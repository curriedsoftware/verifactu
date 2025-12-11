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
use serde::Deserialize;
use verifactu::schema::{
    ConsultaFactuSistemaFacturacion, RegistroFactura, RegistroFacturacionAlta, SiNo,
    SoapEnvelopeConsulta, SoapEnvelopeReg, SuministroInformacion, TipoFactura,
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
fn parses_voluntary_alta_normal_example() {
    let suministro = parse_suministro(VOLUNTARIA_ALTA_NORMAL);
    let alta = expect_single_alta(&suministro);
    assert!(matches!(alta.tipo_factura, TipoFactura::F1));
    assert_eq!(alta.desglose.detalle_desglose.len(), 2);
    assert!(alta.destinatarios.as_ref().is_some());
}

#[test]
fn parses_voluntary_subsanacion_example() {
    let suministro = parse_suministro(VOLUNTARIA_SUBSANACION);
    let alta = expect_single_alta(&suministro);
    assert!(matches!(alta.subsanacion, Some(SiNo::S)));
}

#[test]
fn parses_voluntary_alta_post_rechazo_example() {
    let suministro = parse_suministro(VOLUNTARIA_ALTA_POST_RECHAZO);
    let alta = expect_single_alta(&suministro);
    assert!(matches!(alta.rechazo_previo, Some(SiNo::X)));
}

#[test]
fn parses_voluntary_anulacion_example() {
    let suministro = parse_suministro(VOLUNTARIA_ANULACION);
    match suministro.registro_factura.first().expect("registro") {
        RegistroFactura::Anulacion(_) => {}
        _ => panic!("expected cancellation"),
    }
}

#[test]
fn parses_voluntary_anulacion_post_rechazo_example() {
    let suministro = parse_suministro(VOLUNTARIA_ANULACION_POST_RECHAZO);
    match suministro.registro_factura.first().expect("registro") {
        RegistroFactura::Anulacion(anulacion) => {
            assert!(matches!(anulacion.rechazo_previo, Some(SiNo::S)));
        }
        _ => panic!("expected cancellation"),
    }
}

#[test]
fn parses_voluntary_anulacion_sin_registro_example() {
    let suministro = parse_suministro(VOLUNTARIA_ANULACION_SIN_REGISTRO);
    match suministro.registro_factura.first().expect("registro") {
        RegistroFactura::Anulacion(anulacion) => {
            assert!(matches!(anulacion.sin_registro_previo, Some(SiNo::S)));
        }
        _ => panic!("expected cancellation"),
    }
}

#[test]
fn parses_voluntary_envio_agrupado_example() {
    let suministro = parse_suministro(VOLUNTARIA_ENVIO_AGRUPADO);
    assert_eq!(suministro.registro_factura.len(), 3);
    let altas = suministro
        .registro_factura
        .iter()
        .filter(|reg| matches!(reg, RegistroFactura::Alta(_)))
        .count();
    let anulaciones = suministro
        .registro_factura
        .iter()
        .filter(|reg| matches!(reg, RegistroFactura::Anulacion(_)))
        .count();
    assert_eq!(altas, 2);
    assert_eq!(anulaciones, 1);
}

#[test]
fn parses_requerimiento_alta_example() {
    let suministro = parse_suministro(REQUERIMIENTO_ALTA_NORMAL);
    assert!(
        suministro
            .cabecera
            .remision_requerimiento
            .as_ref()
            .is_some()
    );
}

#[test]
fn parses_requerimiento_subsanacion_example() {
    let suministro = parse_suministro(REQUERIMIENTO_SUBSANACION);
    let alta = expect_single_alta(&suministro);
    assert!(matches!(alta.subsanacion, Some(SiNo::S)));
    assert!(
        suministro
            .cabecera
            .remision_requerimiento
            .as_ref()
            .is_some()
    );
}

#[test]
fn parses_requerimiento_anulacion_example() {
    let suministro = parse_suministro(REQUERIMIENTO_ANULACION);
    match suministro.registro_factura.first().expect("registro") {
        RegistroFactura::Anulacion(_) => {}
        _ => panic!("expected cancellation"),
    }
    assert!(
        suministro
            .cabecera
            .remision_requerimiento
            .as_ref()
            .is_some()
    );
}

#[test]
fn parses_consulta_basica_example() {
    let consulta = parse_consulta(CONSULTA_BASICA);
    assert!(consulta.cabecera.obligado_emision.is_some());
    let periodo = consulta.filtro_consulta.periodo_imputacion;
    assert_eq!(periodo.ejercicio.as_str(), "2024");
}

#[test]
fn parses_consulta_filtrada_destinatario_example() {
    let consulta = parse_consulta(CONSULTA_FILTRADA_DESTINATARIO);
    assert!(consulta.filtro_consulta.contraparte.is_some());
}

#[test]
fn parses_consulta_filtrada_factura_example() {
    let consulta = parse_consulta(CONSULTA_FILTRADA_FACTURA);
    let rango = consulta
        .filtro_consulta
        .fecha_expedicion_factura
        .as_ref()
        .and_then(|f| f.rango_fecha_expedicion.as_ref())
        .expect("rango fecha");
    assert_eq!(rango.desde.as_str(), "02-11-2024");
    assert_eq!(rango.hasta.as_str(), "13-11-2024");
}

#[test]
fn parses_consulta_paginada_example() {
    let consulta = parse_consulta(CONSULTA_PAGINADA);
    assert!(consulta.cabecera.destinatario.is_some());
    assert!(consulta.filtro_consulta.clave_paginacion.is_some());
}

fn parse_suministro(xml: &str) -> SuministroInformacion {
    let envelope: SoapEnvelopeReg<SuministroInformacion> = from_str(xml).expect("valid xml");
    envelope.body.payload
}

fn parse_consulta(xml: &str) -> ConsultaFactuSistemaFacturacion {
    let envelope: SoapEnvelopeConsulta<ConsultaFactuSistemaFacturacion> =
        from_str(xml).expect("valid xml");
    envelope.body.payload
}

fn expect_single_alta<'a>(suministro: &'a SuministroInformacion) -> &'a RegistroFacturacionAlta {
    assert_eq!(suministro.registro_factura.len(), 1);
    match suministro.registro_factura.first().expect("registro") {
        RegistroFactura::Alta(alta) => alta,
        _ => panic!("expected alta"),
    }
}
