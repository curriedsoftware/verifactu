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

use sha2::{Digest, Sha256};

use crate::schema::{RegistroFactura, RegistroFacturacionAlta, RegistroFacturacionAnulacion};

pub trait Hashable {
    fn hash(&self, prev_huella: Option<&str>) -> String;
}

/// A field value as it enters the huella: the AEAT specification (v0.1.2)
/// strips leading and trailing whitespace from each value before concatenating
/// them, and nothing inside the value.
fn value(field: &str) -> &str {
    field.trim()
}

/// The components AEAT concatenates to compute an **Alta** record's huella.
///
/// Each value is trimmed of leading and trailing whitespace as it is hashed, as
/// the specification requires, so a padded value hashes like the bare one. The
/// record still carries the value as given: trimming is what the hash does, not
/// what the XML sends.
///
/// Grouping them into a single input struct keeps the huella format defined in
/// exactly one place: both the [`Hashable`] impl for [`RegistroFacturacionAlta`]
/// (used when building a record to submit) and external recomputation (e.g.
/// verifying the integrity of a chain returned by the AEAT `consulta` endpoint)
/// build one of these and call [`AltaHuellaInput::huella`], so the byte layout
/// fed to SHA-256 can never drift between the two.
pub struct AltaHuellaInput<'a> {
    pub id_emisor_factura: &'a str,
    pub num_serie_factura: &'a str,
    pub fecha_expedicion_factura: &'a str,
    pub tipo_factura: &'a str,
    pub cuota_total: &'a str,
    pub importe_total: &'a str,
    /// Previous record's huella; `None` (rendered as the empty string) for the
    /// first record of the chain.
    pub prev_huella: Option<&'a str>,
    pub fecha_hora_huso_gen_registro: &'a str,
}

impl AltaHuellaInput<'_> {
    /// Compute the SHA-256 huella for this Alta record.
    pub fn huella(&self) -> String {
        format!(
            "{:X}",
            Sha256::digest(format!(
                "IDEmisorFactura={}&NumSerieFactura={}&FechaExpedicionFactura={}&TipoFactura={}&CuotaTotal={}&ImporteTotal={}&Huella={}&FechaHoraHusoGenRegistro={}",
                value(self.id_emisor_factura),
                value(self.num_serie_factura),
                value(self.fecha_expedicion_factura),
                value(self.tipo_factura),
                value(self.cuota_total),
                value(self.importe_total),
                value(self.prev_huella.unwrap_or("")),
                value(self.fecha_hora_huso_gen_registro),
            ))
        )
    }
}

/// The components AEAT concatenates to compute an **Anulación** record's huella.
/// Single source of truth for the Anulación huella format, trimmed the same
/// way; see [`AltaHuellaInput`] for the rationale.
pub struct AnulacionHuellaInput<'a> {
    pub id_emisor_factura_anulada: &'a str,
    pub num_serie_factura_anulada: &'a str,
    pub fecha_expedicion_factura_anulada: &'a str,
    /// Previous record's huella; `None` (rendered as the empty string) for the
    /// first record of the chain.
    pub prev_huella: Option<&'a str>,
    pub fecha_hora_huso_gen_registro: &'a str,
}

impl AnulacionHuellaInput<'_> {
    /// Compute the SHA-256 huella for this Anulación record.
    pub fn huella(&self) -> String {
        format!(
            "{:X}",
            Sha256::digest(format!(
                "IDEmisorFacturaAnulada={}&NumSerieFacturaAnulada={}&FechaExpedicionFacturaAnulada={}&Huella={}&FechaHoraHusoGenRegistro={}",
                value(self.id_emisor_factura_anulada),
                value(self.num_serie_factura_anulada),
                value(self.fecha_expedicion_factura_anulada),
                value(self.prev_huella.unwrap_or("")),
                value(self.fecha_hora_huso_gen_registro),
            ))
        )
    }
}

