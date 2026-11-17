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
        impl_string_type!($name, $name::validate);
    };
    // `$received` validates a value read from XML, which may be laxer than
    // constructing one: an AEAT response must stay readable.
    ($name:ident, $received:path) => {
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
                $received(&value).map_err(serde::de::Error::custom)?;
                Ok(Self(value))
            }
        }
    };
}

/// A closed XSD enumeration: one variant per `<enumeration value="..."/>`,
/// exchanged as that literal value. A value outside the enumeration cannot be
/// built, and is refused when read.
macro_rules! xsd_enum {
    (
        $(#[$meta:meta])*
        $name:ident {
            $($(#[$variant_meta:meta])* $variant:ident => $value:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $($(#[$variant_meta])* $variant),+
        }

        impl $name {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => $value),+
                }
            }
        }

        impl TryFrom<&str> for $name {
            type Error = ValidationError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                match value {
                    $($value => Ok(Self::$variant),)+
                    other => Err(ValidationError::new(
                        stringify!($name),
                        format!(
                            "expected one of {}, got {other:?}",
                            [$($value),+].join(", ")
                        ),
                    )),
                }
            }
        }

        impl TryFrom<String> for $name {
            type Error = ValidationError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::try_from(value.as_str())
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
                Self::try_from(value.as_str()).map_err(serde::de::Error::custom)
            }
        }
    };
}

/// A value that enters the huella must carry no leading or trailing
/// whitespace: the hash trims it (AEAT huella specification v0.1.2) but the
/// XML sends it as given, so a padded value would identify one invoice at the
/// AEAT and hash as another. Uses the same definition of whitespace as the
/// hash's `str::trim`.
fn validate_no_outer_whitespace(
    value: &str,
    type_name: &'static str,
) -> Result<(), ValidationError> {
    if value.trim() != value {
        return Err(ValidationError::new(
            type_name,
            "cannot start or end with whitespace",
        ));
    }
    Ok(())
}

macro_rules! string_max_type {
    ($name:ident, $max_len:expr) => {
        string_max_type!($name, $max_len, |_value: &str| Ok(()));
    };
    // A type used in a huella field: rejects outer whitespace as well.
    ($name:ident, $max_len:expr, hashed) => {
        string_max_type!($name, $max_len, |value: &str| {
            validate_no_outer_whitespace(value, stringify!($name))
        });
    };
    ($name:ident, $max_len:expr, $extra:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(String);

        impl $name {
            fn validate(value: &str) -> Result<(), ValidationError> {
                Self::validate_len(value)?;
                let extra: fn(&str) -> Result<(), ValidationError> = $extra;
                extra(value)
            }

            fn validate_len(value: &str) -> Result<(), ValidationError> {
                let len = value.chars().count();
                if len > $max_len {
                    return Err(ValidationError::new(
                        stringify!($name),
                        format!("must contain at most {} characters, got {}", $max_len, len),
                    ));
                }
                Ok(())
            }
        }

        // A padded value read back from the AEAT (say, a record another SIF
        // sent) is kept as received; the huella trims it anyway.
        impl_string_type!($name, $name::validate_len);
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

// `RegistroAnterior/NumSerieFactura` (a huella field) is `TextMax60Type` in the
// XSD; every other `NumSerieFactura` is the stricter [`TextoIDFactura`].
string_max_type!(TextMax60, 60, hashed);

/// `NumSerieFactura` (`TextoIDFacturaType`): 1 to 60 characters. On top of the
/// XSD, the AEAT refuses (error 1104) any character outside printable ASCII
/// (32 to 126) and the five characters `"`, `'`, `<`, `>` and `=`. It is a
/// huella field, so it cannot start or end with whitespace either.
///
/// A value read from XML is only held to the maximum length, so a record
/// another system sent stays readable.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextoIDFactura(String);

impl TextoIDFactura {
    const MAX_LEN: usize = 60;
    const FORBIDDEN: [char; 5] = ['"', '\'', '<', '>', '='];

    fn validate(value: &str) -> Result<(), ValidationError> {
        if value.is_empty() {
            return Err(ValidationError::new(
                "TextoIDFactura",
                "value cannot be empty",
            ));
        }
        Self::validate_len(value)?;
        if let Some(invalid) = value
            .chars()
            .find(|c| !matches!(c, ' '..='~') || Self::FORBIDDEN.contains(c))
        {
            return Err(ValidationError::new(
                "TextoIDFactura",
                format!(
                    "can only contain printable ASCII characters other than \" ' < > =, got {invalid:?}"
                ),
            ));
        }
        validate_no_outer_whitespace(value, "TextoIDFactura")
    }

    fn validate_len(value: &str) -> Result<(), ValidationError> {
        let len = value.chars().count();
        if len > Self::MAX_LEN {
            return Err(ValidationError::new(
                "TextoIDFactura",
                format!(
                    "must contain at most {} characters, got {}",
                    Self::MAX_LEN,
                    len
                ),
            ));
        }
        Ok(())
    }
}

impl_string_type!(TextoIDFactura, TextoIDFactura::validate_len);

/// The pieces of an `xs:dateTime` literal that the two levels of strictness
/// of [`FechaHoraHuso`] tell apart.
struct XsdDateTime {
    year_digits: usize,
    negative_year: bool,
    has_fraction: bool,
    has_timezone: bool,
}

/// Parses the lexical form of `xs:dateTime`:
/// `-?YYYY-MM-DDThh:mm:ss(.s+)?(Z|(+|-)hh:mm)?`.
fn parse_xsd_date_time(value: &str) -> Option<XsdDateTime> {
    fn two_digits(bytes: &[u8]) -> Option<u32> {
        match bytes {
            [a, b] if a.is_ascii_digit() && b.is_ascii_digit() => {
                Some(u32::from(a - b'0') * 10 + u32::from(b - b'0'))
            }
            _ => None,
        }
    }

    if !value.is_ascii() {
        return None;
    }
    let (negative_year, rest) = match value.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, value),
    };
    let (date, time) = rest.split_once('T')?;

    let mut date_parts = date.split('-');
    let year = date_parts.next()?;
    let month = two_digits(date_parts.next()?.as_bytes())?;
    let day = two_digits(date_parts.next()?.as_bytes())?;
    if date_parts.next().is_some() || year.len() < 4 || !year.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if !is_valid_calendar_date(year.parse().ok()?, month, day) {
        return None;
    }

    let time = time.as_bytes();
    if time.len() < 8 || time[2] != b':' || time[5] != b':' {
        return None;
    }
    let hour = two_digits(&time[0..2])?;
    let minute = two_digits(&time[3..5])?;
    let second = two_digits(&time[6..8])?;
    let end_of_day = hour == 24 && minute == 0 && second == 0;
    if !(end_of_day || (hour <= 23 && minute <= 59 && second <= 59)) {
        return None;
    }

    let mut rest = &time[8..];
    let mut has_fraction = false;
    if let [b'.', fraction @ ..] = rest {
        let digits = fraction.iter().take_while(|b| b.is_ascii_digit()).count();
        if digits == 0 {
            return None;
        }
        if end_of_day && fraction[..digits].iter().any(|b| *b != b'0') {
            return None;
        }
        has_fraction = true;
        rest = &fraction[digits..];
    }

    let has_timezone = match rest {
        [] => false,
        [b'Z'] => true,
        [b'+' | b'-', offset @ ..] if offset.len() == 5 && offset[2] == b':' => {
            let offset_hour = two_digits(&offset[0..2])?;
            let offset_minute = two_digits(&offset[3..5])?;
            if !(offset_hour < 14 && offset_minute <= 59 || offset_hour == 14 && offset_minute == 0)
            {
                return None;
            }
            true
        }
        _ => return None,
    };

    Some(XsdDateTime {
        year_digits: year.len(),
        negative_year,
        has_fraction,
        has_timezone,
    })
}

/// `FechaHoraHusoGenRegistro`: the moment a record was generated, with its
/// time zone. The XSD type is `xs:dateTime` and the AEAT documents the format
/// `YYYY-MM-DDThh:mm:ssTZD` (ISO 8601, for example
/// `2024-01-01T19:20:30+01:00`); anything else is error 1244.
///
/// The value is kept as the exact string it was built from or read as, because
/// that string is what the huella hashes and what the XML sends: it is never
/// reformatted.
///
/// Building one requires whole seconds and an explicit time zone (`Z` or
/// `+hh:mm` / `-hh:mm`). A value read from XML is held to `xs:dateTime` only,
/// so fractional seconds or a missing time zone in a response stay readable.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FechaHoraHuso(String);

