/***
 * Copyright (c) 2025 Rafael Fernández López <ereslibre@curried.software>
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

#![allow(dead_code)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::large_enum_variant)]

use quick_xml::se::to_string;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use std::{borrow::Cow, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    type_name: &'static str,
    message: Cow<'static, str>,
}

impl ValidationError {
    fn new(type_name: &'static str, message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            type_name,
            message: message.into(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.type_name, self.message)
    }
}

impl std::error::Error for ValidationError {}

macro_rules! impl_string_type {
    ($name:ident) => {
        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<&str> for $name {
            type Error = ValidationError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::validate(value)?;
                Ok(Self(value.to_owned()))
            }
        }

        impl TryFrom<String> for $name {
            type Error = ValidationError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::validate(&value)?;
                Ok(Self(value))
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl std::ops::Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                self.as_str()
            }
        }

        impl std::borrow::Borrow<str> for $name {
            fn borrow(&self) -> &str {
                self.as_str()
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::try_from(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

macro_rules! string_max_type {
    ($name:ident, $max_len:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(String);

        impl $name {
            fn validate(value: &str) -> Result<(), ValidationError> {
                let len = value.chars().count();
                if len > $max_len {
                    Err(ValidationError::new(
                        stringify!($name),
                        format!("must contain at most {} characters, got {}", $max_len, len),
                    ))
                } else {
                    Ok(())
                }
            }
        }

        impl_string_type!($name);
    };
}

string_max_type!(StringMax2, 2);
string_max_type!(StringMax15, 15);
string_max_type!(StringMax16, 16);
string_max_type!(StringMax18, 18);
string_max_type!(StringMax20, 20);
string_max_type!(StringMax25, 25);
string_max_type!(StringMax30, 30);
string_max_type!(StringMax34, 34);
string_max_type!(StringMax40, 40);
string_max_type!(StringMax50, 50);
string_max_type!(StringMax60, 60);
string_max_type!(StringMax64, 64);
string_max_type!(StringMax65, 65);
string_max_type!(StringMax70, 70);
string_max_type!(StringMax100, 100);
string_max_type!(StringMax120, 120);
string_max_type!(StringMax150, 150);
string_max_type!(StringMax250, 250);
string_max_type!(StringMax1500, 1500);

fn validate_unsigned_decimal(
    value: &str,
    min_integer_digits: usize,
    max_integer_digits: usize,
    max_decimal_digits: usize,
    allow_empty_decimal: bool,
    type_name: &'static str,
) -> Result<(), ValidationError> {
    if value.is_empty() {
        return Err(ValidationError::new(type_name, "value cannot be empty"));
    }

    let mut parts = value.split('.');
    let integer_part = parts.next().unwrap_or_default();
    let decimal_part = parts.next();

    if parts.next().is_some() {
        return Err(ValidationError::new(
            type_name,
            "value cannot contain more than one decimal separator",
        ));
    }

    if !integer_part.chars().all(|c| c.is_ascii_digit()) {
        return Err(ValidationError::new(
            type_name,
            "integer part must contain only digits",
        ));
    }

    let integer_len = integer_part.len();
    if integer_len < min_integer_digits || integer_len > max_integer_digits {
        return Err(ValidationError::new(
            type_name,
            format!(
                "integer part must contain between {} and {} digits",
                min_integer_digits, max_integer_digits
            ),
        ));
    }

    if let Some(decimal) = decimal_part {
        if decimal.is_empty() && !allow_empty_decimal {
            return Err(ValidationError::new(
                type_name,
                "decimal separator requires at least one digit",
            ));
        }
        if !decimal.chars().all(|c| c.is_ascii_digit()) {
            return Err(ValidationError::new(
                type_name,
                "decimal part must contain only digits",
            ));
        }
        if decimal.len() > max_decimal_digits {
            return Err(ValidationError::new(
                type_name,
                format!(
                    "decimal part can contain at most {} digits",
                    max_decimal_digits
                ),
            ));
        }
    }

    Ok(())
}

fn validate_signed_decimal(
    value: &str,
    min_integer_digits: usize,
    max_integer_digits: usize,
    max_decimal_digits: usize,
    allow_empty_decimal: bool,
    type_name: &'static str,
) -> Result<(), ValidationError> {
    if value.is_empty() {
        return Err(ValidationError::new(type_name, "value cannot be empty"));
    }

    let (digits, sign_consumed) = match value.as_bytes()[0] {
        b'+' | b'-' => (value.get(1..).unwrap_or_default(), true),
        _ => (value, false),
    };

    if sign_consumed && digits.is_empty() {
        return Err(ValidationError::new(
            type_name,
            "value must contain digits after the sign",
        ));
    }

    validate_unsigned_decimal(
        digits,
        min_integer_digits,
        max_integer_digits,
        max_decimal_digits,
        allow_empty_decimal,
        type_name,
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tipo2_2(String);

impl Tipo2_2 {
    fn validate(value: &str) -> Result<(), ValidationError> {
        validate_unsigned_decimal(value, 1, 3, 2, true, "Tipo2_2")
    }
}

impl_string_type!(Tipo2_2);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tipo3(String);

impl Tipo3 {
    fn validate(value: &str) -> Result<(), ValidationError> {
        if value.len() > 3 {
            return Err(ValidationError::new(
                "Tipo3",
                "must contain at most 3 digits",
            ));
        }
        if !value.chars().all(|c| c.is_ascii_digit()) {
            return Err(ValidationError::new("Tipo3", "must contain only digits"));
        }
        Ok(())
    }
}

impl_string_type!(Tipo3);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tipo6(String);

impl Tipo6 {
    fn validate(value: &str) -> Result<(), ValidationError> {
        if value.len() > 4 {
            return Err(ValidationError::new(
                "Tipo6",
                "must contain at most 4 digits",
            ));
        }
        if !value.chars().all(|c| c.is_ascii_digit()) {
            return Err(ValidationError::new("Tipo6", "must contain only digits"));
        }
        Ok(())
    }
}

impl_string_type!(Tipo6);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImporteSgn12_2(String);

impl ImporteSgn12_2 {
    fn validate(value: &str) -> Result<(), ValidationError> {
        validate_signed_decimal(value, 1, 12, 2, true, "ImporteSgn12_2")
    }
}

impl_string_type!(ImporteSgn12_2);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImporteSgn14_2(String);

impl ImporteSgn14_2 {
    fn validate(value: &str) -> Result<(), ValidationError> {
        validate_signed_decimal(value, 1, 14, 2, true, "ImporteSgn14_2")
    }
}

impl_string_type!(ImporteSgn14_2);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Fecha(String);

enum DateOrder {
    DayMonthYear,
}

impl Fecha {
    fn validate(value: &str) -> Result<(), ValidationError> {
        if let Some((year, month, day)) = parse_date(value, DateOrder::DayMonthYear)
            && is_valid_calendar_date(year, month, day)
        {
            return Ok(());
        }

        Err(ValidationError::new(
            "Fecha",
            "must follow the DD-MM-YYYY format",
        ))
    }
}

impl_string_type!(Fecha);

fn parse_date(value: &str, order: DateOrder) -> Option<(i32, u32, u32)> {
    let mut parts = value.split('-');
    let first = parts.next()?;
    let second = parts.next()?;
    let third = parts.next()?;
    if parts.next().is_some() {
        return None;
    }

    match order {
        DateOrder::DayMonthYear => {
            if first.len() != 2 || second.len() != 2 || third.len() != 4 {
                return None;
            }
            let day = first.parse().ok()?;
            let month = second.parse().ok()?;
            let year = third.parse().ok()?;
            Some((year, month, day))
        }
    }
}

fn is_valid_calendar_date(year: i32, month: u32, day: u32) -> bool {
    if !(1..=12).contains(&month) {
        return false;
    }

    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => unreachable!(),
    };

    day != 0 && day <= max_day
}

string_max_type!(TextMax60, 60);
string_max_type!(TextMax500, 500);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Year4(String);

impl Year4 {
    fn validate(value: &str) -> Result<(), ValidationError> {
        if value.len() != 4 {
            return Err(ValidationError::new(
                "Year4",
                "year must contain exactly four digits",
            ));
        }

        if !value.chars().all(|c| c.is_ascii_digit()) {
            return Err(ValidationError::new(
                "Year4",
                "year must contain digits only",
            ));
        }

        Ok(())
    }
}

impl_string_type!(Year4);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdOperacionesTrascendenciaTributaria(String);

impl IdOperacionesTrascendenciaTributaria {
    fn validate(value: &str) -> Result<(), ValidationError> {
        let len = value.chars().count();
        if len == 0 {
            return Err(ValidationError::new(
                "IdOperacionesTrascendenciaTributaria",
                "value cannot be empty",
            ));
        }

        if len > 4 {
            return Err(ValidationError::new(
                "IdOperacionesTrascendenciaTributaria",
                "value must contain at most 4 characters",
            ));
        }

        if !value
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            return Err(ValidationError::new(
                "IdOperacionesTrascendenciaTributaria",
                "value must be composed of uppercase letters and digits",
            ));
        }

        Ok(())
    }
}

impl_string_type!(IdOperacionesTrascendenciaTributaria);

// Custom NIF/NIE type that keeps the literal value as provided by AEAT.
// Official identifiers are 9 characters long, but the public examples use
// placeholders such as "AAAA", so we keep the payload flexible and let
// downstream validation enforce the length when required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NIF(String);

impl NIF {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn validate(value: &str) -> Result<(), ValidationError> {
        // NIF format is enforced server-side by AEAT, and the public examples
        // use placeholders such as "AAAA", so we only reject the empty string
        // here. Validation strictness is deliberately independent of the target
        // environment (test vs. production), which is now a runtime concern.
        if value.is_empty() {
            return Err(ValidationError::new("NIF", "cannot be empty"));
        }
        Ok(())
    }
}

impl std::fmt::Display for NIF {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for NIF {
    type Error = ValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::validate(value)?;
        Ok(Self(value.to_owned()))
    }
}

impl TryFrom<String> for NIF {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::validate(&value)?;
        Ok(Self(value))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Periodo {
    #[serde(rename = "01")]
    Enero,
    #[serde(rename = "02")]
    Febrero,
    #[serde(rename = "03")]
    Marzo,
    #[serde(rename = "04")]
    Abril,
    #[serde(rename = "05")]
    Mayo,
    #[serde(rename = "06")]
    Junio,
    #[serde(rename = "07")]
    Julio,
    #[serde(rename = "08")]
    Agosto,
    #[serde(rename = "09")]
    Septiembre,
    #[serde(rename = "10")]
    Octubre,
    #[serde(rename = "11")]
    Noviembre,
    #[serde(rename = "12")]
    Diciembre,
}

impl TryFrom<&str> for Periodo {
    type Error = ValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "01" => Periodo::Enero,
            "02" => Periodo::Febrero,
            "03" => Periodo::Marzo,
            "04" => Periodo::Abril,
            "05" => Periodo::Mayo,
            "06" => Periodo::Junio,
            "07" => Periodo::Julio,
            "08" => Periodo::Agosto,
            "09" => Periodo::Septiembre,
            "10" => Periodo::Octubre,
            "11" => Periodo::Noviembre,
            "12" => Periodo::Diciembre,
            other => {
                return Err(ValidationError::new(
                    "Periodo",
                    format!("expected a month code \"01\"..\"12\", got {other:?}"),
                ));
            }
        })
    }
}

impl TryFrom<String> for Periodo {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Periodo::try_from(value.as_str())
    }
}

/// Identifies a party by exactly one of a Spanish NIF or a foreign identifier
/// (`IDOtro`). Modeled as a sum type so that the "neither" and "both" states —
/// which the AEAT schema rejects — are unrepresentable.
#[derive(Debug, Clone)]
pub enum Identificador {
    Nif(NIF),
    IdOtro(IDOtro),
}

/// Builds an `Identificador` from the two optional wire fields, enforcing that
/// exactly one is present.
fn identificador_from_wire<E: serde::de::Error>(
    type_name: &str,
    nif: Option<NIF>,
    id_otro: Option<IDOtro>,
) -> Result<Identificador, E> {
    match (nif, id_otro) {
        (Some(nif), None) => Ok(Identificador::Nif(nif)),
        (None, Some(id_otro)) => Ok(Identificador::IdOtro(id_otro)),
        (Some(_), Some(_)) => Err(E::custom(format!(
            "{type_name} cannot have both NIF and IDOtro"
        ))),
        (None, None) => Err(E::custom(format!(
            "{type_name} must have either NIF or IDOtro"
        ))),
    }
}

#[derive(Debug, Clone)]
pub struct PersonaFisicaJuridicaES {
    pub nombre_razon: StringMax120,
    pub identificador: Identificador,
}

#[derive(Serialize, Deserialize)]
struct PersonaFisicaJuridicaESWire {
    #[serde(rename = "sum1:NombreRazon", alias = "NombreRazon")]
    nombre_razon: StringMax120,
    #[serde(
        rename = "sum1:NIF",
        alias = "NIF",
        skip_serializing_if = "Option::is_none"
    )]
    nif: Option<NIF>,
    #[serde(
        rename = "sum1:IDOtro",
        alias = "IDOtro",
        skip_serializing_if = "Option::is_none"
    )]
    id_otro: Option<IDOtro>,
}

impl Serialize for PersonaFisicaJuridicaES {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let (nif, id_otro) = match &self.identificador {
            Identificador::Nif(nif) => (Some(nif.clone()), None),
            Identificador::IdOtro(id_otro) => (None, Some(id_otro.clone())),
        };
        PersonaFisicaJuridicaESWire {
            nombre_razon: self.nombre_razon.clone(),
            nif,
            id_otro,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PersonaFisicaJuridicaES {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = PersonaFisicaJuridicaESWire::deserialize(deserializer)?;
        Ok(Self {
            nombre_razon: wire.nombre_razon,
            identificador: identificador_from_wire(
                "PersonaFisicaJuridicaES",
                wire.nif,
                wire.id_otro,
            )?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PersonaFisicaJuridicaConsulta {
    pub nombre_razon: StringMax120,
    pub identificador: Identificador,
}

#[derive(Serialize, Deserialize)]
struct PersonaFisicaJuridicaConsultaWire {
    #[serde(rename = "sum:NombreRazon", alias = "NombreRazon")]
    nombre_razon: StringMax120,
    #[serde(
        rename = "sum:NIF",
        alias = "NIF",
        skip_serializing_if = "Option::is_none"
    )]
    nif: Option<NIF>,
    #[serde(
        rename = "sum:IDOtro",
        alias = "IDOtro",
        skip_serializing_if = "Option::is_none"
    )]
    id_otro: Option<IDOtro>,
}

impl Serialize for PersonaFisicaJuridicaConsulta {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let (nif, id_otro) = match &self.identificador {
            Identificador::Nif(nif) => (Some(nif.clone()), None),
            Identificador::IdOtro(id_otro) => (None, Some(id_otro.clone())),
        };
        PersonaFisicaJuridicaConsultaWire {
            nombre_razon: self.nombre_razon.clone(),
            nif,
            id_otro,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PersonaFisicaJuridicaConsulta {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = PersonaFisicaJuridicaConsultaWire::deserialize(deserializer)?;
        Ok(Self {
            nombre_razon: wire.nombre_razon,
            identificador: identificador_from_wire(
                "PersonaFisicaJuridicaConsulta",
                wire.nif,
                wire.id_otro,
            )?,
        })
    }
}

impl From<PersonaFisicaJuridicaES> for PersonaFisicaJuridicaConsulta {
    fn from(value: PersonaFisicaJuridicaES) -> Self {
        Self {
            nombre_razon: value.nombre_razon,
            identificador: value.identificador,
        }
    }
}

impl From<PersonaFisicaJuridicaConsulta> for PersonaFisicaJuridicaES {
    fn from(value: PersonaFisicaJuridicaConsulta) -> Self {
        Self {
            nombre_razon: value.nombre_razon,
            identificador: value.identificador,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Incidencia {
    S,
    N,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinRequerimiento {
    S,
    N,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MostrarSistemaInformatico {
    S,
    N,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MostrarNombreRazonEmisor {
    S,
    N,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndicadorRepresentante {
    S,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeneradoPor {
    E,
    D,
    T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonaFisicaJuridicaID {
    #[serde(rename = "02")]
    NifIva,
    #[serde(rename = "03")]
    Pasaporte,
    #[serde(rename = "04")]
    IDEnPaisResidencia,
    #[serde(rename = "05")]
    CertificadoResidencia,
    #[serde(rename = "06")]
    OtroDocumentoProbatorio,
    #[serde(rename = "07")]
    NoCensado,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TipoHuella {
    #[serde(rename = "01")]
    Sha256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cabecera {
    #[serde(rename = "sum1:ObligadoEmision", alias = "ObligadoEmision")]
    pub obligado_emision: PersonaFisicaJuridicaES,
    #[serde(
        rename = "sum1:Representante",
        alias = "Representante",
        skip_serializing_if = "Option::is_none"
    )]
    pub representante: Option<PersonaFisicaJuridicaES>,
    #[serde(
        rename = "sum1:RemisionVoluntaria",
        alias = "RemisionVoluntaria",
        skip_serializing_if = "Option::is_none"
    )]
    pub remision_voluntaria: Option<RemisionVoluntaria>,
    #[serde(
        rename = "sum1:RemisionRequerimiento",
        alias = "RemisionRequerimiento",
        skip_serializing_if = "Option::is_none"
    )]
    pub remision_requerimiento: Option<RemisionRequerimiento>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemisionVoluntaria {
    // Both children are minOccurs="0" in the AEAT XSD. FechaFinVeriFactu is only
    // populated on the final submission, when the taxpayer ceases VeriFactu use;
    // for ordinary continuous submission it is omitted and only Incidencia is set.
    #[serde(
        rename = "sum1:FechaFinVeriFactu",
        alias = "FechaFinVeriFactu",
        skip_serializing_if = "Option::is_none"
    )]
    pub fecha_fin_veri_factu: Option<Fecha>,
    #[serde(rename = "sum1:Incidencia", alias = "Incidencia")]
    pub incidencia: Incidencia,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemisionRequerimiento {
    #[serde(rename = "sum1:RefRequerimiento", alias = "RefRequerimiento")]
    pub ref_requerimiento: StringMax18,
    #[serde(
        rename = "sum1:FinRequerimiento",
        alias = "FinRequerimiento",
        skip_serializing_if = "Option::is_none"
    )]
    pub fin_requerimiento: Option<FinRequerimiento>,
}

// Main information submission structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "sum:RegFactuSistemaFacturacion")]
pub struct SuministroInformacion {
    #[serde(rename = "sum:Cabecera", alias = "Cabecera")]
    pub cabecera: Cabecera,
    /// Maximum 1000 entries as per XSD specification (maxOccurs=1000)
    #[serde(
        rename = "sum:RegistroFactura",
        alias = "RegistroFactura",
        deserialize_with = "deserialize_registro_factura",
        serialize_with = "serialize_registro_factura"
    )]
    pub registro_factura: Vec<RegistroFactura>,
}

impl SuministroInformacion {
    /// Creates a new SuministroInformacion with validation of XSD constraints
    pub fn new(
        cabecera: Cabecera,
        registro_factura: Vec<RegistroFactura>,
    ) -> Result<Self, ValidationError> {
        if registro_factura.is_empty() {
            return Err(ValidationError::new(
                "SuministroInformacion",
                "must contain at least one registro_factura",
            ));
        }
        if registro_factura.len() > 1000 {
            return Err(ValidationError::new(
                "SuministroInformacion",
                format!(
                    "cannot exceed 1000 registro_factura entries, got {}",
                    registro_factura.len()
                ),
            ));
        }
        Ok(Self {
            cabecera,
            registro_factura,
        })
    }
}

pub trait IntoSoapXml {
    fn to_xml(&self) -> String;
    fn soap_envelope_namespaces(&self) -> &'static str;
}

impl IntoSoapXml for SuministroInformacion {
    fn to_xml(&self) -> String {
        to_string(&self).expect("valid xml").trim().to_string()
    }

    fn soap_envelope_namespaces(&self) -> &'static str {
        r#"xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:sum="https://www2.agenciatributaria.gob.es/static_files/common/internet/dep/aplicaciones/es/aeat/tike/cont/ws/SuministroLR.xsd" xmlns:sum1="https://www2.agenciatributaria.gob.es/static_files/common/internet/dep/aplicaciones/es/aeat/tike/cont/ws/SuministroInformacion.xsd""#
    }
}

fn deserialize_registro_factura<'de, D>(deserializer: D) -> Result<Vec<RegistroFactura>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct RegistroWrapper {
        #[serde(
            rename = "sum1:RegistroAlta",
            alias = "RegistroAlta",
            skip_serializing_if = "Option::is_none"
        )]
        alta: Option<RegistroFacturacionAlta>,
        #[serde(
            rename = "sum1:RegistroAnulacion",
            alias = "RegistroAnulacion",
            skip_serializing_if = "Option::is_none"
        )]
        anulacion: Option<RegistroFacturacionAnulacion>,
    }

    let wrappers = Vec::<RegistroWrapper>::deserialize(deserializer)?;
    wrappers
        .into_iter()
        .map(|wrapper| {
            if let Some(alta) = wrapper.alta {
                Ok(RegistroFactura::Alta(alta))
            } else if let Some(anulacion) = wrapper.anulacion {
                Ok(RegistroFactura::Anulacion(anulacion))
            } else {
                Err(serde::de::Error::custom(
                    "RegistroFactura element missing RegistroAlta or RegistroAnulacion",
                ))
            }
        })
        .collect()
}

fn serialize_registro_factura<S>(
    registros: &Vec<RegistroFactura>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;

    #[derive(Serialize)]
    struct RegistroWrapper {
        #[serde(
            rename = "sum1:RegistroAlta",
            alias = "RegistroAlta",
            skip_serializing_if = "Option::is_none"
        )]
        alta: Option<RegistroFacturacionAlta>,
        #[serde(
            rename = "sum1:RegistroAnulacion",
            alias = "RegistroAnulacion",
            skip_serializing_if = "Option::is_none"
        )]
        anulacion: Option<RegistroFacturacionAnulacion>,
    }

    let mut seq = serializer.serialize_seq(Some(registros.len()))?;
    for registro in registros {
        let wrapper = match registro {
            RegistroFactura::Alta(alta) => RegistroWrapper {
                alta: Some(alta.clone()),
                anulacion: None,
            },
            RegistroFactura::Anulacion(anulacion) => RegistroWrapper {
                alta: None,
                anulacion: Some(anulacion.clone()),
            },
        };
        seq.serialize_element(&wrapper)?;
    }
    seq.end()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistroFactura {
    #[serde(rename = "sum1:RegistroAlta", alias = "RegistroAlta")]
    Alta(RegistroFacturacionAlta),
    #[serde(rename = "sum1:RegistroAnulacion", alias = "RegistroAnulacion")]
    Anulacion(RegistroFacturacionAnulacion),
}

impl RegistroFactura {
    /// Computes and sets the hash (huella) for this invoice record.
    ///
    /// # Arguments
    /// * `prev_huella` - Optional hash from the previous record in the chain
    pub fn compute_hash(&mut self, prev_huella: Option<&str>) {
        use crate::hashing::Hashable;
        let hash = self.hash(prev_huella);
        match self {
            Self::Alta(registro) => {
                registro.huella = hash
                    .as_str()
                    .try_into()
                    .expect("SHA256 hash should always fit in StringMax64");
            }
            Self::Anulacion(registro) => {
                registro.huella = hash
                    .as_str()
                    .try_into()
                    .expect("SHA256 hash should always fit in StringMax64");
            }
        }
    }
}

// Invoice book submission (libro registro)
// Note: The XSD may use different naming - this follows the pattern from the existing stubs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuministroLR {
    #[serde(rename = "sum:Cabecera", alias = "Cabecera")]
    pub cabecera: Cabecera,
    #[serde(rename = "sum:Registros", alias = "Registros")]
    pub registros: Vec<RegistroFactura>,
}

// Invoice book query (consulta libro registro)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "con:ConsultaFactuSistemaFacturacion")]
pub struct ConsultaFactuSistemaFacturacion {
    #[serde(rename = "con:Cabecera", alias = "Cabecera")]
    pub cabecera: CabeceraConsulta,
    #[serde(rename = "con:FiltroConsulta", alias = "FiltroConsulta")]
    pub filtro_consulta: FiltroConsulta,
    #[serde(
        rename = "con:DatosAdicionalesRespuesta",
        alias = "DatosAdicionalesRespuesta",
        skip_serializing_if = "Option::is_none"
    )]
    pub datos_adicionales_respuesta: Option<DatosAdicionalesRespuesta>,
}

pub type ConsultaLR = ConsultaFactuSistemaFacturacion;

impl IntoSoapXml for ConsultaFactuSistemaFacturacion {
    fn to_xml(&self) -> String {
        to_string(&self).expect("valid xml").trim().to_string()
    }

    fn soap_envelope_namespaces(&self) -> &'static str {
        r#"xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:con="https://www2.agenciatributaria.gob.es/static_files/common/internet/dep/aplicaciones/es/aeat/tike/cont/ws/ConsultaLR.xsd" xmlns:sum="https://www2.agenciatributaria.gob.es/static_files/common/internet/dep/aplicaciones/es/aeat/tike/cont/ws/SuministroInformacion.xsd""#
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CabeceraConsulta {
    #[serde(rename = "sum:IDVersion", alias = "IDVersion")]
    pub id_version: StringMax16,
    #[serde(
        rename = "sum:ObligadoEmision",
        alias = "ObligadoEmision",
        skip_serializing_if = "Option::is_none"
    )]
    pub obligado_emision: Option<PersonaFisicaJuridicaConsulta>,
    #[serde(
        rename = "sum:Destinatario",
        alias = "Destinatario",
        skip_serializing_if = "Option::is_none"
    )]
    pub destinatario: Option<PersonaFisicaJuridicaConsulta>,
    #[serde(
        rename = "sum:IndicadorRepresentante",
        alias = "IndicadorRepresentante",
        skip_serializing_if = "Option::is_none"
    )]
    pub indicador_representante: Option<IndicadorRepresentante>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatosAdicionalesRespuesta {
    #[serde(
        rename = "con:MostrarNombreRazonEmisor",
        alias = "MostrarNombreRazonEmisor",
        skip_serializing_if = "Option::is_none"
    )]
    pub mostrar_nombre_razon_emisor: Option<MostrarNombreRazonEmisor>,
    #[serde(
        rename = "con:MostrarSistemaInformatico",
        alias = "MostrarSistemaInformatico",
        skip_serializing_if = "Option::is_none"
    )]
    pub mostrar_sistema_informatico: Option<MostrarSistemaInformatico>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiltroConsulta {
    #[serde(rename = "con:PeriodoImputacion", alias = "PeriodoImputacion")]
    pub periodo_imputacion: PeriodoImputacion,
    #[serde(
        rename = "con:NumSerieFactura",
        alias = "NumSerieFactura",
        skip_serializing_if = "Option::is_none"
    )]
    pub num_serie_factura: Option<TextMax60>,
    #[serde(
        rename = "con:Contraparte",
        alias = "Contraparte",
        skip_serializing_if = "Option::is_none"
    )]
    pub contraparte: Option<PersonaFisicaJuridicaConsulta>,
    #[serde(
        rename = "con:FechaExpedicionFactura",
        alias = "FechaExpedicionFactura",
        skip_serializing_if = "Option::is_none"
    )]
    pub fecha_expedicion_factura: Option<FechaExpedicionFiltro>,
    #[serde(
        rename = "con:SistemaInformatico",
        alias = "SistemaInformatico",
        skip_serializing_if = "Option::is_none"
    )]
    pub sistema_informatico: Option<SistemaInformaticoConsulta>,
    #[serde(
        rename = "con:RefExterna",
        alias = "RefExterna",
        skip_serializing_if = "Option::is_none"
    )]
    pub ref_externa: Option<TextMax60>,
    #[serde(
        rename = "con:ClavePaginacion",
        alias = "ClavePaginacion",
        skip_serializing_if = "Option::is_none"
    )]
    pub clave_paginacion: Option<ClavePaginacion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodoImputacion {
    #[serde(rename = "sum:Ejercicio", alias = "Ejercicio")]
    pub ejercicio: Year4,
    #[serde(rename = "sum:Periodo", alias = "Periodo")]
    pub periodo: Periodo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FechaExpedicionFiltro {
    #[serde(
        rename = "sum:RangoFechaExpedicion",
        alias = "RangoFechaExpedicion",
        skip_serializing_if = "Option::is_none"
    )]
    pub rango_fecha_expedicion: Option<RangoFechaExpedicion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangoFechaExpedicion {
    #[serde(rename = "sum:Desde", alias = "Desde")]
    pub desde: Fecha,
    #[serde(rename = "sum:Hasta", alias = "Hasta")]
    pub hasta: Fecha,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClavePaginacion {
    #[serde(
        rename = "sum:IDEmisorFactura",
        alias = "IDEmisorFactura",
        skip_serializing_if = "Option::is_none"
    )]
    pub id_emisor_factura: Option<NIF>,
    #[serde(
        rename = "sum:NumSerieFactura",
        alias = "NumSerieFactura",
        skip_serializing_if = "Option::is_none"
    )]
    pub num_serie_factura: Option<TextMax60>,
    #[serde(
        rename = "sum:FechaExpedicionFactura",
        alias = "FechaExpedicionFactura",
        skip_serializing_if = "Option::is_none"
    )]
    pub fecha_expedicion_factura: Option<Fecha>,
}

// ISO 3166-1 alpha-2 country codes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CountryType {
    AF,
    AL,
    DE,
    AD,
    AO,
    AI,
    AQ,
    AG,
    SA,
    DZ,
    AR,
    AM,
    AW,
    AU,
    AT,
    AZ,
    BS,
    BD,
    BB,
    BH,
    BE,
    BZ,
    BJ,
    BM,
    BY,
    BO,
    BA,
    BW,
    BR,
    BN,
    BG,
    BF,
    BI,
    BT,
    CV,
    KH,
    CM,
    CA,
    QA,
    TD,
    CL,
    CN,
    CY,
    CO,
    KM,
    CG,
    CD,
    KP,
    KR,
    CI,
    CR,
    HR,
    CU,
    CW,
    DK,
    DM,
    EC,
    EG,
    SV,
    AE,
    ER,
    SK,
    SI,
    ES,
    US,
    EE,
    ET,
    PH,
    FI,
    FJ,
    FR,
    GA,
    GM,
    GE,
    GH,
    GI,
    GD,
    GR,
    GL,
    GP,
    GU,
    GT,
    GF,
    GN,
    GW,
    GQ,
    GY,
    HT,
    HN,
    HK,
    HU,
    IN,
    ID,
    IQ,
    IR,
    IE,
    BV,
    CX,
    IS,
    KY,
    CC,
    CK,
    FO,
    GS,
    HM,
    FK,
    MP,
    MH,
    UM,
    PN,
    SB,
    TC,
    VG,
    VI,
    WF,
    AX,
    IL,
    IT,
    JM,
    JP,
    JE,
    JO,
    KZ,
    KE,
    KG,
    KI,
    KW,
    LA,
    LS,
    LV,
    LB,
    LR,
    LY,
    LI,
    LT,
    LU,
    MO,
    MK,
    MG,
    MY,
    MW,
    MV,
    ML,
    MT,
    MA,
    MQ,
    MU,
    MR,
    YT,
    MX,
    FM,
    MD,
    MC,
    MN,
    ME,
    MS,
    MZ,
    MM,
    NA,
    NR,
    NP,
    NI,
    NE,
    NG,
    NU,
    NF,
    NO,
    NC,
    NZ,
    OM,
    NL,
    PK,
    PW,
    PA,
    PG,
    PY,
    PE,
    PF,
    PL,
    PT,
    PR,
    GB,
    CF,
    CZ,
    RE,
    RO,
    RW,
    RU,
    EH,
    WS,
    AS,
    BL,
    KN,
    SM,
    MF,
    PM,
    VC,
    SH,
    LC,
    ST,
    SN,
    RS,
    SC,
    SL,
    SG,
    SX,
    SY,
    SO,
    LK,
    SZ,
    ZA,
    SD,
    SS,
    SE,
    CH,
    SR,
    SJ,
    TH,
    TW,
    TZ,
    TJ,
    IO,
    TF,
    PS,
    TL,
    TG,
    TK,
    TO,
    TT,
    TN,
    TM,
    TR,
    TV,
    UA,
    UG,
    UY,
    UZ,
    VU,
    VA,
    VE,
    VN,
    YE,
    DJ,
    ZM,
    ZW,
}

// Estado for submission responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EstadoRegistroSuministro {
    Correcto,
    AceptadoConErrores,
    Incorrecto,
}

// Estado for query responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EstadoRegistroConsulta {
    Correcto,
    AceptadoConErrores,
    Anulado,
}

pub type ErrorDetalle = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CalificacionOperacion {
    S1,
    S2,
    N1,
    N2,
}

// Single unified Detalle structure representing DetalleDesgloseType from XSD
// All fields are optional except BaseImponibleOimporteNoSujeto which is required
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detalle {
    #[serde(
        rename = "sum1:Impuesto",
        alias = "Impuesto",
        skip_serializing_if = "Option::is_none"
    )]
    pub impuesto: Option<Impuesto>,
    #[serde(
        rename = "sum1:ClaveRegimen",
        alias = "ClaveRegimen",
        skip_serializing_if = "Option::is_none"
    )]
    pub clave_regimen: Option<IdOperacionesTrascendenciaTributaria>,
    #[serde(
        rename = "sum1:CalificacionOperacion",
        alias = "CalificacionOperacion",
        skip_serializing_if = "Option::is_none"
    )]
    pub calificacion_operacion: Option<CalificacionOperacion>,
    #[serde(
        rename = "sum1:OperacionExenta",
        alias = "OperacionExenta",
        skip_serializing_if = "Option::is_none"
    )]
    pub operacion_exenta: Option<OperacionExenta>,
    #[serde(
        rename = "sum1:TipoImpositivo",
        alias = "TipoImpositivo",
        skip_serializing_if = "Option::is_none"
    )]
    pub tipo_impositivo: Option<Tipo2_2>,
    #[serde(
        rename = "sum1:BaseImponibleOimporteNoSujeto",
        alias = "BaseImponibleOimporteNoSujeto"
    )]
    pub base_imponible_o_importe_no_sujeto: ImporteSgn12_2,
    #[serde(
        rename = "sum1:BaseImponibleACoste",
        alias = "BaseImponibleACoste",
        skip_serializing_if = "Option::is_none"
    )]
    pub base_imponible_a_coste: Option<ImporteSgn12_2>,
    #[serde(
        rename = "sum1:CuotaRepercutida",
        alias = "CuotaRepercutida",
        skip_serializing_if = "Option::is_none"
    )]
    pub cuota_repercutida: Option<ImporteSgn12_2>,
    #[serde(
        rename = "sum1:TipoRecargoEquivalencia",
        alias = "TipoRecargoEquivalencia",
        skip_serializing_if = "Option::is_none"
    )]
    pub tipo_recargo_equivalencia: Option<Tipo2_2>,
    #[serde(
        rename = "sum1:CuotaRecargoEquivalencia",
        alias = "CuotaRecargoEquivalencia",
        skip_serializing_if = "Option::is_none"
    )]
    pub cuota_recargo_equivalencia: Option<ImporteSgn12_2>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperacionExenta {
    E1, // Operación interior (Art. 20 LIVA)
    E2, // Exportación no comunitaria (Art. 21 LIVA)
    E3, // Asimiladas a exportaciones (Art. 22 LIVA)
    E4, // Régimen aduanero (Art. 23 y 24 LIVA)
    E5, // Exportación intracomunitaria (Art. 25 LIVA)
    E6, // Otras exenciones
    E7, // Exenta por el Art. 110 Ley 4/2012 (Régimen Especial de Pequeños Empresarios o Profesionales)
    E8, // Exenta otros (casos no cubiertos por otras causas de exención)
}

impl Default for Detalle {
    fn default() -> Self {
        Self {
            impuesto: None,
            clave_regimen: None,
            calificacion_operacion: None,
            operacion_exenta: None,
            tipo_impositivo: None,
            base_imponible_o_importe_no_sujeto: "0".try_into().expect("valid base"),
            base_imponible_a_coste: None,
            cuota_repercutida: None,
            tipo_recargo_equivalencia: None,
            cuota_recargo_equivalencia: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Impuesto {
    #[serde(rename = "01")]
    IVA, // Impuesto sobre el Valor Añadido
    #[serde(rename = "02")]
    IPSI, // Impuesto sobre la Producción, los Servicios y la Importación (Ceuta/Melilla)
    #[serde(rename = "03")]
    IGIC, // Impuesto General Indirecto Canario (Canary Islands)
    #[serde(rename = "05")]
    Otros,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum TipoFactura {
    #[default]
    F1, // Standard invoice
    F2, // Simplified invoice
    R1, // Corrective invoice (error in taxable base)
    R2, // Corrective invoice (article 80.1, 80.2, 80.6 LIVA)
    R3, // Corrective invoice (article 80.3 LIVA)
    R4, // Corrective invoice (article 80.4 LIVA)
    R5, // Corrective invoice (insolvency proceedings)
    F3, // Replacement for simplified invoices
}

impl std::fmt::Display for TipoFactura {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TipoFactura::F1 => write!(f, "F1"),
            TipoFactura::F2 => write!(f, "F2"),
            TipoFactura::R1 => write!(f, "R1"),
            TipoFactura::R2 => write!(f, "R2"),
            TipoFactura::R3 => write!(f, "R3"),
            TipoFactura::R4 => write!(f, "R4"),
            TipoFactura::R5 => write!(f, "R5"),
            TipoFactura::F3 => write!(f, "F3"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TipoRectificativa {
    S, // Substitutive
    I, // Incremental
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SiNo {
    S,
    N,
    X,
}

// Additional supporting types
// Invoice identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDFactura {
    #[serde(rename = "sum1:IDEmisorFactura", alias = "IDEmisorFactura")]
    pub id_emisor_factura: NIF,
    #[serde(rename = "sum1:NumSerieFactura", alias = "NumSerieFactura")]
    pub num_serie_factura: TextMax60,
    #[serde(
        rename = "sum1:FechaExpedicionFactura",
        alias = "FechaExpedicionFactura"
    )]
    pub fecha_expedicion_factura: Fecha,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDFacturaConsulta {
    #[serde(rename = "sum:IDEmisorFactura", alias = "IDEmisorFactura")]
    pub id_emisor_factura: NIF,
    #[serde(rename = "sum:NumSerieFactura", alias = "NumSerieFactura")]
    pub num_serie_factura: TextMax60,
    #[serde(
        rename = "sum:FechaExpedicionFactura",
        alias = "FechaExpedicionFactura"
    )]
    pub fecha_expedicion_factura: Fecha,
}

impl From<IDFactura> for IDFacturaConsulta {
    fn from(value: IDFactura) -> Self {
        Self {
            id_emisor_factura: value.id_emisor_factura,
            num_serie_factura: value.num_serie_factura,
            fecha_expedicion_factura: value.fecha_expedicion_factura,
        }
    }
}

impl From<IDFacturaConsulta> for IDFactura {
    fn from(value: IDFacturaConsulta) -> Self {
        Self {
            id_emisor_factura: value.id_emisor_factura,
            num_serie_factura: value.num_serie_factura,
            fecha_expedicion_factura: value.fecha_expedicion_factura,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDFacturaAnulada {
    #[serde(
        rename = "sum1:IDEmisorFacturaAnulada",
        alias = "IDEmisorFacturaAnulada"
    )]
    pub id_emisor_factura_anulada: NIF,
    #[serde(
        rename = "sum1:NumSerieFacturaAnulada",
        alias = "NumSerieFacturaAnulada"
    )]
    pub num_serie_factura_anulada: TextMax60,
    #[serde(
        rename = "sum1:FechaExpedicionFacturaAnulada",
        alias = "FechaExpedicionFacturaAnulada"
    )]
    pub fecha_expedicion_factura_anulada: Fecha,
}

// Chaining/blockchain structure for invoice integrity.
//
// A record is either the first in the chain (`PrimerRegistro`) or links to the
// previous record (`RegistroAnterior`); exactly one applies. Modeled as a sum
// type so the "neither" and "both" states the AEAT schema rejects cannot be
// constructed.
#[derive(Debug, Clone)]
pub enum Encadenamiento {
    /// This record is the first of the chain. Per the AEAT XSD
    /// (`PrimerRegistroCadenaType`) the element value is fixed to "S", so the
    /// variant carries no data — `N` is not a representable (or valid) state.
    PrimerRegistro,
    RegistroAnterior(RegistroAnterior),
}

/// The `PrimerRegistro` element value. The AEAT XSD restricts
/// `PrimerRegistroCadenaType` to the single enumeration value "S", so this type
/// holds no data and always (de)serializes as the string "S".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PrimerRegistroMarker;

impl Serialize for PrimerRegistroMarker {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str("S")
    }
}

impl<'de> Deserialize<'de> for PrimerRegistroMarker {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        if value == "S" {
            Ok(PrimerRegistroMarker)
        } else {
            Err(D::Error::custom(format!(
                "PrimerRegistro must be 'S', got '{value}'"
            )))
        }
    }
}

#[derive(Serialize, Deserialize)]
struct EncadenamientoWire {
    #[serde(
        rename = "sum1:PrimerRegistro",
        alias = "PrimerRegistro",
        skip_serializing_if = "Option::is_none"
    )]
    primer_registro: Option<PrimerRegistroMarker>,
    #[serde(
        rename = "sum1:RegistroAnterior",
        alias = "RegistroAnterior",
        skip_serializing_if = "Option::is_none"
    )]
    registro_anterior: Option<RegistroAnterior>,
}

impl Serialize for Encadenamiento {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let (primer_registro, registro_anterior) = match self {
            Encadenamiento::PrimerRegistro => (Some(PrimerRegistroMarker), None),
            Encadenamiento::RegistroAnterior(registro) => (None, Some(registro.clone())),
        };
        EncadenamientoWire {
            primer_registro,
            registro_anterior,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Encadenamiento {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = EncadenamientoWire::deserialize(deserializer)?;
        match (wire.primer_registro, wire.registro_anterior) {
            (Some(_), None) => Ok(Encadenamiento::PrimerRegistro),
            (None, Some(registro)) => Ok(Encadenamiento::RegistroAnterior(registro)),
            (Some(_), Some(_)) => Err(D::Error::custom(
                "Encadenamiento cannot have both PrimerRegistro and RegistroAnterior",
            )),
            (None, None) => Err(D::Error::custom(
                "Encadenamiento must have either PrimerRegistro or RegistroAnterior",
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistroAnterior {
    #[serde(rename = "sum1:IDEmisorFactura", alias = "IDEmisorFactura")]
    pub id_emisor_factura: NIF,
    #[serde(rename = "sum1:NumSerieFactura", alias = "NumSerieFactura")]
    pub num_serie_factura: TextMax60,
    #[serde(
        rename = "sum1:FechaExpedicionFactura",
        alias = "FechaExpedicionFactura"
    )]
    pub fecha_expedicion_factura: Fecha,
    #[serde(rename = "sum1:Huella", alias = "Huella")]
    pub huella: StringMax64,
}

// Software system information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SistemaInformatico {
    #[serde(rename = "sum1:NombreRazon", alias = "NombreRazon")]
    pub nombre_razon: StringMax120,
    #[serde(rename = "sum1:NIF", alias = "NIF")]
    pub nif: NIF,
    #[serde(
        rename = "sum1:NombreSistemaInformatico",
        alias = "NombreSistemaInformatico"
    )]
    pub nombre_sistema_informatico: StringMax30,
    #[serde(rename = "sum1:IdSistemaInformatico", alias = "IdSistemaInformatico")]
    pub id_sistema_informatico: StringMax2,
    #[serde(rename = "sum1:Version", alias = "Version")]
    pub version: StringMax50,
    #[serde(rename = "sum1:NumeroInstalacion", alias = "NumeroInstalacion")]
    pub numero_instalacion: StringMax100,
    #[serde(
        rename = "sum1:TipoUsoPosibleSoloVerifactu",
        alias = "TipoUsoPosibleSoloVerifactu"
    )]
    pub tipo_uso_posible_solo_verifactu: SiNo,
    #[serde(rename = "sum1:TipoUsoPosibleMultiOT", alias = "TipoUsoPosibleMultiOT")]
    pub tipo_uso_posible_multi_ot: SiNo,
    #[serde(rename = "sum1:IndicadorMultiplesOT", alias = "IndicadorMultiplesOT")]
    pub indicador_multiples_ot: SiNo,
}

// Software system information for queries (simplified structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SistemaInformaticoConsulta {
    #[serde(
        rename = "sum:NombreSistemaInformatico",
        alias = "NombreSistemaInformatico",
        skip_serializing_if = "Option::is_none"
    )]
    pub nombre_sistema_informatico: Option<StringMax30>,
    #[serde(
        rename = "sum:IdSistemaInformatico",
        alias = "IdSistemaInformatico",
        skip_serializing_if = "Option::is_none"
    )]
    pub id_sistema_informatico: Option<StringMax2>,
}

// Tax breakdown structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Desglose {
    /// Maximum 12 entries as per XSD specification (maxOccurs=12)
    #[serde(rename = "sum1:DetalleDesglose", alias = "DetalleDesglose")]
    detalle_desglose: Vec<Detalle>,
}

impl Desglose {
    /// Returns the tax breakdown detail lines.
    pub fn detalle_desglose(&self) -> &[Detalle] {
        &self.detalle_desglose
    }

    /// Creates a new Desglose with validation of XSD constraints
    pub fn new(detalle_desglose: Vec<Detalle>) -> Result<Self, ValidationError> {
        if detalle_desglose.is_empty() {
            return Err(ValidationError::new(
                "Desglose",
                "must contain at least one detalle_desglose",
            ));
        }
        if detalle_desglose.len() > 12 {
            return Err(ValidationError::new(
                "Desglose",
                format!(
                    "cannot exceed 12 detalle_desglose entries, got {}",
                    detalle_desglose.len()
                ),
            ));
        }
        Ok(Self { detalle_desglose })
    }
}

// Destinatarios (recipient) information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Destinatarios {
    /// Maximum 1000 entries as per XSD specification (maxOccurs=1000)
    #[serde(rename = "sum1:IDDestinatario", alias = "IDDestinatario")]
    pub destinatarios: Vec<Destinatario>,
}

impl Destinatarios {
    /// Creates a new Destinatarios with validation of XSD constraints
    pub fn new(destinatarios: Vec<Destinatario>) -> Result<Self, ValidationError> {
        if destinatarios.is_empty() {
            return Err(ValidationError::new(
                "Destinatarios",
                "must contain at least one destinatario",
            ));
        }
        if destinatarios.len() > 1000 {
            return Err(ValidationError::new(
                "Destinatarios",
                format!(
                    "cannot exceed 1000 destinatarios entries, got {}",
                    destinatarios.len()
                ),
            ));
        }
        Ok(Self { destinatarios })
    }
}

#[derive(Debug, Clone)]
pub struct Destinatario {
    pub nombre_razon: StringMax120,
    pub identificador: Identificador,
}

#[derive(Serialize, Deserialize)]
struct DestinatarioWire {
    #[serde(rename = "sum1:NombreRazon", alias = "NombreRazon")]
    nombre_razon: StringMax120,
    #[serde(
        rename = "sum1:NIF",
        alias = "NIF",
        skip_serializing_if = "Option::is_none"
    )]
    nif: Option<NIF>,
    #[serde(
        rename = "sum1:IDOtro",
        alias = "IDOtro",
        skip_serializing_if = "Option::is_none"
    )]
    id_otro: Option<IDOtro>,
}

impl Serialize for Destinatario {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let (nif, id_otro) = match &self.identificador {
            Identificador::Nif(nif) => (Some(nif.clone()), None),
            Identificador::IdOtro(id_otro) => (None, Some(id_otro.clone())),
        };
        DestinatarioWire {
            nombre_razon: self.nombre_razon.clone(),
            nif,
            id_otro,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Destinatario {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = DestinatarioWire::deserialize(deserializer)?;
        Ok(Self {
            nombre_razon: wire.nombre_razon,
            identificador: identificador_from_wire("Destinatario", wire.nif, wire.id_otro)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDOtro {
    #[serde(rename = "sum1:CodigoPais", alias = "CodigoPais")]
    pub codigo_pais: CountryType,
    #[serde(rename = "sum1:IDType", alias = "IDType")]
    pub id_type: PersonaFisicaJuridicaID,
    #[serde(rename = "sum1:ID", alias = "ID")]
    pub id: StringMax20,
}

// Import rectification details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImporteRectificacion {
    #[serde(rename = "sum1:BaseRectificada", alias = "BaseRectificada")]
    pub base_rectificada: ImporteSgn12_2,
    #[serde(rename = "sum1:CuotaRectificada", alias = "CuotaRectificada")]
    pub cuota_rectificada: ImporteSgn12_2,
    #[serde(
        rename = "sum1:CuotaRecargoRectificado",
        alias = "CuotaRecargoRectificado",
        skip_serializing_if = "Option::is_none"
    )]
    pub cuota_recargo_rectificado: Option<ImporteSgn12_2>,
}

/// Serde adapter for the submission `FacturasRectificadas` element. The AEAT
/// XSD (`RegistroFacturacionAltaType`) nests each reference in an
/// `<IDFacturaRectificada>` element inside `<FacturasRectificadas>`. A bare
/// `Vec<IDFactura>` under the `sum1:FacturasRectificadas` field name would
/// instead serialize as repeated `<FacturasRectificadas>` elements with the
/// `IDFactura` fields inlined, which AEAT rejects with error 4102. This adapter
/// re-introduces the required `<IDFacturaRectificada>` wrapper while keeping the
/// ergonomic `Option<Vec<IDFactura>>` field type.
mod facturas_rectificadas_serde {
    use super::IDFactura;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize)]
    struct WrapperRef<'a> {
        #[serde(rename = "sum1:IDFacturaRectificada")]
        id_factura_rectificada: &'a [IDFactura],
    }

    #[derive(Deserialize)]
    struct Wrapper {
        #[serde(
            rename = "sum1:IDFacturaRectificada",
            alias = "IDFacturaRectificada",
            default
        )]
        id_factura_rectificada: Vec<IDFactura>,
    }

    pub(super) fn serialize<S: Serializer>(
        value: &Option<Vec<IDFactura>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(facturas) => WrapperRef {
                id_factura_rectificada: facturas,
            }
            .serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Vec<IDFactura>>, D::Error> {
        let wrapper = Option::<Wrapper>::deserialize(deserializer)?;
        Ok(wrapper.map(|w| w.id_factura_rectificada))
    }
}

/// Serde adapter for the submission `FacturasSustituidas` element — the
/// substituted-invoice counterpart of [`facturas_rectificadas_serde`], nesting
/// each reference in `<IDFacturaSustituida>` per the AEAT XSD.
mod facturas_sustituidas_serde {
    use super::IDFactura;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize)]
    struct WrapperRef<'a> {
        #[serde(rename = "sum1:IDFacturaSustituida")]
        id_factura_sustituida: &'a [IDFactura],
    }

    #[derive(Deserialize)]
    struct Wrapper {
        #[serde(
            rename = "sum1:IDFacturaSustituida",
            alias = "IDFacturaSustituida",
            default
        )]
        id_factura_sustituida: Vec<IDFactura>,
    }

    pub(super) fn serialize<S: Serializer>(
        value: &Option<Vec<IDFactura>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(facturas) => WrapperRef {
                id_factura_sustituida: facturas,
            }
            .serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Vec<IDFactura>>, D::Error> {
        let wrapper = Option::<Wrapper>::deserialize(deserializer)?;
        Ok(wrapper.map(|w| w.id_factura_sustituida))
    }
}

// Main invoice registration structure (alta = new registration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistroFacturacionAlta {
    #[serde(rename = "sum1:IDVersion", alias = "IDVersion")]
    pub id_version: StringMax16,
    #[serde(rename = "sum1:IDFactura", alias = "IDFactura")]
    pub id_factura: IDFactura,
    #[serde(
        rename = "sum1:RefExterna",
        alias = "RefExterna",
        skip_serializing_if = "Option::is_none"
    )]
    pub ref_externa: Option<TextMax60>,
    #[serde(rename = "sum1:NombreRazonEmisor", alias = "NombreRazonEmisor")]
    pub nombre_razon_emisor: StringMax120,
    #[serde(
        rename = "sum1:Subsanacion",
        alias = "Subsanacion",
        skip_serializing_if = "Option::is_none"
    )]
    pub subsanacion: Option<SiNo>,
    #[serde(
        rename = "sum1:RechazoPrevio",
        alias = "RechazoPrevio",
        skip_serializing_if = "Option::is_none"
    )]
    pub rechazo_previo: Option<SiNo>,
    #[serde(rename = "sum1:TipoFactura", alias = "TipoFactura")]
    pub tipo_factura: TipoFactura,
    #[serde(
        rename = "sum1:TipoRectificativa",
        alias = "TipoRectificativa",
        skip_serializing_if = "Option::is_none"
    )]
    pub tipo_rectificativa: Option<TipoRectificativa>,
    /// Maximum 1000 entries as per XSD specification (maxOccurs=1000).
    /// Each reference is nested in an `<IDFacturaRectificada>` element inside
    /// `<FacturasRectificadas>` per the AEAT XSD; see [`facturas_rectificadas_serde`].
    #[serde(
        rename = "sum1:FacturasRectificadas",
        alias = "FacturasRectificadas",
        default,
        skip_serializing_if = "Option::is_none",
        with = "facturas_rectificadas_serde"
    )]
    pub facturas_rectificadas: Option<Vec<IDFactura>>,
    /// Maximum 1000 entries as per XSD specification (maxOccurs=1000).
    /// Each reference is nested in an `<IDFacturaSustituida>` element inside
    /// `<FacturasSustituidas>` per the AEAT XSD; see [`facturas_sustituidas_serde`].
    #[serde(
        rename = "sum1:FacturasSustituidas",
        alias = "FacturasSustituidas",
        default,
        skip_serializing_if = "Option::is_none",
        with = "facturas_sustituidas_serde"
    )]
    pub facturas_sustituidas: Option<Vec<IDFactura>>,
    #[serde(
        rename = "sum1:ImporteRectificacion",
        alias = "ImporteRectificacion",
        skip_serializing_if = "Option::is_none"
    )]
    pub importe_rectificacion: Option<ImporteRectificacion>,
    #[serde(
        rename = "sum1:FechaOperacion",
        alias = "FechaOperacion",
        skip_serializing_if = "Option::is_none"
    )]
    pub fecha_operacion: Option<Fecha>,
    #[serde(rename = "sum1:DescripcionOperacion", alias = "DescripcionOperacion")]
    pub descripcion_operacion: TextMax500,
    #[serde(
        rename = "sum1:FacturaSimplificadaArt7273",
        alias = "FacturaSimplificadaArt7273",
        skip_serializing_if = "Option::is_none"
    )]
    pub factura_simplificada_art7273: Option<SiNo>,
    #[serde(
        rename = "sum1:FacturaSinIdentifDestinatarioArt61d",
        alias = "FacturaSinIdentifDestinatarioArt61d",
        skip_serializing_if = "Option::is_none"
    )]
    pub factura_sin_identif_destinatario_art61d: Option<SiNo>,
    #[serde(
        rename = "sum1:Macrodato",
        alias = "Macrodato",
        skip_serializing_if = "Option::is_none"
    )]
    pub macrodato: Option<SiNo>,
    #[serde(
        rename = "sum1:EmitidaPorTerceroODestinatario",
        alias = "EmitidaPorTerceroODestinatario",
        skip_serializing_if = "Option::is_none"
    )]
    pub emitida_por_tercero_o_destinatario: Option<GeneradoPor>,
    #[serde(
        rename = "sum1:Tercero",
        alias = "Tercero",
        skip_serializing_if = "Option::is_none"
    )]
    pub tercero: Option<PersonaFisicaJuridicaES>,
    #[serde(
        rename = "sum1:Destinatarios",
        alias = "Destinatarios",
        skip_serializing_if = "Option::is_none"
    )]
    pub destinatarios: Option<Destinatarios>,
    #[serde(
        rename = "sum1:Cupon",
        alias = "Cupon",
        skip_serializing_if = "Option::is_none"
    )]
    pub cupon: Option<SiNo>,
    #[serde(rename = "sum1:Desglose", alias = "Desglose")]
    pub desglose: Desglose,
    #[serde(rename = "sum1:CuotaTotal", alias = "CuotaTotal")]
    pub cuota_total: ImporteSgn12_2,
    #[serde(rename = "sum1:ImporteTotal", alias = "ImporteTotal")]
    pub importe_total: ImporteSgn14_2,
    #[serde(rename = "sum1:Encadenamiento", alias = "Encadenamiento")]
    pub encadenamiento: Encadenamiento,
    #[serde(rename = "sum1:SistemaInformatico", alias = "SistemaInformatico")]
    pub sistema_informatico: SistemaInformatico,
    #[serde(
        rename = "sum1:FechaHoraHusoGenRegistro",
        alias = "FechaHoraHusoGenRegistro"
    )]
    pub fecha_hora_huso_gen_registro: String, // DateTime with timezone
    #[serde(
        rename = "sum1:NumRegistroAcuerdoFacturacion",
        alias = "NumRegistroAcuerdoFacturacion",
        skip_serializing_if = "Option::is_none"
    )]
    pub num_registro_acuerdo_facturacion: Option<StringMax15>,
    #[serde(
        rename = "sum1:IdAcuerdoSistemaInformatico",
        alias = "IdAcuerdoSistemaInformatico",
        skip_serializing_if = "Option::is_none"
    )]
    pub id_acuerdo_sistema_informatico: Option<StringMax16>,
    #[serde(rename = "sum1:TipoHuella", alias = "TipoHuella")]
    pub tipo_huella: TipoHuella,
    #[serde(rename = "sum1:Huella", alias = "Huella")]
    pub huella: StringMax64,
}

impl RegistroFacturacionAlta {
    /// Validates XSD constraints for this record
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate IDVersion is "1.0"
        if self.id_version.as_str() != "1.0" {
            return Err(ValidationError::new(
                "RegistroFacturacionAlta",
                format!("IDVersion must be '1.0', got '{}'", self.id_version),
            ));
        }

        // Validate facturas_rectificadas length
        if let Some(ref facturas) = self.facturas_rectificadas
            && facturas.len() > 1000
        {
            return Err(ValidationError::new(
                "RegistroFacturacionAlta",
                format!(
                    "cannot exceed 1000 facturas_rectificadas, got {}",
                    facturas.len()
                ),
            ));
        }

        // Validate facturas_sustituidas length
        if let Some(ref facturas) = self.facturas_sustituidas
            && facturas.len() > 1000
        {
            return Err(ValidationError::new(
                "RegistroFacturacionAlta",
                format!(
                    "cannot exceed 1000 facturas_sustituidas, got {}",
                    facturas.len()
                ),
            ));
        }

        // Validate desglose has at least 1 and at most 12 entries
        if self.desglose.detalle_desglose.is_empty() {
            return Err(ValidationError::new(
                "RegistroFacturacionAlta",
                "desglose must contain at least one detalle",
            ));
        }
        if self.desglose.detalle_desglose.len() > 12 {
            return Err(ValidationError::new(
                "RegistroFacturacionAlta",
                format!(
                    "desglose cannot exceed 12 detalle entries, got {}",
                    self.desglose.detalle_desglose.len()
                ),
            ));
        }

        // Validate destinatarios if present
        if let Some(ref destinatarios) = self.destinatarios {
            if destinatarios.destinatarios.is_empty() {
                return Err(ValidationError::new(
                    "RegistroFacturacionAlta",
                    "destinatarios must contain at least one entry if present",
                ));
            }
            if destinatarios.destinatarios.len() > 1000 {
                return Err(ValidationError::new(
                    "RegistroFacturacionAlta",
                    format!(
                        "cannot exceed 1000 destinatarios, got {}",
                        destinatarios.destinatarios.len()
                    ),
                ));
            }
        }

        Ok(())
    }

    /// Computes and sets the hash (huella) for this invoice record.
    ///
    /// # Arguments
    /// * `prev_huella` - Optional hash from the previous record in the chain
    ///
    /// # Panics
    /// Panics if the computed hash cannot be converted to StringMax64 (should not happen as SHA256 produces 64 hex chars)
    pub fn compute_hash(&mut self, prev_huella: Option<&str>) {
        use crate::hashing::Hashable;
        let hash = self.hash(prev_huella);
        self.huella = hash
            .as_str()
            .try_into()
            .expect("SHA256 hash should always fit in StringMax64");
    }
}

// Invoice cancellation structure (anulacion = cancellation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistroFacturacionAnulacion {
    #[serde(rename = "sum1:IDVersion", alias = "IDVersion")]
    pub id_version: StringMax16,
    #[serde(rename = "sum1:IDFactura", alias = "IDFactura")]
    pub id_factura: IDFacturaAnulada,
    #[serde(
        rename = "sum1:RefExterna",
        alias = "RefExterna",
        skip_serializing_if = "Option::is_none"
    )]
    pub ref_externa: Option<TextMax60>,
    #[serde(
        rename = "sum1:SinRegistroPrevio",
        alias = "SinRegistroPrevio",
        skip_serializing_if = "Option::is_none"
    )]
    pub sin_registro_previo: Option<SiNo>,
    #[serde(
        rename = "sum1:RechazoPrevio",
        alias = "RechazoPrevio",
        skip_serializing_if = "Option::is_none"
    )]
    pub rechazo_previo: Option<SiNo>,
    #[serde(
        rename = "sum1:GeneradoPor",
        alias = "GeneradoPor",
        skip_serializing_if = "Option::is_none"
    )]
    pub generado_por: Option<GeneradoPor>,
    #[serde(
        rename = "sum1:Generador",
        alias = "Generador",
        skip_serializing_if = "Option::is_none"
    )]
    pub generador: Option<PersonaFisicaJuridicaES>,
    #[serde(rename = "sum1:Encadenamiento", alias = "Encadenamiento")]
    pub encadenamiento: Encadenamiento,
    #[serde(rename = "sum1:SistemaInformatico", alias = "SistemaInformatico")]
    pub sistema_informatico: SistemaInformatico,
    #[serde(
        rename = "sum1:FechaHoraHusoGenRegistro",
        alias = "FechaHoraHusoGenRegistro"
    )]
    pub fecha_hora_huso_gen_registro: String, // DateTime with timezone
    #[serde(rename = "sum1:TipoHuella", alias = "TipoHuella")]
    pub tipo_huella: TipoHuella,
    #[serde(rename = "sum1:Huella", alias = "Huella")]
    pub huella: StringMax64,
    // Note: Digital signature (Signature element) would require XML signature handling
}

impl RegistroFacturacionAnulacion {
    /// Validates XSD constraints for this cancellation record
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate IDVersion is "1.0"
        if self.id_version.as_str() != "1.0" {
            return Err(ValidationError::new(
                "RegistroFacturacionAnulacion",
                format!("IDVersion must be '1.0', got '{}'", self.id_version),
            ));
        }
        Ok(())
    }
}

// Response types
// Based on RespuestaSuministro.xsd from AEAT

// Estado types - two different enums per XSD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EstadoEnvio {
    Correcto,
    ParcialmenteCorrecto,
    Incorrecto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EstadoRegistro {
    Correcto,
    AceptadoConErrores,
    Incorrecto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TipoOperacion {
    Alta,
    Anulacion,
}

// Main response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespuestaSuministro {
    #[serde(rename = "CSV", skip_serializing_if = "Option::is_none")]
    pub csv: Option<String>,
    #[serde(rename = "DatosPresentacion", skip_serializing_if = "Option::is_none")]
    pub datos_presentacion: Option<DatosPresentacion>,
    #[serde(rename = "Cabecera")]
    pub cabecera: CabeceraRespuesta,
    #[serde(rename = "TiempoEsperaEnvio")]
    pub tiempo_espera_envio: u32,
    #[serde(rename = "EstadoEnvio")]
    pub estado_envio: EstadoEnvio,
    #[serde(rename = "RespuestaLinea", default)]
    pub respuesta_linea: Vec<RespuestaLinea>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CabeceraRespuesta {
    #[serde(rename = "ObligadoEmision")]
    pub obligado_emision: PersonaFisicaJuridicaES,
    #[serde(rename = "Representante", skip_serializing_if = "Option::is_none")]
    pub representante: Option<PersonaFisicaJuridicaES>,
    #[serde(rename = "RemisionVoluntaria", skip_serializing_if = "Option::is_none")]
    pub remision_voluntaria: Option<RemisionVoluntaria>,
    #[serde(
        rename = "RemisionRequerimiento",
        skip_serializing_if = "Option::is_none"
    )]
    pub remision_requerimiento: Option<RemisionRequerimiento>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespuestaLinea {
    #[serde(rename = "IDFactura")]
    pub id_factura: IDFacturaRespuesta,
    #[serde(rename = "Operacion")]
    pub operacion: OperacionRespuesta,
    #[serde(rename = "RefExterna", skip_serializing_if = "Option::is_none")]
    pub ref_externa: Option<TextMax60>,
    #[serde(rename = "EstadoRegistro")]
    pub estado_registro: EstadoRegistro,
    #[serde(
        rename = "CodigoErrorRegistro",
        skip_serializing_if = "Option::is_none"
    )]
    pub codigo_error_registro: Option<u32>,
    #[serde(
        rename = "DescripcionErrorRegistro",
        skip_serializing_if = "Option::is_none"
    )]
    pub descripcion_error_registro: Option<String>,
    #[serde(rename = "RegistroDuplicado", skip_serializing_if = "Option::is_none")]
    pub registro_duplicado: Option<RegistroDuplicado>,
}

impl RespuestaLinea {
    /// The typed backend error code reported for this record, if any.
    pub fn backend_error(&self) -> Option<crate::errors::BackendError> {
        self.codigo_error_registro
            .map(crate::errors::BackendError::from_code)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistroDuplicado {
    #[serde(rename = "IdPeticionRegistroDuplicado")]
    pub id_peticion_registro_duplicado: String,
    #[serde(rename = "EstadoRegistroDuplicado")]
    pub estado_registro_duplicado: EstadoRegistroDuplicado,
    #[serde(
        rename = "CodigoErrorRegistro",
        skip_serializing_if = "Option::is_none"
    )]
    pub codigo_error_registro: Option<u32>,
    #[serde(
        rename = "DescripcionErrorRegistro",
        skip_serializing_if = "Option::is_none"
    )]
    pub descripcion_error_registro: Option<String>,
}

impl RegistroDuplicado {
    /// The typed backend error code reported for this duplicate record, if any.
    pub fn backend_error(&self) -> Option<crate::errors::BackendError> {
        self.codigo_error_registro
            .map(crate::errors::BackendError::from_code)
    }
}

// EstadoRegistroSFType in SuministroInformacion.xsd.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EstadoRegistroDuplicado {
    Correcta,
    AceptadaConErrores,
    Anulada,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDFacturaRespuesta {
    #[serde(rename = "IDEmisorFactura")]
    pub id_emisor_factura: String, // Can be "AAAA" in test data
    #[serde(rename = "NumSerieFactura")]
    pub num_serie_factura: String,
    #[serde(rename = "FechaExpedicionFactura")]
    pub fecha_expedicion_factura: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperacionRespuesta {
    #[serde(rename = "TipoOperacion")]
    pub tipo_operacion: TipoOperacion,
    #[serde(rename = "Subsanacion", skip_serializing_if = "Option::is_none")]
    pub subsanacion: Option<SiNo>,
    #[serde(rename = "RechazoPrevio", skip_serializing_if = "Option::is_none")]
    pub rechazo_previo: Option<SiNo>,
    #[serde(rename = "SinRegistroPrevio", skip_serializing_if = "Option::is_none")]
    pub sin_registro_previo: Option<SiNo>,
}

// Query response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespuestaConsultaFactuSistemaFacturacion {
    #[serde(rename = "Cabecera")]
    pub cabecera: CabeceraConsulta,
    #[serde(rename = "PeriodoImputacion")]
    pub periodo_imputacion: PeriodoImputacion,
    #[serde(rename = "IndicadorPaginacion")]
    pub indicador_paginacion: IndicadorPaginacion,
    #[serde(rename = "ResultadoConsulta")]
    pub resultado_consulta: ResultadoConsulta,
    #[serde(
        rename = "RegistroRespuestaConsultaFactuSistemaFacturacion",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub registros: Vec<RegistroRespuestaConsultaRegFacturacion>,
    #[serde(rename = "ClavePaginacion", skip_serializing_if = "Option::is_none")]
    pub clave_paginacion: Option<ClavePaginacion>,
}

pub type RespuestaConsultaLR = RespuestaConsultaFactuSistemaFacturacion;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndicadorPaginacion {
    S,
    N,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResultadoConsulta {
    ConDatos,
    SinDatos,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistroRespuestaConsultaRegFacturacion {
    #[serde(rename = "IDFactura")]
    pub id_factura: IDFactura,
    #[serde(rename = "DatosRegistroFacturacion")]
    pub datos_registro_facturacion: RespuestaDatosRegistroFacturacion,
    #[serde(rename = "DatosPresentacion", skip_serializing_if = "Option::is_none")]
    pub datos_presentacion: Option<DatosPresentacion>,
    #[serde(rename = "EstadoRegistro")]
    pub estado_registro: EstadoRegFactu,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatosPresentacion {
    #[serde(rename = "NIFPresentador", skip_serializing_if = "Option::is_none")]
    pub nif_presentador: Option<NIF>,
    #[serde(
        rename = "TimestampPresentacion",
        skip_serializing_if = "Option::is_none"
    )]
    pub timestamp_presentacion: Option<String>,
    // Only present in the consulta response (DatosPresentacion2Type); absent
    // from the suministro response (DatosPresentacionType).
    #[serde(rename = "IdPeticion", skip_serializing_if = "Option::is_none")]
    pub id_peticion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstadoRegFactu {
    #[serde(rename = "TimestampUltimaModificacion")]
    pub timestamp_ultima_modificacion: String, // dateTime
    #[serde(rename = "EstadoRegistro")]
    pub estado_registro: EstadoRegistroConsulta,
    #[serde(
        rename = "CodigoErrorRegistro",
        skip_serializing_if = "Option::is_none"
    )]
    pub codigo_error_registro: Option<ErrorDetalle>,
    #[serde(
        rename = "DescripcionErrorRegistro",
        skip_serializing_if = "Option::is_none"
    )]
    pub descripcion_error_registro: Option<TextMax500>,
}

impl EstadoRegFactu {
    /// The typed backend error code reported for this record, if any.
    pub fn backend_error(&self) -> Option<crate::errors::BackendError> {
        self.codigo_error_registro
            .map(|code| crate::errors::BackendError::from_code(code as u32))
    }
}

// Wrapper for the rectified-invoice list in a consulta response
// (FacturasRectificadas -> IDFacturaRectificada*, IDFacturaARType).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacturasRectificadas {
    #[serde(rename = "IDFacturaRectificada", alias = "sum1:IDFacturaRectificada")]
    pub facturas: Vec<IDFactura>,
}

// Wrapper for the substituted-invoice list in a consulta response
// (FacturasSustituidas -> IDFacturaSustituida*, IDFacturaARType).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacturasSustituidas {
    #[serde(rename = "IDFacturaSustituida", alias = "sum1:IDFacturaSustituida")]
    pub facturas: Vec<IDFactura>,
}

// Field sequence mirrors RespuestaDatosRegistroFacturacionType in
// RespuestaConsultaLR.xsd. Every element is optional (minOccurs=0). Note this
// type intentionally has no IDVersion: the consulta response does not echo it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespuestaDatosRegistroFacturacion {
    #[serde(rename = "NombreRazonEmisor", skip_serializing_if = "Option::is_none")]
    pub nombre_razon_emisor: Option<StringMax120>,
    #[serde(rename = "RefExterna", skip_serializing_if = "Option::is_none")]
    pub ref_externa: Option<TextMax60>,
    #[serde(rename = "Subsanacion", skip_serializing_if = "Option::is_none")]
    pub subsanacion: Option<SiNo>,
    #[serde(rename = "RechazoPrevio", skip_serializing_if = "Option::is_none")]
    pub rechazo_previo: Option<SiNo>,
    #[serde(rename = "SinRegistroPrevio", skip_serializing_if = "Option::is_none")]
    pub sin_registro_previo: Option<SiNo>,
    #[serde(rename = "GeneradoPor", skip_serializing_if = "Option::is_none")]
    pub generado_por: Option<GeneradoPor>,
    #[serde(rename = "Generador", skip_serializing_if = "Option::is_none")]
    pub generador: Option<PersonaFisicaJuridicaConsulta>,
    #[serde(rename = "TipoFactura", skip_serializing_if = "Option::is_none")]
    pub tipo_factura: Option<TipoFactura>,
    #[serde(rename = "TipoRectificativa", skip_serializing_if = "Option::is_none")]
    pub tipo_rectificativa: Option<TipoRectificativa>,
    #[serde(
        rename = "FacturasRectificadas",
        skip_serializing_if = "Option::is_none"
    )]
    pub facturas_rectificadas: Option<FacturasRectificadas>,
    #[serde(
        rename = "FacturasSustituidas",
        skip_serializing_if = "Option::is_none"
    )]
    pub facturas_sustituidas: Option<FacturasSustituidas>,
    #[serde(
        rename = "ImporteRectificacion",
        skip_serializing_if = "Option::is_none"
    )]
    pub importe_rectificacion: Option<ImporteRectificacion>,
    #[serde(rename = "FechaOperacion", skip_serializing_if = "Option::is_none")]
    pub fecha_operacion: Option<Fecha>,
    #[serde(
        rename = "DescripcionOperacion",
        skip_serializing_if = "Option::is_none"
    )]
    pub descripcion_operacion: Option<TextMax500>,
    #[serde(
        rename = "FacturaSimplificadaArt7273",
        skip_serializing_if = "Option::is_none"
    )]
    pub factura_simplificada_art7273: Option<SiNo>,
    #[serde(
        rename = "FacturaSinIdentifDestinatarioArt61d",
        skip_serializing_if = "Option::is_none"
    )]
    pub factura_sin_identif_destinatario_art61d: Option<SiNo>,
    #[serde(rename = "Macrodato", skip_serializing_if = "Option::is_none")]
    pub macrodato: Option<SiNo>,
    #[serde(
        rename = "EmitidaPorTerceroODestinatario",
        skip_serializing_if = "Option::is_none"
    )]
    pub emitida_por_tercero_o_destinatario: Option<GeneradoPor>,
    #[serde(rename = "Tercero", skip_serializing_if = "Option::is_none")]
    pub tercero: Option<PersonaFisicaJuridicaConsulta>,
    #[serde(rename = "Destinatarios", skip_serializing_if = "Option::is_none")]
    pub destinatarios: Option<Destinatarios>,
    #[serde(rename = "Cupon", skip_serializing_if = "Option::is_none")]
    pub cupon: Option<SiNo>,
    #[serde(rename = "Desglose", skip_serializing_if = "Option::is_none")]
    pub desglose: Option<Desglose>,
    #[serde(rename = "CuotaTotal", skip_serializing_if = "Option::is_none")]
    pub cuota_total: Option<ImporteSgn12_2>,
    #[serde(rename = "ImporteTotal", skip_serializing_if = "Option::is_none")]
    pub importe_total: Option<ImporteSgn14_2>,
    #[serde(rename = "Encadenamiento", skip_serializing_if = "Option::is_none")]
    pub encadenamiento: Option<Encadenamiento>,
    #[serde(rename = "SistemaInformatico", skip_serializing_if = "Option::is_none")]
    pub sistema_informatico: Option<SistemaInformatico>,
    #[serde(
        rename = "FechaHoraHusoGenRegistro",
        skip_serializing_if = "Option::is_none"
    )]
    pub fecha_hora_huso_gen_registro: Option<String>, // dateTime with timezone
    #[serde(
        rename = "NumRegistroAcuerdoFacturacion",
        skip_serializing_if = "Option::is_none"
    )]
    pub num_registro_acuerdo_facturacion: Option<StringMax15>,
    #[serde(
        rename = "IdAcuerdoSistemaInformatico",
        skip_serializing_if = "Option::is_none"
    )]
    pub id_acuerdo_sistema_informatico: Option<StringMax16>,
    #[serde(rename = "TipoHuella", skip_serializing_if = "Option::is_none")]
    pub tipo_huella: Option<TipoHuella>,
    #[serde(rename = "Huella", skip_serializing_if = "Option::is_none")]
    pub huella: Option<StringMax64>,
    #[serde(rename = "NifRepresentante", skip_serializing_if = "Option::is_none")]
    pub nif_representante: Option<NIF>,
    #[serde(rename = "FechaFinVeriFactu", skip_serializing_if = "Option::is_none")]
    pub fecha_fin_verifactu: Option<Fecha>,
    #[serde(rename = "Incidencia", skip_serializing_if = "Option::is_none")]
    pub incidencia: Option<Incidencia>,
}

#[derive(Deserialize, Serialize)]
pub struct SoapEnvelopeReg<T> {
    #[serde(rename = "Header", skip_serializing)]
    pub _header: Option<serde::de::IgnoredAny>,
    #[serde(rename = "Body")]
    pub body: SoapBodyReg<T>,
}

fn render_soap_envelope(namespaces: &str, payload: &str) -> String {
    format!(
        "<?xml version=\"1.0\"?>\n\
<soapenv:Envelope {namespaces}><soapenv:Header/><soapenv:Body>{payload}</soapenv:Body></soapenv:Envelope>",
        namespaces = namespaces,
        payload = payload
    )
}

impl<T: IntoSoapXml> SoapEnvelopeReg<T> {
    pub fn to_xml(&self) -> String {
        let payload = &self.body.payload;
        let namespaces = payload.soap_envelope_namespaces();
        let xml_payload = payload.to_xml();
        render_soap_envelope(namespaces, xml_payload.as_str())
    }
}

#[derive(Deserialize)]
pub struct SoapEnvelopeRespuestaReg<T> {
    #[serde(rename = "Header")]
    pub _header: Option<serde::de::IgnoredAny>,
    #[serde(rename = "Body")]
    pub body: SoapBodyRespuestaReg<T>,
}

#[derive(Deserialize)]
pub struct SoapBodyRespuestaReg<T> {
    #[serde(rename = "RespuestaRegFactuSistemaFacturacion")]
    pub payload: T,
}

#[derive(Deserialize)]
pub struct SoapEnvelopeRespuestaConsulta<T> {
    #[serde(rename = "Header")]
    pub _header: Option<serde::de::IgnoredAny>,
    #[serde(rename = "Body")]
    pub body: SoapBodyRespuestaConsulta<T>,
}

#[derive(Deserialize)]
pub struct SoapBodyRespuestaConsulta<T> {
    #[serde(
        rename = "RespuestaConsultaLRFactuSistemaFacturacion",
        alias = "RespuestaConsultaFactuSistemaFacturacion"
    )]
    pub payload: T,
}

#[derive(Deserialize)]
pub struct SoapEnvelopeConsulta<T> {
    #[serde(rename = "Header")]
    pub _header: Option<serde::de::IgnoredAny>,
    #[serde(rename = "Body")]
    pub body: SoapBodyConsulta<T>,
}

impl<T: IntoSoapXml> SoapEnvelopeConsulta<T> {
    pub fn to_xml(&self) -> String {
        let payload = &self.body.payload;
        let namespaces = payload.soap_envelope_namespaces();
        let xml_payload = payload.to_xml();
        render_soap_envelope(namespaces, xml_payload.as_str())
    }
}

#[derive(Deserialize)]
pub struct SoapBodyConsulta<T> {
    #[serde(rename = "ConsultaFactuSistemaFacturacion")]
    pub payload: T,
}

#[derive(Deserialize, Serialize)]
pub struct SoapBodyReg<T> {
    #[serde(rename = "RegFactuSistemaFacturacion")]
    pub payload: T,
}

/// SOAP envelope carrying a `<Fault>` in its body. AEAT returns this (rather
/// than the expected response element) for header/authorization/format errors,
/// e.g. an `ObligadoEmision` NIF that is malformed, unknown or not authorized.
#[derive(Debug, Deserialize)]
pub struct SoapFaultEnvelope {
    #[serde(rename = "Header")]
    pub _header: Option<serde::de::IgnoredAny>,
    #[serde(rename = "Body")]
    pub body: SoapFaultBody,
}

#[derive(Debug, Deserialize)]
pub struct SoapFaultBody {
    #[serde(rename = "Fault")]
    pub fault: SoapFault,
}

#[derive(Debug, Deserialize)]
pub struct SoapFault {
    #[serde(rename = "faultcode")]
    pub faultcode: String,
    #[serde(rename = "faultstring")]
    pub faultstring: String,
}

impl SoapFault {
    /// AEAT prefixes the fault string with `Codigo[NNNN].`. When present,
    /// return the bare numeric code so callers can match on it.
    pub fn codigo(&self) -> Option<&str> {
        self.faultstring
            .strip_prefix("Codigo[")
            .and_then(|rest| rest.split(']').next())
    }

    /// The typed backend error code for this fault, when AEAT included a
    /// recognizable `Codigo[NNNN]` prefix in the fault string. This lets
    /// callers match on a specific [`crate::errors::BackendError`] rather than
    /// parsing the human-readable fault string.
    pub fn backend_error(&self) -> Option<crate::errors::BackendError> {
        self.codigo()
            .and_then(|codigo| codigo.parse::<u32>().ok())
            .map(crate::errors::BackendError::from_code)
    }
}

impl std::fmt::Display for SoapFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.faultstring, self.faultcode)
    }
}
