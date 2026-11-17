# verifactu

A Rust library for communicating with the Spanish Tax Agency's VERI*FACTU system.

## Overview

`verifactu` is a Rust library that provides type-safe bindings and utilities for interacting with the VERI*FACTU API (Sistema de Verificación de Facturas) from the Spanish Tax Agency (Agencia Tributaria Española). This library handles XML serialization/deserialization, request/response validation, and provides helpers for generating QR codes for invoices.

## Features

- **Type-safe schema definitions** for VERI*FACTU XML requests and responses
- **XML serialization/deserialization** using `quick-xml` and `serde`
- **QR code generation** for invoice verification
- **Request validation** with proper error handling
- **Test and production environments** selected at runtime, defaulting to the safe test environment
- **Comprehensive test suite** with real-world examples

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
verifactu = "0.1"
```

## Strict when building, lenient when reading

The schema types hold a request to `SuministroInformacion.xsd`, so a value the
AEAT would refuse cannot be built in the first place:

- `NIF` is exactly 9 characters.
- `TextoIDFactura` (`NumSerieFactura`) is 1 to 60 printable ASCII characters,
  without `"`, `'`, `<`, `>` or `=` (AEAT error 1104).
- `SiNo` is `S` or `N`. Only the alta's `RechazoPrevio` has the third value `X`.
- `IdOperacionesTrascendenciaTributaria` (`ClaveRegimen`) and `IDVersion` are
  the XSD enumerations.
- `ImporteTotal` is an `ImporteSgn12_2`: at most 12 integer digits.
- `FechaHoraHuso` (`FechaHoraHusoGenRegistro`) is built from a
  `chrono::DateTime` or from a `YYYY-MM-DDThh:mm:ssTZD` string, and is sent and
  hashed as that exact string.
- `PersonaFisicaJuridicaES`, the `ObligadoEmision` and `Representante` of a
  `Cabecera`, is identified by NIF only. `PersonaFisicaJuridica` also accepts a
  foreign identifier.

Reading is deliberately laxer where the AEAT returns more than a request may
carry: a NIF of any length, a `NumSerieFactura` with any characters and any
`xs:dateTime` are kept exactly as received.

## Choosing the environment

The target AEAT environment is a runtime choice, not a build flag. It defaults
to the safe test ("preproducción") environment, so reaching production always
requires an explicit, visible opt-in:

```rust
use verifactu::{Client, Environment};

// Test environment (the default).
let client = Client::new(http, Environment::Test);

// Production — submissions here are legally binding.
let client = Client::new(http, Environment::Production);

// Or resolve from the VERIFACTU_ENV variable (defaults to Test;
// only VERIFACTU_ENV=production selects production).
let client = Client::new(http, Environment::from_env());
```

The `verifactu` CLI defaults to the test environment and only targets production
when passed `--production`, prompting for interactive confirmation first (use
`--yes` to skip the prompt in automation).