impl Hashable for RegistroFacturacionAlta {
    fn hash(&self, prev_huella: Option<&str>) -> String {
        AltaHuellaInput {
            id_emisor_factura: &self.id_factura.id_emisor_factura.to_string(),
            num_serie_factura: self.id_factura.num_serie_factura.as_ref(),
            fecha_expedicion_factura: self.id_factura.fecha_expedicion_factura.as_ref(),
            tipo_factura: &self.tipo_factura.to_string(),
            cuota_total: self.cuota_total.as_ref(),
            importe_total: self.importe_total.as_ref(),
            prev_huella,
            fecha_hora_huso_gen_registro: &self.fecha_hora_huso_gen_registro,
        }
        .huella()
    }
}

impl Hashable for RegistroFacturacionAnulacion {
    fn hash(&self, prev_huella: Option<&str>) -> String {
        AnulacionHuellaInput {
            id_emisor_factura_anulada: &self.id_factura.id_emisor_factura_anulada.to_string(),
            num_serie_factura_anulada: self.id_factura.num_serie_factura_anulada.as_ref(),
            fecha_expedicion_factura_anulada: self
                .id_factura
                .fecha_expedicion_factura_anulada
                .as_ref(),
            prev_huella,
            fecha_hora_huso_gen_registro: &self.fecha_hora_huso_gen_registro,
        }
        .huella()
    }
}

