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
    RespuestaSuministro, SoapBodyRespuestaReg, SoapEnvelopeReg, SoapEnvelopeRespuestaReg,
    TipoFactura,
};

const BAD_NIF_HTTP_STATUS_CODE: &str = include_str!("data/recorded_responses/bad_nif.http-status");
const BAD_NIF_BODY: &str = include_str!("data/recorded_responses/bad_nif.xml");

fn parse_respuesta_suministro(xml: &str) -> RespuestaSuministro {
    let envelope: SoapEnvelopeRespuestaReg<RespuestaSuministro> = from_str(xml).expect("valid xml");
    envelope.body.payload
}

#[test]
fn parses_bad_nif_example() {
    let suministro = parse_respuesta_suministro(BAD_NIF_BODY);
    assert_eq!(
        quick_xml::se::to_string(&suministro).expect("valid xml"),
        quick_xml::se::to_string(
            &from_str::<RespuestaSuministro>(
                &quick_xml::se::to_string(&suministro).expect("valid xml")
            )
            .expect("valid xml")
        )
        .expect("valid xml")
    )
}
