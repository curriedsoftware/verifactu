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

use quick_xml::de::from_str;
use verifactu::schema::{
    ConsultaFactuSistemaFacturacion, SoapEnvelopeConsulta, SoapEnvelopeReg, SuministroInformacion,
};

const VOLUNTARIA_ALTA_NORMAL: &str =
    include_str!("data/pdf_examples/remision_voluntaria_alta_normal.xml");
const VOLUNTARIA_SUBSANACION: &str =
    include_str!("data/pdf_examples/remision_voluntaria_subsanacion.xml");
const VOLUNTARIA_ALTA_POST_RECHAZO: &str =
    include_str!("data/pdf_examples/remision_voluntaria_alta_post_rechazo.xml");
const VOLUNTARIA_ANULACION: &str =
    include_str!("data/pdf_examples/remision_voluntaria_anulacion.xml");
const VOLUNTARIA_ANULACION_POST_RECHAZO: &str =
    include_str!("data/pdf_examples/remision_voluntaria_anulacion_post_rechazo.xml");
const VOLUNTARIA_ANULACION_SIN_REGISTRO: &str =
    include_str!("data/pdf_examples/remision_voluntaria_anulacion_sin_registro.xml");
const VOLUNTARIA_ENVIO_AGRUPADO: &str =
    include_str!("data/pdf_examples/remision_voluntaria_envio_agrupado.xml");
const REQUERIMIENTO_ALTA_NORMAL: &str =
    include_str!("data/pdf_examples/requerimiento_alta_normal.xml");
const REQUERIMIENTO_SUBSANACION: &str =
    include_str!("data/pdf_examples/requerimiento_subsanacion.xml");
const REQUERIMIENTO_ANULACION: &str = include_str!("data/pdf_examples/requerimiento_anulacion.xml");
const CONSULTA_BASICA: &str = include_str!("data/pdf_examples/consulta_presentadas_basica.xml");
const CONSULTA_FILTRADA_DESTINATARIO: &str =
    include_str!("data/pdf_examples/consulta_presentadas_filtrada_destinatario.xml");
const CONSULTA_FILTRADA_FACTURA: &str =
    include_str!("data/pdf_examples/consulta_presentadas_filtrada_factura.xml");
const CONSULTA_PAGINADA: &str = include_str!("data/pdf_examples/consulta_presentadas_paginada.xml");

#[test]
fn encodes_voluntary_alta_normal_example() {
    test_roundtrip_suministro(VOLUNTARIA_ALTA_NORMAL);
}

#[test]
fn encodes_voluntary_subsanacion_example() {
    test_roundtrip_suministro(VOLUNTARIA_SUBSANACION);
}

#[test]
fn encodes_voluntary_alta_post_rechazo_example() {
    test_roundtrip_suministro(VOLUNTARIA_ALTA_POST_RECHAZO);
}

#[test]
fn encodes_voluntary_anulacion_example() {
    test_roundtrip_suministro(VOLUNTARIA_ANULACION);
}

#[test]
fn encodes_voluntary_anulacion_post_rechazo_example() {
    test_roundtrip_suministro(VOLUNTARIA_ANULACION_POST_RECHAZO);
}

#[test]
fn encodes_voluntary_anulacion_sin_registro_example() {
    test_roundtrip_suministro(VOLUNTARIA_ANULACION_SIN_REGISTRO);
}

#[test]
fn encodes_voluntary_envio_agrupado_example() {
    test_roundtrip_suministro(VOLUNTARIA_ENVIO_AGRUPADO);
}

#[test]
fn encodes_requerimiento_alta_example() {
    test_roundtrip_suministro(REQUERIMIENTO_ALTA_NORMAL);
}

#[test]
fn encodes_requerimiento_subsanacion_example() {
    test_roundtrip_suministro(REQUERIMIENTO_SUBSANACION);
}

#[test]
fn encodes_requerimiento_anulacion_example() {
    test_roundtrip_suministro(REQUERIMIENTO_ANULACION);
}

#[test]
fn encodes_consulta_basica_example() {
    test_roundtrip_consulta(CONSULTA_BASICA);
}

#[test]
fn encodes_consulta_filtrada_destinatario_example() {
    test_roundtrip_consulta(CONSULTA_FILTRADA_DESTINATARIO);
}

#[test]
fn encodes_consulta_filtrada_factura_example() {
    test_roundtrip_consulta(CONSULTA_FILTRADA_FACTURA);
}

#[test]
fn encodes_consulta_paginada_example() {
    test_roundtrip_consulta(CONSULTA_PAGINADA);
}

fn test_roundtrip_suministro(xml: &str) {
    // Parse the original XML
    let envelope: SoapEnvelopeReg<SuministroInformacion> = from_str(&xml).expect("valid xml");

    // "La prueba del algodón", as we say in spanish
    let generated_xml = envelope.to_xml();
    let reparsed_envelope: SoapEnvelopeReg<SuministroInformacion> =
        from_str(&generated_xml).expect("valid xml");
    assert_eq!(generated_xml, reparsed_envelope.to_xml(),);
}

fn test_roundtrip_consulta(xml: &str) {
    // Parse the original XML
    let envelope: SoapEnvelopeConsulta<ConsultaFactuSistemaFacturacion> =
        from_str(xml).expect("valid xml");

    // "La prueba del algodón", as we say in spanish
    let generated_xml = envelope.to_xml();
    let reparsed_envelope: SoapEnvelopeConsulta<ConsultaFactuSistemaFacturacion> =
        from_str(&generated_xml).expect("valid xml");
    assert_eq!(generated_xml, reparsed_envelope.to_xml(),);
}
