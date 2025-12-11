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

pub mod endpoints;
pub mod errors;
pub mod hashing;
mod qr;
pub mod schema;

#[macro_export]
macro_rules! request {
    ($client: ident, $record: ident, $endpoint: path, $response_type: ty) => {{
        let xml_body = $record.to_xml();
        eprintln!("sending XML to {}", $endpoint);
        println!("{}", xml_body);

        let response_text = $client
            .post($endpoint)
            .header("Content-Type", "text/xml; charset=utf-8")
            .body(xml_body)
            .send()
            .await
            .map_err(|err| errors::Error::RequestError(format!("{:?}", err)))?
            .text()
            .await
            .map_err(|err| errors::Error::RequestError(format!("{:?}", err)))?;

        println!("response as text was {}", response_text);

        let envelope: schema::SoapEnvelopeRespuestaReg<$response_type> =
            quick_xml::de::from_str(&response_text)
                .map_err(|err| errors::Error::RequestError(format!("{:?}", err)))?;

        Ok(envelope.body.payload)
    }};
}

// pub async fn alta(
//     client: &Client,
//     record: &schema::SuministroInformacion,
// ) -> Result<RespuestaSuministro, errors::Error> {
//     request!(client, record, endpoints::SISTEMA_VERIFACTU)
// }

// pub async fn alta_subsanacion(
//     client: &Client,
//     record: &schema::SuministroInformacion,
// ) -> Result<RespuestaSuministro, errors::Error> {
//     request!(client, record, endpoints::SISTEMA_VERIFACTU)
// }

// pub async fn alta_por_rechazo(
//     client: &Client,
//     record: &schema::SuministroInformacion,
// ) -> Result<RespuestaSuministro, errors::Error> {
//     request!(client, record, endpoints::SISTEMA_VERIFACTU)
// }

// pub async fn anulacion(
//     client: &Client,
//     record: &schema::SuministroInformacion,
// ) -> Result<RespuestaSuministro, errors::Error> {
//     request!(client, record, endpoints::SISTEMA_VERIFACTU)
// }

// pub async fn anulacion_tras_rechazo(
//     client: &Client,
//     record: &schema::SuministroInformacion,
// ) -> Result<RespuestaSuministro, errors::Error> {
//     request!(client, record, endpoints::SISTEMA_VERIFACTU)
// }

// pub async fn anulacion_registro_desconocido(
//     client: &Client,
//     record: &schema::SuministroInformacion,
// ) -> Result<RespuestaSuministro, errors::Error> {
//     request!(client, record, endpoints::SISTEMA_VERIFACTU)
// }

// pub async fn requerimiento_alta(
//     client: &Client,
//     record: &schema::SuministroInformacion,
// ) -> Result<RespuestaSuministro, errors::Error> {
//     request!(client, record, endpoints::SISTEMA_REQUERIMIENTO)
// }

// pub async fn requerimiento_alta_subsanacion(
//     client: &Client,
//     record: &schema::SuministroInformacion,
// ) -> Result<RespuestaSuministro, errors::Error> {
//     request!(client, record, endpoints::SISTEMA_REQUERIMIENTO)
// }

// pub async fn requerimiento_anulacion(
//     client: &Client,
//     record: &schema::SuministroInformacion,
// ) -> Result<RespuestaSuministro, errors::Error> {
//     request!(client, record, endpoints::SISTEMA_REQUERIMIENTO)
// }

// pub async fn consulta(
//     client: &Client,
//     record: &schema::ConsultaFactuSistemaFacturacion,
// ) -> Result<RespuestaConsultaLR, errors::Error> {
//     request!(client, record, endpoints::SISTEMA_VERIFACTU)
// }
