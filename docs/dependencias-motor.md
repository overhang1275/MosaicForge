# Dependencias del motor CPU

Fecha: 2026-10-14. Registro obtenido con cargo metadata --locked; versiones fijadas en services/engine/Cargo.lock.

Licencias declaradas por los paquetes (no sustituye revisar sus textos de licencia al distribuir). Todas las dependencias listadas declaran licencias open source. LICENSE del proyecto sigue pendiente; los crates locales son publish=false.

Directas runtime: serde, serde_json, regress (requerida por los patrones de tipos generados) y sha2. Directas build: typify sin features por defecto, serde_json, syn y prettyplease. No se instalaron Java, Valkey, PostgreSQL, CUDA ni Python/ML para esta tarea.

| Paquete | Versión | Licencia declarada |
|---|---|---|
| allocator-api2 | 0.2.21 | MIT OR Apache-2.0 |
| block-buffer | 0.10.4 | MIT OR Apache-2.0 |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 |
| cpufeatures | 0.2.17 | MIT OR Apache-2.0 |
| crypto-common | 0.1.7 | MIT OR Apache-2.0 |
| digest | 0.10.7 | MIT OR Apache-2.0 |
| dyn-clone | 1.0.20 | MIT OR Apache-2.0 |
| equivalent | 1.0.3 | Apache-2.0 OR MIT |
| foldhash | 0.2.0 | Zlib |
| generic-array | 0.14.7 | MIT |
| hashbrown | 0.16.1 | MIT OR Apache-2.0 |
| heck | 0.5.0 | MIT OR Apache-2.0 |
| itoa | 1.0.18 | MIT OR Apache-2.0 |
| libc | 0.2.190 | MIT OR Apache-2.0 |
| log | 0.4.34 | MIT OR Apache-2.0 |
| memchr | 2.8.3 | Unlicense OR MIT |
| prettyplease | 0.2.37 | MIT OR Apache-2.0 |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| regress | 0.10.5 | MIT OR Apache-2.0 |
| regress | 0.12.0 | MIT OR Apache-2.0 |
| schemars | 0.8.22 | MIT |
| schemars_derive | 0.8.22 | MIT |
| semver | 1.0.28 | MIT OR Apache-2.0 |
| serde | 1.0.229 | MIT OR Apache-2.0 |
| serde_core | 1.0.229 | MIT OR Apache-2.0 |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 |
| serde_derive_internals | 0.29.1 | MIT OR Apache-2.0 |
| serde_json | 1.0.154 | MIT OR Apache-2.0 |
| sha2 | 0.10.9 | MIT OR Apache-2.0 |
| syn | 2.0.119 | MIT OR Apache-2.0 |
| syn | 3.0.9 | MIT OR Apache-2.0 |
| thiserror | 2.0.21 | MIT OR Apache-2.0 |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 |
| typenum | 1.20.1 | MIT OR Apache-2.0 |
| typify | 0.8.0 | Apache-2.0 |
| typify-impl | 0.8.0 | Apache-2.0 |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| version_check | 0.9.5 | MIT/Apache-2.0 |
| zmij | 1.0.23 | MIT |

## Compatibilidad comprobada

- Windows, Rust 1.99.0; build de contratos y diez pruebas pasan sin GPU ni Python.
- Versiones de API/contratos iniciales: schema 1, reglas 1, motor 0.1.0.
- Tipos generados en OUT_DIR a partir de contracts/engine/v1; no se versionan los archivos generados.
- Configuration hash: SHA-256 de JSON de los tipos validados, con claves ordenadas (serde_json sin preserve_order), excluyendo count y startIndex.
- Rasterizadores, RNG, CLI y benchmark pertenecen a tareas posteriores y no se verificaron todavía.

Fuente del generador: https://github.com/oxidecomputer/typify
