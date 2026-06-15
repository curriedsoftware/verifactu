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

use crate::{endpoints::Environment, errors::Error};
use image::Luma;
use qrcode::QrCode;
use url::Url;
use urlencoding::encode;

use crate::schema::{RegistroFacturacionAlta, RegistroFacturacionAnulacion};

macro_rules! impl_qr {
    ($uri_expr:expr, $size:expr) => {{
        let uri = $uri_expr;
        let code = QrCode::new(&uri).map_err(|_| Error::QrCodeGenerationFailed)?;
        let image = code
            .render::<Luma<u8>>()
            .quiet_zone(false)
            .min_dimensions($size, $size)
            .build();

        let mut png_bytes = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_bytes);
        image
            .write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|_| Error::QrCodeGenerationFailed)?;

        png_bytes
    }};
}

impl RegistroFacturacionAlta {
    pub fn uri(&self, environment: Environment) -> String {
        let mut uri = Url::parse(environment.endpoints().qr_url).expect("valid URL");
        uri.set_query(Some(&format!(
            "nif={}&numserie={}&fecha={}&importe={}",
            encode(self.id_factura.id_emisor_factura.as_str()),
            encode(&self.id_factura.num_serie_factura),
            encode(&self.id_factura.fecha_expedicion_factura),
            encode(&self.importe_total),
        )));
        uri.to_string()
    }

    pub fn qr(&self, environment: Environment, size: u32) -> Result<Vec<u8>, Error> {
        Ok(impl_qr!(self.uri(environment), size))
    }
}

impl RegistroFacturacionAnulacion {
    pub fn uri(&self, environment: Environment, importe_total: &str) -> String {
        let mut uri = Url::parse(environment.endpoints().qr_url).expect("valid URL");
        uri.set_query(Some(&format!(
            "nif={}&numserie={}&fecha={}&importe={}",
            encode(self.id_factura.id_emisor_factura_anulada.as_str()),
            encode(&self.id_factura.num_serie_factura_anulada),
            encode(&self.id_factura.fecha_expedicion_factura_anulada),
            encode(importe_total),
        )));
        uri.to_string()
    }

    pub fn qr(
        &self,
        environment: Environment,
        importe_total: &str,
        size: u32,
    ) -> Result<Vec<u8>, Error> {
        Ok(impl_qr!(self.uri(environment, importe_total), size))
    }
}
