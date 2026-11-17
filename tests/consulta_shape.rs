//! Pins the wire shape of a consulta request against `ConsultaLR.xsd` and
//! `SuministroInformacion.xsd`.
//!
//! The consulta envelope declares the `con` and `sum` prefixes only, so every
//! element must be written under one of them, in the order the schema's
//! sequences give. The expected documents below were validated against the
//! official XSDs with `xmllint --schema`.

use quick_xml::de::from_str;
use verifactu::schema::{
    CabeceraConsulta, ClavePaginacion, ConsultaFactuSistemaFacturacion, Consultante, CountryType,
    DatosAdicionalesRespuesta, FechaExpedicionFiltro, FiltroConsulta, IDOtro, Identificador,
    IndicadorRepresentante, MostrarNombreRazonEmisor, MostrarSistemaInformatico, PeriodoImputacion,
    PersonaFisicaJuridicaConsulta, PersonaFisicaJuridicaESConsulta, PersonaFisicaJuridicaID,
    RangoFechaExpedicion, RespuestaConsultaLR, SiNo, SistemaInformaticoConsulta, SoapBodyConsulta,
    SoapEnvelopeConsulta, SoapEnvelopeRespuestaConsulta,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const ENVELOPE_OPEN: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
    r#"<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" "#,
    r#"xmlns:con="https://www2.agenciatributaria.gob.es/static_files/common/internet/dep/aplicaciones/es/aeat/tike/cont/ws/ConsultaLR.xsd" "#,
    r#"xmlns:sum="https://www2.agenciatributaria.gob.es/static_files/common/internet/dep/aplicaciones/es/aeat/tike/cont/ws/SuministroInformacion.xsd">"#,
    "<soapenv:Header/><soapenv:Body>",
);
const ENVELOPE_CLOSE: &str = "</soapenv:Body></soapenv:Envelope>";

fn envelope(payload: ConsultaFactuSistemaFacturacion) -> String {
    SoapEnvelopeConsulta {
        _header: None,
        body: SoapBodyConsulta { payload },
    }
    .to_xml()
}

fn persona_es(
    nombre: &str,
    nif: &str,
) -> Result<PersonaFisicaJuridicaESConsulta, Box<dyn std::error::Error>> {
    Ok(PersonaFisicaJuridicaESConsulta {
        nombre_razon: nombre.try_into()?,
        nif: nif.try_into()?,
    })
}

fn foreign_id(id: &str) -> Result<Identificador, Box<dyn std::error::Error>> {
    Ok(Identificador::IdOtro(IDOtro {
        codigo_pais: Some(CountryType::DE),
        id_type: PersonaFisicaJuridicaID::NifIva,
        id: id.try_into()?,
    }))
}

fn periodo() -> Result<PeriodoImputacion, Box<dyn std::error::Error>> {
    Ok(PeriodoImputacion {
        ejercicio: "2026".try_into()?,
        periodo: "01".try_into()?,
    })
}

fn empty_filtro() -> Result<FiltroConsulta, Box<dyn std::error::Error>> {
    Ok(FiltroConsulta {
        periodo_imputacion: periodo()?,
        num_serie_factura: None,
        contraparte: None,
        fecha_expedicion_factura: None,
        sistema_informatico: None,
        ref_externa: None,
        clave_paginacion: None,
    })
}