impl FechaHoraHuso {
    /// Formats `date_time` once, to whole seconds and with its numeric offset
    /// (`2024-01-01T19:20:30+01:00`).
    pub fn from_datetime<Tz>(date_time: &chrono::DateTime<Tz>) -> Result<Self, ValidationError>
    where
        Tz: chrono::TimeZone,
        Tz::Offset: fmt::Display,
    {
        Self::try_from(date_time.format("%Y-%m-%dT%H:%M:%S%:z").to_string())
    }

    fn validate(value: &str) -> Result<(), ValidationError> {
        match parse_xsd_date_time(value) {
            Some(parsed)
                if parsed.year_digits == 4
                    && !parsed.negative_year
                    && !parsed.has_fraction
                    && parsed.has_timezone =>
            {
                Ok(())
            }
            _ => Err(ValidationError::new(
                "FechaHoraHuso",
                "must follow the YYYY-MM-DDThh:mm:ssTZD format, where TZD is Z, +hh:mm or -hh:mm",
            )),
        }
    }

    fn validate_received(value: &str) -> Result<(), ValidationError> {
        match parse_xsd_date_time(value) {
            Some(_) => Ok(()),
            None => Err(ValidationError::new(
                "FechaHoraHuso",
                "must be a valid xs:dateTime",
            )),
        }
    }
}

impl_string_type!(FechaHoraHuso, FechaHoraHuso::validate_received);
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

xsd_enum! {
    /// `ClaveRegimen` (`IdOperacionesTrascendenciaTributariaType`): the VAT or
    /// IGIC regime of a breakdown line. The codes are those of lists L8A and
    /// L8B of the AEAT specification; 12, 13 and 16 are not part of the
    /// enumeration.
    IdOperacionesTrascendenciaTributaria {
        C01 => "01",
        C02 => "02",
        C03 => "03",
        C04 => "04",
        C05 => "05",
        C06 => "06",
        C07 => "07",
        C08 => "08",
        C09 => "09",
        C10 => "10",
        C11 => "11",
        C14 => "14",
        C15 => "15",
        C17 => "17",
        C18 => "18",
        C19 => "19",
        C20 => "20",
        C21 => "21",
    }
}

xsd_enum! {
    /// `IDVersion` (`VersionType`): the version of the record format.
    IDVersion {
        V1_0 => "1.0",
    }
}

/// A Spanish NIF (`NIFType`): exactly 9 characters.
///
/// A value read from XML is kept as received, whatever its length: the AEAT's
/// public examples use placeholders such as "AAAA", and a response must stay
/// readable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NIF(String);

impl NIF {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn validate(value: &str) -> Result<(), ValidationError> {
        let len = value.chars().count();
        if len != 9 {
            return Err(ValidationError::new(
                "NIF",
                format!("must contain exactly 9 characters, got {len}"),
            ));
        }
        // `IDEmisorFactura` is a huella field.
        validate_no_outer_whitespace(value, "NIF")
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

/// A Spanish person or company, identified by NIF only
/// (`PersonaFisicaJuridicaESType`): the `ObligadoEmision` and `Representante`
/// of a `Cabecera`. A foreign identifier (`IDOtro`) is not representable.
#[derive(Debug, Clone, Serialize)]
pub struct PersonaFisicaJuridicaES {
    #[serde(rename = "sum1:NombreRazon")]
    pub nombre_razon: StringMax120,
    #[serde(rename = "sum1:NIF")]
    pub nif: NIF,
}

impl<'de> Deserialize<'de> for PersonaFisicaJuridicaES {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Wire {
            #[serde(rename = "sum1:NombreRazon", alias = "NombreRazon")]
            nombre_razon: StringMax120,
            #[serde(rename = "sum1:NIF", alias = "NIF")]
            nif: Option<NIF>,
            #[serde(rename = "sum1:IDOtro", alias = "IDOtro")]
            id_otro: Option<serde::de::IgnoredAny>,
        }

