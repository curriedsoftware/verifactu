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

#[derive(Debug)]
pub enum Error {
    RequestError(String),
    SoapFault(crate::schema::SoapFault),
    QrCodeGenerationFailed,
    IoError(std::io::Error),
    PemError(String),
    ReqwestError(reqwest::Error),
}

/// How AEAT treats a given backend error code.
///
/// Codes are documented in three groups in `errores.properties`, each with a
/// different effect on the submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendErrorCategory {
    /// The whole submission (envío) is rejected.
    SubmissionRejected,
    /// The invoice record is rejected (or the whole petition if the error is
    /// raised on the header).
    RecordRejected,
    /// The record is accepted into the system but must be corrected
    /// (subsanado) afterwards.
    AcceptedWithErrors,
}

/// A documented VeriFactu backend (AEAT) error code.
///
/// AEAT returns numeric error codes both in SOAP faults (prefixed as
/// `Codigo[NNNN].` in the fault string) and per-record in the `RespuestaLinea`
/// / consulta responses. This enum lets callers match on the specific code
/// instead of inspecting the human-readable Spanish description string.
///
/// The official list is published at:
/// <https://prewww2.aeat.es/static_files/common/internet/dep/aplicaciones/es/aeat/tikeV1.0/cont/ws/errores.properties>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendError {
    // --- Errors that reject the whole submission (envío) ---
    /// 4102: El XML no cumple el esquema. Falta informar campo obligatorio.
    SchemaMissingRequiredField,
    /// 4103: Se ha producido un error inesperado al parsear el XML.
    UnexpectedXmlParseError,
    /// 4104: Error en la cabecera: el valor del campo NIF del bloque
    /// ObligadoEmision no está identificado.
    HeaderObligadoEmisionNifNotIdentified,
    /// 4105: Error en la cabecera: el valor del campo NIF del bloque
    /// Representante no está identificado.
    HeaderRepresentanteNifNotIdentified,
    /// 4106: El formato de fecha es incorrecto.
    InvalidDateFormat,
    /// 4107: El NIF no está identificado en el censo de la AEAT.
    NifNotInCensus,
    /// 4108: Error técnico al obtener el certificado.
    CertificateRetrievalError,
    /// 4109: El formato del NIF es incorrecto.
    InvalidNifFormat,
    /// 4110: Error técnico al comprobar los apoderamientos.
    PowersOfAttorneyCheckError,
    /// 4111: Error técnico al crear el trámite.
    ProcedureCreationError,
    /// 4112: El titular del certificado debe ser Obligado Emisión, Colaborador
    /// Social, Apoderado o Sucesor.
    CertificateHolderNotAuthorized,
    /// 4113: El XML no cumple con el esquema: se ha superado el límite permitido
    /// de registros para el bloque.
    BlockRecordLimitExceeded,
    /// 4114: El XML no cumple con el esquema: se ha superado el límite máximo
    /// permitido de facturas a registrar.
    InvoiceRegistrationLimitExceeded,
    /// 4115: El valor del campo NIF del bloque ObligadoEmision es incorrecto.
    ObligadoEmisionNifInvalid,
    /// 4116: Error en la cabecera: el campo NIF del bloque ObligadoEmision tiene
    /// un formato incorrecto.
    HeaderObligadoEmisionNifFormatInvalid,
    /// 4117: Error en la cabecera: el campo NIF del bloque Representante tiene un
    /// formato incorrecto.
    HeaderRepresentanteNifFormatInvalid,
    /// 4118: Error técnico: la dirección no se corresponde con el fichero de
    /// entrada.
    AddressMismatchWithInputFile,
    /// 4119: Error al informar caracteres cuya codificación no es UTF-8.
    NonUtf8Encoding,
    /// 4120: Error en la cabecera: el valor del campo FechaFinVeriFactu es
    /// incorrecto, debe ser 31-12-20XX, donde XX corresponde con el año actual o
    /// el anterior.
    HeaderFechaFinVeriFactuInvalid,
    /// 4121: Error en la cabecera: el valor del campo Incidencia es incorrecto.
    HeaderIncidenciaInvalid,
    /// 4122: Error en la cabecera: el valor del campo RefRequerimiento es
    /// incorrecto.
    HeaderRefRequerimientoInvalid,
    /// 4123: Error en la cabecera: el valor del campo NIF del bloque
    /// Representante no está identificado en el censo de la AEAT.
    HeaderRepresentanteNifNotInCensus,
    /// 4124: Error en la cabecera: el valor del campo Nombre del bloque
    /// Representante no está identificado en el censo de la AEAT.
    HeaderRepresentanteNombreNotInCensus,
    /// 4125: Error en la cabecera: Si el envío es por requerimiento el campo
    /// RefRequerimiento es obligatorio.
    HeaderRefRequerimientoRequired,
    /// 4126: Error en la cabecera: el campo RefRequerimiento solo debe
    /// informarse en sistemas en remisiones al endpoint del servicio a usar para
    /// la contestación a requerimientos de registros de facturación.
    HeaderRefRequerimientoNotAllowed,
    /// 4127: Error en la cabecera: la remisión voluntaria solo debe informarse
    /// en sistemas VERIFACTU.
    HeaderRemisionVoluntariaOnlyVerifactu,
    /// 4128: Error técnico en la recuperación del valor del Gestor de Tablas.
    TableManagerRetrievalError,
    /// 4129: Error en la cabecera: el campo FinRequerimiento es obligatorio.
    HeaderFinRequerimientoRequired,
    /// 4130: Error en la cabecera: el campo FinRequerimiento solo debe
    /// informarse en sistemas No VERIFACTU.
    HeaderFinRequerimientoOnlyNonVerifactu,
    /// 4131: Error en la cabecera: el valor del campo FinRequerimiento es
    /// incorrecto.
    HeaderFinRequerimientoInvalid,
    /// 4132: El titular del certificado debe ser el destinatario que realiza la
    /// consulta, un Apoderado o Sucesor.
    CertificateHolderNotRecipientOrAuthorized,
    /// 4133: Error en la cabecera: el valor del campo RefRequerimiento no es
    /// alfanumérico.
    HeaderRefRequerimientoNotAlphanumeric,
    /// 3500: Error técnico de base de datos: error en la integridad de la
    /// información.
    DatabaseIntegrityError,
    /// 3501: Error técnico de base de datos.
    DatabaseError,
    /// 3502: La factura consultada para el suministro de
    /// pagos/cobros/inmuebles no existe.
    QueriedInvoiceNotFound,
    /// 3503: La factura especificada no pertenece al titular registrado en el
    /// sistema.
    InvoiceNotOwnedByHolder,
    /// 4134: Servicio no activo.
    ServiceInactive,
    /// 4135: Esta URL no puede ser utilizada mediante GET.
    UrlNotUsableViaGet,
    /// 4136: No se ha enviado el nodo RegistroAlta o el anterior al nodo
    /// RegistroAlta no es correcto.
    MissingOrInvalidRegistroAltaNode,
    /// 4137: No se ha enviado el nodo RegistroAnulacion o el anterior al nodo
    /// RegistroAnulacion no es correcto.
    MissingOrInvalidRegistroAnulacionNode,
    /// 4138: Petición vacía en el XML o encoding incorrecto.
    EmptyRequestOrInvalidEncoding,
    /// 4139: Servicio no habilitado en producción.
    ServiceNotEnabledInProduction,
    /// 4140: No puede acceder a la consulta de facturas al no estar apoderado en
    /// los trámites necesarios.
    NotAuthorizedForInvoiceQuery,
    /// 4141: Su acceso al sistema VERIFACTU ha sido suspendido temporalmente.
    AccessSuspended,

    // --- Errors that reject the invoice (or the whole petition if raised on
    //     the header) ---
    /// 1100: Valor o tipo incorrecto del campo.
    InvalidFieldValueOrType,
    /// 1101: El valor del campo CodigoPais es incorrecto.
    InvalidCodigoPais,
    /// 1102: El valor del campo IDType es incorrecto.
    InvalidIdType,
    /// 1103: El valor del campo ID es incorrecto.
    InvalidId,
    /// 1104: El valor del campo NumSerieFactura es incorrecto.
    InvalidNumSerieFactura,
    /// 1105: El valor del campo FechaExpedicionFactura es incorrecto.
    InvalidFechaExpedicionFactura,
    /// 1106: El valor del campo TipoFactura no está incluido en la lista de
    /// valores permitidos.
    InvalidTipoFactura,
    /// 1107: El valor del campo TipoRectificativa es incorrecto.
    InvalidTipoRectificativa,
    /// 1108: El NIF del IDEmisorFactura debe ser el mismo que el NIF del
    /// ObligadoEmision.
    IdEmisorFacturaNifMismatch,
    /// 1109: El NIF no está identificado en el censo de la AEAT.
    DestinatarioNifNotInCensus1109,
    /// 1110: El NIF no está identificado en el censo de la AEAT.
    DestinatarioNifNotInCensus1110,
    /// 1111: El campo CodigoPais es obligatorio cuando IDType es distinto de
    /// NIF-IVA (02).
    CodigoPaisRequiredForNonNifIva,
    /// 1112: El campo FechaExpedicionFactura es superior a la fecha actual.
    FechaExpedicionFacturaInFuture,
    /// 1114: Si la factura es de tipo rectificativa, el campo TipoRectificativa
    /// debe tener valor.
    TipoRectificativaRequired,
    /// 1115: Si la factura no es de tipo rectificativa, el campo
    /// TipoRectificativa no debe tener valor.
    TipoRectificativaNotAllowed,
    /// 1116: Debe informarse el campo FacturasSustituidas sólo si la factura es
    /// de tipo F3.
    FacturasSustituidasOnlyF3,
    /// 1117: Si la factura no es de tipo rectificativa, el bloque
    /// FacturasRectificadas no podrá venir informado.
    FacturasRectificadasNotAllowed,
    /// 1118: Si la factura es de tipo rectificativa por sustitución el bloque
    /// ImporteRectificacion es obligatorio.
    ImporteRectificacionRequired,
    /// 1119: Si la factura no es de tipo rectificativa por sustitución el bloque
    /// ImporteRectificacion no debe tener valor.
    ImporteRectificacionNotAllowed,
    /// 1120: Valor de campo IDEmisorFactura del bloque IDFactura con tipo
    /// incorrecto.
    IdEmisorFacturaWrongType,
    /// 1121: El campo ID no está identificado en el censo de la AEAT.
    IdNotInCensus,
    /// 1122: El campo CodigoPais indicado no coincide con los dos primeros
    /// dígitos del identificador.
    CodigoPaisIdentifierMismatch,
    /// 1123: El formato del NIF es incorrecto.
    InvalidNifFormat1123,
    /// 1124: El valor del campo TipoImpositivo no está incluido en la lista de
    /// valores permitidos.
    InvalidTipoImpositivo,
    /// 1125: El valor del campo FechaOperacion tiene una fecha superior a la
    /// permitida.
    FechaOperacionAboveAllowed,
    /// 1126: El valor del CodigoPais solo puede ser ES cuando el IDType sea
    /// Pasaporte (03) o No Censado (07). Si IDType es No Censado (07) el
    /// CodigoPais debe ser ES (España).
    CodigoPaisEsOnlyPasaporteOrNoCensado,
    /// 1127: El valor del campo TipoRecargoEquivalencia no está incluido en la
    /// lista de valores permitidos.
    InvalidTipoRecargoEquivalencia,
    /// 1128: No existe acuerdo de facturación.
    NoFacturacionAgreement,
    /// 1129: Error técnico al obtener el acuerdo de facturación.
    FacturacionAgreementRetrievalError,
    /// 1130: El campo NumSerieFactura contiene caracteres no permitidos.
    NumSerieFacturaInvalidChars,
    /// 1131: El valor del campo ID ha de ser el NIF de una persona física cuando
    /// el campo IDType tiene valor No Censado (07).
    IdMustBeNaturalPersonNifForNoCensado,
    /// 1132: El valor del campo TipoImpositivo es incorrecto, el valor informado
    /// solo es permitido para FechaOperacion o FechaExpedicionFactura inferior o
    /// igual al año 2012.
    TipoImpositivoOnlyBefore2012,
    /// 1133: El valor del campo FechaExpedicionFactura no debe ser inferior a la
    /// fecha actual menos veinte años.
    FechaExpedicionFacturaTooOld,
    /// 1134: El valor del campo FechaOperacion no debe ser inferior a la fecha
    /// actual menos veinte años.
    FechaOperacionTooOld,
    /// 1135: El valor del campo TipoRecargoEquivalencia es incorrecto, el valor
    /// informado solo es permitido para FechaOperacion o FechaExpedicionFactura
    /// inferior o igual al año 2012.
    TipoRecargoEquivalenciaOnlyBefore2012,
    /// 1136: El campo FacturaSimplificadaArticulos7273 solo acepta valores N o
    /// S.
    FacturaSimplificadaArticulos7273InvalidValue,
    /// 1137: El campo Macrodato solo acepta valores N o S.
    MacrodatoInvalidValue,
    /// 1138: El campo Macrodato solo debe ser informado con valor S si el valor
    /// de ImporteTotal es igual o superior a +-100.000.000.
    MacrodatoOnlyIfTotalAbove100M,
    /// 1139: Si el campo ImporteTotal está informado y es igual o superior a
    /// +-100.000.000 el campo Macrodato debe estar informado con valor S.
    MacrodatoRequiredIfTotalAbove100M,
    /// 1140: Los campos CuotaRepercutida y BaseImponibleACoste deben tener el
    /// mismo signo.
    CuotaRepercutidaBaseImponibleACosteSignMismatch,
    /// 1142: El campo CuotaRepercutida tiene un valor incorrecto para el valor
    /// de los campos BaseImponibleOimporteNoSujeto y TipoImpositivo
    /// suministrados.
    CuotaRepercutidaInvalidForBaseImponible,
    /// 1143: Los campos CuotaRepercutida y BaseImponibleOimporteNoSujeto deben
    /// tener el mismo signo.
    CuotaRepercutidaBaseImponibleSignMismatch,
    /// 1144: El campo CuotaRepercutida tiene un valor incorrecto para el valor
    /// de los campos BaseImponibleACoste y TipoImpositivo suministrados.
    CuotaRepercutidaInvalidForBaseImponibleACoste,
    /// 1145: Formato de fecha incorrecto.
    InvalidDateFormat1145,
    /// 1146: Sólo se permite que la fecha de expedicion de la factura sea
    /// anterior a la fecha operación si los detalles del desglose son
    /// ClaveRegimen 14 o 15 e Impuesto 01, 03 o vacío.
    FechaExpedicionBeforeOperacionRestriction,
    /// 1147: Si ClaveRegimen es 14, FechaOperacion es obligatoria y debe ser
    /// posterior a la FechaExpedicionFactura.
    ClaveRegimen14FechaOperacionRequired,
    /// 1148: Si la ClaveRegimen es 14, el campo TipoFactura debe ser F1, R1, R2,
    /// R3 o R4.
    ClaveRegimen14TipoFacturaRestriction,
    /// 1149: Si ClaveRegimen es 14, el NIF de Destinatarios debe estar
    /// identificado en el censo de la AEAT y comenzar por P, Q, S o V.
    ClaveRegimen14DestinatarioNifRestriction,
    /// 1150: Cuando TipoFactura sea F2 y no este informado
    /// NumRegistroAcuerdoFacturacion o FacturaSinIdentifDestinatarioArt61d no
    /// sea S el sumatorio de BaseImponibleOimporteNoSujeto y CuotaRepercutida de
    /// todas las líneas de detalle no podrá ser superior a 3.000.
    F2BaseImponibleSumLimit,
    /// 1151: El campo EmitidaPorTerceroODestinatario solo acepta valores T o D.
    EmitidaPorTerceroODestinatarioInvalidValue,
    /// 1152: La fecha de expedición no puede ser inferior al 28 de octubre de
    /// 2024.
    FechaExpedicionBeforeOct2024,
    /// 1153: Valor del campo RechazoPrevio no válido, solo podrá incluirse el
    /// campo RechazoPrevio con valor X si se ha informado el campo Subsanacion y
    /// tiene el valor S.
    RechazoPrevioInvalidValue,
    /// 1154: El NIF del emisor de la factura rectificada/sustitutiva no se ha
    /// podido identificar en el censo de la AEAT.
    RectifiedInvoiceIssuerNifNotIdentified,
    /// 1155: Se está informando el bloque Tercero sin estar informado el campo
    /// EmitidaPorTerceroODestinatario.
    TerceroBlockWithoutEmitidaPorTerceroODestinatario,
    /// 1156: Para el bloque IDOtro y IDType NIF-IVA (02), el valor de
    /// TipoFactura es incorrecto.
    IdOtroNifIvaInvalidTipoFactura,
    /// 1157: El valor de cupón solo puede ser S o N si está informado. El valor
    /// de cupón sólo puede ser S si el tipo de factura es R1 o R5.
    CuponInvalidValue,
    /// 1158: Se está informando EmitidaPorTerceroODestinatario, pero no se
    /// informa el bloque correspondiente.
    EmitidaPorTerceroODestinatarioMissingBlock,
    /// 1159: Se está informando del bloque Tercero cuando se indica que se va a
    /// informar de Destinatario.
    TerceroBlockWhileDestinatario,
    /// 1160: Si el TipoImpositivo es 5%, sólo se admite TipoRecargoEquivalencia
    /// 0,5 o 0,62.
    TipoImpositivo5RecargoRestriction,
    /// 1161: El valor del campo RechazoPrevio no es válido, no podrá incluirse el
    /// campo RechazoPrevio con valor S si no se ha informado del campo
    /// Subsanacion o tiene el valor N.
    RechazoPrevioSInvalidWithoutSubsanacion,
    /// 1162: Si el TipoImpositivo es 21%, sólo se admite TipoRecargoEquivalencia
    /// 5,2 ó 1,75.
    TipoImpositivo21RecargoRestriction,
    /// 1163: Si el TipoImpositivo es 10%, sólo se admite TipoRecargoEquivalencia
    /// 1,4.
    TipoImpositivo10RecargoRestriction,
    /// 1164: Si el TipoImpositivo es 4%, sólo se admite TipoRecargoEquivalencia
    /// 0,5.
    TipoImpositivo4RecargoRestriction,
    /// 1165: Si el TipoImpositivo es 0% sólo se admite TipoRecargoEquivalencia
    /// 0% entre el 1 de enero de 2023 y el 30 de septiembre de 2024.
    TipoImpositivo0RecargoRestriction,
    /// 1166: Si el TipoImpositivo es 2% entre el 1 de octubre de 2024 y el 31 de
    /// diciembre de 2024, sólo se admite TipoRecargoEquivalencia 0,26.
    TipoImpositivo2RecargoRestriction,
    /// 1167: Si el TipoImpositivo es 5% sólo se admite TipoRecargoEquivalencia
    /// 0,5 si Fecha Operacion (Fecha Expedicion Factura si no se informa
    /// FechaOperacion) es mayor o igual que el 1 de julio de 2022 y el 31 de
    /// diciembre de 2022.
    TipoImpositivo5Recargo05Restriction,
    /// 1168: Si el TipoImpositivo es 5% sólo se admite TipoRecargoEquivalencia
    /// 0,62 si Fecha Operacion (Fecha Expedicion Factura si no se informa
    /// FechaOperacion) es mayor o igual que el 1 de enero de 2023 y el 30 de
    /// septiembre de 2024.
    TipoImpositivo5Recargo062Restriction,
    /// 1169: Si el TipoImpositivo es 7,5% entre el 1 de octubre de 2024 y el 31
    /// de diciembre de 2024, sólo se admite TipoRecargoEquivalencia 1.
    TipoImpositivo75RecargoRestriction,
    /// 1170: Si el TipoImpositivo es 0%, desde el 1 de octubre del 2024, sólo se
    /// admite TipoRecargoEquivalencia 0,26.
    TipoImpositivo0RecargoFromOct2024,
    /// 1171: El valor del campo Subsanacion o RechazoPrevio no se encuentra en
    /// los valores permitidos.
    SubsanacionOrRechazoPrevioInvalid,
    /// 1172: El valor del campo NIF u ObligadoEmision son nulos.
    NifOrObligadoEmisionNull,
    /// 1173: Sólo se permite que la fecha de operación sea superior a la fecha
    /// actual si los detalles del desglose son ClaveRegimen 14 o 15 e Impuesto
    /// IVA(01) o IGIC(03) o vacío.
    FechaOperacionFutureRestriction,
    /// 1174: El valor del campo FechaExpedicionFactura del bloque
    /// RegistroAnterior es incorrecto.
    RegistroAnteriorFechaExpedicionInvalid,
    /// 1175: El valor del campo NumSerieFactura del bloque RegistroAnterior es
    /// incorrecto.
    RegistroAnteriorNumSerieFacturaInvalid,
    /// 1176: El valor de campo NIF del bloque SistemaInformatico es incorrecto.
    SistemaInformaticoNifInvalid,
    /// 1177: El valor de campo IdSistemaInformatico del bloque
    /// SistemaInformatico es incorrecto.
    IdSistemaInformaticoInvalid,
    /// 1178: Error en el bloque de Tercero.
    TerceroBlockError,
    /// 1179: Error en el bloque de SistemaInformatico.
    SistemaInformaticoBlockError,
    /// 1180: Error en el bloque de Encadenamiento.
    EncadenamientoBlockError,
    /// 1181: El valor del campo CalificacionOperacion es incorrecto.
    InvalidCalificacionOperacion,
    /// 1182: El valor del campo OperacionExenta es incorrecto.
    InvalidOperacionExenta,
    /// 1183: El campo FacturaSimplificadaArticulos7273 solo se podrá rellenar
    /// con S si TipoFactura es de tipo F1 o F3 o R1 o R2 o R3 o R4.
    FacturaSimplificadaArticulos7273TipoFacturaRestriction,
    /// 1184: El campo FacturaSinIdentifDestinatarioArt61d solo acepta valores S
    /// o N.
    FacturaSinIdentifDestinatarioArt61dInvalidValue,
    /// 1185: El campo FacturaSinIdentifDestinatarioArt61d solo se podrá rellenar
    /// con S si TipoFactura es de tipo F2 o R5.
    FacturaSinIdentifDestinatarioArt61dTipoFacturaRestriction,
    /// 1186: Si EmitidaPorTercerosODestinatario es igual a T el bloque Tercero
    /// será de cumplimentación obligatoria.
    TerceroRequiredWhenEmitidaPorTercero,
    /// 1187: Sólo se podrá cumplimentarse el bloque Tercero si el valor de
    /// EmitidaPorTercerosODestinatario es T.
    TerceroOnlyWhenEmitidaPorTercero,
    /// 1188: El NIF del bloque Tercero debe ser diferente al NIF del
    /// ObligadoEmision.
    TerceroNifMustDifferFromObligadoEmision,
    /// 1189: Si TipoFactura es F1 o F3 o R1 o R2 o R3 o R4 el bloque
    /// Destinatarios tiene que estar cumplimentado.
    DestinatariosRequiredForTipoFactura,
    /// 1190: Si TipoFactura es F2 o R5 el bloque Destinatarios no puede estar
    /// cumplimentado.
    DestinatariosNotAllowedForF2R5,
    /// 1191: Si TipoFactura es R3 sólo se admitirá NIF o IDType = No Censado
    /// (07).
    R3OnlyNifOrNoCensado,
    /// 1192: Si TipoFactura es R2 sólo se admitirá NIF o IDType = No Censado (07)
    /// o NIF-IVA (02).
    R2OnlyNifNoCensadoOrNifIva,
    /// 1193: En el bloque Destinatarios si se identifica mediante NIF, el NIF
    /// debe estar identificado y ser distinto del NIF ObligadoEmision.
    DestinatariosNifMustBeIdentifiedAndDiffer,
    /// 1194: El valor del campo TipoImpositivo es incorrecto, el valor informado
    /// solo es permitido para FechaOperacion o FechaExpedicionFactura posterior
    /// o igual a 1 de julio de 2022 e inferior o igual a 30 de septiembre de
    /// 2024.
    TipoImpositivoMidPeriodRestriction,
    /// 1195: Al menos uno de los dos campos OperacionExenta o
    /// CalificacionOperacion deben estar informados.
    OperacionExentaOrCalificacionRequired,
    /// 1196: OperacionExenta o CalificacionOperacion no pueden ser ambos
    /// informados ya que son excluyentes entre sí.
    OperacionExentaAndCalificacionExclusive,
    /// 1197: Si CalificacionOperacion tiene valor S2 TipoFactura solo puede ser
    /// F1, F3, R1, R2, R3 y R4.
    S2TipoFacturaRestriction,
    /// 1198: Si CalificacionOperacion tiene valor S2 TipoImpositivo y
    /// CuotaRepercutida deberan tener valor 0.
    S2TipoImpositivoCuotaZero,
    /// 1199: Si Impuesto es '01' (IVA), '03' (IGIC) o no se cumplimenta y
    /// ClaveRegimen es 01 no pueden marcarse la OperacionExenta E2, E3.
    ExentaE2E3NotAllowedForClaveRegimen01,
    /// 1200: Si ClaveRegimen es 03 CalificacionOperacion sólo puede ser S1.
    ClaveRegimen03OnlyS1,
    /// 1201: Si ClaveRegimen es 04 CalificacionOperacion sólo puede ser S2 o
    /// bien OperacionExenta.
    ClaveRegimen04OnlyS2OrExenta,
    /// 1202: Si ClaveRegimen es 06 TipoFactura no puede ser F2, F3, R5 y
    /// BaseImponibleACoste debe estar cumplimentado.
    ClaveRegimen06Restriction,
    /// 1203: Si ClaveRegimen es 07 OperacionExenta no puede ser E2, E3, E4 y E5
    /// o CalificacionOperacion no puede ser S2, N1, N2.
    ClaveRegimen07Restriction,
    /// 1205: Si ClaveRegimen es 10 CalificacionOperacion tiene que ser N1,
    /// TipoFactura F1 y Destinatarios estar identificada mediante NIF.
    ClaveRegimen10Restriction,
    /// 1206: Si ClaveRegimen es 11 TipoImpositivo ha de ser 21%.
    ClaveRegimen11TipoImpositivo21,
    /// 1207: La CuotaRepercutida solo podrá ser distinta de 0 si
    /// CalificacionOperacion es S1.
    CuotaRepercutidaNonZeroOnlyS1,
    /// 1208: Si CalificacionOperacion es S1 y BaseImponibleACoste no está
    /// cumplimentada, TipoImpositivo y CuotaRepercutida son obligatorios.
    S1TipoImpositivoCuotaRequired,
    /// 1209: Si CalificacionOperacion es S1 y ClaveRegimen es 06, TipoImpositivo
    /// y CuotaRepercutida son obligatorios.
    S1ClaveRegimen06TipoImpositivoCuotaRequired,
    /// 1210: El campo ImporteTotal tiene un valor incorrecto para el valor de
    /// los campos BaseImponibleOimporteNoSujeto, CuotaRepercutida y
    /// CuotaRecargoEquivalencia suministrados.
    ImporteTotalInvalid,
    /// 1211: El bloque Tercero no puede estar identificado con IDType=No Censado
    /// (07).
    TerceroNoCensadoNotAllowed,
    /// 1212: El campo TipoUsoPosibleSoloVerifactu solo acepta valores N o S.
    TipoUsoPosibleSoloVerifactuInvalidValue,
    /// 1213: El campo TipoUsoPosibleMultiOT solo acepta valores N o S.
    TipoUsoPosibleMultiOTInvalidValue,
    /// 1214: El campo NumeroOTAlta debe ser numérico positivo de 4 posiciones.
    NumeroOTAltaInvalid,
    /// 1215: Error en el bloque de ObligadoEmision.
    ObligadoEmisionBlockError,
    /// 1216: El campo CuotaTotal tiene un valor incorrecto para el valor de los
    /// campos CuotaRepercutida y CuotaRecargoEquivalencia suministrados.
    CuotaTotalInvalid,
    /// 1217: Error identificando el IDEmisorFactura.
    IdEmisorFacturaIdentificationError,
    /// 1218: El valor del campo Impuesto es incorrecto.
    InvalidImpuesto,
    /// 1219: El valor del campo IDEmisorFactura es incorrecto.
    InvalidIdEmisorFactura,
    /// 1220: El valor del campo NombreSistemaInformatico es incorrecto.
    InvalidNombreSistemaInformatico,
    /// 1221: El valor del campo IDType del sistema informático es incorrecto.
    InvalidSistemaInformaticoIdType,
    /// 1222: El valor del campo ID del bloque IDOtro es incorrecto.
    InvalidIdOtroId,
    /// 1223: En el bloque SistemaInformatico si se cumplimenta NIF, no deberá
    /// existir la agrupación IDOtro y viceversa, pero es obligatorio que se
    /// cumplimente uno de los dos.
    SistemaInformaticoNifOrIdOtro,
    /// 1224: Si se informa el campo GeneradoPor deberá existir la agrupación
    /// Generador y viceversa.
    GeneradoPorRequiresGenerador,
    /// 1225: El valor del campo GeneradoPor es incorrecto.
    InvalidGeneradoPor,
    /// 1226: El campo IndicadorMultiplesOT solo acepta valores N o S.
    IndicadorMultiplesOTInvalidValue,
    /// 1227: Si el campo GeneradoPor es igual a E debe estar relleno el campo
    /// NIF del bloque Generador.
    GeneradoPorERequiresGeneradorNif,
    /// 1228: En el bloque Generador si se cumplimenta NIF, no deberá existir la
    /// agrupación IDOtro y viceversa, pero es obligatorio que se cumplimente uno
    /// de los dos.
    GeneradorNifOrIdOtro,
    /// 1229: Si el valor de GeneradoPor es igual a T el valor del campo IDType
    /// del bloque Generador no debe ser No Censado (07).
    GeneradoPorTGeneradorIdTypeNotNoCensado,
    /// 1230: Si el valor de GeneradoPor es igual a D y el CodigoPais tiene valor
    /// ES (España), el valor del campo IDType del bloque Generador debe ser
    /// Pasaporte (03) o No Censado (07).
    GeneradoPorDEsGeneradorIdTypeRestriction,
    /// 1231: El valor del campo IDType del bloque Generador es incorrecto.
    InvalidGeneradorIdType,
    /// 1232: Si se identifica a través de la agrupación IDOtro y CodigoPais
    /// tiene valor ES (España), el campo IDType debe valer Pasaporte (03).
    IdOtroEsIdTypePasaporte,
    /// 1233: Si se identifica a través de la agrupación IDOtro y CodigoPais
    /// tiene valor ES (España), el campo IDType debe valer No Censado (07).
    IdOtroEsIdTypeNoCensado,
    /// 1234: Si se identifica a través de la agrupación IDOtro y CodigoPais
    /// tiene valor ES (España), el campo IDType debe valer Pasaporte (03) o No
    /// Censado (07).
    IdOtroEsIdTypePasaporteOrNoCensado,
    /// 1235: El valor del campo TipoImpositivo es incorrecto, el valor informado
    /// sólo es permitido para FechaOperacion o FechaExpedicionFactura posterior
    /// o igual a 1 de octubre de 2024 e inferior o igual a 31 de diciembre de
    /// 2024.
    TipoImpositivoOct2024Restriction1235,
    /// 1236: El valor del campo TipoImpositivo es incorrecto, el valor informado
    /// solo es permitido para FechaOperacion o FechaExpedicionFactura posterior
    /// o igual a 1 de octubre de 2024 e inferior o igual a 31 de diciembre de
    /// 2024.
    TipoImpositivoOct2024Restriction1236,
    /// 1237: El valor del campo CalificacionOperacion está informado como
    /// Operación No sujeta (N1 o N2) y el impuesto es IVA. No se puede informar
    /// de los campos TipoImpositivo, CuotaRepercutida, TipoRecargoEquivalencia y
    /// CuotaRecargoEquivalencia.
    N1N2IvaFieldsNotAllowed,
    /// 1238: Si la operacion es exenta no se puede informar ninguno de los
    /// campos TipoImpositivo, CuotaRepercutida, TipoRecargoEquivalencia y
    /// CuotaRecargoEquivalencia.
    ExentaFieldsNotAllowed,
    /// 1239: Error en el bloque Destinatario.
    DestinatarioBlockError,
    /// 1240: Error en el bloque de IdEmisorFactura.
    IdEmisorFacturaBlockError,
    /// 1241: Error técnico al obtener el SistemaInformatico.
    SistemaInformaticoRetrievalError,
    /// 1242: No existe el sistema informático.
    SistemaInformaticoNotFound,
    /// 1243: Error técnico al obtener el cálculo de la fecha del huso horario.
    TimeZoneDateCalculationError,
    /// 1244: El campo FechaHoraHusoGenRegistro tiene un formato incorrecto.
    FechaHoraHusoGenRegistroInvalidFormat,
    /// 1245: Si el campo Impuesto está vacío o tiene valor IVA(01) o IPSI(02) o
    /// IGIC(03) el campo ClaveRegimen debe de estar cumplimentado.
    ClaveRegimenRequiredForImpuesto,
    /// 1246: El valor del campo ClaveRegimen es incorrecto.
    InvalidClaveRegimen,
    /// 1247: El valor del campo TipoHuella es incorrecto.
    InvalidTipoHuella,
    /// 1248: El valor del campo Periodo es incorrecto.
    InvalidPeriodo,
    /// 1249: El valor del campo IndicadorRepresentante tiene un valor
    /// incorrecto.
    InvalidIndicadorRepresentante,
    /// 1250: El valor de fecha desde debe ser menor que el valor de fecha hasta
    /// en RangoFechaExpedicion.
    RangoFechaExpedicionDesdeAfterHasta,
    /// 1251: El valor del campo IdVersion tiene un valor incorrecto.
    InvalidIdVersion,
    /// 1252: Si ClaveRegimen es 08 el campo CalificacionOperacion tiene que
    /// tener el valor N2 e ir siempre informado.
    ClaveRegimen08CalificacionN2Required,
    /// 1253: El valor del campo RefExterna tiene un valor incorrecto.
    InvalidRefExterna,
    /// 1254: Si FechaOperacion (FechaExpedicionFactura si no se informa
    /// FechaOperacion) es anterior a 01/01/2021 no se permite el valor 'XI' para
    /// Identificaciones NIF-IVA.
    NifIvaXiNotAllowedBefore2021,
    /// 1255: Si FechaOperacion (FechaExpedicionFactura si no se informa
    /// FechaOperacion) es mayor o igual que 01/02/2021 no se permite el valor
    /// 'GB' para Identificaciones NIF-IVA.
    NifIvaGbNotAllowedFrom2021,
    /// 1256: Error técnico al obtener el límite de la fecha de expedición.
    FechaExpedicionLimitRetrievalError,
    /// 1257: El campo BaseImponibleACoste solo puede estar cumplimentado si la
    /// ClaveRegimen es = '06' o Impuesto = '02' (IPSI) o Impuesto = '05'
    /// (Otros).
    BaseImponibleACosteRestriction,
    /// 1258: El valor de campo NIF del bloque Generador es incorrecto.
    GeneradorNifInvalid,
    /// 1259: En el bloque Generador si se identifica mediante NIF, el NIF debe
    /// estar identificado y ser distinto del NIF ObligadoEmision.
    GeneradorNifMustBeIdentifiedAndDiffer,
    /// 1260: El campo ClaveRegimen solo debe de estar cumplimentado si el campo
    /// Impuesto está vacío o tiene valor IVA(01) o IPSI(02) o IGIC(03).
    ClaveRegimenOnlyForImpuesto,
    /// 1261: El campo IndicadorRepresentante solo debe de estar cumplimentado si
    /// se consulta por ObligadoEmision.
    IndicadorRepresentanteOnlyForObligadoEmision,
    /// 1262: La longitud de huella no cumple con las especificaciones.
    HuellaLengthInvalid,
    /// 1263: La longitud del tipo de huella no cumple con las especificaciones.
    TipoHuellaLengthInvalid,
    /// 1264: La longitud del campo primer Registro no cumple con las
    /// especificaciones.
    PrimerRegistroLengthInvalid,
    /// 1265: La longitud del campo tipo factura no cumple con las
    /// especificaciones.
    TipoFacturaLengthInvalid,
    /// 1266: La longitud del campo cuota total no cumple con las
    /// especificaciones.
    CuotaTotalLengthInvalid,
    /// 1267: La longitud del campo importe total no cumple con las
    /// especificaciones.
    ImporteTotalLengthInvalid,
    /// 1268: La longitud del campo FechaHoraHusoGenRegistro no cumple con las
    /// especificaciones.
    FechaHoraHusoGenRegistroLengthInvalid,
    /// 1269: El bloque Registro Anterior no esta informado correctamente.
    RegistroAnteriorBlockInvalid,
    /// 1270: El valor del campo MostrarNombreRazonEmisor tiene un valor
    /// incorrecto.
    InvalidMostrarNombreRazonEmisor,
    /// 1271: El valor del campo MostrarSistemaInformatico tiene un valor
    /// incorrecto.
    InvalidMostrarSistemaInformatico,
    /// 1272: Si se consulta por Destinatario el valor del campo
    /// MostrarSistemaInformatico debe valer 'N' o no estar cumplimentado.
    MostrarSistemaInformaticoDestinatarioRestriction,
    /// 1273: Error en el bloque de Generador.
    GeneradorBlockError,
    /// 1274: Valor incorrecto campo primer registro.
    InvalidPrimerRegistro,
    /// 1275: Valor incorrecto campo RechazoPrevio.
    InvalidRechazoPrevio,
    /// 1276: Valor incorrecto campo SinRegistroPrevio.
    InvalidSinRegistroPrevio,
    /// 1277: Valor incorrecto del TipoRecargoEquivalencia para el tipo
    /// impositivo 0%.
    InvalidTipoRecargoEquivalenciaForZeroRate,
    /// 1278: El valor de la huella del registro anterior debe ser diferente a la
    /// huella del registro actual.
    PreviousHashMustDifferFromCurrent,
    /// 1281: Solo se puede cumplimentar TipoRecargoEquivalencia y
    /// CuotaRecargoEquivalencia cuando CalificacionOperacion tiene valor S1.
    RecargoEquivalenciaOnlyS1,
    /// 1282: Si el NIF de la cabecera es persona fisica se debe informar tambien
    /// de su NombreRazon.
    HeaderNaturalPersonRequiresNombreRazon,
    /// 1283: Si el NIF de la contraparte es persona fisica se debe informar
    /// tambien de su NombreRazon.
    CounterpartyNaturalPersonRequiresNombreRazon,
    /// 1284: Si se ha informado de TipoRecargoEquivalencia tambien se debe
    /// informar de CuotaRecargoEquivalencia y viceversa.
    TipoRecargoEquivalenciaRequiresCuota,
    /// 1285: Se han encontracado varios Sistemas Informáticos con los datos
    /// suministrados, debe filtrar la consulta por más campos del Sistema
    /// Informático.
    MultipleSistemasInformaticosFound,
    /// 1286: Si el impuesto es IVA(01), IGIC(03) o vacio, si ClaveRegimen es 02
    /// solo se podrá informar OperacionExenta.
    ClaveRegimen02OnlyOperacionExenta,
    /// 1287: El valor del campo contiene carácteres no validos (<, >, ", ', =).
    FieldContainsInvalidChars,
    /// 1288: Error técnico en la validación de la fecha de
    /// expedición/operación.
    ExpeditionOperationDateValidationError,
    /// 1289: Si Impuesto es IVA(01) o vacio y si el campo OperacionExenta es
    /// igual a 'E5' sólo deberá existir la agrupación IDOtro en el bloque
    /// Destinatario.
    ExentaE5RequiresIdOtro,
    /// 1290: El campo ID no contiene un NIF con formato correcto.
    IdNotValidNifFormat,
    /// 1291: El HASH del Registro anterior no es alfanumérico.
    PreviousHashNotAlphanumeric,
    /// 1292: El HASH no es alfanumérico.
    HashNotAlphanumeric,
    /// 1293: Si ClaveRegimen es 20 el campo CalificacionOperacion tiene que
    /// tener el valor N2 e ir siempre informado.
    ClaveRegimen20CalificacionN2Required,
    /// 3000: Registro de facturación duplicado.
    DuplicateRecord,
    /// 3001: El registro de facturación ya ha sido dado de baja.
    RecordAlreadyCancelled,
    /// 3002: No existe el registro de facturación.
    RecordDoesNotExist,
    /// 3003: El presentador no tiene los permisos necesarios para actualizar
    /// este registro de facturación.
    PresenterLacksPermissions,
    /// 3004: No es posible modificar la factura ya que ha sido dada de alta vía
    /// formulario.
    CannotModifyInvoiceFiledViaForm,

    // --- Errors that accept the record but require later correction
    //     (subsanación) ---
    /// 2000: El cálculo de la huella suministrada es incorrecta.
    IncorrectHashCalculation,
    /// 2001: El NIF del bloque Destinatarios no está identificado en el censo de
    /// la AEAT.
    DestinatariosNifNotInCensus,
    /// 2002: La longitud de huella del registro anterior no cumple con las
    /// especificaciones.
    PreviousRecordHashLengthInvalid,
    /// 2003: El contenido de la huella del registro anterior no cumple con las
    /// especificaciones.
    PreviousRecordHashContentInvalid,
    /// 2004: El valor del campo FechaHoraHusoGenRegistro debe ser la fecha
    /// actual del sistema de la AEAT, admitiéndose un margen de error.
    FechaHoraHusoGenRegistroOutOfRange,
    /// 2005: El campo ImporteTotal tiene un valor incorrecto para el valor de
    /// los campos BaseImponibleOimporteNoSujeto, CuotaRepercutida y
    /// CuotaRecargoEquivalencia suministrados.
    ImporteTotalInvalid2005,
    /// 2006: El campo CuotaTotal tiene un valor incorrecto para el valor de los
    /// campos CuotaRepercutida y CuotaRecargoEquivalencia suministrados.
    CuotaTotalInvalid2006,
    /// 2007: No debe informarse como primer registro, existen facturas emitidas
    /// con el obligado emisión y el sistema informático actual.
    ShouldNotBeFirstRecord,
    /// 2008: El valor de la huella del registro anterior debe ser diferente a la
    /// huella del registro actual.
    PreviousHashMustDifferFromCurrent2008,
    /// 2009: Si el campo Impuesto tiene valor IPSI(02) el campo ClaveRegimen
    /// debe de estar cumplimentado.
    ClaveRegimenRequiredForIpsi,

    /// An undocumented or future error code returned by AEAT.
    Unknown(u32),
}