impl Hashable for RegistroFactura {
    fn hash(&self, prev_huella: Option<&str>) -> String {
        match self {
            Self::Alta(registro_facturacion_alta) => registro_facturacion_alta.hash(prev_huella),
            Self::Anulacion(registro_facturacion_anulacion) => {
                registro_facturacion_anulacion.hash(prev_huella)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AltaHuellaInput, AnulacionHuellaInput};

    // First record of a chain: the canonical example from the AEAT huella
    // documentation. Locking this exact digest guards the byte layout fed to
    // SHA-256 against accidental drift.
    #[test]
    fn alta_first_record_matches_aeat_example() {
        let huella = AltaHuellaInput {
            id_emisor_factura: "89890001K",
            num_serie_factura: "12345678/G33",
            fecha_expedicion_factura: "01-01-2024",
            tipo_factura: "F1",
            cuota_total: "12.35",
            importe_total: "123.45",
            prev_huella: None,
            fecha_hora_huso_gen_registro: "2024-01-01T19:20:30+01:00",
        }
        .huella();
        assert_eq!(
            huella,
            "3C464DAF61ACB827C65FDA19F352A4E3BDC2C640E9E9FC4CC058073F38F12F60"
        );
    }

    // A chained Alta: the previous record's huella feeds the `Huella` field.
    #[test]
    fn alta_chained_record_includes_prev_huella() {
        let huella = AltaHuellaInput {
            id_emisor_factura: "89890001K",
            num_serie_factura: "12345679/G34",
            fecha_expedicion_factura: "02-01-2024",
            tipo_factura: "F1",
            cuota_total: "21.00",
            importe_total: "121.00",
            prev_huella: Some("3C464DAF61ACB827C65FDA19F352A4E3BDC2C640E9E9FC4CC058073F38F12F60"),
            fecha_hora_huso_gen_registro: "2024-01-02T19:20:30+01:00",
        }
        .huella();
        assert_eq!(
            huella,
            "A4D415F68A52925E62EF21E6AADDDEC9BB69C46D29E26B0CE14F728D9D31778A"
        );
    }

    // Anulación uses the `*Anulada` field names and omits the amount/tipo fields.
    #[test]
    fn anulacion_first_record_format() {
        let huella = AnulacionHuellaInput {
            id_emisor_factura_anulada: "89890001K",
            num_serie_factura_anulada: "12345678/G33",
            fecha_expedicion_factura_anulada: "01-01-2024",
            prev_huella: None,
            fecha_hora_huso_gen_registro: "2024-01-03T19:20:30+01:00",
        }
        .huella();
        assert_eq!(
            huella,
            "9AC16B332D14EDC61801758056AD892CB25005778B412E2064365655A48B7B2A"
        );
    }

    // The second record of the AEAT huella documentation's chain, chained onto
    // the first example above.
    #[test]
    fn alta_second_record_matches_aeat_example() {
        let huella = AltaHuellaInput {
            id_emisor_factura: "89890001K",
            num_serie_factura: "12345679/G34",
            fecha_expedicion_factura: "01-01-2024",
            tipo_factura: "F1",
            cuota_total: "12.35",
            importe_total: "123.45",
            prev_huella: Some("3C464DAF61ACB827C65FDA19F352A4E3BDC2C640E9E9FC4CC058073F38F12F60"),
            fecha_hora_huso_gen_registro: "2024-01-01T19:20:35+01:00",
        }
        .huella();
        assert_eq!(
            huella,
            "F7B94CFD8924EDFF273501B01EE5153E4CE8F259766F88CF6ACB8935802A2B97"
        );
    }

    // The anulación example of the AEAT huella documentation, cancelling the
    // second record and chained onto it.
    #[test]
    fn anulacion_matches_aeat_example() {
        let huella = AnulacionHuellaInput {
            id_emisor_factura_anulada: "89890001K",
            num_serie_factura_anulada: "12345679/G34",
            fecha_expedicion_factura_anulada: "01-01-2024",
            prev_huella: Some("F7B94CFD8924EDFF273501B01EE5153E4CE8F259766F88CF6ACB8935802A2B97"),
            fecha_hora_huso_gen_registro: "2024-01-01T19:20:40+01:00",
        }
        .huella();
        assert_eq!(
            huella,
            "177547C0D57AC74748561D054A9CEC14B4C4EA23D1BEFD6F2E69E3A388F90C68"
        );
    }

    // Leading and trailing whitespace on any value, of any kind (spaces, tabs,
    // newlines), hashes like the bare value: the first AEAT example, padded.
    #[test]
    fn alta_trims_each_value_before_hashing() {
        let huella = AltaHuellaInput {
            id_emisor_factura: " 89890001K",
            num_serie_factura: "12345678/G33 ",
            fecha_expedicion_factura: "\t01-01-2024",
            tipo_factura: "F1\n",
            cuota_total: "  12.35",
            importe_total: "123.45  ",
            prev_huella: Some("   "),
            fecha_hora_huso_gen_registro: " 2024-01-01T19:20:30+01:00 ",
        }
        .huella();
        assert_eq!(
            huella,
            "3C464DAF61ACB827C65FDA19F352A4E3BDC2C640E9E9FC4CC058073F38F12F60"
        );
    }

    // The anulación example, padded, hashes like the bare one.
    #[test]
    fn anulacion_trims_each_value_before_hashing() {
        let huella = AnulacionHuellaInput {
            id_emisor_factura_anulada: "89890001K ",
            num_serie_factura_anulada: " 12345679/G34",
            fecha_expedicion_factura_anulada: " 01-01-2024 ",
            prev_huella: Some(
                " F7B94CFD8924EDFF273501B01EE5153E4CE8F259766F88CF6ACB8935802A2B97\n",
            ),
            fecha_hora_huso_gen_registro: "\t2024-01-01T19:20:40+01:00",
        }
        .huella();
        assert_eq!(
            huella,
            "177547C0D57AC74748561D054A9CEC14B4C4EA23D1BEFD6F2E69E3A388F90C68"
        );
    }

    // Only the ends are trimmed: whitespace inside a value is part of it.
    #[test]
    fn inner_whitespace_is_kept() {
        let huella = |num_serie| {
            AltaHuellaInput {
                id_emisor_factura: "89890001K",
                num_serie_factura: num_serie,
                fecha_expedicion_factura: "01-01-2024",
                tipo_factura: "F1",
                cuota_total: "12.35",
                importe_total: "123.45",
                prev_huella: None,
                fecha_hora_huso_gen_registro: "2024-01-01T19:20:30+01:00",
            }
            .huella()
        };
        assert_ne!(huella("12345678 /G33"), huella("12345678/G33"));
        assert_eq!(huella(" 12345678 /G33 "), huella("12345678 /G33"));
    }

    // An empty `prev_huella` and `Some("")` must hash identically -- both render
    // an empty `Huella=` field.
    #[test]
    fn none_and_empty_prev_huella_are_equivalent() {
        let base = |prev| {
            AnulacionHuellaInput {
                id_emisor_factura_anulada: "89890001K",
                num_serie_factura_anulada: "A1",
                fecha_expedicion_factura_anulada: "01-01-2024",
                prev_huella: prev,
                fecha_hora_huso_gen_registro: "2024-01-03T00:00:00+01:00",
            }
            .huella()
        };
        assert_eq!(base(None), base(Some("")));
    }
}
