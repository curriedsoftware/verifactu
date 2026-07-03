/***
 * Copyright (c) 2026 Rafael Fernández López <ereslibre@curried.software>
 *
 * Permission is hereby granted, free of charge, to any person
 * obtaining a copy of this software and associated documentation
 * files (the "Software"), to deal in the Software without
 * restriction, including without limitation the rights to use, copy,
 * modify, merge, publish, distribute, sublicense, and/or sell copies
 * of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be
 * included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
 * NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
 * BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
 * ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 ***/

use std::{
    env,
    error::Error,
    fs,
    io::{self, Write},
};

use clap::{Args, Parser, Subcommand};
use const_oid::ObjectIdentifier;
use pkcs8::{EncryptedPrivateKeyInfo, der::Decode};
use prettytable::{Table, format, row};
use quick_xml::se::to_string;
use reqwest::{Client, Identity};
use verifactu::{
    self, Environment,
    errors::DataError,
    hashing::{AltaHuellaInput, AnulacionHuellaInput},
    schema::{
        Encadenamiento, EstadoRegistroConsulta, Identificador, IndicadorPaginacion,
        PeriodoImputacion as SchemaPeriodoImputacion, PersonaFisicaJuridicaConsulta,
        RegistroRespuestaConsultaRegFacturacion, RespuestaConsultaLR, ResultadoConsulta, SiNo,
    },
};
use x509_cert::{Certificate, der::DecodePem};

const CERTIFICATE_ENV: &str = "VERIFACTU_CERTIFICATE_PEM_PATH";
const PRIVATE_KEY_ENV: &str = "VERIFACTU_PRIVATE_KEY_PEM_PATH";
const PRIVATE_KEY_PASSPHRASE_ENV: &str = "VERIFACTU_PRIVATE_KEY_PASSPHRASE";

// Subject DN attribute OIDs we read from the AEAT certificate to derive the
// ObligadoEmision identity.
const OID_ORGANIZATION_NAME: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.4.10");
const OID_ORGANIZATION_IDENTIFIER: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.4.97");
const OID_COMMON_NAME: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.4.3");
const OID_SERIAL_NUMBER: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.4.5");
const OID_GIVEN_NAME: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.4.42");
const OID_SURNAME: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.4.4");

#[derive(Args, Clone, Debug)]
struct FiltroConsulta {
    /// Periodo de imputación
    #[command(flatten)]
    periodo_imputacion: PeriodoImputacion,
    /// Número de serie de factura
    #[arg(short, long)]
    num_serie_factura: Option<String>,
    /// Contraparte
    #[arg(short, long)]
    contraparte: Option<String>,
    /// Fecha de expedición de factura
    #[arg(short, long)]
    fecha_expedicion_factura: Option<String>,
    /// Sistema informático
    #[arg(short, long)]
    sistema_informatico: Option<String>,
    /// Referencia externa
    #[arg(short, long)]
    referencia_externa: Option<String>,
}

impl TryFrom<FiltroConsulta> for verifactu::schema::FiltroConsulta {
    type Error = verifactu::errors::DataError;

    fn try_from(value: FiltroConsulta) -> Result<Self, Self::Error> {
        Ok(Self {
            periodo_imputacion: SchemaPeriodoImputacion {
                ejercicio: value
                    .periodo_imputacion
                    .ejercicio
                    .try_into()
                    .map_err(|error| {
                        DataError::InvalidData(format!("ejercicio is invalid: {error}"))
                    })?,
                periodo: value
                    .periodo_imputacion
                    .periodo
                    .try_into()
                    .map_err(|error| {
                        DataError::InvalidData(format!("periodo is invalid: {error}"))
                    })?,
            },
            num_serie_factura: None,
            contraparte: None,
            fecha_expedicion_factura: None,
            sistema_informatico: None,
            ref_externa: None,
            clave_paginacion: None,
        })
    }
}