impl BackendError {
    /// Map a numeric AEAT error code to its typed representation, returning
    /// [`BackendError::Unknown`] for codes not present in the documented list.
    pub fn from_code(code: u32) -> Self {
        match code {
            4102 => Self::SchemaMissingRequiredField,
            4103 => Self::UnexpectedXmlParseError,
            4104 => Self::HeaderObligadoEmisionNifNotIdentified,
            4105 => Self::HeaderRepresentanteNifNotIdentified,
            4106 => Self::InvalidDateFormat,
            4107 => Self::NifNotInCensus,
            4108 => Self::CertificateRetrievalError,
            4109 => Self::InvalidNifFormat,
            4110 => Self::PowersOfAttorneyCheckError,
            4111 => Self::ProcedureCreationError,
            4112 => Self::CertificateHolderNotAuthorized,
            4113 => Self::BlockRecordLimitExceeded,
            4114 => Self::InvoiceRegistrationLimitExceeded,
            4115 => Self::ObligadoEmisionNifInvalid,
            4116 => Self::HeaderObligadoEmisionNifFormatInvalid,
            4117 => Self::HeaderRepresentanteNifFormatInvalid,
            4118 => Self::AddressMismatchWithInputFile,
            4119 => Self::NonUtf8Encoding,
            4120 => Self::HeaderFechaFinVeriFactuInvalid,
            4121 => Self::HeaderIncidenciaInvalid,
            4122 => Self::HeaderRefRequerimientoInvalid,
            4123 => Self::HeaderRepresentanteNifNotInCensus,
            4124 => Self::HeaderRepresentanteNombreNotInCensus,
            4125 => Self::HeaderRefRequerimientoRequired,
            4126 => Self::HeaderRefRequerimientoNotAllowed,
            4127 => Self::HeaderRemisionVoluntariaOnlyVerifactu,
            4128 => Self::TableManagerRetrievalError,
            4129 => Self::HeaderFinRequerimientoRequired,
            4130 => Self::HeaderFinRequerimientoOnlyNonVerifactu,
            4131 => Self::HeaderFinRequerimientoInvalid,
            4132 => Self::CertificateHolderNotRecipientOrAuthorized,
            4133 => Self::HeaderRefRequerimientoNotAlphanumeric,
            3500 => Self::DatabaseIntegrityError,
            3501 => Self::DatabaseError,
            3502 => Self::QueriedInvoiceNotFound,
            3503 => Self::InvoiceNotOwnedByHolder,
            4134 => Self::ServiceInactive,
            4135 => Self::UrlNotUsableViaGet,
            4136 => Self::MissingOrInvalidRegistroAltaNode,
            4137 => Self::MissingOrInvalidRegistroAnulacionNode,
            4138 => Self::EmptyRequestOrInvalidEncoding,
            4139 => Self::ServiceNotEnabledInProduction,
            4140 => Self::NotAuthorizedForInvoiceQuery,
            4141 => Self::AccessSuspended,
            1100 => Self::InvalidFieldValueOrType,
            1101 => Self::InvalidCodigoPais,
            1102 => Self::InvalidIdType,
            1103 => Self::InvalidId,
            1104 => Self::InvalidNumSerieFactura,
            1105 => Self::InvalidFechaExpedicionFactura,
            1106 => Self::InvalidTipoFactura,
            1107 => Self::InvalidTipoRectificativa,
            1108 => Self::IdEmisorFacturaNifMismatch,
            1109 => Self::DestinatarioNifNotInCensus1109,
            1110 => Self::DestinatarioNifNotInCensus1110,
            1111 => Self::CodigoPaisRequiredForNonNifIva,
            1112 => Self::FechaExpedicionFacturaInFuture,
            1114 => Self::TipoRectificativaRequired,
            1115 => Self::TipoRectificativaNotAllowed,
            1116 => Self::FacturasSustituidasOnlyF3,
            1117 => Self::FacturasRectificadasNotAllowed,
            1118 => Self::ImporteRectificacionRequired,
            1119 => Self::ImporteRectificacionNotAllowed,
            1120 => Self::IdEmisorFacturaWrongType,
            1121 => Self::IdNotInCensus,
            1122 => Self::CodigoPaisIdentifierMismatch,
            1123 => Self::InvalidNifFormat1123,
            1124 => Self::InvalidTipoImpositivo,
            1125 => Self::FechaOperacionAboveAllowed,
            1126 => Self::CodigoPaisEsOnlyPasaporteOrNoCensado,
            1127 => Self::InvalidTipoRecargoEquivalencia,
            1128 => Self::NoFacturacionAgreement,
            1129 => Self::FacturacionAgreementRetrievalError,
            1130 => Self::NumSerieFacturaInvalidChars,
            1131 => Self::IdMustBeNaturalPersonNifForNoCensado,
            1132 => Self::TipoImpositivoOnlyBefore2012,
            1133 => Self::FechaExpedicionFacturaTooOld,
            1134 => Self::FechaOperacionTooOld,
            1135 => Self::TipoRecargoEquivalenciaOnlyBefore2012,
            1136 => Self::FacturaSimplificadaArticulos7273InvalidValue,
            1137 => Self::MacrodatoInvalidValue,
            1138 => Self::MacrodatoOnlyIfTotalAbove100M,
            1139 => Self::MacrodatoRequiredIfTotalAbove100M,
            1140 => Self::CuotaRepercutidaBaseImponibleACosteSignMismatch,
            1142 => Self::CuotaRepercutidaInvalidForBaseImponible,
            1143 => Self::CuotaRepercutidaBaseImponibleSignMismatch,
            1144 => Self::CuotaRepercutidaInvalidForBaseImponibleACoste,
            1145 => Self::InvalidDateFormat1145,
            1146 => Self::FechaExpedicionBeforeOperacionRestriction,
            1147 => Self::ClaveRegimen14FechaOperacionRequired,
            1148 => Self::ClaveRegimen14TipoFacturaRestriction,
            1149 => Self::ClaveRegimen14DestinatarioNifRestriction,
            1150 => Self::F2BaseImponibleSumLimit,
            1151 => Self::EmitidaPorTerceroODestinatarioInvalidValue,
            1152 => Self::FechaExpedicionBeforeOct2024,
            1153 => Self::RechazoPrevioInvalidValue,
            1154 => Self::RectifiedInvoiceIssuerNifNotIdentified,
            1155 => Self::TerceroBlockWithoutEmitidaPorTerceroODestinatario,
            1156 => Self::IdOtroNifIvaInvalidTipoFactura,
            1157 => Self::CuponInvalidValue,
            1158 => Self::EmitidaPorTerceroODestinatarioMissingBlock,
            1159 => Self::TerceroBlockWhileDestinatario,
            1160 => Self::TipoImpositivo5RecargoRestriction,
            1161 => Self::RechazoPrevioSInvalidWithoutSubsanacion,
            1162 => Self::TipoImpositivo21RecargoRestriction,
            1163 => Self::TipoImpositivo10RecargoRestriction,
            1164 => Self::TipoImpositivo4RecargoRestriction,
            1165 => Self::TipoImpositivo0RecargoRestriction,
            1166 => Self::TipoImpositivo2RecargoRestriction,
            1167 => Self::TipoImpositivo5Recargo05Restriction,
            1168 => Self::TipoImpositivo5Recargo062Restriction,
            1169 => Self::TipoImpositivo75RecargoRestriction,
            1170 => Self::TipoImpositivo0RecargoFromOct2024,
            1171 => Self::SubsanacionOrRechazoPrevioInvalid,
            1172 => Self::NifOrObligadoEmisionNull,
            1173 => Self::FechaOperacionFutureRestriction,
            1174 => Self::RegistroAnteriorFechaExpedicionInvalid,
            1175 => Self::RegistroAnteriorNumSerieFacturaInvalid,
            1176 => Self::SistemaInformaticoNifInvalid,
            1177 => Self::IdSistemaInformaticoInvalid,
            1178 => Self::TerceroBlockError,
            1179 => Self::SistemaInformaticoBlockError,
            1180 => Self::EncadenamientoBlockError,
            1181 => Self::InvalidCalificacionOperacion,
            1182 => Self::InvalidOperacionExenta,
            1183 => Self::FacturaSimplificadaArticulos7273TipoFacturaRestriction,
            1184 => Self::FacturaSinIdentifDestinatarioArt61dInvalidValue,
            1185 => Self::FacturaSinIdentifDestinatarioArt61dTipoFacturaRestriction,
            1186 => Self::TerceroRequiredWhenEmitidaPorTercero,
            1187 => Self::TerceroOnlyWhenEmitidaPorTercero,
            1188 => Self::TerceroNifMustDifferFromObligadoEmision,
            1189 => Self::DestinatariosRequiredForTipoFactura,
            1190 => Self::DestinatariosNotAllowedForF2R5,
            1191 => Self::R3OnlyNifOrNoCensado,
            1192 => Self::R2OnlyNifNoCensadoOrNifIva,
            1193 => Self::DestinatariosNifMustBeIdentifiedAndDiffer,
            1194 => Self::TipoImpositivoMidPeriodRestriction,
            1195 => Self::OperacionExentaOrCalificacionRequired,
            1196 => Self::OperacionExentaAndCalificacionExclusive,
            1197 => Self::S2TipoFacturaRestriction,
            1198 => Self::S2TipoImpositivoCuotaZero,
            1199 => Self::ExentaE2E3NotAllowedForClaveRegimen01,
            1200 => Self::ClaveRegimen03OnlyS1,
            1201 => Self::ClaveRegimen04OnlyS2OrExenta,
            1202 => Self::ClaveRegimen06Restriction,
            1203 => Self::ClaveRegimen07Restriction,
            1205 => Self::ClaveRegimen10Restriction,
            1206 => Self::ClaveRegimen11TipoImpositivo21,
            1207 => Self::CuotaRepercutidaNonZeroOnlyS1,
            1208 => Self::S1TipoImpositivoCuotaRequired,
            1209 => Self::S1ClaveRegimen06TipoImpositivoCuotaRequired,
            1210 => Self::ImporteTotalInvalid,
            1211 => Self::TerceroNoCensadoNotAllowed,
            1212 => Self::TipoUsoPosibleSoloVerifactuInvalidValue,
            1213 => Self::TipoUsoPosibleMultiOTInvalidValue,
            1214 => Self::NumeroOTAltaInvalid,
            1215 => Self::ObligadoEmisionBlockError,
            1216 => Self::CuotaTotalInvalid,
            1217 => Self::IdEmisorFacturaIdentificationError,
            1218 => Self::InvalidImpuesto,
            1219 => Self::InvalidIdEmisorFactura,
            1220 => Self::InvalidNombreSistemaInformatico,
            1221 => Self::InvalidSistemaInformaticoIdType,
            1222 => Self::InvalidIdOtroId,
            1223 => Self::SistemaInformaticoNifOrIdOtro,
            1224 => Self::GeneradoPorRequiresGenerador,
            1225 => Self::InvalidGeneradoPor,
            1226 => Self::IndicadorMultiplesOTInvalidValue,
            1227 => Self::GeneradoPorERequiresGeneradorNif,
            1228 => Self::GeneradorNifOrIdOtro,
            1229 => Self::GeneradoPorTGeneradorIdTypeNotNoCensado,
            1230 => Self::GeneradoPorDEsGeneradorIdTypeRestriction,
            1231 => Self::InvalidGeneradorIdType,
            1232 => Self::IdOtroEsIdTypePasaporte,
            1233 => Self::IdOtroEsIdTypeNoCensado,
            1234 => Self::IdOtroEsIdTypePasaporteOrNoCensado,
            1235 => Self::TipoImpositivoOct2024Restriction1235,
            1236 => Self::TipoImpositivoOct2024Restriction1236,
            1237 => Self::N1N2IvaFieldsNotAllowed,
            1238 => Self::ExentaFieldsNotAllowed,
            1239 => Self::DestinatarioBlockError,
            1240 => Self::IdEmisorFacturaBlockError,
            1241 => Self::SistemaInformaticoRetrievalError,
            1242 => Self::SistemaInformaticoNotFound,
            1243 => Self::TimeZoneDateCalculationError,
            1244 => Self::FechaHoraHusoGenRegistroInvalidFormat,
            1245 => Self::ClaveRegimenRequiredForImpuesto,
            1246 => Self::InvalidClaveRegimen,
            1247 => Self::InvalidTipoHuella,
            1248 => Self::InvalidPeriodo,
            1249 => Self::InvalidIndicadorRepresentante,
            1250 => Self::RangoFechaExpedicionDesdeAfterHasta,
            1251 => Self::InvalidIdVersion,
            1252 => Self::ClaveRegimen08CalificacionN2Required,
            1253 => Self::InvalidRefExterna,
            1254 => Self::NifIvaXiNotAllowedBefore2021,
            1255 => Self::NifIvaGbNotAllowedFrom2021,
            1256 => Self::FechaExpedicionLimitRetrievalError,
            1257 => Self::BaseImponibleACosteRestriction,
            1258 => Self::GeneradorNifInvalid,
            1259 => Self::GeneradorNifMustBeIdentifiedAndDiffer,
            1260 => Self::ClaveRegimenOnlyForImpuesto,
            1261 => Self::IndicadorRepresentanteOnlyForObligadoEmision,
            1262 => Self::HuellaLengthInvalid,
            1263 => Self::TipoHuellaLengthInvalid,
            1264 => Self::PrimerRegistroLengthInvalid,
            1265 => Self::TipoFacturaLengthInvalid,
            1266 => Self::CuotaTotalLengthInvalid,
            1267 => Self::ImporteTotalLengthInvalid,
            1268 => Self::FechaHoraHusoGenRegistroLengthInvalid,
            1269 => Self::RegistroAnteriorBlockInvalid,
            1270 => Self::InvalidMostrarNombreRazonEmisor,
            1271 => Self::InvalidMostrarSistemaInformatico,
            1272 => Self::MostrarSistemaInformaticoDestinatarioRestriction,
            1273 => Self::GeneradorBlockError,
            1274 => Self::InvalidPrimerRegistro,
            1275 => Self::InvalidRechazoPrevio,
            1276 => Self::InvalidSinRegistroPrevio,
            1277 => Self::InvalidTipoRecargoEquivalenciaForZeroRate,
            1278 => Self::PreviousHashMustDifferFromCurrent,
            1281 => Self::RecargoEquivalenciaOnlyS1,
            1282 => Self::HeaderNaturalPersonRequiresNombreRazon,
            1283 => Self::CounterpartyNaturalPersonRequiresNombreRazon,
            1284 => Self::TipoRecargoEquivalenciaRequiresCuota,
            1285 => Self::MultipleSistemasInformaticosFound,
            1286 => Self::ClaveRegimen02OnlyOperacionExenta,
            1287 => Self::FieldContainsInvalidChars,
            1288 => Self::ExpeditionOperationDateValidationError,
            1289 => Self::ExentaE5RequiresIdOtro,
            1290 => Self::IdNotValidNifFormat,
            1291 => Self::PreviousHashNotAlphanumeric,
            1292 => Self::HashNotAlphanumeric,
            1293 => Self::ClaveRegimen20CalificacionN2Required,
            3000 => Self::DuplicateRecord,
            3001 => Self::RecordAlreadyCancelled,
            3002 => Self::RecordDoesNotExist,
            3003 => Self::PresenterLacksPermissions,
            3004 => Self::CannotModifyInvoiceFiledViaForm,
            2000 => Self::IncorrectHashCalculation,
            2001 => Self::DestinatariosNifNotInCensus,
            2002 => Self::PreviousRecordHashLengthInvalid,
            2003 => Self::PreviousRecordHashContentInvalid,
            2004 => Self::FechaHoraHusoGenRegistroOutOfRange,
            2005 => Self::ImporteTotalInvalid2005,
            2006 => Self::CuotaTotalInvalid2006,
            2007 => Self::ShouldNotBeFirstRecord,
            2008 => Self::PreviousHashMustDifferFromCurrent2008,
            2009 => Self::ClaveRegimenRequiredForIpsi,
            other => Self::Unknown(other),
        }
    }