/// A query by the issuer with every filter set, a foreign `Contraparte` and a
/// fully populated `SistemaInformatico` produced by a foreign company.
fn consulta_with_every_filter()
-> Result<ConsultaFactuSistemaFacturacion, Box<dyn std::error::Error>> {
    Ok(ConsultaFactuSistemaFacturacion {
        cabecera: CabeceraConsulta {
            id_version: "1.0".try_into()?,
            consultante: Consultante::ObligadoEmision(persona_es("Issuer SL", "B12345678")?),
            indicador_representante: Some(IndicadorRepresentante::S),
        },
        filtro_consulta: FiltroConsulta {
            periodo_imputacion: periodo()?,
            num_serie_factura: Some("INV-1".try_into()?),
            contraparte: Some(PersonaFisicaJuridicaConsulta {
                nombre_razon: "Kunde GmbH".try_into()?,
                identificador: foreign_id("DE123456789")?,
            }),
            fecha_expedicion_factura: Some(FechaExpedicionFiltro {
                rango_fecha_expedicion: Some(RangoFechaExpedicion {
                    desde: Some("01-01-2026".try_into()?),
                    hasta: Some("31-01-2026".try_into()?),
                }),
            }),
            sistema_informatico: Some(SistemaInformaticoConsulta {
                nombre_razon: "Software GmbH".try_into()?,
                identificador: foreign_id("DE987654321")?,
                nombre_sistema_informatico: Some("Facturador".try_into()?),
                id_sistema_informatico: "01".try_into()?,
                version: Some("1.0".try_into()?),
                numero_instalacion: "1".try_into()?,
                tipo_uso_posible_solo_verifactu: Some(SiNo::S),
                tipo_uso_posible_multi_ot: Some(SiNo::N),
                indicador_multiples_ot: Some(SiNo::N),
            }),
            ref_externa: Some("ref".try_into()?),
            clave_paginacion: Some(ClavePaginacion {
                id_emisor_factura: Some("B12345678".try_into()?),
                num_serie_factura: Some("INV-1".try_into()?),
                fecha_expedicion_factura: Some("15-01-2026".try_into()?),
            }),
        },
        datos_adicionales_respuesta: Some(DatosAdicionalesRespuesta {
            mostrar_nombre_razon_emisor: Some(MostrarNombreRazonEmisor::S),
            mostrar_sistema_informatico: Some(MostrarSistemaInformatico::S),
        }),
    })
}

#[test]
fn consulta_with_every_filter_has_the_schema_shape() -> TestResult {
    let xml = envelope(consulta_with_every_filter()?);

    let expected = [
        ENVELOPE_OPEN,
        "<con:ConsultaFactuSistemaFacturacion>",
        "<con:Cabecera>",
        "<sum:IDVersion>1.0</sum:IDVersion>",
        "<sum:ObligadoEmision>",
        "<sum:NombreRazon>Issuer SL</sum:NombreRazon>",
        "<sum:NIF>B12345678</sum:NIF>",
        "</sum:ObligadoEmision>",
        "<sum:IndicadorRepresentante>S</sum:IndicadorRepresentante>",
        "</con:Cabecera>",
        "<con:FiltroConsulta>",
        "<con:PeriodoImputacion>",
        "<sum:Ejercicio>2026</sum:Ejercicio>",
        "<sum:Periodo>01</sum:Periodo>",
        "</con:PeriodoImputacion>",
        "<con:NumSerieFactura>INV-1</con:NumSerieFactura>",
        "<con:Contraparte>",
        "<sum:NombreRazon>Kunde GmbH</sum:NombreRazon>",
        "<sum:IDOtro>",
        "<sum:CodigoPais>DE</sum:CodigoPais>",
        "<sum:IDType>02</sum:IDType>",
        "<sum:ID>DE123456789</sum:ID>",
        "</sum:IDOtro>",
        "</con:Contraparte>",
        "<con:FechaExpedicionFactura>",
        "<sum:RangoFechaExpedicion>",
        "<sum:Desde>01-01-2026</sum:Desde>",
        "<sum:Hasta>31-01-2026</sum:Hasta>",
        "</sum:RangoFechaExpedicion>",
        "</con:FechaExpedicionFactura>",
        "<con:SistemaInformatico>",
        "<sum:NombreRazon>Software GmbH</sum:NombreRazon>",
        "<sum:IDOtro>",
        "<sum:CodigoPais>DE</sum:CodigoPais>",
        "<sum:IDType>02</sum:IDType>",
        "<sum:ID>DE987654321</sum:ID>",
        "</sum:IDOtro>",
        "<sum:NombreSistemaInformatico>Facturador</sum:NombreSistemaInformatico>",
        "<sum:IdSistemaInformatico>01</sum:IdSistemaInformatico>",
        "<sum:Version>1.0</sum:Version>",
        "<sum:NumeroInstalacion>1</sum:NumeroInstalacion>",
        "<sum:TipoUsoPosibleSoloVerifactu>S</sum:TipoUsoPosibleSoloVerifactu>",
        "<sum:TipoUsoPosibleMultiOT>N</sum:TipoUsoPosibleMultiOT>",
        "<sum:IndicadorMultiplesOT>N</sum:IndicadorMultiplesOT>",
        "</con:SistemaInformatico>",
        "<con:RefExterna>ref</con:RefExterna>",
        "<con:ClavePaginacion>",
        "<sum:IDEmisorFactura>B12345678</sum:IDEmisorFactura>",
        "<sum:NumSerieFactura>INV-1</sum:NumSerieFactura>",
        "<sum:FechaExpedicionFactura>15-01-2026</sum:FechaExpedicionFactura>",
        "</con:ClavePaginacion>",
        "</con:FiltroConsulta>",
        "<con:DatosAdicionalesRespuesta>",
        "<con:MostrarNombreRazonEmisor>S</con:MostrarNombreRazonEmisor>",
        "<con:MostrarSistemaInformatico>S</con:MostrarSistemaInformatico>",
        "</con:DatosAdicionalesRespuesta>",
        "</con:ConsultaFactuSistemaFacturacion>",
        ENVELOPE_CLOSE,
    ]
    .concat();

    assert_eq!(xml, expected);
    Ok(())
}