#[derive(Args, Clone, Debug)]
struct PeriodoImputacion {
    /// Ejercicio
    ejercicio: String,
    /// Periodo
    periodo: String,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    /// Check "registros de facturación"
    Consulta {
        #[command(flatten)]
        filtro: FiltroConsulta,
        /// Show result in XML format.
        #[arg(long)]
        xml: bool,
    },
    /// Verify the integrity of a registered hash chain by recomputing every
    /// record's huella from the data AEAT returns for a period (`consulta`), and
    /// checking it against the huella AEAT stored. Exits non-zero on any
    /// mismatch.
    VerifyChain {
        #[command(flatten)]
        filtro: FiltroConsulta,
    },
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Target the REAL AEAT production systems instead of the test
    /// ("preproducción") environment. Submissions to production are legally
    /// binding; you will be asked to confirm interactively unless `--yes` is
    /// also given. Without this flag, the safe test environment is used.
    #[arg(long, global = true)]
    production: bool,
    /// Skip the interactive confirmation prompt when targeting production.
    /// Intended for non-interactive/automated use; use with care.
    #[arg(long, global = true)]
    yes: bool,
    /// NIF del obligado a emisión. If omitted, it's derived from the user
    /// certificate. Applies to every command that needs an ObligadoEmision
    /// identity (consulta, verify-chain, ...).
    #[arg(long, global = true)]
    obligado_emision: Option<String>,
    #[command(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let environment = resolve_environment(cli.production, cli.yes)?;

    match cli.command {
        Command::Consulta { filtro, xml } => {
            let client = build_verifactu_client(environment)?;
            let obligado_emision = build_obligado_emision(cli.obligado_emision)?;
            let result = client
                .consulta(&verifactu::schema::ConsultaFactuSistemaFacturacion {
                    cabecera: verifactu::schema::CabeceraConsulta {
                        id_version: "1.0".try_into()?,
                        obligado_emision: Some(obligado_emision),
                        destinatario: None,
                        indicador_representante: None,
                    },
                    filtro_consulta: filtro.try_into()?,
                    datos_adicionales_respuesta: None,
                })
                .await?;
            if xml {
                let xml_output = to_string(&result).expect("serialization should not fail");
                println!("{}", xml_output);
            } else {
                print_consulta_table(&result);
            }
        }
        Command::VerifyChain { filtro } => {
            let client = build_verifactu_client(environment)?;
            let obligado_emision = build_obligado_emision(cli.obligado_emision)?;
            let registros = fetch_all_registros(&client, obligado_emision, filtro).await?;
            let checks = verify_chain(&registros);
            print_chain_report(&checks);
            let failures = checks.iter().filter(|c| c.outcome.is_failure()).count();
            if failures > 0 {
                return Err(
                    format!("chain integrity check failed: {failures} bad record(s)").into(),
                );
            }
        }
    }

    Ok(())
}

/// Query the AEAT `consulta` endpoint for a period, following pagination until
/// every registro has been collected.
async fn fetch_all_registros(
    client: &verifactu::Client,
    obligado_emision: PersonaFisicaJuridicaConsulta,
    filtro: FiltroConsulta,
) -> Result<Vec<RegistroRespuestaConsultaRegFacturacion>, Box<dyn Error>> {
    let base_filtro: verifactu::schema::FiltroConsulta = filtro.try_into()?;

    let mut registros = Vec::new();
    let mut clave_paginacion = None;

    loop {
        let mut filtro_consulta = base_filtro.clone();
        filtro_consulta.clave_paginacion = clave_paginacion.clone();

        let result = client
            .consulta(&verifactu::schema::ConsultaFactuSistemaFacturacion {
                cabecera: verifactu::schema::CabeceraConsulta {
                    id_version: "1.0".try_into()?,
                    obligado_emision: Some(obligado_emision.clone()),
                    destinatario: None,
                    indicador_representante: None,
                },
                filtro_consulta,
                datos_adicionales_respuesta: None,
            })
            .await?;

        if matches!(result.resultado_consulta, ResultadoConsulta::SinDatos) {
            break;
        }

        registros.extend(result.registros);

        match result.indicador_paginacion {
            IndicadorPaginacion::S => {
                clave_paginacion = result.clave_paginacion;
                // Defensive: AEAT signalled more pages but gave no cursor. Stop
                // rather than loop forever on the same page.
                if clave_paginacion.is_none() {
                    break;
                }
            }
            IndicadorPaginacion::N => break,
        }
    }

    Ok(registros)
}