    /// The numeric AEAT error code for this error.
    pub fn code(&self) -> u32 {
        match self {
            Self::SchemaMissingRequiredField => 4102,
            Self::UnexpectedXmlParseError => 4103,
            Self::HeaderObligadoEmisionNifNotIdentified => 4104,
            Self::HeaderRepresentanteNifNotIdentified => 4105,
            Self::InvalidDateFormat => 4106,
            Self::NifNotInCensus => 4107,
            Self::CertificateRetrievalError => 4108,
            Self::InvalidNifFormat => 4109,
            Self::PowersOfAttorneyCheckError => 4110,
            Self::ProcedureCreationError => 4111,
            Self::CertificateHolderNotAuthorized => 4112,
            Self::BlockRecordLimitExceeded => 4113,
            Self::InvoiceRegistrationLimitExceeded => 4114,
            Self::ObligadoEmisionNifInvalid => 4115,
            Self::HeaderObligadoEmisionNifFormatInvalid => 4116,
            Self::HeaderRepresentanteNifFormatInvalid => 4117,
            Self::AddressMismatchWithInputFile => 4118,
            Self::NonUtf8Encoding => 4119,
            Self::HeaderFechaFinVeriFactuInvalid => 4120,
            Self::HeaderIncidenciaInvalid => 4121,
            Self::HeaderRefRequerimientoInvalid => 4122,
            Self::HeaderRepresentanteNifNotInCensus => 4123,
            Self::HeaderRepresentanteNombreNotInCensus => 4124,
            Self::HeaderRefRequerimientoRequired => 4125,
            Self::HeaderRefRequerimientoNotAllowed => 4126,
            Self::HeaderRemisionVoluntariaOnlyVerifactu => 4127,
            Self::TableManagerRetrievalError => 4128,
            Self::HeaderFinRequerimientoRequired => 4129,
            Self::HeaderFinRequerimientoOnlyNonVerifactu => 4130,
            Self::HeaderFinRequerimientoInvalid => 4131,
            Self::CertificateHolderNotRecipientOrAuthorized => 4132,
            Self::HeaderRefRequerimientoNotAlphanumeric => 4133,
            Self::DatabaseIntegrityError => 3500,
            Self::DatabaseError => 3501,
            Self::QueriedInvoiceNotFound => 3502,
            Self::InvoiceNotOwnedByHolder => 3503,
            Self::ServiceInactive => 4134,
            Self::UrlNotUsableViaGet => 4135,
            Self::MissingOrInvalidRegistroAltaNode => 4136,
            Self::MissingOrInvalidRegistroAnulacionNode => 4137,
            Self::EmptyRequestOrInvalidEncoding => 4138,
            Self::ServiceNotEnabledInProduction => 4139,
            Self::NotAuthorizedForInvoiceQuery => 4140,
            Self::AccessSuspended => 4141,
            Self::InvalidFieldValueOrType => 1100,
            Self::InvalidCodigoPais => 1101,
            Self::InvalidIdType => 1102,
            Self::InvalidId => 1103,
            Self::InvalidNumSerieFactura => 1104,
            Self::InvalidFechaExpedicionFactura => 1105,
            Self::InvalidTipoFactura => 1106,
            Self::InvalidTipoRectificativa => 1107,
            Self::IdEmisorFacturaNifMismatch => 1108,
            Self::DestinatarioNifNotInCensus1109 => 1109,
            Self::DestinatarioNifNotInCensus1110 => 1110,
            Self::CodigoPaisRequiredForNonNifIva => 1111,
            Self::FechaExpedicionFacturaInFuture => 1112,
            Self::TipoRectificativaRequired => 1114,
            Self::TipoRectificativaNotAllowed => 1115,
            Self::FacturasSustituidasOnlyF3 => 1116,
            Self::FacturasRectificadasNotAllowed => 1117,
            Self::ImporteRectificacionRequired => 1118,
            Self::ImporteRectificacionNotAllowed => 1119,
            Self::IdEmisorFacturaWrongType => 1120,
            Self::IdNotInCensus => 1121,
            Self::CodigoPaisIdentifierMismatch => 1122,
            Self::InvalidNifFormat1123 => 1123,
            Self::InvalidTipoImpositivo => 1124,
            Self::FechaOperacionAboveAllowed => 1125,
            Self::CodigoPaisEsOnlyPasaporteOrNoCensado => 1126,
            Self::InvalidTipoRecargoEquivalencia => 1127,
            Self::NoFacturacionAgreement => 1128,
            Self::FacturacionAgreementRetrievalError => 1129,
            Self::NumSerieFacturaInvalidChars => 1130,
            Self::IdMustBeNaturalPersonNifForNoCensado => 1131,
            Self::TipoImpositivoOnlyBefore2012 => 1132,
            Self::FechaExpedicionFacturaTooOld => 1133,
            Self::FechaOperacionTooOld => 1134,
            Self::TipoRecargoEquivalenciaOnlyBefore2012 => 1135,
            Self::FacturaSimplificadaArticulos7273InvalidValue => 1136,
            Self::MacrodatoInvalidValue => 1137,
            Self::MacrodatoOnlyIfTotalAbove100M => 1138,
            Self::MacrodatoRequiredIfTotalAbove100M => 1139,
            Self::CuotaRepercutidaBaseImponibleACosteSignMismatch => 1140,
            Self::CuotaRepercutidaInvalidForBaseImponible => 1142,
            Self::CuotaRepercutidaBaseImponibleSignMismatch => 1143,
            Self::CuotaRepercutidaInvalidForBaseImponibleACoste => 1144,
            Self::InvalidDateFormat1145 => 1145,
            Self::FechaExpedicionBeforeOperacionRestriction => 1146,
            Self::ClaveRegimen14FechaOperacionRequired => 1147,
            Self::ClaveRegimen14TipoFacturaRestriction => 1148,
            Self::ClaveRegimen14DestinatarioNifRestriction => 1149,
            Self::F2BaseImponibleSumLimit => 1150,
            Self::EmitidaPorTerceroODestinatarioInvalidValue => 1151,
            Self::FechaExpedicionBeforeOct2024 => 1152,
            Self::RechazoPrevioInvalidValue => 1153,
            Self::RectifiedInvoiceIssuerNifNotIdentified => 1154,
            Self::TerceroBlockWithoutEmitidaPorTerceroODestinatario => 1155,
            Self::IdOtroNifIvaInvalidTipoFactura => 1156,
            Self::CuponInvalidValue => 1157,
            Self::EmitidaPorTerceroODestinatarioMissingBlock => 1158,
            Self::TerceroBlockWhileDestinatario => 1159,
            Self::TipoImpositivo5RecargoRestriction => 1160,
            Self::RechazoPrevioSInvalidWithoutSubsanacion => 1161,
            Self::TipoImpositivo21RecargoRestriction => 1162,
            Self::TipoImpositivo10RecargoRestriction => 1163,
            Self::TipoImpositivo4RecargoRestriction => 1164,
            Self::TipoImpositivo0RecargoRestriction => 1165,
            Self::TipoImpositivo2RecargoRestriction => 1166,
            Self::TipoImpositivo5Recargo05Restriction => 1167,
            Self::TipoImpositivo5Recargo062Restriction => 1168,
            Self::TipoImpositivo75RecargoRestriction => 1169,
            Self::TipoImpositivo0RecargoFromOct2024 => 1170,
            Self::SubsanacionOrRechazoPrevioInvalid => 1171,
            Self::NifOrObligadoEmisionNull => 1172,
            Self::FechaOperacionFutureRestriction => 1173,
            Self::RegistroAnteriorFechaExpedicionInvalid => 1174,
            Self::RegistroAnteriorNumSerieFacturaInvalid => 1175,
            Self::SistemaInformaticoNifInvalid => 1176,
            Self::IdSistemaInformaticoInvalid => 1177,
            Self::TerceroBlockError => 1178,
            Self::SistemaInformaticoBlockError => 1179,
            Self::EncadenamientoBlockError => 1180,
            Self::InvalidCalificacionOperacion => 1181,
            Self::InvalidOperacionExenta => 1182,
            Self::FacturaSimplificadaArticulos7273TipoFacturaRestriction => 1183,
            Self::FacturaSinIdentifDestinatarioArt61dInvalidValue => 1184,
            Self::FacturaSinIdentifDestinatarioArt61dTipoFacturaRestriction => 1185,
            Self::TerceroRequiredWhenEmitidaPorTercero => 1186,
            Self::TerceroOnlyWhenEmitidaPorTercero => 1187,
            Self::TerceroNifMustDifferFromObligadoEmision => 1188,
            Self::DestinatariosRequiredForTipoFactura => 1189,
            Self::DestinatariosNotAllowedForF2R5 => 1190,
            Self::R3OnlyNifOrNoCensado => 1191,
            Self::R2OnlyNifNoCensadoOrNifIva => 1192,
            Self::DestinatariosNifMustBeIdentifiedAndDiffer => 1193,
            Self::TipoImpositivoMidPeriodRestriction => 1194,
            Self::OperacionExentaOrCalificacionRequired => 1195,
            Self::OperacionExentaAndCalificacionExclusive => 1196,
            Self::S2TipoFacturaRestriction => 1197,
            Self::S2TipoImpositivoCuotaZero => 1198,
            Self::ExentaE2E3NotAllowedForClaveRegimen01 => 1199,
            Self::ClaveRegimen03OnlyS1 => 1200,
            Self::ClaveRegimen04OnlyS2OrExenta => 1201,
            Self::ClaveRegimen06Restriction => 1202,
            Self::ClaveRegimen07Restriction => 1203,
            Self::ClaveRegimen10Restriction => 1205,
            Self::ClaveRegimen11TipoImpositivo21 => 1206,
            Self::CuotaRepercutidaNonZeroOnlyS1 => 1207,
            Self::S1TipoImpositivoCuotaRequired => 1208,
            Self::S1ClaveRegimen06TipoImpositivoCuotaRequired => 1209,
            Self::ImporteTotalInvalid => 1210,
            Self::TerceroNoCensadoNotAllowed => 1211,
            Self::TipoUsoPosibleSoloVerifactuInvalidValue => 1212,
            Self::TipoUsoPosibleMultiOTInvalidValue => 1213,
            Self::NumeroOTAltaInvalid => 1214,
            Self::ObligadoEmisionBlockError => 1215,
            Self::CuotaTotalInvalid => 1216,
            Self::IdEmisorFacturaIdentificationError => 1217,
            Self::InvalidImpuesto => 1218,
            Self::InvalidIdEmisorFactura => 1219,
            Self::InvalidNombreSistemaInformatico => 1220,
            Self::InvalidSistemaInformaticoIdType => 1221,
            Self::InvalidIdOtroId => 1222,
            Self::SistemaInformaticoNifOrIdOtro => 1223,
            Self::GeneradoPorRequiresGenerador => 1224,
            Self::InvalidGeneradoPor => 1225,
            Self::IndicadorMultiplesOTInvalidValue => 1226,
            Self::GeneradoPorERequiresGeneradorNif => 1227,
            Self::GeneradorNifOrIdOtro => 1228,
            Self::GeneradoPorTGeneradorIdTypeNotNoCensado => 1229,
            Self::GeneradoPorDEsGeneradorIdTypeRestriction => 1230,
            Self::InvalidGeneradorIdType => 1231,
            Self::IdOtroEsIdTypePasaporte => 1232,
            Self::IdOtroEsIdTypeNoCensado => 1233,
            Self::IdOtroEsIdTypePasaporteOrNoCensado => 1234,
            Self::TipoImpositivoOct2024Restriction1235 => 1235,
            Self::TipoImpositivoOct2024Restriction1236 => 1236,
            Self::N1N2IvaFieldsNotAllowed => 1237,
            Self::ExentaFieldsNotAllowed => 1238,
            Self::DestinatarioBlockError => 1239,
            Self::IdEmisorFacturaBlockError => 1240,
            Self::SistemaInformaticoRetrievalError => 1241,
            Self::SistemaInformaticoNotFound => 1242,
            Self::TimeZoneDateCalculationError => 1243,
            Self::FechaHoraHusoGenRegistroInvalidFormat => 1244,
            Self::ClaveRegimenRequiredForImpuesto => 1245,
            Self::InvalidClaveRegimen => 1246,
            Self::InvalidTipoHuella => 1247,
            Self::InvalidPeriodo => 1248,
            Self::InvalidIndicadorRepresentante => 1249,
            Self::RangoFechaExpedicionDesdeAfterHasta => 1250,
            Self::InvalidIdVersion => 1251,
            Self::ClaveRegimen08CalificacionN2Required => 1252,
            Self::InvalidRefExterna => 1253,
            Self::NifIvaXiNotAllowedBefore2021 => 1254,
            Self::NifIvaGbNotAllowedFrom2021 => 1255,
            Self::FechaExpedicionLimitRetrievalError => 1256,
            Self::BaseImponibleACosteRestriction => 1257,
            Self::GeneradorNifInvalid => 1258,
            Self::GeneradorNifMustBeIdentifiedAndDiffer => 1259,
            Self::ClaveRegimenOnlyForImpuesto => 1260,
            Self::IndicadorRepresentanteOnlyForObligadoEmision => 1261,
            Self::HuellaLengthInvalid => 1262,
            Self::TipoHuellaLengthInvalid => 1263,
            Self::PrimerRegistroLengthInvalid => 1264,
            Self::TipoFacturaLengthInvalid => 1265,
            Self::CuotaTotalLengthInvalid => 1266,
            Self::ImporteTotalLengthInvalid => 1267,
            Self::FechaHoraHusoGenRegistroLengthInvalid => 1268,
            Self::RegistroAnteriorBlockInvalid => 1269,
            Self::InvalidMostrarNombreRazonEmisor => 1270,
            Self::InvalidMostrarSistemaInformatico => 1271,
            Self::MostrarSistemaInformaticoDestinatarioRestriction => 1272,
            Self::GeneradorBlockError => 1273,
            Self::InvalidPrimerRegistro => 1274,
            Self::InvalidRechazoPrevio => 1275,
            Self::InvalidSinRegistroPrevio => 1276,
            Self::InvalidTipoRecargoEquivalenciaForZeroRate => 1277,
            Self::PreviousHashMustDifferFromCurrent => 1278,
            Self::RecargoEquivalenciaOnlyS1 => 1281,
            Self::HeaderNaturalPersonRequiresNombreRazon => 1282,
            Self::CounterpartyNaturalPersonRequiresNombreRazon => 1283,
            Self::TipoRecargoEquivalenciaRequiresCuota => 1284,
            Self::MultipleSistemasInformaticosFound => 1285,
            Self::ClaveRegimen02OnlyOperacionExenta => 1286,
            Self::FieldContainsInvalidChars => 1287,
            Self::ExpeditionOperationDateValidationError => 1288,
            Self::ExentaE5RequiresIdOtro => 1289,
            Self::IdNotValidNifFormat => 1290,
            Self::PreviousHashNotAlphanumeric => 1291,
            Self::HashNotAlphanumeric => 1292,
            Self::ClaveRegimen20CalificacionN2Required => 1293,
            Self::DuplicateRecord => 3000,
            Self::RecordAlreadyCancelled => 3001,
            Self::RecordDoesNotExist => 3002,
            Self::PresenterLacksPermissions => 3003,
            Self::CannotModifyInvoiceFiledViaForm => 3004,
            Self::IncorrectHashCalculation => 2000,
            Self::DestinatariosNifNotInCensus => 2001,
            Self::PreviousRecordHashLengthInvalid => 2002,
            Self::PreviousRecordHashContentInvalid => 2003,
            Self::FechaHoraHusoGenRegistroOutOfRange => 2004,
            Self::ImporteTotalInvalid2005 => 2005,
            Self::CuotaTotalInvalid2006 => 2006,
            Self::ShouldNotBeFirstRecord => 2007,
            Self::PreviousHashMustDifferFromCurrent2008 => 2008,
            Self::ClaveRegimenRequiredForIpsi => 2009,
            Self::Unknown(code) => *code,
        }
    }

