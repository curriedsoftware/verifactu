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

use verifactu::schema::{Incidencia, RemisionVoluntaria};

/// Ordinary continuous submission: only `Incidencia` is set. The optional
/// `FechaFinVeriFactu` is omitted (it is only emitted on the final submission
/// when the taxpayer ceases VeriFactu use).
#[test]
fn remision_voluntaria_without_fecha_fin_omits_the_element() {
    let remision = RemisionVoluntaria {
        fecha_fin_veri_factu: None,
        incidencia: Incidencia::N,
    };

    let xml = quick_xml::se::to_string(&remision).expect("valid xml");
    assert!(
        !xml.contains("FechaFinVeriFactu"),
        "FechaFinVeriFactu must be omitted when None, got {xml}"
    );
    assert!(
        xml.contains("<sum1:Incidencia>N</sum1:Incidencia>"),
        "{xml}"
    );
}

/// Final submission: both children are present, `FechaFinVeriFactu` first.
#[test]
fn remision_voluntaria_with_fecha_fin_includes_the_element() {
    let remision = RemisionVoluntaria {
        fecha_fin_veri_factu: Some("31-12-2025".try_into().expect("valid fecha")),
        incidencia: Incidencia::S,
    };

    let xml = quick_xml::se::to_string(&remision).expect("valid xml");
    assert!(
        xml.contains("<sum1:FechaFinVeriFactu>31-12-2025</sum1:FechaFinVeriFactu>"),
        "{xml}"
    );
    assert!(
        xml.contains("<sum1:Incidencia>S</sum1:Incidencia>"),
        "{xml}"
    );
}
