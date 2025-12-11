# verifactu

A Rust library for communicating with the Spanish Tax Agency's VERI*FACTU system.

## Overview

`verifactu` is a Rust library that provides type-safe bindings and utilities for interacting with the VERI*FACTU API (Sistema de Verificación de Facturas) from the Spanish Tax Agency (Agencia Tributaria Española). This library handles XML serialization/deserialization, request/response validation, and provides helpers for generating QR codes for invoices.

## Features

- **Type-safe schema definitions** for VERI*FACTU XML requests and responses
- **XML serialization/deserialization** using `quick-xml` and `serde`
- **QR code generation** for invoice verification
- **Request validation** with proper error handling
- **Production and development modes** via feature flags
- **Comprehensive test suite** with real-world examples

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
verifactu = "0.1"
```

For production use:

```toml
[dependencies]
verifactu = { version = "0.1", features = ["production"] }
```