/// Outcome of recomputing one record's huella.
enum CheckOutcome {
    /// The recomputed huella matches the one AEAT stored.
    Ok,
    /// Recomputed huella differs from the stored one (tampered or corrupt).
    Mismatch { stored: String, recomputed: String },
    /// A field required to recompute the huella was absent from the response.
    Missing(&'static str),
}

impl CheckOutcome {
    fn is_failure(&self) -> bool {
        !matches!(self, CheckOutcome::Ok)
    }
}

/// How a record links into the chain, derived from its `Encadenamiento`.
enum LinkStatus {
    /// First record of the chain (`PrimerRegistro`).
    First,
    /// Chains onto a record whose huella is present in the queried set.
    LinkedInSet,
    /// Chains onto a huella not in the queried set (e.g. a prior period).
    LinkedOutsideSet,
    /// No `Encadenamiento` element at all -- anomalous; AEAT always sets one.
    MissingEncadenamiento,
}

/// The verification result for a single record.
struct RecordCheck {
    index: usize,
    kind: &'static str,
    num_serie: String,
    fecha: String,
    outcome: CheckOutcome,
    link: LinkStatus,
}

/// Recompute and verify the huella of every record AEAT returned.
///
/// Each record carries the previous record's huella inside its own
/// `Encadenamiento::RegistroAnterior`, so the per-record check is self-contained:
/// recompute the huella from the record's fields plus that embedded predecessor
/// huella and compare against the huella AEAT stored. Any tampering with a
/// record's content *or* with the predecessor huella it claims breaks the match.
fn verify_chain(registros: &[RegistroRespuestaConsultaRegFacturacion]) -> Vec<RecordCheck> {
    // Index records by their stored huella so we can classify each link as
    // pointing inside or outside the queried set.
    let huellas: std::collections::HashSet<&str> = registros
        .iter()
        .filter_map(|r| r.datos_registro_facturacion.huella.as_ref())
        .map(|h| h.as_str())
        .collect();

    registros
        .iter()
        .enumerate()
        .map(|(index, record)| {
            let datos = &record.datos_registro_facturacion;
            let is_anulacion = matches!(
                record.estado_registro.estado_registro,
                EstadoRegistroConsulta::Anulado
            );

            let prev_huella: Option<&str> = match &datos.encadenamiento {
                Some(Encadenamiento::RegistroAnterior(prev)) => Some(prev.huella.as_str()),
                Some(Encadenamiento::PrimerRegistro) | None => None,
            };

            let link = match &datos.encadenamiento {
                Some(Encadenamiento::PrimerRegistro) => LinkStatus::First,
                Some(Encadenamiento::RegistroAnterior(prev)) => {
                    if huellas.contains(prev.huella.as_str()) {
                        LinkStatus::LinkedInSet
                    } else {
                        LinkStatus::LinkedOutsideSet
                    }
                }
                None => LinkStatus::MissingEncadenamiento,
            };

            let outcome = recompute_outcome(record, is_anulacion, prev_huella);

            RecordCheck {
                index,
                kind: if is_anulacion { "anulacion" } else { "alta" },
                num_serie: record.id_factura.num_serie_factura.to_string(),
                fecha: record.id_factura.fecha_expedicion_factura.to_string(),
                outcome,
                link,
            }
        })
        .collect()
}

/// Recompute a single record's huella and compare it to the stored value.
fn recompute_outcome(
    record: &RegistroRespuestaConsultaRegFacturacion,
    is_anulacion: bool,
    prev_huella: Option<&str>,
) -> CheckOutcome {
    let datos = &record.datos_registro_facturacion;

    let Some(stored) = datos.huella.as_ref().map(|h| h.as_str()) else {
        return CheckOutcome::Missing("Huella");
    };
    let Some(fecha_hora) = datos.fecha_hora_huso_gen_registro.as_deref() else {
        return CheckOutcome::Missing("FechaHoraHusoGenRegistro");
    };

    let id_emisor = record.id_factura.id_emisor_factura.to_string();
    let num_serie = record.id_factura.num_serie_factura.to_string();
    let fecha = record.id_factura.fecha_expedicion_factura.to_string();

    let recomputed = if is_anulacion {
        AnulacionHuellaInput {
            id_emisor_factura_anulada: &id_emisor,
            num_serie_factura_anulada: &num_serie,
            fecha_expedicion_factura_anulada: &fecha,
            prev_huella,
            fecha_hora_huso_gen_registro: fecha_hora,
        }
        .huella()
    } else {
        let Some(tipo_factura) = datos.tipo_factura.as_ref() else {
            return CheckOutcome::Missing("TipoFactura");
        };
        let Some(cuota_total) = datos.cuota_total.as_ref() else {
            return CheckOutcome::Missing("CuotaTotal");
        };
        let Some(importe_total) = datos.importe_total.as_ref() else {
            return CheckOutcome::Missing("ImporteTotal");
        };
        AltaHuellaInput {
            id_emisor_factura: &id_emisor,
            num_serie_factura: &num_serie,
            fecha_expedicion_factura: &fecha,
            tipo_factura: &tipo_factura.to_string(),
            cuota_total: cuota_total.as_ref(),
            importe_total: importe_total.as_ref(),
            prev_huella,
            fecha_hora_huso_gen_registro: fecha_hora,
        }
        .huella()
    };

    if recomputed == stored {
        CheckOutcome::Ok
    } else {
        CheckOutcome::Mismatch {
            stored: stored.to_owned(),
            recomputed,
        }
    }
}

/// Print the per-record verification results and a final summary.
fn print_chain_report(checks: &[RecordCheck]) {
    if checks.is_empty() {
        println!("No registros returned for the requested period; nothing to verify.");
        return;
    }

    println!(
        "\n=== Verifying Verifactu hash chain ({} record(s)) ===\n",
        checks.len()
    );

    for check in checks {
        let link = match check.link {
            LinkStatus::First => "primer registro",
            LinkStatus::LinkedInSet => "← prev in set",
            LinkStatus::LinkedOutsideSet => "← prev outside set",
            LinkStatus::MissingEncadenamiento => "⚠ no encadenamiento",
        };
        match &check.outcome {
            CheckOutcome::Ok => {
                println!(
                    "  ✓ [{}] {} {} ({}) [{link}] huella OK",
                    check.index + 1,
                    check.kind,
                    check.num_serie,
                    check.fecha,
                );
            }
            CheckOutcome::Mismatch { stored, recomputed } => {
                println!(
                    "  ✗ [{}] {} {} ({}) [{link}] HUELLA MISMATCH\n      stored:     {stored}\n      recomputed: {recomputed}",
                    check.index + 1,
                    check.kind,
                    check.num_serie,
                    check.fecha,
                );
            }
            CheckOutcome::Missing(field) => {
                println!(
                    "  ✗ [{}] {} {} ({}) [{link}] cannot recompute: missing {field}",
                    check.index + 1,
                    check.kind,
                    check.num_serie,
                    check.fecha,
                );
            }
        }
    }

    let failures = checks.iter().filter(|c| c.outcome.is_failure()).count();
    println!();
    if failures == 0 {
        println!(
            "✓ Chain integrity verified: all {} huellas match.",
            checks.len()
        );
    } else {
        println!("✗ Chain integrity check found {failures} bad record(s).");
    }
}

/// Render a `consulta` response as a human-readable table using `prettytable`.
fn print_consulta_table(result: &RespuestaConsultaLR) {
    println!(
        "Periodo: {}/{:?}  ·  Resultado: {:?}  ·  Registros: {}",
        result.periodo_imputacion.ejercicio,
        result.periodo_imputacion.periodo,
        result.resultado_consulta,
        result.registros.len(),
    );

    if result.registros.is_empty() {
        println!("No hay registros.");
        return;
    }

    // Stringify an optional field, falling back to a dash when absent.
    fn opt<T: std::fmt::Display>(value: &Option<T>) -> String {
        value
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "-".to_owned())
    }

