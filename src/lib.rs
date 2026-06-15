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

#[doc(hidden)]
#[macro_export]
macro_rules! __build_soap_xml {
    ($record: ident) => {{
        use $crate::schema::IntoSoapXml as _;
        let xml_payload = $record.to_xml();
        let namespaces = $record.soap_envelope_namespaces();
        format!(
            "<?xml version=\"1.0\"?>\n\
             <soapenv:Envelope {namespaces}><soapenv:Header/><soapenv:Body>\
             {payload}\
             </soapenv:Body></soapenv:Envelope>",
            namespaces = namespaces,
            payload = xml_payload
        )
    }};
}

#[macro_export]
macro_rules! request {
    ($client: ident, $record: ident, $endpoint: path, $envelope_type: ty) => {{
        let xml_body = $crate::__build_soap_xml!($record);

        tracing::debug!(endpoint = $endpoint, body = %xml_body, "sending SOAP request");

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

        tracing::debug!(response = %response_text, "received SOAP response");

        match quick_xml::de::from_str::<$envelope_type>(&response_text) {
            Ok(envelope) => Ok(envelope.body.payload),
            Err(parse_err) => {
                // AEAT signals header/authorization/format errors with a SOAP
                // Fault in the body rather than the expected response element.
                // Surface the fault message instead of a misleading
                // "missing field" deserialization error.
                match quick_xml::de::from_str::<$crate::schema::SoapFaultEnvelope>(&response_text) {
                    Ok(fault_envelope) => Err(errors::Error::SoapFault(fault_envelope.body.fault)),
                    Err(_) => Err(errors::Error::RequestError(format!("{:?}", parse_err))),
                }
            }
        }
    }};
}

#[macro_export]
macro_rules! log_request {
    ($record: ident, $endpoint: path) => {
        $crate::log_request!($record, $endpoint, tracing::Level::DEBUG)
    };
    ($record: ident, $endpoint: path, $level: expr) => {{
        let xml_body = $crate::__build_soap_xml!($record);

        match $level {
            tracing::Level::TRACE => tracing::trace!(endpoint = $endpoint, body = %xml_body, "would send SOAP request"),
            tracing::Level::DEBUG => tracing::debug!(endpoint = $endpoint, body = %xml_body, "would send SOAP request"),
            tracing::Level::INFO => tracing::info!(endpoint = $endpoint, body = %xml_body, "would send SOAP request"),
            tracing::Level::WARN => tracing::warn!(endpoint = $endpoint, body = %xml_body, "would send SOAP request"),
            tracing::Level::ERROR => tracing::error!(endpoint = $endpoint, body = %xml_body, "would send SOAP request"),
        }
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
