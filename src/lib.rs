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

pub use endpoints::{Endpoints, Environment};

/// A VeriFactu client bound to a specific [`Environment`].
///
/// The environment is fixed when the client is constructed and is carried on
/// every request, so the test-vs-production decision is explicit and visible at
/// the construction site rather than hidden in a build flag. Construct it with
/// the desired environment via [`Client::new`]; [`Environment`] defaults to
/// [`Environment::Test`], so the safe path requires no extra ceremony.
pub struct Client {
    http: reqwest::Client,
    environment: Environment,
}

impl Client {
    /// Build a client from an underlying `reqwest::Client` and the target
    /// [`Environment`].
    pub fn new(http: reqwest::Client, environment: Environment) -> Self {
        Self { http, environment }
    }

    /// The underlying HTTP client.
    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// The environment this client targets.
    pub fn environment(&self) -> Environment {
        self.environment
    }
}

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
    ($client: expr, $record: ident, $endpoint_field: ident, $envelope_type: ty) => {{
        let __environment = $client.environment();
        let endpoint = __environment.endpoints().$endpoint_field;
        let xml_body = $crate::__build_soap_xml!($record);

        tracing::debug!(endpoint, environment = ?__environment, body = %xml_body, "sending SOAP request");

        let response_text = $client
            .http()
            .post(endpoint)
            .header("Content-Type", "text/xml; charset=utf-8")
            .body(xml_body)
            .send()
            .await
            .map_err(|err| $crate::errors::Error::RequestError(format!("{:?}", err)))?
            .text()
            .await
            .map_err(|err| $crate::errors::Error::RequestError(format!("{:?}", err)))?;

        tracing::debug!(response = %response_text, "received SOAP response");

        match quick_xml::de::from_str::<$envelope_type>(&response_text) {
            Ok(envelope) => Ok(envelope.body.payload),
            Err(parse_err) => {
                // AEAT signals header/authorization/format errors with a SOAP
                // Fault in the body rather than the expected response element.
                // Surface the fault message instead of a misleading
                // "missing field" deserialization error.
                match quick_xml::de::from_str::<$crate::schema::SoapFaultEnvelope>(&response_text) {
                    Ok(fault_envelope) => Err($crate::errors::Error::SoapFault(fault_envelope.body.fault)),
                    Err(_) => Err($crate::errors::Error::RequestError(format!("{:?}", parse_err))),
                }
            }
        }
    }};
}

#[macro_export]
macro_rules! log_request {
    ($record: ident, $endpoint: expr) => {
        $crate::log_request!($record, $endpoint, tracing::Level::DEBUG)
    };
    ($record: ident, $endpoint: expr, $level: expr) => {{
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

impl Client {
    pub async fn alta(
        &self,
        record: &schema::SuministroInformacion,
    ) -> Result<schema::RespuestaSuministro, errors::Error> {
        request!(
            self,
            record,
            sistema_verifactu,
            schema::SoapEnvelopeRespuestaReg<schema::RespuestaSuministro>
        )
    }

    pub async fn alta_subsanacion(
        &self,
        record: &schema::SuministroInformacion,
    ) -> Result<schema::RespuestaSuministro, errors::Error> {
        request!(
            self,
            record,
            sistema_verifactu,
            schema::SoapEnvelopeRespuestaReg<schema::RespuestaSuministro>
        )
    }

    pub async fn alta_por_rechazo(
        &self,
        record: &schema::SuministroInformacion,
    ) -> Result<schema::RespuestaSuministro, errors::Error> {
        request!(
            self,
            record,
            sistema_verifactu,
            schema::SoapEnvelopeRespuestaReg<schema::RespuestaSuministro>
        )
    }

    pub async fn anulacion(
        &self,
        record: &schema::SuministroInformacion,
    ) -> Result<schema::RespuestaSuministro, errors::Error> {
        request!(
            self,
            record,
            sistema_verifactu,
            schema::SoapEnvelopeRespuestaReg<schema::RespuestaSuministro>
        )
    }

    pub async fn anulacion_tras_rechazo(
        &self,
        record: &schema::SuministroInformacion,
    ) -> Result<schema::RespuestaSuministro, errors::Error> {
        request!(
            self,
            record,
            sistema_verifactu,
            schema::SoapEnvelopeRespuestaReg<schema::RespuestaSuministro>
        )
    }

    pub async fn anulacion_registro_desconocido(
        &self,
        record: &schema::SuministroInformacion,
    ) -> Result<schema::RespuestaSuministro, errors::Error> {
        request!(
            self,
            record,
            sistema_verifactu,
            schema::SoapEnvelopeRespuestaReg<schema::RespuestaSuministro>
        )
    }

    pub async fn requerimiento_alta(
        &self,
        record: &schema::SuministroInformacion,
    ) -> Result<schema::RespuestaSuministro, errors::Error> {
        request!(
            self,
            record,
            sistema_requerimiento,
            schema::SoapEnvelopeRespuestaReg<schema::RespuestaSuministro>
        )
    }

    pub async fn requerimiento_alta_subsanacion(
        &self,
        record: &schema::SuministroInformacion,
    ) -> Result<schema::RespuestaSuministro, errors::Error> {
        request!(
            self,
            record,
            sistema_requerimiento,
            schema::SoapEnvelopeRespuestaReg<schema::RespuestaSuministro>
        )
    }

    pub async fn requerimiento_anulacion(
        &self,
        record: &schema::SuministroInformacion,
    ) -> Result<schema::RespuestaSuministro, errors::Error> {
        request!(
            self,
            record,
            sistema_requerimiento,
            schema::SoapEnvelopeRespuestaReg<schema::RespuestaSuministro>
        )
    }

    pub async fn consulta(
        &self,
        record: &schema::ConsultaFactuSistemaFacturacion,
    ) -> Result<schema::RespuestaConsultaLR, errors::Error> {
        request!(
            self,
            record,
            sistema_verifactu,
            schema::SoapEnvelopeRespuestaConsulta<schema::RespuestaConsultaLR>
        )
    }
}
