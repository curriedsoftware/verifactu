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

#[derive(Debug)]
pub enum DataError {
    InvalidData(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::RequestError(message) => write!(f, "request error: {message}"),
            Error::SoapFault(fault) => write!(f, "AEAT returned an error: {fault}"),
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