    let mut table = Table::new();
    let mut table_format = *format::consts::FORMAT_BOX_CHARS;
    // Set the title row off from the data with a heavier double-line border.
    table_format.separator(
        format::LinePosition::Title,
        format::LineSeparator::new('═', '╪', '╞', '╡'),
    );
    table.set_format(table_format);
    table.set_titles(row![
        "NIF Emisor",
        "Núm. Serie",
        "Fecha Exp.",
        "Destinatario",
        "Subsanación",
        "Tipo",
        "Importe Total",
        "Cuota Total",
        "Estado",
    ]);

    // Recipient name, condensed: show the first and a "+N" hint when there are more.
    fn destinatario(datos: &verifactu::schema::RespuestaDatosRegistroFacturacion) -> String {
        match &datos.destinatarios {
            Some(d) => {
                let first = d
                    .destinatarios
                    .first()
                    .map(|dest| dest.nombre_razon.to_string())
                    .unwrap_or_else(|| "-".to_owned());
                match d.destinatarios.len() {
                    0 | 1 => first,
                    n => format!("{first} (+{})", n - 1),
                }
            }
            None => "-".to_owned(),
        }
    }

    for registro in &result.registros {
        let datos = &registro.datos_registro_facturacion;
        table.add_row(row![
            registro.id_factura.id_emisor_factura,
            registro.id_factura.num_serie_factura,
            registro.id_factura.fecha_expedicion_factura,
            destinatario(datos),
            match datos.subsanacion {
                Some(SiNo::S) => "Sí",
                Some(SiNo::N) => "No",
                Some(SiNo::X) | None => "-",
            },
            datos
                .tipo_factura
                .as_ref()
                .map(|tipo| format!("{tipo:?}"))
                .unwrap_or_else(|| "-".to_owned()),
            opt(&datos.importe_total),
            opt(&datos.cuota_total),
            format!("{:?}", registro.estado_registro.estado_registro),
        ]);
    }