/// The envelope binds `con` and `sum` and nothing else, so any other prefix in
/// the document is an undeclared one and the request is not well-formed.
#[test]
fn consulta_uses_only_the_prefixes_its_envelope_declares() -> TestResult {
    let xml = envelope(consulta_with_every_filter()?);

    for tag in xml.split('<').skip(1) {
        let name = tag
            .trim_start_matches('/')
            .split(['>', ' ', '/'])
            .next()
            .unwrap_or_default();
        if name.starts_with('?') {
            continue;
        }
        let prefix = name.split_once(':').map(|(prefix, _)| prefix);
        assert!(
            matches!(prefix, Some("soapenv" | "con" | "sum")),
            "element <{name}> does not use a prefix the consulta envelope declares"
        );
    }
    Ok(())
}

/// The minimal `SistemaInformatico` filter: the producer by NIF, the system
/// code and the installation number, which the schema makes mandatory.
#[test]
fn sistema_informatico_filter_with_only_mandatory_fields() -> TestResult {
    let mut filtro = empty_filtro()?;
    filtro.sistema_informatico = Some(SistemaInformaticoConsulta {
        nombre_razon: "Software SL".try_into()?,
        identificador: Identificador::Nif("B87654321".try_into()?),
        nombre_sistema_informatico: None,
        id_sistema_informatico: "01".try_into()?,
        version: None,
        numero_instalacion: "1".try_into()?,
        tipo_uso_posible_solo_verifactu: None,
        tipo_uso_posible_multi_ot: None,
        indicador_multiples_ot: None,
    });
    let xml = envelope(ConsultaFactuSistemaFacturacion {
        cabecera: CabeceraConsulta {
            id_version: "1.0".try_into()?,
            consultante: Consultante::ObligadoEmision(persona_es("Issuer SL", "B12345678")?),
            indicador_representante: None,
        },
        filtro_consulta: filtro,
        datos_adicionales_respuesta: None,
    });

    assert!(
        xml.contains(concat!(
            "<con:SistemaInformatico>",
            "<sum:NombreRazon>Software SL</sum:NombreRazon>",
            "<sum:NIF>B87654321</sum:NIF>",
            "<sum:IdSistemaInformatico>01</sum:IdSistemaInformatico>",
            "<sum:NumeroInstalacion>1</sum:NumeroInstalacion>",
            "</con:SistemaInformatico>",
        )),
        "unexpected SistemaInformatico filter in {xml}"
    );
    Ok(())
}