    /// The official Spanish description of this error code, as published by
    /// AEAT. Returns `None` for [`BackendError::Unknown`].
    pub fn description(&self) -> Option<&'static str> {
        let description = match self {
            Self::SchemaMissingRequiredField => {
                "El XML no cumple el esquema. Falta informar campo obligatorio."
            }
            Self::UnexpectedXmlParseError => {
                "Se ha producido un error inesperado al parsear el XML."
            }
            Self::HeaderObligadoEmisionNifNotIdentified => {
                "Error en la cabecera: el valor del campo NIF del bloque ObligadoEmision no está identificado."
            }
            Self::HeaderRepresentanteNifNotIdentified => {
                "Error en la cabecera: el valor del campo NIF del bloque Representante no está identificado."
            }
            Self::InvalidDateFormat => "El formato de fecha es incorrecto.",
            Self::NifNotInCensus => "El NIF no está identificado en el censo de la AEAT.",
            Self::CertificateRetrievalError => "Error técnico al obtener el certificado.",
            Self::InvalidNifFormat => "El formato del NIF es incorrecto.",
            Self::PowersOfAttorneyCheckError => "Error técnico al comprobar los apoderamientos.",
            Self::ProcedureCreationError => "Error técnico al crear el trámite.",
            Self::CertificateHolderNotAuthorized => {
                "El titular del certificado debe ser Obligado Emisión, Colaborador Social, Apoderado o Sucesor."
            }
            Self::BlockRecordLimitExceeded => {
                "El XML no cumple con el esquema: se ha superado el límite permitido de registros para el bloque."
            }
            Self::InvoiceRegistrationLimitExceeded => {
                "El XML no cumple con el esquema: se ha superado el límite máximo permitido de facturas a registrar."
            }
            Self::ObligadoEmisionNifInvalid => {
                "El valor del campo NIF del bloque ObligadoEmision es incorrecto."
            }
            Self::HeaderObligadoEmisionNifFormatInvalid => {
                "Error en la cabecera: el campo NIF del bloque ObligadoEmision tiene un formato incorrecto."
            }
            Self::HeaderRepresentanteNifFormatInvalid => {
                "Error en la cabecera: el campo NIF del bloque Representante tiene un formato incorrecto."
            }
            Self::AddressMismatchWithInputFile => {
                "Error técnico: la dirección no se corresponde con el fichero de entrada."
            }
            Self::NonUtf8Encoding => "Error al informar caracteres cuya codificación no es UTF-8.",
            Self::HeaderFechaFinVeriFactuInvalid => {
                "Error en la cabecera: el valor del campo FechaFinVeriFactu es incorrecto, debe ser 31-12-20XX, donde XX corresponde con el año actual o el anterior."
            }
            Self::HeaderIncidenciaInvalid => {
                "Error en la cabecera: el valor del campo Incidencia es incorrecto."
            }
            Self::HeaderRefRequerimientoInvalid => {
                "Error en la cabecera: el valor del campo RefRequerimiento es incorrecto."
            }
            Self::HeaderRepresentanteNifNotInCensus => {
                "Error en la cabecera: el valor del campo NIF del bloque Representante no está identificado en el censo de la AEAT."
            }
            Self::HeaderRepresentanteNombreNotInCensus => {
                "Error en la cabecera: el valor del campo Nombre del bloque Representante no está identificado en el censo de la AEAT."
            }
            Self::HeaderRefRequerimientoRequired => {
                "Error en la cabecera: Si el envío es por requerimiento el campo RefRequerimiento es obligatorio."
            }
            Self::HeaderRefRequerimientoNotAllowed => {
                "Error en la cabecera: el campo RefRequerimiento solo debe informarse en sistemas en remisiones al endpoint del servicio a usar para la contestación a requerimientos de registros de facturación."
            }
            Self::HeaderRemisionVoluntariaOnlyVerifactu => {
                "Error en la cabecera: la remisión voluntaria solo debe informarse en sistemas VERIFACTU."
            }
            Self::TableManagerRetrievalError => {
                "Error técnico en la recuperación del valor del Gestor de Tablas."
            }
            Self::HeaderFinRequerimientoRequired => {
                "Error en la cabecera: el campo FinRequerimiento es obligatorio."
            }
            Self::HeaderFinRequerimientoOnlyNonVerifactu => {
                "Error en la cabecera: el campo FinRequerimiento solo debe informarse en sistemas No VERIFACTU."
            }
            Self::HeaderFinRequerimientoInvalid => {
                "Error en la cabecera: el valor del campo FinRequerimiento es incorrecto."
            }
            Self::CertificateHolderNotRecipientOrAuthorized => {
                "El titular del certificado debe ser el destinatario que realiza la consulta, un Apoderado o Sucesor."
            }
            Self::HeaderRefRequerimientoNotAlphanumeric => {
                "Error en la cabecera: el valor del campo RefRequerimiento no es alfanumérico."
            }
            Self::DatabaseIntegrityError => {
                "Error técnico de base de datos: error en la integridad de la información."
            }
            Self::DatabaseError => "Error técnico de base de datos.",
            Self::QueriedInvoiceNotFound => {
                "La factura consultada para el suministro de pagos/cobros/inmuebles no existe."
            }
            Self::InvoiceNotOwnedByHolder => {
                "La factura especificada no pertenece al titular registrado en el sistema."
            }
            Self::ServiceInactive => "Servicio no activo.",
            Self::UrlNotUsableViaGet => "Esta URL no puede ser utilizada mediante GET.",
            Self::MissingOrInvalidRegistroAltaNode => {
                "No se ha enviado el nodo RegistroAlta o el anterior al nodo RegistroAlta no es correcto."
            }
            Self::MissingOrInvalidRegistroAnulacionNode => {
                "No se ha enviado el nodo RegistroAnulacion o el anterior al nodo RegistroAnulacion no es correcto."
            }
            Self::EmptyRequestOrInvalidEncoding => {
                "Petición vacía en el XML o encoding incorrecto."
            }
            Self::ServiceNotEnabledInProduction => "Servicio no habilitado en producción.",
            Self::NotAuthorizedForInvoiceQuery => {
                "No puede acceder a la consulta de facturas al no estar apoderado en los trámites necesarios."
            }
            Self::AccessSuspended => {
                "Le informamos que su acceso al sistema VERIFACTU ha sido suspendido temporalmente para realizar cualquier solicitud. Para resolver este inconveniente, le solicitamos que se ponga en contacto con nuestro equipo de soporte a través del buzón de correo electrónico verifactu@correo.aeat.es, donde le atenderán con la mayor brevedad posible."
            }
            Self::InvalidFieldValueOrType => "Valor o tipo incorrecto del campo.",
            Self::InvalidCodigoPais => "El valor del campo CodigoPais es incorrecto.",
            Self::InvalidIdType => "El valor del campo IDType es incorrecto.",
            Self::InvalidId => "El valor del campo ID es incorrecto.",
            Self::InvalidNumSerieFactura => "El valor del campo NumSerieFactura es incorrecto.",
            Self::InvalidFechaExpedicionFactura => {
                "El valor del campo FechaExpedicionFactura es incorrecto."
            }
            Self::InvalidTipoFactura => {
                "El valor del campo TipoFactura no está incluido en la lista de valores permitidos."
            }
            Self::InvalidTipoRectificativa => "El valor del campo TipoRectificativa es incorrecto.",
            Self::IdEmisorFacturaNifMismatch => {
                "El NIF del IDEmisorFactura debe ser el mismo que el NIF del ObligadoEmision."
            }
            Self::DestinatarioNifNotInCensus1109 => {
                "El NIF no está identificado en el censo de la AEAT."
            }
            Self::DestinatarioNifNotInCensus1110 => {
                "El NIF no está identificado en el censo de la AEAT."
            }
            Self::CodigoPaisRequiredForNonNifIva => {
                "El campo CodigoPais es obligatorio cuando IDType es distinto de NIF-IVA (02)."
            }
            Self::FechaExpedicionFacturaInFuture => {
                "El campo FechaExpedicionFactura es superior a la fecha actual."
            }
            Self::TipoRectificativaRequired => {
                "Si la factura es de tipo rectificativa, el campo TipoRectificativa debe tener valor."
            }
            Self::TipoRectificativaNotAllowed => {
                "Si la factura no es de tipo rectificativa, el campo TipoRectificativa no debe tener valor."
            }
            Self::FacturasSustituidasOnlyF3 => {
                "Debe informarse el campo FacturasSustituidas sólo si la factura es de tipo F3."
            }
            Self::FacturasRectificadasNotAllowed => {
                "Si la factura no es de tipo rectificativa, el bloque FacturasRectificadas no podrá venir informado."
            }
            Self::ImporteRectificacionRequired => {
                "Si la factura es de tipo rectificativa por sustitución el bloque ImporteRectificacion es obligatorio."
            }
            Self::ImporteRectificacionNotAllowed => {
                "Si la factura no es de tipo rectificativa por sustitución el bloque ImporteRectificacion no debe tener valor."
            }
            Self::IdEmisorFacturaWrongType => {
                "Valor de campo IDEmisorFactura del bloque IDFactura con tipo incorrecto."
            }
            Self::IdNotInCensus => "El campo ID no está identificado en el censo de la AEAT.",
            Self::CodigoPaisIdentifierMismatch => {
                "El campo CodigoPais indicado no coincide con los dos primeros dígitos del identificador."
            }
            Self::InvalidNifFormat1123 => "El formato del NIF es incorrecto.",
            Self::InvalidTipoImpositivo => {
                "El valor del campo TipoImpositivo no está incluido en la lista de valores permitidos."
            }
            Self::FechaOperacionAboveAllowed => {
                "El valor del campo FechaOperacion tiene una fecha superior a la permitida."
            }
            Self::CodigoPaisEsOnlyPasaporteOrNoCensado => {
                "El valor del CodigoPais solo puede ser ES cuando el IDType sea Pasaporte (03) o No Censado (07). Si IDType es No Censado (07) el CodigoPais debe ser ES (España)."
            }
            Self::InvalidTipoRecargoEquivalencia => {
                "El valor del campo TipoRecargoEquivalencia no está incluido en la lista de valores permitidos."
            }
            Self::NoFacturacionAgreement => "No existe acuerdo de facturación.",
            Self::FacturacionAgreementRetrievalError => {
                "Error técnico al obtener el acuerdo de facturación."
            }
            Self::NumSerieFacturaInvalidChars => {
                "El campo NumSerieFactura contiene caracteres no permitidos."
            }
            Self::IdMustBeNaturalPersonNifForNoCensado => {
                "El valor del campo ID ha de ser el NIF de una persona física cuando el campo IDType tiene valor No Censado (07)."
            }
            Self::TipoImpositivoOnlyBefore2012 => {
                "El valor del campo TipoImpositivo es incorrecto, el valor informado solo es permitido para FechaOperacion o FechaExpedicionFactura inferior o igual al año 2012."
            }
            Self::FechaExpedicionFacturaTooOld => {
                "El valor del campo FechaExpedicionFactura no debe ser inferior a la fecha actual menos veinte años."
            }
            Self::FechaOperacionTooOld => {
                "El valor del campo FechaOperacion no debe ser inferior a la fecha actual menos veinte años."
            }
            Self::TipoRecargoEquivalenciaOnlyBefore2012 => {
                "El valor del campo TipoRecargoEquivalencia es incorrecto, el valor informado solo es permitido para FechaOperacion o FechaExpedicionFactura inferior o igual al año 2012."
            }
            Self::FacturaSimplificadaArticulos7273InvalidValue => {
                "El campo FacturaSimplificadaArticulos7273 solo acepta valores N o S."
            }
            Self::MacrodatoInvalidValue => "El campo Macrodato solo acepta valores N o S.",
            Self::MacrodatoOnlyIfTotalAbove100M => {
                "El campo Macrodato solo debe ser informado con valor S si el valor de ImporteTotal es igual o superior a +-100.000.000"
            }
            Self::MacrodatoRequiredIfTotalAbove100M => {
                "Si el campo ImporteTotal está informado y es igual o superior a +-100.000.000 el campo Macrodato debe estar informado con valor S."
            }
            Self::CuotaRepercutidaBaseImponibleACosteSignMismatch => {
                "Los campos CuotaRepercutida y BaseImponibleACoste deben tener el mismo signo."
            }
            Self::CuotaRepercutidaInvalidForBaseImponible => {
                "El campo CuotaRepercutida tiene un valor incorrecto para el valor de los campos BaseImponibleOimporteNoSujeto y TipoImpositivo suministrados."
            }
            Self::CuotaRepercutidaBaseImponibleSignMismatch => {
                "Los campos CuotaRepercutida y BaseImponibleOimporteNoSujeto deben tener el mismo signo."
            }
            Self::CuotaRepercutidaInvalidForBaseImponibleACoste => {
                "El campo CuotaRepercutida tiene un valor incorrecto para el valor de los campos BaseImponibleACoste y TipoImpositivo suministrados."
            }
            Self::InvalidDateFormat1145 => "Formato de fecha incorrecto.",
            Self::FechaExpedicionBeforeOperacionRestriction => {
                "Sólo se permite que la fecha de expedicion de la factura sea anterior a la fecha operación si los detalles del desglose son ClaveRegimen 14 o 15 e Impuesto 01, 03 o vacío."
            }
            Self::ClaveRegimen14FechaOperacionRequired => {
                "Si ClaveRegimen es 14, FechaOperacion es obligatoria y debe ser posterior a la FechaExpedicionFactura."
            }
            Self::ClaveRegimen14TipoFacturaRestriction => {
                "Si la ClaveRegimen es 14, el campo TipoFactura debe ser F1, R1, R2, R3 o R4."
            }
            Self::ClaveRegimen14DestinatarioNifRestriction => {
                "Si ClaveRegimen es 14, el NIF de Destinatarios debe estar identificado en el censo de la AEAT y comenzar por P, Q, S o V."
            }
            Self::F2BaseImponibleSumLimit => {
                "Cuando TipoFactura sea F2 y no este informado NumRegistroAcuerdoFacturacion o FacturaSinIdentifDestinatarioArt61d no sea S el sumatorio de BaseImponibleOimporteNoSujeto y CuotaRepercutida de todas las líneas de detalle no podrá ser superior a 3.000."
            }
            Self::EmitidaPorTerceroODestinatarioInvalidValue => {
                "El campo EmitidaPorTerceroODestinatario solo acepta valores T o D."
            }
            Self::FechaExpedicionBeforeOct2024 => {
                "La fecha de expedición no puede ser inferior al 28 de octubre de 2024."
            }
            Self::RechazoPrevioInvalidValue => {
                "Valor del campo RechazoPrevio no válido, solo podrá incluirse el campo RechazoPrevio con valor X si se ha informado el campo Subsanacion y tiene el valor S."
            }
            Self::RectifiedInvoiceIssuerNifNotIdentified => {
                "El NIF del emisor de la factura rectificada/sustitutiva no se ha podido identificar en el censo de la AEAT."
            }
            Self::TerceroBlockWithoutEmitidaPorTerceroODestinatario => {
                "Se está informando el bloque Tercero sin estar informado el campo EmitidaPorTerceroODestinatario."
            }
            Self::IdOtroNifIvaInvalidTipoFactura => {
                "Para el bloque IDOtro y IDType NIF-IVA (02), el valor de TipoFactura es incorrecto."
            }
            Self::CuponInvalidValue => {
                "El valor de cupón solo puede ser S o N si está informado. El valor de cupón sólo puede ser S si el tipo de factura es R1 o R5."
            }
            Self::EmitidaPorTerceroODestinatarioMissingBlock => {
                "Se está informando EmitidaPorTerceroODestinatario, pero no se informa el bloque correspondiente."
            }
            Self::TerceroBlockWhileDestinatario => {
                "Se está informando del bloque Tercero cuando se indica que se va a informar de Destinatario."
            }
            Self::TipoImpositivo5RecargoRestriction => {
                "Si el TipoImpositivo es 5%, sólo se admite TipoRecargoEquivalencia 0,5 o 0,62."
            }
            Self::RechazoPrevioSInvalidWithoutSubsanacion => {
                "El valor del campo RechazoPrevio no es válido, no podrá incluirse el campo RechazoPrevio con valor S si no se ha informado del campo Subsanacion o tiene el valor N."
            }
            Self::TipoImpositivo21RecargoRestriction => {
                "Si el TipoImpositivo es 21%, sólo se admite TipoRecargoEquivalencia 5,2 ó 1,75."
            }
            Self::TipoImpositivo10RecargoRestriction => {
                "Si el TipoImpositivo es 10%, sólo se admite TipoRecargoEquivalencia 1,4."
            }
            Self::TipoImpositivo4RecargoRestriction => {
                "Si el TipoImpositivo es 4%, sólo se admite TipoRecargoEquivalencia 0,5."
            }
            Self::TipoImpositivo0RecargoRestriction => {
                "Si el TipoImpositivo es 0% sólo se admite TipoRecargoEquivalencia 0% entre el 1 de enero de 2023 y el 30 de septiembre de 2024."
            }
            Self::TipoImpositivo2RecargoRestriction => {
                "Si el TipoImpositivo es 2% entre el 1 de octubre de 2024 y el 31 de diciembre de 2024, sólo se admite TipoRecargoEquivalencia 0,26."
            }
            Self::TipoImpositivo5Recargo05Restriction => {
                "Si el TipoImpositivo es 5% sólo se admite TipoRecargoEquivalencia 0,5 si Fecha Operacion (Fecha Expedicion Factura si no se informa FechaOperacion) es mayor o igual que el 1 de julio de 2022 y el 31 de diciembre de 2022."
            }
            Self::TipoImpositivo5Recargo062Restriction => {
                "Si el TipoImpositivo es 5% sólo se admite TipoRecargoEquivalencia 0,62 si Fecha Operacion (Fecha Expedicion Factura si no se informa FechaOperacion) es mayor o igual que el 1 de enero de 2023 y el 30 de septiembre de 2024."
            }
            Self::TipoImpositivo75RecargoRestriction => {
                "Si el TipoImpositivo es 7,5% entre el 1 de octubre de 2024 y el 31 de diciembre de 2024, sólo se admite TipoRecargoEquivalencia 1."
            }
            Self::TipoImpositivo0RecargoFromOct2024 => {
                "Si el TipoImpositivo es 0%, desde el 1 de octubre del 2024, sólo se admite TipoRecargoEquivalencia 0,26."
            }
            Self::SubsanacionOrRechazoPrevioInvalid => {
                "El valor del campo Subsanacion o RechazoPrevio no se encuentra en los valores permitidos."
            }
            Self::NifOrObligadoEmisionNull => "El valor del campo NIF u ObligadoEmision son nulos.",
            Self::FechaOperacionFutureRestriction => {
                "Sólo se permite que la fecha de operación sea superior a la fecha actual si los detalles del desglose son ClaveRegimen 14 o 15 e Impuesto IVA(01) o IGIC(03) o vacío."
            }
            Self::RegistroAnteriorFechaExpedicionInvalid => {
                "El valor del campo FechaExpedicionFactura del bloque RegistroAnteriores incorrecto."
            }
            Self::RegistroAnteriorNumSerieFacturaInvalid => {
                "El valor del campo NumSerieFactura del bloque RegistroAnterior es incorrecto."
            }
            Self::SistemaInformaticoNifInvalid => {
                "El valor de campo NIF del bloque SistemaInformatico es incorrecto."
            }
            Self::IdSistemaInformaticoInvalid => {
                "El valor de campo IdSistemaInformatico del bloque SistemaInformatico es incorrecto."
            }
            Self::TerceroBlockError => "Error en el bloque de Tercero.",
            Self::SistemaInformaticoBlockError => "Error en el bloque de SistemaInformatico.",
            Self::EncadenamientoBlockError => "Error en el bloque de Encadenamiento.",
            Self::InvalidCalificacionOperacion => {
                "El valor del campo CalificacionOperacion es incorrecto."
            }
            Self::InvalidOperacionExenta => "El valor del campo OperacionExenta es incorrecto.",
            Self::FacturaSimplificadaArticulos7273TipoFacturaRestriction => {
                "El campo FacturaSimplificadaArticulos7273 solo se podrá rellenar con S si TipoFactura es de tipo F1 o F3 o R1 o R2 o R3 o R4."
            }
            Self::FacturaSinIdentifDestinatarioArt61dInvalidValue => {
                "El campo FacturaSinIdentifDestinatarioArt61d solo acepta valores S o N."
            }
            Self::FacturaSinIdentifDestinatarioArt61dTipoFacturaRestriction => {
                "El campo FacturaSinIdentifDestinatarioArt61d solo se podrá rellenar con S si TipoFactura es de tipo F2 o R5."
            }
            Self::TerceroRequiredWhenEmitidaPorTercero => {
                "Si EmitidaPorTercerosODestinatario es igual a T el bloque Tercero será de cumplimentación obligatoria."
            }
            Self::TerceroOnlyWhenEmitidaPorTercero => {
                "Sólo se podrá cumplimentarse el bloque Tercero si el valor de EmitidaPorTercerosODestinatario es T."
            }
            Self::TerceroNifMustDifferFromObligadoEmision => {
                "El NIF del bloque Tercero debe ser diferente al NIF del ObligadoEmision."
            }
            Self::DestinatariosRequiredForTipoFactura => {
                "Si TipoFactura es F1 o F3 o R1 o R2 o R3 o R4 el bloque Destinatarios tiene que estar cumplimentado."
            }
            Self::DestinatariosNotAllowedForF2R5 => {
                "Si TipoFactura es F2 o R5 el bloque Destinatarios no puede estar cumplimentado."
            }
            Self::R3OnlyNifOrNoCensado => {
                "Si TipoFactura es R3 sólo se admitirá NIF o IDType = No Censado (07)."
            }
            Self::R2OnlyNifNoCensadoOrNifIva => {
                "Si TipoFactura es R2 sólo se admitirá NIF o IDType = No Censado (07) o NIF-IVA (02)."
            }
            Self::DestinatariosNifMustBeIdentifiedAndDiffer => {
                "En el bloque Destinatarios si se identifica mediante NIF, el NIF debe estar identificado y ser distinto del NIF ObligadoEmision."
            }
            Self::TipoImpositivoMidPeriodRestriction => {
                "El valor del campo TipoImpositivo es incorrecto, el valor informado solo es permitido para FechaOperacion o FechaExpedicionFactura posterior o igual a 1 de julio de 2022 e inferior o igual a 30 de septiembre de 2024."
            }
            Self::OperacionExentaOrCalificacionRequired => {
                "Al menos uno de los dos campos OperacionExenta o CalificacionOperacion deben estar informados."
            }
            Self::OperacionExentaAndCalificacionExclusive => {
                "OperacionExenta o CalificacionOperacion no pueden ser ambos informados ya que son excluyentes entre sí."
            }
            Self::S2TipoFacturaRestriction => {
                "Si CalificacionOperacion tiene valor Operación Sujeta y No exenta - Con inversión del sujeto pasivo (S2) TipoFactura solo puede ser F1, F3, R1, R2, R3 y R4."
            }
            Self::S2TipoImpositivoCuotaZero => {
                "Si CalificacionOperacion tiene valor Operación Sujeta y No exenta - Con inversión del sujeto pasivo (S2) TipoImpositivo y CuotaRepercutida deberan tener valor 0."
            }
            Self::ExentaE2E3NotAllowedForClaveRegimen01 => {
                "Si Impuesto es '01' (IVA), '03' (IGIC) o no se cumplimenta y ClaveRegimen es 01 no pueden marcarse la OperacionExenta E2, E3."
            }
            Self::ClaveRegimen03OnlyS1 => {
                "Si ClaveRegimen es 03 CalificacionOperacion sólo puede ser Operación Sujeta y No exenta - Sin inversión del sujeto pasivo (S1)."
            }
            Self::ClaveRegimen04OnlyS2OrExenta => {
                "Si ClaveRegimen es 04 CalificacionOperacion sólo puede ser Operación Sujeta y No exenta - Con inversión del sujeto pasivo (S2) o bien OperacionExenta."
            }
            Self::ClaveRegimen06Restriction => {
                "Si ClaveRegimen es 06 TipoFactura no puede ser F2, F3, R5 y BaseImponibleACoste debe estar cumplimentado."
            }
            Self::ClaveRegimen07Restriction => {
                "Si ClaveRegimen es 07 OperacionExenta no puede ser E2, E3, E4 y E5 o CalificacionOperacion no puede ser S2, N1, N2."
            }
            Self::ClaveRegimen10Restriction => {
                "Si ClaveRegimen es 10 CalificacionOperacion tiene que ser N1, TipoFactura F1 y Destinatarios estar identificada mediante NIF."
            }
            Self::ClaveRegimen11TipoImpositivo21 => {
                "Si ClaveRegimen es 11 TipoImpositivo ha de ser 21%."
            }
            Self::CuotaRepercutidaNonZeroOnlyS1 => {
                "La CuotaRepercutida solo podrá ser distinta de 0 si CalificacionOperacion es Operación Sujeta y No exenta - Sin inversión del sujeto pasivo (S1)."
            }
            Self::S1TipoImpositivoCuotaRequired => {
                "Si CalificacionOperacion es Operación Sujeta y No exenta - Sin inversión del sujeto pasivo (S1) y BaseImponibleACoste no está cumplimentada, TipoImpositivo y CuotaRepercutida son obligatorios."
            }
            Self::S1ClaveRegimen06TipoImpositivoCuotaRequired => {
                "Si CalificacionOperacion es Operación Sujeta y No exenta - Sin inversión del sujeto pasivo (S1) y ClaveRegimen es 06, TipoImpositivo y CuotaRepercutida son obligatorios."
            }
            Self::ImporteTotalInvalid => {
                "El campo ImporteTotal tiene un valor incorrecto para el valor de los campos BaseImponibleOimporteNoSujeto, CuotaRepercutida y CuotaRecargoEquivalencia suministrados."
            }
            Self::TerceroNoCensadoNotAllowed => {
                "El bloque Tercero no puede estar identificado con IDType=No Censado (07)."
            }
            Self::TipoUsoPosibleSoloVerifactuInvalidValue => {
                "El campo TipoUsoPosibleSoloVerifactu solo acepta valores N o S."
            }
            Self::TipoUsoPosibleMultiOTInvalidValue => {
                "El campo TipoUsoPosibleMultiOT solo acepta valores N o S."
            }
            Self::NumeroOTAltaInvalid => {
                "El campo NumeroOTAlta debe ser númerico positivo de 4 posiciones."
            }
            Self::ObligadoEmisionBlockError => "Error en el bloque de ObligadoEmision.",
            Self::CuotaTotalInvalid => {
                "El campo CuotaTotal tiene un valor incorrecto para el valor de los campos CuotaRepercutida y CuotaRecargoEquivalencia suministrados."
            }
            Self::IdEmisorFacturaIdentificationError => "Error identificando el IDEmisorFactura.",
            Self::InvalidImpuesto => "El valor del campo Impuesto es incorrecto.",
            Self::InvalidIdEmisorFactura => "El valor del campo IDEmisorFactura es incorrecto.",
            Self::InvalidNombreSistemaInformatico => {
                "El valor del campo NombreSistemaInformatico es incorrecto."
            }
            Self::InvalidSistemaInformaticoIdType => {
                "El valor del campo IDType del sistema informático es incorrecto."
            }
            Self::InvalidIdOtroId => "El valor del campo ID del bloque IDOtro es incorrecto.",
            Self::SistemaInformaticoNifOrIdOtro => {
                "En el bloque SistemaInformatico si se cumplimenta NIF, no deberá existir la agrupación IDOtro y viceversa, pero es obligatorio que se cumplimente uno de los dos."
            }
            Self::GeneradoPorRequiresGenerador => {
                "Si se informa el campo GeneradoPor deberá existir la agrupación Generador y viceversa."
            }
            Self::InvalidGeneradoPor => "El valor del campo GeneradoPor es incorrecto.",
            Self::IndicadorMultiplesOTInvalidValue => {
                "El campo IndicadorMultiplesOT solo acepta valores N o S."
            }
            Self::GeneradoPorERequiresGeneradorNif => {
                "Si el campo GeneradoPor es igual a E debe estar relleno el campo NIF del bloque Generador."
            }
            Self::GeneradorNifOrIdOtro => {
                "En el bloque Generador si se cumplimenta NIF, no deberá existir la agrupación IDOtro y viceversa, pero es obligatorio que se cumplimente uno de los dos."
            }
            Self::GeneradoPorTGeneradorIdTypeNotNoCensado => {
                "Si el valor de GeneradoPor es igual a T el valor del campo IDType del bloque Generador no debe ser No Censado (07)."
            }
            Self::GeneradoPorDEsGeneradorIdTypeRestriction => {
                "Si el valor de GeneradoPor es igual a D y el CodigoPais tiene valor ES (España), el valor del campo IDType del bloque Generador debe ser Pasaporte (03) o No Censado (07)."
            }
            Self::InvalidGeneradorIdType => {
                "El valor del campo IDType del bloque Generador es incorrecto."
            }
            Self::IdOtroEsIdTypePasaporte => {
                "Si se identifica a través de la agrupación IDOtro y CodigoPais tiene valor ES (España), el campo IDType debe valer Pasaporte (03)."
            }
            Self::IdOtroEsIdTypeNoCensado => {
                "Si se identifica a través de la agrupación IDOtro y CodigoPais tiene valor ES (España), el campo IDType debe valer No Censado (07)."
            }
            Self::IdOtroEsIdTypePasaporteOrNoCensado => {
                "Si se identifica a través de la agrupación IDOtro y CodigoPais tiene valor ES (España), el campo IDType debe valer Pasaporte (03) o No Censado (07)."
            }
            Self::TipoImpositivoOct2024Restriction1235 => {
                "El valor del campo TipoImpositivo es incorrecto, el valor informado sólo es permitido para FechaOperacion o FechaExpedicionFactura posterior o igual a 1 de octubre de 2024 e inferior o igual a 31 de diciembre de 2024."
            }
            Self::TipoImpositivoOct2024Restriction1236 => {
                "El valor del campo TipoImpositivo es incorrecto, el valor informado solo es permitido para FechaOperacion o FechaExpedicionFactura posterior o igual a 1 de octubre de 2024 e inferior o igual a 31 de diciembre de 2024."
            }
            Self::N1N2IvaFieldsNotAllowed => {
                "El valor del campo CalificacionOperacion está informado como Operación No sujeta (N1 o N2) y el impuesto es IVA. No se puede informar de los campos TipoImpositivo, CuotaRepercutida, TipoRecargoEquivalencia y CuotaRecargoEquivalencia."
            }
            Self::ExentaFieldsNotAllowed => {
                "Si la operacion es exenta no se puede informar ninguno de los campos TipoImpositivo, CuotaRepercutida, TipoRecargoEquivalencia y CuotaRecargoEquivalencia."
            }
            Self::DestinatarioBlockError => "Error en el bloque Destinatario.",
            Self::IdEmisorFacturaBlockError => "Error en el bloque de IdEmisorFactura.",
            Self::SistemaInformaticoRetrievalError => {
                "Error técnico al obtener el SistemaInformatico."
            }
            Self::SistemaInformaticoNotFound => "No existe el sistema informático.",
            Self::TimeZoneDateCalculationError => {
                "Error técnico al obtener el cálculo de la fecha del huso horario."
            }
            Self::FechaHoraHusoGenRegistroInvalidFormat => {
                "El campo FechaHoraHusoGenRegistro tiene un formato incorrecto."
            }
            Self::ClaveRegimenRequiredForImpuesto => {
                "Si el campo Impuesto está vacío o tiene valor IVA(01) o IPSI(02) o IGIC(03) el campo ClaveRegimen debe de estar cumplimentado."
            }
            Self::InvalidClaveRegimen => "El valor del campo ClaveRegimen es incorrecto.",
            Self::InvalidTipoHuella => "El valor del campo TipoHuella es incorrecto.",
            Self::InvalidPeriodo => "El valor del campo Periodo es incorrecto.",
            Self::InvalidIndicadorRepresentante => {
                "El valor del campo IndicadorRepresentante tiene un valor incorrecto."
            }
            Self::RangoFechaExpedicionDesdeAfterHasta => {
                "El valor de fecha desde debe ser menor que el valor de fecha hasta en RangoFechaExpedicion."
            }
            Self::InvalidIdVersion => "El valor del campo IdVersion tiene un valor incorrecto",
            Self::ClaveRegimen08CalificacionN2Required => {
                "Si ClaveRegimen es 08 el campo CalificacionOperacion tiene que tener el valor Operación No sujeta por reglas de localización (N2) e ir siempre informado."
            }
            Self::InvalidRefExterna => "El valor del campo RefExterna tiene un valor incorrecto.",
            Self::NifIvaXiNotAllowedBefore2021 => {
                "Si FechaOperacion (FechaExpedicionFactura si no se informa FechaOperacion) es anterior a 01/01/2021 no se permite el valor 'XI' para Identificaciones NIF-IVA"
            }
            Self::NifIvaGbNotAllowedFrom2021 => {
                "Si FechaOperacion (FechaExpedicionFactura si no se informa FechaOperacion) es mayor o igual que 01/02/2021 no se permite el valor 'GB' para Identificaciones NIF-IVA"
            }
            Self::FechaExpedicionLimitRetrievalError => {
                "Error técnico al obtener el límite de la fecha de expedición."
            }
            Self::BaseImponibleACosteRestriction => {
                "El campo BaseImponibleACoste solo puede estar cumplimentado si la ClaveRegimen es = '06' o Impuesto = '02' (IPSI) o Impuesto = '05' (Otros)."
            }
            Self::GeneradorNifInvalid => {
                "El valor de campo NIF del bloque Generador es incorrecto."
            }
            Self::GeneradorNifMustBeIdentifiedAndDiffer => {
                "En el bloque Generador si se identifica mediante NIF, el NIF debe estar identificado y ser distinto del NIF ObligadoEmision."
            }
            Self::ClaveRegimenOnlyForImpuesto => {
                "El campo ClaveRegimen solo debe de estar cumplimentado si el campo Impuesto está vacío o tiene valor IVA(01) o IPSI(02) o IGIC(03)"
            }
            Self::IndicadorRepresentanteOnlyForObligadoEmision => {
                "El campo IndicadorRepresentante solo debe de estar cumplimentado si se consulta por ObligadoEmision"
            }
            Self::HuellaLengthInvalid => {
                "La longitud de huella no cumple con las especificaciones."
            }
            Self::TipoHuellaLengthInvalid => {
                "La longitud del tipo de huella no cumple con las especificaciones."
            }
            Self::PrimerRegistroLengthInvalid => {
                "La longitud del campo primer Registro no cumple con las especificaciones."
            }
            Self::TipoFacturaLengthInvalid => {
                "La longitud del campo tipo factura no cumple con las especificaciones."
            }
            Self::CuotaTotalLengthInvalid => {
                "La longitud del campo cuota total no cumple con las especificaciones."
            }
            Self::ImporteTotalLengthInvalid => {
                "La longitud del campo importe total no cumple con las especificaciones."
            }
            Self::FechaHoraHusoGenRegistroLengthInvalid => {
                "La longitud del campo FechaHoraHusoGenRegistro no cumple con las especificaciones."
            }
            Self::RegistroAnteriorBlockInvalid => {
                "El bloque Registro Anterior no esta informado correctamente."
            }
            Self::InvalidMostrarNombreRazonEmisor => {
                "El valor del campo MostrarNombreRazonEmisor tiene un valor incorrecto."
            }
            Self::InvalidMostrarSistemaInformatico => {
                "El valor del campo MostrarSistemaInformatico tiene un valor incorrecto."
            }
            Self::MostrarSistemaInformaticoDestinatarioRestriction => {
                "Si se consulta por Destinatario el valor del campo MostrarSistemaInformatico debe valer 'N' o no estar cumplimentado."
            }
            Self::GeneradorBlockError => "Error en el bloque de Generador.",
            Self::InvalidPrimerRegistro => "Valor incorrecto campo primer registro",
            Self::InvalidRechazoPrevio => "Valor incorrecto campo RechazoPrevio",
            Self::InvalidSinRegistroPrevio => "Valor incorrecto campo SinRegistroPrevio",
            Self::InvalidTipoRecargoEquivalenciaForZeroRate => {
                "Valor incorrecto del TipoRecargoEquivalencia para el tipo impositivo 0%."
            }
            Self::PreviousHashMustDifferFromCurrent => {
                "El valor de la huella del registro anterior debe ser diferente a la huella del registro actual"
            }
            Self::RecargoEquivalenciaOnlyS1 => {
                "Solo se puede cumplimentar TipoRecargoEquivalencia y CuotaRecargoEquivalencia cuando CalificacionOperacion tiene valor Operación Sujeta y No exenta - Sin inversión del sujeto pasivo (S1)"
            }
            Self::HeaderNaturalPersonRequiresNombreRazon => {
                "Si el NIF de la cabecera es persona fisica se debe informar tambien de su NombreRazon"
            }
            Self::CounterpartyNaturalPersonRequiresNombreRazon => {
                "Si el NIF de la contraparte es persona fisica se debe informar tambien de su NombreRazon"
            }
            Self::TipoRecargoEquivalenciaRequiresCuota => {
                "Si se ha informado de TipoRecargoEquivalencia tambien se debe informar de CuotaRecargoEquivalencia y viceversa."
            }
            Self::MultipleSistemasInformaticosFound => {
                "Se han encontracado varios Sistemas Informáticos con los datos suministrados, debe filtrar la consulta por más campos del Sistema Informático."
            }
            Self::ClaveRegimen02OnlyOperacionExenta => {
                "Si el impuesto es IVA(01), IGIC(03) o vacio, si ClaveRegimen es 02 solo se podrá informar OperacionExenta."
            }
            Self::FieldContainsInvalidChars => {
                "El valor del campo %s contiene carácteres no validos (<, >, \", ', =)."
            }
            Self::ExpeditionOperationDateValidationError => {
                "Error técnico en la validación de la fecha de expedición/operación."
            }
            Self::ExentaE5RequiresIdOtro => {
                "Si Impuesto es IVA(01) o vacio y si el campo OperacionExenta es igual a 'E5' sólo deberá existir la agrupación IDOtro en el bloque Destinatario."
            }
            Self::IdNotValidNifFormat => "El campo ID no contiene un NIF con formato correcto.",
            Self::PreviousHashNotAlphanumeric => {
                "El HASH del Registro anterior no es alfanumérico."
            }
            Self::HashNotAlphanumeric => "El HASH no es alfanumérico.",
            Self::ClaveRegimen20CalificacionN2Required => {
                "Si ClaveRegimen es 20 el campo CalificacionOperacion tiene que tener el valor Operación No sujeta por reglas de localización (N2) e ir siempre informado."
            }
            Self::DuplicateRecord => "Registro de facturación duplicado.",
            Self::RecordAlreadyCancelled => "El registro de facturación ya ha sido dado de baja.",
            Self::RecordDoesNotExist => "No existe el registro de facturación.",
            Self::PresenterLacksPermissions => {
                "El presentador no tiene los permisos necesarios para actualizar este registro de facturación."
            }
            Self::CannotModifyInvoiceFiledViaForm => {
                "No es posible modificar la factura ya que ha sido dada de alta vía formulario."
            }
            Self::IncorrectHashCalculation => "El cálculo de la huella suministrada es incorrecta.",
            Self::DestinatariosNifNotInCensus => {
                "El NIF del bloque Destinatarios no está identificado en el censo de la AEAT."
            }
            Self::PreviousRecordHashLengthInvalid => {
                "La longitud de huella del registro anterior no cumple con las especificaciones."
            }
            Self::PreviousRecordHashContentInvalid => {
                "El contenido de la huella del registro anterior no cumple con las especificaciones."
            }
            Self::FechaHoraHusoGenRegistroOutOfRange => {
                "El valor del campo FechaHoraHusoGenRegistro debe ser la fecha actual del sistema de la AEAT, admitiéndose un margen de error de:"
            }
            Self::ImporteTotalInvalid2005 => {
                "El campo ImporteTotal tiene un valor incorrecto para el valor de los campos BaseImponibleOimporteNoSujeto, CuotaRepercutida y CuotaRecargoEquivalencia suministrados."
            }
            Self::CuotaTotalInvalid2006 => {
                "El campo CuotaTotal tiene un valor incorrecto para el valor de los campos CuotaRepercutida y CuotaRecargoEquivalencia suministrados."
            }
            Self::ShouldNotBeFirstRecord => {
                "No debe informarse como primer registro, existen facturas emitidas con el obligado emisión y el sistema informático actual."
            }
            Self::PreviousHashMustDifferFromCurrent2008 => {
                "El valor de la huella del registro anterior debe ser diferente a la huella del registro actual."
            }
            Self::ClaveRegimenRequiredForIpsi => {
                "Si el campo Impuesto tiene valor IPSI(02) el campo ClaveRegimen debe de estar cumplimentado."
            }
            Self::Unknown(_) => return None,
        };
        Some(description)
    }

    /// How AEAT treats this error: whether it rejects the whole submission,
    /// rejects the individual record, or accepts the record subject to later
    /// correction.
    pub fn category(&self) -> BackendErrorCategory {
        match self.code() {
            2000..=2999 => BackendErrorCategory::AcceptedWithErrors,
            3000..=3499 => BackendErrorCategory::RecordRejected,
            3500..=3999 => BackendErrorCategory::SubmissionRejected,
            4000..=4999 => BackendErrorCategory::SubmissionRejected,
            // 1xxx and anything else default to a per-record rejection.
            _ => BackendErrorCategory::RecordRejected,
        }
    }
}

