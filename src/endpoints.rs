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

use std::env;

/// Environment variable read by [`Environment::from_env`] to select the target
/// environment. Anything other than an explicit production value resolves to
/// the safe [`Environment::Test`] default.
pub const ENVIRONMENT_ENV: &str = "VERIFACTU_ENV";

/// Which AEAT environment requests are sent to.
///
/// This is a runtime decision, deliberately defaulting to [`Environment::Test`]
/// so that the safe path is the path of least resistance: reaching production
/// requires an explicit, visible choice at the call site (see
/// [`Environment::from_env`] and the CLI `--production` flag). Targeting the
/// real AEAT systems submits binding fiscal records, so it must never be the
/// implicit default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Environment {
    /// AEAT pre-production ("preproducción") systems. Safe for development and
    /// testing; accepts placeholder identifiers.
    #[default]
    Test,
    /// Real AEAT production systems. Submissions here are legally binding.
    Production,
}

/// The set of AEAT endpoint URLs for a given [`Environment`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Endpoints {
    pub sistema_verifactu: &'static str,
    pub sistema_verifactu_sello: &'static str,
    pub sistema_requerimiento: &'static str,
    pub sistema_requerimiento_sello: &'static str,
    pub qr_url: &'static str,
}

const PRODUCTION_ENDPOINTS: Endpoints = Endpoints {
    sistema_verifactu: "https://www1.agenciatributaria.gob.es/wlpl/TIKE-CONT/ws/SistemaFacturacion/VerifactuSOAP",
    sistema_verifactu_sello: "https://www10.agenciatributaria.gob.es/wlpl/TIKE-CONT/ws/SistemaFacturacion/VerifactuSOAP",
    sistema_requerimiento: "https://www1.agenciatributaria.gob.es/wlpl/TIKE-CONT/ws/SistemaFacturacion/RequerimientoSOAP",
    sistema_requerimiento_sello: "https://www10.agenciatributaria.gob.es/wlpl/TIKE-CONT/ws/SistemaFacturacion/RequerimientoSOAP",
    qr_url: "https://www2.agenciatributaria.gob.es/wlpl/TIKE-CONT/ValidarQR",
};

const TEST_ENDPOINTS: Endpoints = Endpoints {
    sistema_verifactu: "https://prewww1.aeat.es/wlpl/TIKE-CONT/ws/SistemaFacturacion/VerifactuSOAP",
    sistema_verifactu_sello: "https://prewww10.aeat.es/wlpl/TIKE-CONT/ws/SistemaFacturacion/VerifactuSOAP",
    sistema_requerimiento: "https://prewww1.aeat.es/wlpl/TIKE-CONT/ws/SistemaFacturacion/RequerimientoSOAP",
    sistema_requerimiento_sello: "https://prewww10.aeat.es/wlpl/TIKE-CONT/ws/SistemaFacturacion/RequerimientoSOAP",
    qr_url: "https://prewww2.aeat.es/wlpl/TIKE-CONT/ValidarQR",
};

impl Environment {
    /// Resolve the environment from the [`ENVIRONMENT_ENV`] environment
    /// variable. Defaults to [`Environment::Test`]; only the explicit values
    /// `production` / `prod` (case-insensitive) select [`Environment::Production`].
    pub fn from_env() -> Self {
        match env::var(ENVIRONMENT_ENV) {
            Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
                "production" | "prod" => Environment::Production,
                _ => Environment::Test,
            },
            Err(_) => Environment::Test,
        }
    }

    /// The endpoint URLs for this environment.
    pub const fn endpoints(self) -> Endpoints {
        match self {
            Environment::Production => PRODUCTION_ENDPOINTS,
            Environment::Test => TEST_ENDPOINTS,
        }
    }

    /// Whether this is the real production environment.
    pub const fn is_production(self) -> bool {
        matches!(self, Environment::Production)
    }
}
