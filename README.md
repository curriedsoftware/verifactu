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