    table.printstd();
}

/// Read the AEAT certificate PEM from the path in `VERIFACTU_CERTIFICATE_PEM_PATH`.
fn read_certificate_pem() -> Result<String, Box<dyn Error>> {
    let certificate_path = env::var(CERTIFICATE_ENV).map_err(|err| {
        format!("missing {CERTIFICATE_ENV} environment variable with path to certificate: {err}")
    })?;
    fs::read_to_string(&certificate_path)
        .map_err(|err| format!("failed to read certificate {certificate_path}: {err}").into())
}

/// Build the consulta `ObligadoEmision` identity. Both the NIF and the company
/// name are derived from the certificate subject; an explicit `obligado_nif`
/// overrides the derived NIF.
fn build_obligado_emision(
    obligado_nif: Option<String>,
) -> Result<PersonaFisicaJuridicaConsulta, Box<dyn Error>> {
    let cert_pem = read_certificate_pem()?;
    let (nombre_razon, derived_nif) = derive_obligado_from_cert(&cert_pem)?;
    let nif = obligado_nif.unwrap_or(derived_nif);

    Ok(PersonaFisicaJuridicaConsulta {
        nombre_razon: nombre_razon
            .as_str()
            .try_into()
            .map_err(|err| format!("invalid NombreRazon derived from certificate: {err}"))?,
        identificador: Identificador::Nif(
            nif.as_str()
                .try_into()
                .map_err(|err| format!("invalid ObligadoEmision NIF: {err}"))?,
        ),
    })
}

/// Derive `(NombreRazon, NIF)` for the ObligadoEmision from the certificate
/// subject DN. Handles both representative-of-legal-person certificates (uses
/// organizationName + organizationIdentifier) and natural-person certificates
/// (uses givenName/surname or commonName + serialNumber).
fn derive_obligado_from_cert(cert_pem: &str) -> Result<(String, String), Box<dyn Error>> {
    let cert = Certificate::from_pem(cert_pem.as_bytes())
        .map_err(|err| format!("failed to parse certificate: {err}"))?;

    let mut organization = None;
    let mut organization_id = None;
    let mut common_name = None;
    let mut serial_number = None;
    let mut given_name = None;
    let mut surname = None;

    for rdn in cert.tbs_certificate.subject.0.iter() {
        for atav in rdn.0.iter() {
            let Ok(value) = std::str::from_utf8(atav.value.value()) else {
                continue;
            };
            let oid = atav.oid;
            let slot = if oid == OID_ORGANIZATION_NAME {
                &mut organization
            } else if oid == OID_ORGANIZATION_IDENTIFIER {
                &mut organization_id
            } else if oid == OID_COMMON_NAME {
                &mut common_name
            } else if oid == OID_SERIAL_NUMBER {
                &mut serial_number
            } else if oid == OID_GIVEN_NAME {
                &mut given_name
            } else if oid == OID_SURNAME {
                &mut surname
            } else {
                continue;
            };
            *slot = Some(value.to_owned());
        }
    }

    // NombreRazon: legal person → organizationName; natural person → "GN SN",
    // falling back to the commonName.
    let nombre_razon = organization
        .or_else(|| match (&given_name, &surname) {
            (Some(gn), Some(sn)) => Some(format!("{gn} {sn}")),
            _ => None,
        })
        .or(common_name)
        .ok_or("certificate subject has no organizationName, given/surname or commonName")?;

    // NIF: legal person → organizationIdentifier ("VATES-<nif>"); natural person
    // → serialNumber ("IDCES-<nif>"). Strip the Spanish prefixes if present.
    let nif = organization_id
        .map(|value| strip_nif_prefix(&value).to_owned())
        .or_else(|| serial_number.map(|value| strip_nif_prefix(&value).to_owned()))
        .ok_or(
            "certificate subject has no organizationIdentifier or serialNumber to derive a NIF",
        )?;

    Ok((nombre_razon, nif))
}