        let wire = Wire::deserialize(deserializer)?;
        if wire.id_otro.is_some() {
            return Err(D::Error::custom(
                "PersonaFisicaJuridicaES cannot have an IDOtro",
            ));
        }
        Ok(Self {
            nombre_razon: wire.nombre_razon,
            nif: wire
                .nif
                .ok_or_else(|| D::Error::custom("PersonaFisicaJuridicaES must have a NIF"))?,
        })
    }
}

/// A person or company identified by a NIF or by a foreign identifier
/// (`PersonaFisicaJuridicaType`): the `Tercero` of an alta and the `Generador`
/// of an anulación.
#[derive(Debug, Clone)]
pub struct PersonaFisicaJuridica {
    pub nombre_razon: StringMax120,
    pub identificador: Identificador,
}

#[derive(Serialize, Deserialize)]
struct PersonaFisicaJuridicaWire {
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

impl Serialize for PersonaFisicaJuridica {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let (nif, id_otro) = match &self.identificador {
            Identificador::Nif(nif) => (Some(nif.clone()), None),
            Identificador::IdOtro(id_otro) => (None, Some(id_otro.clone())),
        };
        PersonaFisicaJuridicaWire {
            nombre_razon: self.nombre_razon.clone(),
            nif,
            id_otro,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PersonaFisicaJuridica {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = PersonaFisicaJuridicaWire::deserialize(deserializer)?;
        Ok(Self {
            nombre_razon: wire.nombre_razon,
            identificador: identificador_from_wire(
                "PersonaFisicaJuridica",
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

impl From<PersonaFisicaJuridica> for PersonaFisicaJuridicaConsulta {
    fn from(value: PersonaFisicaJuridica) -> Self {
        Self {
            nombre_razon: value.nombre_razon,
            identificador: value.identificador,
        }
    }
}

impl From<PersonaFisicaJuridicaES> for PersonaFisicaJuridicaConsulta {
    fn from(value: PersonaFisicaJuridicaES) -> Self {
        Self {
            nombre_razon: value.nombre_razon,
            identificador: Identificador::Nif(value.nif),
        }
    }
}

impl From<PersonaFisicaJuridicaConsulta> for PersonaFisicaJuridica {
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
    pub id_version: IDVersion,
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
    pub num_serie_factura: Option<TextoIDFactura>,
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
    pub num_serie_factura: Option<TextoIDFactura>,
    #[serde(
        rename = "sum:FechaExpedicionFactura",
        alias = "FechaExpedicionFactura",
        skip_serializing_if = "Option::is_none"
    )]
    pub fecha_expedicion_factura: Option<Fecha>,
}

// Country codes accepted by AEAT: the `CountryType2` enumeration of the
// VeriFactu `SuministroInformacion.xsd`, sorted alphabetically. Mostly ISO 3166-1
// alpha-2 plus a few AEAT-specific codes ISO does not assign (`XG`, `QU`,
// `XB`, `XU`, `XN`). ISO codes AEAT does not enumerate — the French overseas
// departments (`GF`, `GP`, `MQ`, `MF`, `BL`), `AX`, `EH` and `SJ` — are
// deliberately absent: AEAT rejects the submission if it receives them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CountryType {
    AD,
    AE,
    AF,
    AG,
    AI,
    AL,
    AM,
    AO,
    AQ,
    AR,
    AS,
    AT,
    AU,
    AW,
    AZ,
    BA,
    BB,
    BD,
    BE,
    BF,
    BG,
    BH,
    BI,
    BJ,
    BM,
    BN,
    BO,
    BQ,
    BR,
    BS,
    BT,
    BV,
    BW,
    BY,
    BZ,
    CA,
    CC,
    CD,
    CF,
    CG,
    CH,
    CI,
    CK,
    CL,
    CM,
    CN,
    CO,
    CR,
    CU,
    CV,
    CW,
    CX,
    CY,
    CZ,
    DE,
    DJ,
    DK,
    DM,
    DO,
    DZ,
    EC,
    EE,
    EG,
    ER,
    ES,
    ET,
    FI,
    FJ,
    FK,
    FM,
    FO,
    FR,
    GA,
    GB,
    GD,
    GE,
    GG,
    GH,
    GI,
    GL,
    GM,
    GN,
    GQ,
    GR,
    GS,
    GT,
    GU,
    GW,
    GY,
    HK,
    HM,
    HN,
    HR,
    HT,
    HU,
    ID,
    IE,
    IL,
    IM,
    IN,
    IO,
    IQ,
    IR,
    IS,
    IT,
    JE,
    JM,
    JO,
    JP,
    KE,
    KG,
    KH,
    KI,
    KM,
    KN,
    KP,
    KR,
    KW,
    KY,
    KZ,
    LA,
    LB,
    LC,
    LI,
    LK,
    LR,
    LS,
    LT,
    LU,
    LV,
    LY,
    MA,
    MC,
    MD,
    ME,
    MG,
    MH,
    MK,
    ML,
    MM,
    MN,
    MO,
    MP,
    MR,
    MS,
    MT,
    MU,
    MV,
    MW,
    MX,
    MY,
    MZ,
    NA,
    NC,
    NE,
    NF,
    NG,
    NI,
    NL,
    NO,
    NP,
    NR,
    NU,
    NZ,
    OM,
    PA,
    PE,
    PF,
    PG,
    PH,
    PK,
    PL,
    PM,
    PN,
    PR,
    PS,
    PT,
    PW,
    PY,
    QA,
    QU,
    RE,
    RO,
    RS,
    RU,
    RW,
    SA,
    SB,
    SC,
    SD,
    SE,
    SG,
    SH,
    SI,
    SK,
    SL,
    SM,
    SN,
    SO,
    SR,
    SS,
    ST,
    SV,
    SX,
    SY,
    SZ,
    TC,
    TD,
    TF,
    TG,
    TH,
    TJ,
    TK,
    TL,
    TM,
    TN,
    TO,
    TR,
    TT,
    TV,
    TW,
    TZ,
    UA,
    UG,
    UM,
    US,
    UY,
    UZ,
    VA,
    VC,
    VE,
    VG,
    VI,
    VN,
    VU,
    WF,
    WS,
    XB,
    XG,
    XN,
    XU,
    YE,
    YT,
    ZA,
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
    R1, // Corrective invoice (error grounded in law, article 80.1, 80.2, 80.6 LIVA)
    R2, // Corrective invoice (article 80.3 LIVA, insolvency proceedings)
    R3, // Corrective invoice (article 80.4 LIVA, uncollectable debts)
    R4, // Corrective invoice (any other cause)
    R5, // Corrective invoice for simplified invoices
    F3, // Replacement for simplified invoices
}

impl TipoFactura {
    /// Whether this is a corrective (rectificativa) invoice type, i.e. one of
    /// R1-R5. Corrective invoices carry the `TipoRectificativa` and
    /// `FacturasRectificadas` fields.
    pub fn is_corrective(&self) -> bool {
        matches!(
            self,
            TipoFactura::R1 | TipoFactura::R2 | TipoFactura::R3 | TipoFactura::R4 | TipoFactura::R5
        )
    }
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

xsd_enum! {
    /// A yes/no flag (`SiNoType` and the other two-valued S/N types of the
    /// XSD).
    SiNo {
        S => "S",
        N => "N",
    }
}

xsd_enum! {
    /// `RechazoPrevio` of an alta (`RechazoPrevioType`). The anulación's
    /// `RechazoPrevio` is a plain [`SiNo`]: it has no `X`.
    RechazoPrevio {
        /// No previous rejection.
        N => "N",
        /// The record was rejected before and is sent again.
        S => "S",
        /// The record was never accepted: there is no previous record of it at
        /// the AEAT. Only valid together with `Subsanacion = S`.
        X => "X",
    }
}

// Additional supporting types
// Invoice identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDFactura {
    #[serde(rename = "sum1:IDEmisorFactura", alias = "IDEmisorFactura")]
    pub id_emisor_factura: NIF,
    #[serde(rename = "sum1:NumSerieFactura", alias = "NumSerieFactura")]
    pub num_serie_factura: TextoIDFactura,
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
    pub num_serie_factura: TextoIDFactura,
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
    pub num_serie_factura_anulada: TextoIDFactura,
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
    pub id_version: IDVersion,
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
    pub rechazo_previo: Option<RechazoPrevio>,
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
    pub tercero: Option<PersonaFisicaJuridica>,
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
    pub importe_total: ImporteSgn12_2,
    #[serde(rename = "sum1:Encadenamiento", alias = "Encadenamiento")]
    pub encadenamiento: Encadenamiento,
    #[serde(rename = "sum1:SistemaInformatico", alias = "SistemaInformatico")]
    pub sistema_informatico: SistemaInformatico,
    #[serde(
        rename = "sum1:FechaHoraHusoGenRegistro",
        alias = "FechaHoraHusoGenRegistro"
    )]
    pub fecha_hora_huso_gen_registro: FechaHoraHuso,
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

        // Cross-field constraints between TipoFactura, TipoRectificativa and the
        // corrective/replacement reference blocks (AEAT XSD + business rules).
        let is_corrective = self.tipo_factura.is_corrective();

        // R1-R5 are rectificativas: they must declare how they rectify
        // (TipoRectificativa) and which invoices they rectify
        // (FacturasRectificadas).
        if is_corrective {
            if self.tipo_rectificativa.is_none() {
                return Err(ValidationError::new(
                    "RegistroFacturacionAlta",
                    format!(
                        "TipoRectificativa is required when TipoFactura is {} (R1-R5)",
                        self.tipo_factura
                    ),
                ));
            }
            if self.facturas_rectificadas.is_none() {
                return Err(ValidationError::new(
                    "RegistroFacturacionAlta",
                    format!(
                        "FacturasRectificadas is required when TipoFactura is {} (R1-R5)",
                        self.tipo_factura
                    ),
                ));
            }
        }

        // TipoRectificativa governs whether the rectified amounts are restated.
        // Substitutive (S) replaces the original, so it must carry
        // ImporteRectificacion; incremental (I) reports only the delta and must
        // not.
        match self.tipo_rectificativa {
            Some(TipoRectificativa::S) if self.importe_rectificacion.is_none() => {
                return Err(ValidationError::new(
                    "RegistroFacturacionAlta",
                    "ImporteRectificacion is required when TipoRectificativa is S (substitutive)",
                ));
            }
            Some(TipoRectificativa::I) if self.importe_rectificacion.is_some() => {
                return Err(ValidationError::new(
                    "RegistroFacturacionAlta",
                    "ImporteRectificacion must not be set when TipoRectificativa is I (incremental)",
                ));
            }
            _ => {}
        }

        // F3 replaces previously issued simplified invoices, so it must list the
        // invoices it substitutes.
        if matches!(self.tipo_factura, TipoFactura::F3) && self.facturas_sustituidas.is_none() {
            return Err(ValidationError::new(
                "RegistroFacturacionAlta",
                "FacturasSustituidas is required when TipoFactura is F3",
            ));
        }

        // Within each Desglose detalle, an operation is either qualified
        // (CalificacionOperacion) or exempt (OperacionExenta) — never both.
        for (index, detalle) in self.desglose.detalle_desglose.iter().enumerate() {
            if detalle.calificacion_operacion.is_some() && detalle.operacion_exenta.is_some() {
                return Err(ValidationError::new(
                    "RegistroFacturacionAlta",
                    format!(
                        "CalificacionOperacion and OperacionExenta are mutually exclusive \
                         (both set in Desglose detalle {index})"
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
    pub id_version: IDVersion,
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
    pub generador: Option<PersonaFisicaJuridica>,
    #[serde(rename = "sum1:Encadenamiento", alias = "Encadenamiento")]
    pub encadenamiento: Encadenamiento,
    #[serde(rename = "sum1:SistemaInformatico", alias = "SistemaInformatico")]
    pub sistema_informatico: SistemaInformatico,
    #[serde(
        rename = "sum1:FechaHoraHusoGenRegistro",
        alias = "FechaHoraHusoGenRegistro"
    )]
    pub fecha_hora_huso_gen_registro: FechaHoraHuso,
    #[serde(rename = "sum1:TipoHuella", alias = "TipoHuella")]
    pub tipo_huella: TipoHuella,
    #[serde(rename = "sum1:Huella", alias = "Huella")]
    pub huella: StringMax64,
    // Note: Digital signature (Signature element) would require XML signature handling
}

impl RegistroFacturacionAnulacion {
    /// Validates XSD constraints for this cancellation record. Everything the
    /// XSD constrains on an anulación is enforced by the field types, so there
    /// is nothing left to check here today.
    pub fn validate(&self) -> Result<(), ValidationError> {
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
    pub rechazo_previo: Option<RechazoPrevio>,
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
    pub rechazo_previo: Option<RechazoPrevio>,
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
    pub importe_total: Option<ImporteSgn12_2>,
    #[serde(rename = "Encadenamiento", skip_serializing_if = "Option::is_none")]
    pub encadenamiento: Option<Encadenamiento>,
    #[serde(rename = "SistemaInformatico", skip_serializing_if = "Option::is_none")]
    pub sistema_informatico: Option<SistemaInformatico>,
    #[serde(
        rename = "FechaHoraHusoGenRegistro",
        skip_serializing_if = "Option::is_none"
    )]
    pub fecha_hora_huso_gen_registro: Option<FechaHoraHuso>,
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
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
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

#[cfg(test)]
mod registro_alta_validation_tests {
    use super::*;

    fn sample_sistema_informatico() -> SistemaInformatico {
        SistemaInformatico {
            nombre_razon: "Productor SL".try_into().unwrap(),
            nif: "B75929299".try_into().unwrap(),
            nombre_sistema_informatico: "uninvoice".try_into().unwrap(),
            id_sistema_informatico: "01".try_into().unwrap(),
            version: "1.0".try_into().unwrap(),
            numero_instalacion: "1".try_into().unwrap(),
            tipo_uso_posible_solo_verifactu: SiNo::S,
            tipo_uso_posible_multi_ot: SiNo::N,
            indicador_multiples_ot: SiNo::N,
        }
    }

    fn sample_id_factura() -> IDFactura {
        IDFactura {
            id_emisor_factura: "B12345678".try_into().unwrap(),
            num_serie_factura: "2025-1".try_into().unwrap(),
            fecha_expedicion_factura: "15-03-2025".try_into().unwrap(),
        }
    }

    fn importe_rectificacion() -> ImporteRectificacion {
        ImporteRectificacion {
            base_rectificada: "100.00".try_into().unwrap(),
            cuota_rectificada: "21.00".try_into().unwrap(),
            cuota_recargo_rectificado: None,
        }
    }

    /// A minimal, valid `F1` standard invoice record. Tests mutate single fields
    /// to exercise one cross-field rule at a time.
    fn valid_f1_alta() -> RegistroFacturacionAlta {
        let detalle = Detalle {
            base_imponible_o_importe_no_sujeto: "100.00".try_into().unwrap(),
            ..Default::default()
        };
        RegistroFacturacionAlta {
            id_version: "1.0".try_into().unwrap(),
            id_factura: sample_id_factura(),
            ref_externa: None,
            nombre_razon_emisor: "Emisor SL".try_into().unwrap(),
            subsanacion: None,
            rechazo_previo: None,
            tipo_factura: TipoFactura::F1,
            tipo_rectificativa: None,
            facturas_rectificadas: None,
            facturas_sustituidas: None,
            importe_rectificacion: None,
            fecha_operacion: None,
            descripcion_operacion: "Servicios".try_into().unwrap(),
            factura_simplificada_art7273: None,
            factura_sin_identif_destinatario_art61d: None,
            macrodato: None,
            emitida_por_tercero_o_destinatario: None,
            tercero: None,
            destinatarios: None,
            cupon: None,
            desglose: Desglose::new(vec![detalle]).unwrap(),
            cuota_total: "21.00".try_into().unwrap(),
            importe_total: "121.00".try_into().unwrap(),
            encadenamiento: Encadenamiento::PrimerRegistro,
            sistema_informatico: sample_sistema_informatico(),
            fecha_hora_huso_gen_registro: "2025-03-15T10:00:00+01:00".try_into().unwrap(),
            num_registro_acuerdo_facturacion: None,
            id_acuerdo_sistema_informatico: None,
            tipo_huella: TipoHuella::Sha256,
            huella: "".try_into().unwrap(),
        }
    }

    /// A valid R1 corrective record: declares both how and what it rectifies.
    fn valid_r1_alta() -> RegistroFacturacionAlta {
        RegistroFacturacionAlta {
            tipo_factura: TipoFactura::R1,
            tipo_rectificativa: Some(TipoRectificativa::I),
            facturas_rectificadas: Some(vec![sample_id_factura()]),
            ..valid_f1_alta()
        }
    }

    #[test]
    fn f1_minimal_record_is_valid() {
        assert!(valid_f1_alta().validate().is_ok());
    }

    #[test]
    fn r1_minimal_record_is_valid() {
        assert!(valid_r1_alta().validate().is_ok());
    }

    #[test]
    fn corrective_requires_tipo_rectificativa() {
        let alta = RegistroFacturacionAlta {
            tipo_rectificativa: None,
            ..valid_r1_alta()
        };
        let err = alta.validate().unwrap_err();
        assert!(err.to_string().contains("TipoRectificativa is required"));
    }

    #[test]
    fn corrective_requires_facturas_rectificadas() {
        let alta = RegistroFacturacionAlta {
            facturas_rectificadas: None,
            ..valid_r1_alta()
        };
        let err = alta.validate().unwrap_err();
        assert!(err.to_string().contains("FacturasRectificadas is required"));
    }

    #[test]
    fn substitutive_requires_importe_rectificacion() {
        let alta = RegistroFacturacionAlta {
            tipo_rectificativa: Some(TipoRectificativa::S),
            importe_rectificacion: None,
            ..valid_r1_alta()
        };
        let err = alta.validate().unwrap_err();
        assert!(err.to_string().contains("ImporteRectificacion is required"));
    }

    #[test]
    fn substitutive_with_importe_rectificacion_is_valid() {
        let alta = RegistroFacturacionAlta {
            tipo_rectificativa: Some(TipoRectificativa::S),
            importe_rectificacion: Some(importe_rectificacion()),
            ..valid_r1_alta()
        };
        assert!(alta.validate().is_ok());
    }

    #[test]
    fn incremental_must_not_set_importe_rectificacion() {
        let alta = RegistroFacturacionAlta {
            tipo_rectificativa: Some(TipoRectificativa::I),
            importe_rectificacion: Some(importe_rectificacion()),
            ..valid_r1_alta()
        };
        let err = alta.validate().unwrap_err();
        assert!(
            err.to_string()
                .contains("ImporteRectificacion must not be set")
        );
    }

    #[test]
    fn f3_requires_facturas_sustituidas() {
        let alta = RegistroFacturacionAlta {
            tipo_factura: TipoFactura::F3,
            facturas_sustituidas: None,
            destinatarios: None,
            ..valid_f1_alta()
        };
        let err = alta.validate().unwrap_err();
        assert!(err.to_string().contains("FacturasSustituidas is required"));
    }

    #[test]
    fn f3_with_facturas_sustituidas_is_valid() {
        let alta = RegistroFacturacionAlta {
            tipo_factura: TipoFactura::F3,
            facturas_sustituidas: Some(vec![sample_id_factura()]),
            ..valid_f1_alta()
        };
        assert!(alta.validate().is_ok());
    }

    #[test]
    fn calificacion_and_exenta_are_mutually_exclusive() {
        let detalle = Detalle {
            base_imponible_o_importe_no_sujeto: "100.00".try_into().unwrap(),
            calificacion_operacion: Some(CalificacionOperacion::S1),
            operacion_exenta: Some(OperacionExenta::E1),
            ..Default::default()
        };
        let alta = RegistroFacturacionAlta {
            desglose: Desglose::new(vec![detalle]).unwrap(),
            ..valid_f1_alta()
        };
        let err = alta.validate().unwrap_err();
        assert!(err.to_string().contains("mutually exclusive"));
    }

    #[test]
    fn detalle_with_only_calificacion_is_valid() {
        let detalle = Detalle {
            base_imponible_o_importe_no_sujeto: "100.00".try_into().unwrap(),
            calificacion_operacion: Some(CalificacionOperacion::S1),
            ..Default::default()
        };
        let alta = RegistroFacturacionAlta {
            desglose: Desglose::new(vec![detalle]).unwrap(),
            ..valid_f1_alta()
        };
        assert!(alta.validate().is_ok());
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;
    use crate::hashing::{AnulacionHuellaInput, Hashable};

    // ---- builders -------------------------------------------------------

    fn sistema_informatico() -> SistemaInformatico {
        SistemaInformatico {
            nombre_razon: "Productor SL".try_into().unwrap(),
            nif: "B75929299".try_into().unwrap(),
            nombre_sistema_informatico: "uninvoice".try_into().unwrap(),
            id_sistema_informatico: "01".try_into().unwrap(),
            version: "1.0".try_into().unwrap(),
            numero_instalacion: "1".try_into().unwrap(),
            tipo_uso_posible_solo_verifactu: SiNo::S,
            tipo_uso_posible_multi_ot: SiNo::N,
            indicador_multiples_ot: SiNo::N,
        }
    }

    fn id_factura() -> IDFactura {
        IDFactura {
            id_emisor_factura: "B12345678".try_into().unwrap(),
            num_serie_factura: "2025-1".try_into().unwrap(),
            fecha_expedicion_factura: "15-03-2025".try_into().unwrap(),
        }
    }

    /// A standard `F1` Alta. Corrective tests mutate `tipo_factura` and the
    /// rectification fields.
    fn alta_record() -> RegistroFacturacionAlta {
        let detalle = Detalle {
            base_imponible_o_importe_no_sujeto: "100.00".try_into().unwrap(),
            ..Default::default()
        };
        RegistroFacturacionAlta {
            id_version: "1.0".try_into().unwrap(),
            id_factura: id_factura(),
            ref_externa: None,
            nombre_razon_emisor: "Emisor SL".try_into().unwrap(),
            subsanacion: None,
            rechazo_previo: None,
            tipo_factura: TipoFactura::F1,
            tipo_rectificativa: None,
            facturas_rectificadas: None,
            facturas_sustituidas: None,
            importe_rectificacion: None,
            fecha_operacion: None,
            descripcion_operacion: "Servicios".try_into().unwrap(),
            factura_simplificada_art7273: None,
            factura_sin_identif_destinatario_art61d: None,
            macrodato: None,
            emitida_por_tercero_o_destinatario: None,
            tercero: None,
            destinatarios: None,
            cupon: None,
            desglose: Desglose::new(vec![detalle]).unwrap(),
            cuota_total: "21.00".try_into().unwrap(),
            importe_total: "121.00".try_into().unwrap(),
            encadenamiento: Encadenamiento::PrimerRegistro,
            sistema_informatico: sistema_informatico(),
            fecha_hora_huso_gen_registro: "2025-03-15T10:00:00+01:00".try_into().unwrap(),
            num_registro_acuerdo_facturacion: None,
            id_acuerdo_sistema_informatico: None,
            tipo_huella: TipoHuella::Sha256,
            huella: "".try_into().unwrap(),
        }
    }

    fn anulacion_record() -> RegistroFacturacionAnulacion {
        RegistroFacturacionAnulacion {
            id_version: "1.0".try_into().unwrap(),
            id_factura: IDFacturaAnulada {
                id_emisor_factura_anulada: "B12345678".try_into().unwrap(),
                num_serie_factura_anulada: "2025-1".try_into().unwrap(),
                fecha_expedicion_factura_anulada: "15-03-2025".try_into().unwrap(),
            },
            ref_externa: None,
            sin_registro_previo: None,
            rechazo_previo: None,
            generado_por: None,
            generador: None,
            encadenamiento: Encadenamiento::PrimerRegistro,
            sistema_informatico: sistema_informatico(),
            fecha_hora_huso_gen_registro: "2025-03-15T11:00:00+01:00".try_into().unwrap(),
            tipo_huella: TipoHuella::Sha256,
            huella: "".try_into().unwrap(),
        }
    }

    fn cabecera() -> Cabecera {
        Cabecera {
            obligado_emision: PersonaFisicaJuridicaES {
                nombre_razon: "Emisor SL".try_into().unwrap(),
                nif: "B12345678".try_into().unwrap(),
            },
            representante: None,
            remision_voluntaria: Some(RemisionVoluntaria {
                fecha_fin_veri_factu: None,
                incidencia: Incidencia::N,
            }),
            remision_requerimiento: None,
        }
    }

    fn corrective_alta(tipo: TipoFactura) -> RegistroFacturacionAlta {
        RegistroFacturacionAlta {
            tipo_factura: tipo,
            tipo_rectificativa: Some(TipoRectificativa::I),
            facturas_rectificadas: Some(vec![id_factura()]),
            ..alta_record()
        }
    }

    // ---- corrective invoice (R1-R5) serialization -----------------------

    #[test]
    fn corrective_types_serialize_their_code() {
        for tipo in [
            TipoFactura::R1,
            TipoFactura::R2,
            TipoFactura::R3,
            TipoFactura::R4,
            TipoFactura::R5,
        ] {
            let alta = corrective_alta(tipo.clone());
            let xml = quick_xml::se::to_string(&alta).expect("valid xml");
            assert!(
                xml.contains(&format!("<sum1:TipoFactura>{tipo}</sum1:TipoFactura>")),
                "missing TipoFactura {tipo} in {xml}"
            );
        }
    }

    #[test]
    fn incremental_corrective_serializes_rectified_references() {
        let alta = corrective_alta(TipoFactura::R1);
        let xml = quick_xml::se::to_string(&alta).expect("valid xml");
        assert!(
            xml.contains("<sum1:TipoRectificativa>I</sum1:TipoRectificativa>"),
            "{xml}"
        );
        // The references are nested in the <IDFacturaRectificada> wrapper AEAT
        // requires (error 4102 otherwise).
        assert!(
            xml.contains("<sum1:FacturasRectificadas><sum1:IDFacturaRectificada>"),
            "{xml}"
        );
        // Incremental corrections must not restate the rectified amounts.
        assert!(!xml.contains("ImporteRectificacion"), "{xml}");
    }

    #[test]
    fn substitutive_corrective_serializes_importe_rectificacion() {
        let alta = RegistroFacturacionAlta {
            tipo_rectificativa: Some(TipoRectificativa::S),
            importe_rectificacion: Some(ImporteRectificacion {
                base_rectificada: "100.00".try_into().unwrap(),
                cuota_rectificada: "21.00".try_into().unwrap(),
                cuota_recargo_rectificado: None,
            }),
            ..corrective_alta(TipoFactura::R1)
        };
        let xml = quick_xml::se::to_string(&alta).expect("valid xml");
        assert!(
            xml.contains("<sum1:TipoRectificativa>S</sum1:TipoRectificativa>"),
            "{xml}"
        );
        assert!(
            xml.contains(
                "<sum1:ImporteRectificacion><sum1:BaseRectificada>100.00</sum1:BaseRectificada>"
            ),
            "{xml}"
        );
        assert!(
            xml.contains("<sum1:CuotaRectificada>21.00</sum1:CuotaRectificada>"),
            "{xml}"
        );
    }

    #[test]
    fn corrective_alta_round_trips_through_xml() {
        let alta = corrective_alta(TipoFactura::R3);
        let xml = quick_xml::se::to_string(&alta).expect("serialize");
        let parsed: RegistroFacturacionAlta = quick_xml::de::from_str(&xml).expect("deserialize");
        assert!(matches!(parsed.tipo_factura, TipoFactura::R3));
        assert!(matches!(
            parsed.tipo_rectificativa,
            Some(TipoRectificativa::I)
        ));
        assert_eq!(parsed.facturas_rectificadas.as_ref().map(Vec::len), Some(1));
    }

    // ---- country codes ---------------------------------------------------

    #[test]
    fn id_otro_round_trips_every_country_code() {
        // Keep this list explicit so the test fails if `CountryType` ever
        // diverges from AEAT's `CountryType2` enumeration.
        let codes = [
            "AD", "AE", "AF", "AG", "AI", "AL", "AM", "AO", "AQ", "AR", "AS", "AT", "AU", "AW",
            "AZ", "BA", "BB", "BD", "BE", "BF", "BG", "BH", "BI", "BJ", "BM", "BN", "BO", "BQ",
            "BR", "BS", "BT", "BV", "BW", "BY", "BZ", "CA", "CC", "CD", "CF", "CG", "CH", "CI",
            "CK", "CL", "CM", "CN", "CO", "CR", "CU", "CV", "CW", "CX", "CY", "CZ", "DE", "DJ",
            "DK", "DM", "DO", "DZ", "EC", "EE", "EG", "ER", "ES", "ET", "FI", "FJ", "FK", "FM",
            "FO", "FR", "GA", "GB", "GD", "GE", "GG", "GH", "GI", "GL", "GM", "GN", "GQ", "GR",
            "GS", "GT", "GU", "GW", "GY", "HK", "HM", "HN", "HR", "HT", "HU", "ID", "IE", "IL",
            "IM", "IN", "IO", "IQ", "IR", "IS", "IT", "JE", "JM", "JO", "JP", "KE", "KG", "KH",
            "KI", "KM", "KN", "KP", "KR", "KW", "KY", "KZ", "LA", "LB", "LC", "LI", "LK", "LR",
            "LS", "LT", "LU", "LV", "LY", "MA", "MC", "MD", "ME", "MG", "MH", "MK", "ML", "MM",
            "MN", "MO", "MP", "MR", "MS", "MT", "MU", "MV", "MW", "MX", "MY", "MZ", "NA", "NC",
            "NE", "NF", "NG", "NI", "NL", "NO", "NP", "NR", "NU", "NZ", "OM", "PA", "PE", "PF",
            "PG", "PH", "PK", "PL", "PM", "PN", "PR", "PS", "PT", "PW", "PY", "QA", "QU", "RE",
            "RO", "RS", "RU", "RW", "SA", "SB", "SC", "SD", "SE", "SG", "SH", "SI", "SK", "SL",
            "SM", "SN", "SO", "SR", "SS", "ST", "SV", "SX", "SY", "SZ", "TC", "TD", "TF", "TG",
            "TH", "TJ", "TK", "TL", "TM", "TN", "TO", "TR", "TT", "TV", "TW", "TZ", "UA", "UG",
            "UM", "US", "UY", "UZ", "VA", "VC", "VE", "VG", "VI", "VN", "VU", "WF", "WS", "XB",
            "XG", "XN", "XU", "YE", "YT", "ZA", "ZM", "ZW",
        ];
        assert_eq!(codes.len(), 246);
        assert!(codes.is_sorted());

        for code in codes {
            let source_xml = format!(
                "<IDOtro><CodigoPais>{code}</CodigoPais><IDType>03</IDType><ID>X1234567</ID></IDOtro>"
            );
            let id_otro =
                quick_xml::de::from_str::<IDOtro>(&source_xml).expect("deserialize country");
            let expected = format!("<sum1:CodigoPais>{code}</sum1:CodigoPais>");
            let xml = quick_xml::se::to_string(&id_otro).expect("serialize");
            assert!(xml.contains(&expected), "{xml}");
            quick_xml::de::from_str::<IDOtro>(&xml).expect("deserialize");
        }
    }

    // ---- mixed Alta/Anulacion batch -------------------------------------

    #[test]
    fn mixed_batch_serializes_both_record_types() {
        let suministro = SuministroInformacion::new(
            cabecera(),
            vec![
                RegistroFactura::Alta(alta_record()),
                RegistroFactura::Anulacion(anulacion_record()),
            ],
        )
        .expect("valid suministro");
        assert_eq!(suministro.registro_factura.len(), 2);

        let xml = quick_xml::se::to_string(&suministro).expect("valid xml");
        assert!(xml.contains("<sum1:RegistroAlta>"), "{xml}");
        assert!(xml.contains("<sum1:RegistroAnulacion>"), "{xml}");
    }

    #[test]
    fn batch_rejects_empty_record_list() {
        let err = SuministroInformacion::new(cabecera(), vec![]).unwrap_err();
        assert!(err.to_string().contains("at least one registro_factura"));
    }

    #[test]
    fn mixed_batch_round_trips_through_xml() {
        let suministro = SuministroInformacion::new(
            cabecera(),
            vec![
                RegistroFactura::Alta(alta_record()),
                RegistroFactura::Anulacion(anulacion_record()),
            ],
        )
        .unwrap();
        let xml = quick_xml::se::to_string(&suministro).expect("serialize");
        let parsed: SuministroInformacion = quick_xml::de::from_str(&xml).expect("deserialize");
        assert_eq!(parsed.registro_factura.len(), 2);
        assert!(matches!(
            parsed.registro_factura[0],
            RegistroFactura::Alta(_)
        ));
        assert!(matches!(
            parsed.registro_factura[1],
            RegistroFactura::Anulacion(_)
        ));
    }

    // ---- Anulacion huella vectors ---------------------------------------
    //
    // AEAT publishes a single fully worked huella example (an Alta record, see
    // `hashing::tests::alta_first_record_matches_aeat_example`). The Anulacion
    // concatenation format is documented but not given as a worked example, so
    // these vectors are pinned from the documented `IDEmisorFacturaAnulada=...`
    // layout and guard the full-struct `Hashable` wiring against drift.

    #[test]
    fn anulacion_struct_hash_matches_documented_layout() {
        let registro = anulacion_record();
        let expected = AnulacionHuellaInput {
            id_emisor_factura_anulada: "B12345678",
            num_serie_factura_anulada: "2025-1",
            fecha_expedicion_factura_anulada: "15-03-2025",
            prev_huella: None,
            fecha_hora_huso_gen_registro: "2025-03-15T11:00:00+01:00",
        }
        .huella();
        assert_eq!(registro.hash(None), expected);
    }

    #[test]
    fn anulacion_struct_hash_known_vector() {
        // Same inputs as `hashing::tests::anulacion_first_record_format`, asserted
        // here through the full `RegistroFacturacionAnulacion` struct.
        let registro = RegistroFacturacionAnulacion {
            id_factura: IDFacturaAnulada {
                id_emisor_factura_anulada: "89890001K".try_into().unwrap(),
                num_serie_factura_anulada: "12345678/G33".try_into().unwrap(),
                fecha_expedicion_factura_anulada: "01-01-2024".try_into().unwrap(),
            },
            fecha_hora_huso_gen_registro: "2024-01-03T19:20:30+01:00".try_into().unwrap(),
            ..anulacion_record()
        };
        assert_eq!(
            registro.hash(None),
            "9AC16B332D14EDC61801758056AD892CB25005778B412E2064365655A48B7B2A"
        );
    }

    #[test]
    fn anulacion_struct_hash_chains_prev_huella() {
        let registro = anulacion_record();
        let prev = "9AC16B332D14EDC61801758056AD892CB25005778B412E2064365655A48B7B2A";
        let chained = registro.hash(Some(prev));
        assert_ne!(chained, registro.hash(None));
        let expected = AnulacionHuellaInput {
            id_emisor_factura_anulada: "B12345678",
            num_serie_factura_anulada: "2025-1",
            fecha_expedicion_factura_anulada: "15-03-2025",
            prev_huella: Some(prev),
            fecha_hora_huso_gen_registro: "2025-03-15T11:00:00+01:00",
        }
        .huella();
        assert_eq!(chained, expected);
    }

    // ---- types as strict as SuministroInformacion.xsd -------------------

    #[test]
    fn si_no_has_no_x() {
        assert_eq!(SiNo::try_from("S"), Ok(SiNo::S));
        assert_eq!(SiNo::try_from("N"), Ok(SiNo::N));
        assert!(SiNo::try_from("X").is_err());
        assert!(SiNo::try_from("").is_err());
    }

    #[test]
    fn alta_rechazo_previo_has_three_values() {
        for (value, expected) in [
            ("N", RechazoPrevio::N),
            ("S", RechazoPrevio::S),
            ("X", RechazoPrevio::X),
        ] {
            assert_eq!(RechazoPrevio::try_from(value), Ok(expected));
            let registro = RegistroFacturacionAlta {
                subsanacion: Some(SiNo::S),
                rechazo_previo: Some(expected),
                ..alta_record()
            };
            let xml = quick_xml::se::to_string_with_root("RegistroAlta", &registro)
                .expect("an alta serializes");
            assert!(xml.contains(&format!("<sum1:RechazoPrevio>{value}</sum1:RechazoPrevio>")));
            let parsed: RegistroFacturacionAlta =
                quick_xml::de::from_str(&xml).expect("the alta reads back");
            assert_eq!(parsed.rechazo_previo, Some(expected));
        }
        assert!(RechazoPrevio::try_from("Z").is_err());
    }

    // RechazoPrevioAnulacionType is S/N: an anulación cannot carry `X`.
    #[test]
    fn anulacion_rechazo_previo_x_is_refused() {
        let registro = RegistroFacturacionAnulacion {
            rechazo_previo: Some(SiNo::S),
            ..anulacion_record()
        };
        let xml = quick_xml::se::to_string_with_root("RegistroAnulacion", &registro)
            .expect("an anulación serializes");
        assert!(quick_xml::de::from_str::<RegistroFacturacionAnulacion>(&xml).is_ok());

        let with_x = xml.replace(
            "<sum1:RechazoPrevio>S</sum1:RechazoPrevio>",
            "<sum1:RechazoPrevio>X</sum1:RechazoPrevio>",
        );
        assert_ne!(with_x, xml, "the record carries RechazoPrevio");
        assert!(quick_xml::de::from_str::<RegistroFacturacionAnulacion>(&with_x).is_err());
    }

    #[test]
    fn nif_is_exactly_nine_characters() {
        assert!(NIF::try_from("B12345678").is_ok());
        assert!(NIF::try_from("89890001K").is_ok());
        for invalid in ["", "X", "AAAA", "B1234567", "ES12345678", "B123456789"] {
            assert!(NIF::try_from(invalid).is_err(), "{invalid:?}");
            assert!(NIF::try_from(invalid.to_owned()).is_err(), "{invalid:?}");
        }
    }

    // The AEAT's own examples carry placeholders such as "AAAA": reading
    // stays lenient.
    #[test]
    fn nif_of_any_length_still_deserializes() {
        let xml = "<IDFactura><IDEmisorFactura>AAAA</IDEmisorFactura><NumSerieFactura>A-1</NumSerieFactura><FechaExpedicionFactura>15-03-2025</FechaExpedicionFactura></IDFactura>";
        let id: IDFactura = quick_xml::de::from_str(xml).expect("a short NIF is readable");
        assert_eq!(id.id_emisor_factura.as_str(), "AAAA");
    }

    #[test]
    fn num_serie_factura_cannot_be_empty_or_too_long() {
        assert!(TextoIDFactura::try_from("").is_err());
        assert!(TextoIDFactura::try_from("A").is_ok());
        assert!(TextoIDFactura::try_from("A".repeat(60)).is_ok());
        assert!(TextoIDFactura::try_from("A".repeat(61)).is_err());
    }

    // Validaciones 3.1.3.1, error 1104: ASCII 32 to 126 without " ' < > =.
    #[test]
    fn num_serie_factura_is_held_to_the_aeat_character_set() {
        for code in 0u32..=0x180 {
            let Some(c) = char::from_u32(code) else {
                continue;
            };
            let value = format!("A{c}1");
            let allowed = (32..=126).contains(&code) && !"\"'<>=".contains(c);
            assert_eq!(
                TextoIDFactura::try_from(value.as_str()).is_ok(),
                allowed,
                "character {code} in {value:?}"
            );
        }
        for valid in ["12345678/G33", "2025-1", "FRA 2025/0001", "A_1.2(3)#4"] {
            assert!(TextoIDFactura::try_from(valid).is_ok(), "{valid:?}");
        }
        for invalid in ["Fra. Nº 1", "F=2025-1", "FRA-ñ", "A\"1\"", "A'1", "<1>"] {
            assert!(TextoIDFactura::try_from(invalid).is_err(), "{invalid:?}");
        }
        // A huella field: a space is allowed inside, never at either end.
        assert!(TextoIDFactura::try_from(" A1").is_err());
        assert!(TextoIDFactura::try_from("A1 ").is_err());
    }

    // What another system registered stays readable, whatever its characters.
    #[test]
    fn num_serie_factura_outside_the_character_set_still_deserializes() {
        let xml = "<IDFactura><IDEmisorFactura>B12345678</IDEmisorFactura><NumSerieFactura>Fra. Nº 1</NumSerieFactura><FechaExpedicionFactura>15-03-2025</FechaExpedicionFactura></IDFactura>";
        let id: IDFactura = quick_xml::de::from_str(xml).expect("the reference is readable");
        assert_eq!(id.num_serie_factura.as_str(), "Fra. Nº 1");
    }

    // ImporteTotal is ImporteSgn12.2Type: twelve integer digits, not fourteen.
    #[test]
    fn importe_total_has_twelve_integer_digits() {
        let twelve: ImporteSgn12_2 = "-123456789012.34".try_into().expect("twelve digits fit");
        let registro = RegistroFacturacionAlta {
            importe_total: twelve,
            ..alta_record()
        };
        assert_eq!(registro.importe_total.as_str(), "-123456789012.34");
        assert!(ImporteSgn12_2::try_from("1234567890123.00").is_err());

        let xml = quick_xml::se::to_string_with_root("RegistroAlta", &alta_record())
            .expect("an alta serializes");
        let too_long = xml.replace(
            "<sum1:ImporteTotal>121.00</sum1:ImporteTotal>",
            "<sum1:ImporteTotal>1234567890123.00</sum1:ImporteTotal>",
        );
        assert_ne!(too_long, xml, "the record carries ImporteTotal");
        assert!(quick_xml::de::from_str::<RegistroFacturacionAlta>(&too_long).is_err());
    }

    #[test]
    fn clave_regimen_is_the_xsd_enumeration() {
        let enumeration = [
            "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "14", "15", "17",
            "18", "19", "20", "21",
        ];
        for code in enumeration {
            let clave = IdOperacionesTrascendenciaTributaria::try_from(code)
                .expect("a code of the enumeration");
            assert_eq!(clave.as_str(), code);
            assert_eq!(clave.to_string(), code);
        }
        for invalid in ["", "1", "00", "12", "13", "16", "22", "99", "ZZZZ", " 01"] {
            assert!(
                IdOperacionesTrascendenciaTributaria::try_from(invalid).is_err(),
                "{invalid:?}"
            );
        }
    }

    #[test]
    fn clave_regimen_serializes_its_code() {
        let detalle = Detalle {
            clave_regimen: Some(IdOperacionesTrascendenciaTributaria::C08),
            ..Default::default()
        };
        let xml = quick_xml::se::to_string_with_root("DetalleDesglose", &detalle)
            .expect("a detalle serializes");
        assert!(xml.contains("<sum1:ClaveRegimen>08</sum1:ClaveRegimen>"));
        let parsed: Detalle = quick_xml::de::from_str(&xml).expect("the detalle reads back");
        assert_eq!(
            parsed.clave_regimen,
            Some(IdOperacionesTrascendenciaTributaria::C08)
        );
        let unknown = xml.replace(">08<", ">99<");
        assert!(quick_xml::de::from_str::<Detalle>(&unknown).is_err());
    }

    #[test]
    fn id_version_is_the_xsd_enumeration() {
        assert_eq!(IDVersion::try_from("1.0"), Ok(IDVersion::V1_0));
        for invalid in ["", "1", "1.1", "2.0", "1.0 "] {
            assert!(IDVersion::try_from(invalid).is_err(), "{invalid:?}");
        }

        let xml = quick_xml::se::to_string_with_root("RegistroAlta", &alta_record())
            .expect("an alta serializes");
        assert!(xml.contains("<sum1:IDVersion>1.0</sum1:IDVersion>"));
        let other = xml.replace(
            "<sum1:IDVersion>1.0</sum1:IDVersion>",
            "<sum1:IDVersion>2.0</sum1:IDVersion>",
        );
        assert!(quick_xml::de::from_str::<RegistroFacturacionAlta>(&other).is_err());
    }

    #[test]
    fn fecha_hora_huso_requires_seconds_and_a_time_zone() {
        for valid in [
            "2024-01-01T19:20:30+01:00",
            "2024-01-01T19:20:30-05:30",
            "2024-01-01T19:20:30+00:00",
            "2024-01-01T19:20:30Z",
            "2024-02-29T23:59:59+14:00",
        ] {
            let fecha = FechaHoraHuso::try_from(valid).expect("a valid timestamp");
            assert_eq!(fecha.as_str(), valid);
        }
        for invalid in [
            "",
            "garbage",
            "2024-01-01",
            "2024-01-01T19:20:30",
            "2024-01-01 19:20:30+01:00",
            "2024-01-01T19:20+01:00",
            "2024-01-01T19:20:30.123+01:00",
            "2024-01-01T19:20:30+0100",
            "2024-01-01T19:20:30+01",
            "2024-01-01T19:20:30+15:00",
            "2024-13-01T19:20:30+01:00",
            "2023-02-29T19:20:30+01:00",
            "2024-01-01T24:20:30+01:00",
            "2024-01-01T19:60:30+01:00",
            "2024-01-01T19:20:60+01:00",
            "24-01-01T19:20:30+01:00",
            "02024-01-01T19:20:30+01:00",
            "-2024-01-01T19:20:30+01:00",
            " 2024-01-01T19:20:30+01:00",
            "2024-01-01T19:20:30+01:00 ",
            "01-01-2024T19:20:30+01:00",
        ] {
            assert!(FechaHoraHuso::try_from(invalid).is_err(), "{invalid:?}");
        }
    }

    #[test]
    fn fecha_hora_huso_is_formatted_once_from_a_date_time() {
        use chrono::TimeZone;

        let madrid = chrono::FixedOffset::east_opt(3600).expect("a valid offset");
        let date_time = madrid
            .with_ymd_and_hms(2024, 1, 1, 19, 20, 30)
            .single()
            .expect("an unambiguous instant");
        let fecha = FechaHoraHuso::from_datetime(&date_time).expect("a valid timestamp");
        assert_eq!(fecha.as_str(), "2024-01-01T19:20:30+01:00");

        // Sub-second precision is dropped, and UTC is written as an offset.
        let utc = chrono::Utc
            .with_ymd_and_hms(2025, 3, 15, 9, 0, 0)
            .single()
            .expect("an unambiguous instant")
            + chrono::Duration::milliseconds(750);
        let fecha = FechaHoraHuso::from_datetime(&utc).expect("a valid timestamp");
        assert_eq!(fecha.as_str(), "2025-03-15T09:00:00+00:00");
    }

    // The response side is xs:dateTime, which allows fractional seconds and a
    // missing time zone. The value is kept exactly as received.
    #[test]
    fn fecha_hora_huso_reads_any_xs_date_time() {
        for received in [
            "2024-11-27T11:54:10+01:00",
            "2024-11-27T11:54:10.123+01:00",
            "2024-11-27T11:54:10",
            "2024-11-27T10:54:10Z",
            "2024-11-27T24:00:00",
        ] {
            let xml =
                format!("<R><FechaHoraHusoGenRegistro>{received}</FechaHoraHusoGenRegistro></R>");
            let datos: RespuestaDatosRegistroFacturacion =
                quick_xml::de::from_str(&xml).expect("an xs:dateTime is readable");
            assert_eq!(
                datos.fecha_hora_huso_gen_registro.as_deref(),
                Some(received)
            );
        }
        let xml = "<R><FechaHoraHusoGenRegistro>garbage</FechaHoraHusoGenRegistro></R>";
        assert!(quick_xml::de::from_str::<RespuestaDatosRegistroFacturacion>(xml).is_err());
    }

    // The typed timestamp is sent and hashed as the exact string it was built
    // from: the first record of the AEAT huella documentation, through the
    // full struct.
    #[test]
    fn typed_timestamp_keeps_the_huella_byte_identical() {
        use chrono::TimeZone;

        let madrid = chrono::FixedOffset::east_opt(3600).expect("a valid offset");
        let date_time = madrid
            .with_ymd_and_hms(2024, 1, 1, 19, 20, 30)
            .single()
            .expect("an unambiguous instant");
        let registro = RegistroFacturacionAlta {
            id_factura: IDFactura {
                id_emisor_factura: "89890001K".try_into().expect("a valid NIF"),
                num_serie_factura: "12345678/G33".try_into().expect("a valid reference"),
                fecha_expedicion_factura: "01-01-2024".try_into().expect("a valid date"),
            },
            cuota_total: "12.35".try_into().expect("a valid amount"),
            importe_total: "123.45".try_into().expect("a valid amount"),
            fecha_hora_huso_gen_registro: FechaHoraHuso::from_datetime(&date_time)
                .expect("a valid timestamp"),
            ..alta_record()
        };
        assert_eq!(
            registro.hash(None),
            "3C464DAF61ACB827C65FDA19F352A4E3BDC2C640E9E9FC4CC058073F38F12F60"
        );
        let xml = quick_xml::se::to_string_with_root("RegistroAlta", &registro)
            .expect("an alta serializes");
        assert!(xml.contains(
            "<sum1:FechaHoraHusoGenRegistro>2024-01-01T19:20:30+01:00</sum1:FechaHoraHusoGenRegistro>"
        ));
    }

    // PersonaFisicaJuridicaESType: NombreRazon and NIF, nothing else.
    #[test]
    fn cabecera_personas_are_nif_only() {
        let cabecera = Cabecera {
            representante: Some(PersonaFisicaJuridicaES {
                nombre_razon: "Asesor SL".try_into().expect("a valid name"),
                nif: "B87654321".try_into().expect("a valid NIF"),
            }),
            ..cabecera()
        };
        let xml = quick_xml::se::to_string_with_root("Cabecera", &cabecera)
            .expect("a cabecera serializes");
        assert!(xml.contains(
            "<sum1:ObligadoEmision><sum1:NombreRazon>Emisor SL</sum1:NombreRazon><sum1:NIF>B12345678</sum1:NIF></sum1:ObligadoEmision>"
        ));
        assert!(xml.contains(
            "<sum1:Representante><sum1:NombreRazon>Asesor SL</sum1:NombreRazon><sum1:NIF>B87654321</sum1:NIF></sum1:Representante>"
        ));
        let parsed: Cabecera = quick_xml::de::from_str(&xml).expect("the cabecera reads back");
        assert_eq!(parsed.obligado_emision.nif.as_str(), "B12345678");

        let id_otro = "<sum1:IDOtro><sum1:CodigoPais>FR</sum1:CodigoPais><sum1:IDType>02</sum1:IDType><sum1:ID>FR12345678901</sum1:ID></sum1:IDOtro>";
        let foreign = xml.replace("<sum1:NIF>B12345678</sum1:NIF>", id_otro);
        assert_ne!(foreign, xml, "the cabecera carries the NIF");
        assert!(quick_xml::de::from_str::<Cabecera>(&foreign).is_err());
    }

    // ---- negative ImporteSgn amounts ------------------------------------

    #[test]
    fn negative_importe_sgn12_2_is_accepted() {
        for value in ["-50.00", "-100", "-0.50", "-1234567890.99"] {
            let importe = ImporteSgn12_2::try_from(value)
                .unwrap_or_else(|err| panic!("{value} should be valid: {err}"));
            assert_eq!(importe.as_ref(), value);
        }
    }

    #[test]
    fn negative_importe_sgn14_2_is_accepted() {
        let importe = ImporteSgn14_2::try_from("-12345678901234.56").expect("valid");
        assert_eq!(importe.as_ref(), "-12345678901234.56");
    }

    #[test]
    fn sign_without_digits_is_rejected() {
        assert!(ImporteSgn12_2::try_from("-").is_err());
    }

    #[test]
    fn negative_amounts_serialize_with_their_sign() {
        // A credit-note style detalle carrying negative base and quota.
        let detalle = Detalle {
            base_imponible_o_importe_no_sujeto: "-100.00".try_into().unwrap(),
            cuota_repercutida: Some("-21.00".try_into().unwrap()),
            ..Default::default()
        };
        let alta = RegistroFacturacionAlta {
            desglose: Desglose::new(vec![detalle]).unwrap(),
            cuota_total: "-21.00".try_into().unwrap(),
            importe_total: "-121.00".try_into().unwrap(),
            ..alta_record()
        };
        let xml = quick_xml::se::to_string(&alta).expect("valid xml");
        assert!(
            xml.contains(
                "<sum1:BaseImponibleOimporteNoSujeto>-100.00</sum1:BaseImponibleOimporteNoSujeto>"
            ),
            "{xml}"
        );
        assert!(
            xml.contains("<sum1:CuotaTotal>-21.00</sum1:CuotaTotal>"),
            "{xml}"
        );
        assert!(
            xml.contains("<sum1:ImporteTotal>-121.00</sum1:ImporteTotal>"),
            "{xml}"
        );
    }
}

#[cfg(test)]
mod outer_whitespace_tests {
    use super::*;

    #[test]
    fn huella_types_reject_outer_whitespace() {
        for padded in [" A-1", "A-1 ", "\tA-1", "A-1\n"] {
            assert!(TextMax60::try_from(padded).is_err(), "{padded:?}");
            assert!(NIF::try_from(padded).is_err(), "{padded:?}");
        }
        assert!(TextMax60::try_from("A 1").is_ok());
        assert!(NIF::try_from("B12345678").is_ok());
    }

    #[test]
    fn other_string_types_keep_whitespace() {
        assert!(TextMax500::try_from(" Servicios ").is_ok());
    }

    // Reading is lenient: a padded record in an AEAT response is kept as
    // received rather than failing the whole response.
    #[test]
    fn padded_values_in_a_response_still_deserialize() {
        let xml = "<IDFactura><IDEmisorFactura> B12345678 </IDEmisorFactura><NumSerieFactura> A-1 </NumSerieFactura><FechaExpedicionFactura>15-03-2025</FechaExpedicionFactura></IDFactura>";
        let id: IDFactura = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(id.num_serie_factura.as_str(), " A-1 ");
        assert_eq!(id.id_emisor_factura.as_str(), " B12345678 ");
    }

    #[test]
    fn response_length_is_still_checked() {
        let xml = format!(
            "<IDFactura><IDEmisorFactura>B12345678</IDEmisorFactura><NumSerieFactura>{}</NumSerieFactura><FechaExpedicionFactura>15-03-2025</FechaExpedicionFactura></IDFactura>",
            "A".repeat(61)
        );
        assert!(quick_xml::de::from_str::<IDFactura>(&xml).is_err());
    }
}
