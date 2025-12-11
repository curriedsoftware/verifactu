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

impl Hashable for RegistroFacturacionAlta {
    fn hash(&self, prev_huella: Option<&str>) -> String {
        format!(
            "{:X}",
            Sha256::digest(format!(
                "IDEmisorFactura={}&NumSerieFactura={}&FechaExpedicionFactura={}&TipoFactura={}&CuotaTotal={}&ImporteTotal={}&Huella={}&FechaHoraHusoGenRegistro={}",
                &self.id_factura.id_emisor_factura,
                &self.id_factura.num_serie_factura,
                &self.id_factura.fecha_expedicion_factura,
                &self.tipo_factura,
                &self.cuota_total,
                &self.importe_total,
                prev_huella.unwrap_or(""),
                &self.fecha_hora_huso_gen_registro,
            ))
        )
    }
}

impl Hashable for RegistroFacturacionAnulacion {
    fn hash(&self, prev_huella: Option<&str>) -> String {
        format!(
            "{:X}",
            Sha256::digest(format!(
                "IDEmisorFacturaAnulada={}&NumSerieFacturaAnulada={}&FechaExpedicionFacturaAnulada={}&Huella={}&FechaHoraHusoGenRegistro={}",
                &self.id_factura.id_emisor_factura_anulada,
                &self.id_factura.num_serie_factura_anulada,
                &self.id_factura.fecha_expedicion_factura_anulada,
                prev_huella.unwrap_or(""),
                &self.fecha_hora_huso_gen_registro,
            ))
        )
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