/// Strip the eIDAS semantics-identifier prefixes that precede the bare NIF in
/// certificate subject attributes. Per ETSI EN 319 412-1 these are a 3-char
/// identity type reference, a 2-char country code and a mandatory hyphen:
/// `VATES-` for a legal person's `organizationIdentifier` and `IDCES-` for a
/// natural person's `serialNumber`.
fn strip_nif_prefix(value: &str) -> &str {
    value
        .strip_prefix("VATES-")
        .or_else(|| value.strip_prefix("IDCES-"))
        .unwrap_or(value)
}

/// Resolve which AEAT environment to target from the CLI flags. Defaults to the
/// safe test environment; production requires the explicit `--production` flag
/// and, unless `--yes` is given, an interactive confirmation typed by the user.
/// This is the application-level guardrail that keeps production from being hit
/// by mistake.
fn resolve_environment(production: bool, assume_yes: bool) -> Result<Environment, Box<dyn Error>> {
    if !production {
        eprintln!("Environment: TEST (preproducción)");
        return Ok(Environment::Test);
    }

    if !assume_yes {
        eprintln!("⚠️  You are about to target the AEAT PRODUCTION environment.");
        eprintln!("⚠️  Requests sent here submit REAL, legally binding fiscal records.");
        eprint!("Type 'production' to continue: ");
        io::stderr().flush().ok();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim() != "production" {
            return Err("production confirmation not given; aborting".into());
        }
    }

    eprintln!("Environment: PRODUCTION");
    Ok(Environment::Production)
}

/// Build a TLS-mutual-auth-ready VeriFactu client bound to `environment`, using
/// the certificate/key paths and passphrase pointed to by the `VERIFACTU_*`
/// environment variables (loaded from `.env.local` via direnv during
/// development).
fn build_verifactu_client(environment: Environment) -> Result<verifactu::Client, Box<dyn Error>> {
    let private_key_path = env::var(PRIVATE_KEY_ENV).map_err(|err| {
        format!("missing {PRIVATE_KEY_ENV} environment variable with path to private key: {err}")
    })?;
    let passphrase = env::var(PRIVATE_KEY_PASSPHRASE_ENV).map_err(|err| {
        format!("missing {PRIVATE_KEY_PASSPHRASE_ENV} environment variable: {err}")
    })?;

    let cert_pem = read_certificate_pem()?;
    let key_pem_content = fs::read_to_string(&private_key_path)
        .map_err(|err| format!("failed to read private key {private_key_path}: {err}"))?;

    let key_pem_parsed = pem::parse(&key_pem_content)
        .map_err(|err| format!("failed to parse encrypted private key PEM: {err}"))?;

    let encrypted_private_key = EncryptedPrivateKeyInfo::from_der(key_pem_parsed.contents())
        .map_err(|err| format!("failed to decode encrypted key: {err:?}"))?;
    let decrypted_key_der = encrypted_private_key
        .decrypt(passphrase)
        .map_err(|err| format!("failed to decrypt private key: {err:?}"))?;

    let key_pem_obj = pem::Pem::new("PRIVATE KEY", decrypted_key_der.as_bytes());
    let decrypted_key_pem = pem::encode(&key_pem_obj);

    let mut pem_data = String::new();
    pem_data.push_str(&decrypted_key_pem);
    pem_data.push_str(&cert_pem);

    let identity = Identity::from_pem(pem_data.as_bytes())
        .map_err(|err| format!("invalid identity: {err}"))?;

    let http = Client::builder()
        .identity(identity)
        .build()
        .map_err(|err| format!("failed to build reqwest client: {err}"))?;

    Ok(verifactu::Client::new(http, environment))
}