impl From<u32> for BackendError {
    fn from(code: u32) -> Self {
        Self::from_code(code)
    }
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.description() {
            Some(description) => write!(f, "[{}] {description}", self.code()),
            None => write!(f, "[{}] unknown VeriFactu error code", self.code()),
        }
    }
}

#[derive(Debug)]
pub enum DataError {
    InvalidData(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::RequestError(message) => write!(f, "request error: {message}"),
            Error::SoapFault(fault) => match fault.backend_error() {
                Some(backend_error) => write!(f, "AEAT returned an error: {backend_error}"),
                None => write!(f, "AEAT returned an error: {fault}"),
            },
            Error::QrCodeGenerationFailed => write!(f, "QR code generation failed"),
            Error::IoError(err) => write!(f, "I/O error: {err}"),
            Error::PemError(message) => write!(f, "PEM error: {message}"),
            Error::ReqwestError(err) => write!(f, "reqwest error: {err}"),
        }
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataError::InvalidData(message) => write!(f, "invalid data: {message}"),
        }
    }
}

impl std::error::Error for DataError {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<pem::PemError> for Error {
    fn from(err: pem::PemError) -> Self {
        Error::PemError(err.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::ReqwestError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every documented AEAT error code, across the three rejection groups.
    const DOCUMENTED_CODES: &[u32] = &[
        // Group: whole-submission rejection.
        4102, 4103, 4104, 4105, 4106, 4107, 4108, 4109, 4110, 4111, 4112, 4113, 4114, 4115, 4116,
        4117, 4118, 4119, 4120, 4121, 4122, 4123, 4124, 4125, 4126, 4127, 4128, 4129, 4130, 4131,
        4132, 4133, 3500, 3501, 3502, 3503, 4134, 4135, 4136, 4137, 4138, 4139, 4140, 4141,
        // Group: invoice/record rejection.
        1100, 1101, 1102, 1103, 1104, 1105, 1106, 1107, 1108, 1109, 1110, 1111, 1112, 1114, 1115,
        1116, 1117, 1118, 1119, 1120, 1121, 1122, 1123, 1124, 1125, 1126, 1127, 1128, 1129, 1130,
        1131, 1132, 1133, 1134, 1135, 1136, 1137, 1138, 1139, 1140, 1142, 1143, 1144, 1145, 1146,
        1147, 1148, 1149, 1150, 1151, 1152, 1153, 1154, 1155, 1156, 1157, 1158, 1159, 1160, 1161,
        1162, 1163, 1164, 1165, 1166, 1167, 1168, 1169, 1170, 1171, 1172, 1173, 1174, 1175, 1176,
        1177, 1178, 1179, 1180, 1181, 1182, 1183, 1184, 1185, 1186, 1187, 1188, 1189, 1190, 1191,
        1192, 1193, 1194, 1195, 1196, 1197, 1198, 1199, 1200, 1201, 1202, 1203, 1205, 1206, 1207,
        1208, 1209, 1210, 1211, 1212, 1213, 1214, 1215, 1216, 1217, 1218, 1219, 1220, 1221, 1222,
        1223, 1224, 1225, 1226, 1227, 1228, 1229, 1230, 1231, 1232, 1233, 1234, 1235, 1236, 1237,
        1238, 1239, 1240, 1241, 1242, 1243, 1244, 1245, 1246, 1247, 1248, 1249, 1250, 1251, 1252,
        1253, 1254, 1255, 1256, 1257, 1258, 1259, 1260, 1261, 1262, 1263, 1264, 1265, 1266, 1267,
        1268, 1269, 1270, 1271, 1272, 1273, 1274, 1275, 1276, 1277, 1278, 1281, 1282, 1283, 1284,
        1285, 1286, 1287, 1288, 1289, 1290, 1291, 1292, 1293, 3000, 3001, 3002, 3003, 3004,
        // Group: accepted, requires later correction (subsanación).
        2000, 2001, 2002, 2003, 2004, 2005, 2006, 2007, 2008, 2009,
    ];

    #[test]
    fn documented_codes_round_trip() {
        for &code in DOCUMENTED_CODES {
            let error = BackendError::from_code(code);
            assert_eq!(
                error.code(),
                code,
                "code {code} did not round-trip through from_code()/code()"
            );
            assert!(
                error.description().is_some(),
                "code {code} is missing a description"
            );
            assert!(
                !matches!(error, BackendError::Unknown(_)),
                "documented code {code} resolved to Unknown"
            );
        }
    }

    #[test]
    fn unknown_codes_are_preserved() {
        let error = BackendError::from_code(9999);
        assert_eq!(error.code(), 9999);
        assert_eq!(error, BackendError::Unknown(9999));
        assert!(error.description().is_none());
    }

    #[test]
    fn category_matches_rejection_group() {
        assert_eq!(
            BackendError::from_code(4102).category(),
            BackendErrorCategory::SubmissionRejected
        );
        assert_eq!(
            BackendError::from_code(3500).category(),
            BackendErrorCategory::SubmissionRejected
        );
        assert_eq!(
            BackendError::from_code(1108).category(),
            BackendErrorCategory::RecordRejected
        );
        assert_eq!(
            BackendError::from_code(3000).category(),
            BackendErrorCategory::RecordRejected
        );
        assert_eq!(
            BackendError::from_code(2000).category(),
            BackendErrorCategory::AcceptedWithErrors
        );
    }
}
