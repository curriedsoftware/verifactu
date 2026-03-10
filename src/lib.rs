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

use schema::IntoSoapXml;

#[macro_export]
macro_rules! request {
    ($client: ident, $record: ident, $endpoint: path, $envelope_type: ty) => {{
        let xml_payload = $record.to_xml();
        let namespaces = $record.soap_envelope_namespaces();
        let xml_body = format!(
            "<?xml version=\"1.0\"?>\n\
             <soapenv:Envelope {namespaces}><soapenv:Header/><soapenv:Body>\
             {payload}\
             </soapenv:Body></soapenv:Envelope>",
            namespaces = namespaces,
            payload = xml_payload
        );

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

        let envelope: $envelope_type = quick_xml::de::from_str(&response_text)
            .map_err(|err| errors::Error::RequestError(format!("{:?}", err)))?;

        Ok(envelope.body.payload)
    }};
}

#[macro_export]
macro_rules! log_request {
    ($record: ident, $endpoint: path) => {{
        let xml_payload = $record.to_xml();
        let namespaces = $record.soap_envelope_namespaces();
        let xml_body = format!(
            "<?xml version=\"1.0\"?>\n\
             <soapenv:Envelope {namespaces}><soapenv:Header/><soapenv:Body>\
             {payload}\
             </soapenv:Body></soapenv:Envelope>",
            namespaces = namespaces,
            payload = xml_payload
        );

        println!("Would have sent the following request to {}", $endpoint);
        println!("{}", xml_body);
    }};
}

pub async fn alta(
    client: &reqwest::Client,
    record: &schema::SuministroInformacion,
) -> Result<schema::RespuestaSuministro, errors::Error> {
    request!(
        client,
        record,
        endpoints::SISTEMA_VERIFACTU,
        schema::SoapEnvelopeRespuestaReg<schema::RespuestaSuministro>
    )
}

pub async fn alta_subsanacion(
    client: &reqwest::Client,
    record: &schema::SuministroInformacion,
) -> Result<schema::RespuestaSuministro, errors::Error> {
    request!(
        client,
        record,
        endpoints::SISTEMA_VERIFACTU,
        schema::SoapEnvelopeRespuestaReg<schema::RespuestaSuministro>
    )
}

pub async fn alta_por_rechazo(
    client: &reqwest::Client,
    record: &schema::SuministroInformacion,
) -> Result<schema::RespuestaSuministro, errors::Error> {
    request!(
        client,
        record,
        endpoints::SISTEMA_VERIFACTU,
        schema::SoapEnvelopeRespuestaReg<schema::RespuestaSuministro>
    )
}

pub async fn anulacion(
    client: &reqwest::Client,
    record: &schema::SuministroInformacion,
) -> Result<schema::RespuestaSuministro, errors::Error> {
    request!(
        client,
        record,
        endpoints::SISTEMA_VERIFACTU,
        schema::SoapEnvelopeRespuestaReg<schema::RespuestaSuministro>
    )
}

pub async fn anulacion_tras_rechazo(
    client: &reqwest::Client,
    record: &schema::SuministroInformacion,
) -> Result<schema::RespuestaSuministro, errors::Error> {
    request!(
        client,
        record,
        endpoints::SISTEMA_VERIFACTU,
        schema::SoapEnvelopeRespuestaReg<schema::RespuestaSuministro>
    )
}

pub async fn anulacion_registro_desconocido(
    client: &reqwest::Client,
    record: &schema::SuministroInformacion,
) -> Result<schema::RespuestaSuministro, errors::Error> {
    request!(
        client,
        record,
        endpoints::SISTEMA_VERIFACTU,
        schema::SoapEnvelopeRespuestaReg<schema::RespuestaSuministro>
    )
}

pub async fn requerimiento_alta(
    client: &reqwest::Client,
    record: &schema::SuministroInformacion,
) -> Result<schema::RespuestaSuministro, errors::Error> {
    request!(
        client,
        record,
        endpoints::SISTEMA_REQUERIMIENTO,
        schema::SoapEnvelopeRespuestaReg<schema::RespuestaSuministro>
    )
}

pub async fn requerimiento_alta_subsanacion(
    client: &reqwest::Client,
    record: &schema::SuministroInformacion,
) -> Result<schema::RespuestaSuministro, errors::Error> {
    request!(
        client,
        record,
        endpoints::SISTEMA_REQUERIMIENTO,
        schema::SoapEnvelopeRespuestaReg<schema::RespuestaSuministro>
    )
}

pub async fn requerimiento_anulacion(
    client: &reqwest::Client,
    record: &schema::SuministroInformacion,
) -> Result<schema::RespuestaSuministro, errors::Error> {
    request!(
        client,
        record,
        endpoints::SISTEMA_REQUERIMIENTO,
        schema::SoapEnvelopeRespuestaReg<schema::RespuestaSuministro>
    )
}

pub async fn consulta(
    client: &reqwest::Client,
    record: &schema::ConsultaFactuSistemaFacturacion,
) -> Result<schema::RespuestaConsultaLR, errors::Error> {
    request!(
        client,
        record,
        endpoints::SISTEMA_VERIFACTU,
        schema::SoapEnvelopeRespuestaConsulta<schema::RespuestaConsultaLR>
    )
}