/// A query by the recipient names the `Destinatario` and never the
/// `ObligadoEmision`: the schema makes the two a choice. An open-ended date
/// range carries only the end that was given.
#[test]
fn consulta_by_destinatario_with_open_ended_range() -> TestResult {
    let mut filtro = empty_filtro()?;
    filtro.fecha_expedicion_factura = Some(FechaExpedicionFiltro {
        rango_fecha_expedicion: Some(RangoFechaExpedicion {
            desde: Some("01-01-2026".try_into()?),
            hasta: None,
        }),
    });
    let xml = envelope(ConsultaFactuSistemaFacturacion {
        cabecera: CabeceraConsulta {
            id_version: "1.0".try_into()?,
            consultante: Consultante::Destinatario(persona_es("Customer SL", "A84532509")?),
            indicador_representante: None,
        },
        filtro_consulta: filtro,
        datos_adicionales_respuesta: None,
    });

    assert!(
        xml.contains(concat!(
            "<con:Cabecera>",
            "<sum:IDVersion>1.0</sum:IDVersion>",
            "<sum:Destinatario>",
            "<sum:NombreRazon>Customer SL</sum:NombreRazon>",
            "<sum:NIF>A84532509</sum:NIF>",
            "</sum:Destinatario>",
            "</con:Cabecera>",
        )),
        "unexpected Cabecera in {xml}"
    );
    assert!(!xml.contains("ObligadoEmision"));
    assert!(
        xml.contains(concat!(
            "<sum:RangoFechaExpedicion>",
            "<sum:Desde>01-01-2026</sum:Desde>",
            "</sum:RangoFechaExpedicion>",
        )),
        "unexpected RangoFechaExpedicion in {xml}"
    );
    Ok(())
}

#[test]
fn consulta_with_every_filter_round_trips() -> TestResult {
    let xml = envelope(consulta_with_every_filter()?);

    let parsed: SoapEnvelopeConsulta<ConsultaFactuSistemaFacturacion> = from_str(&xml)?;

    assert_eq!(parsed.to_xml(), xml);
    Ok(())
}

/// A header naming both parties, or neither, is not a consulta header.
#[test]
fn cabecera_must_name_exactly_one_party() {
    let both = concat!(
        "<Cabecera><IDVersion>1.0</IDVersion>",
        "<ObligadoEmision><NombreRazon>A</NombreRazon><NIF>B12345678</NIF></ObligadoEmision>",
        "<Destinatario><NombreRazon>B</NombreRazon><NIF>A84532509</NIF></Destinatario>",
        "</Cabecera>",
    );
    let neither = "<Cabecera><IDVersion>1.0</IDVersion></Cabecera>";

    assert!(from_str::<CabeceraConsulta>(both).is_err());
    assert!(from_str::<CabeceraConsulta>(neither).is_err());
}

/// The response side reads the same shared types under AEAT's own prefixes.
/// The sample is synthetic and validates against `RespuestaConsultaLR.xsd`: a
/// record whose `Tercero` and `IDDestinatario` are foreign (`IDOtro`).
#[test]
fn respuesta_with_foreign_parties_parses() -> TestResult {
    let xml = include_str!("data/consulta/respuesta_idotro.xml");

    let envelope: SoapEnvelopeRespuestaConsulta<RespuestaConsultaLR> = from_str(xml)?;
    let respuesta = envelope.body.payload;

    assert!(matches!(
        respuesta.cabecera.consultante,
        Consultante::ObligadoEmision(_)
    ));
    let registro = respuesta.registros.first().ok_or("one registro")?;
    let datos = &registro.datos_registro_facturacion;
    let tercero = datos.tercero.as_ref().ok_or("a Tercero")?;
    match &tercero.identificador {
        Identificador::IdOtro(id_otro) => assert_eq!(id_otro.id.as_str(), "DE111111111"),
        Identificador::Nif(_) => return Err("Tercero should be identified by IDOtro".into()),
    }
    let destinatarios = datos.destinatarios.as_ref().ok_or("Destinatarios")?;
    let destinatario = destinatarios
        .destinatarios
        .first()
        .ok_or("one IDDestinatario")?;
    match &destinatario.identificador {
        Identificador::IdOtro(id_otro) => assert_eq!(id_otro.id.as_str(), "DE123456789"),
        Identificador::Nif(_) => return Err("IDDestinatario should be identified by IDOtro".into()),
    }
    assert!(datos.sistema_informatico.is_some());
    Ok(())
}
