# Dependencias del motor CPU

Fecha: 2026-10-14. Registro obtenido con cargo metadata --locked; versiones fijadas en services/engine/Cargo.lock.

Licencias declaradas por los paquetes (no sustituye revisar sus textos de licencia al distribuir). Todas las dependencias listadas declaran licencias open source. LICENSE del proyecto sigue pendiente; los crates locales son publish=false.

Directas runtime: serde, serde_json, regress (patrones generados), sha2, quick-xml 0.42.0, usvg y resvg 0.47.0 sin features por defecto y svgtypes 0.16.1. resvg/tiny-skia se adelantan a 4.4 para medir cobertura CPU; no hay API general de exportación PNG todavía. Directas build: typify sin features por defecto, serde_json, syn y prettyplease. No se instalaron Java, Valkey, PostgreSQL, CUDA ni Python/ML para esta tarea.

| Paquete | Versión | Licencia declarada |
|---|---|---|
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 |
| allocator-api2 | 0.2.21 | MIT OR Apache-2.0 |
| arrayref | 0.3.9 | BSD-2-Clause |
| arrayvec | 0.7.8 | MIT OR Apache-2.0 |
| autocfg | 1.5.1 | Apache-2.0 OR MIT |
| base64 | 0.22.1 | MIT OR Apache-2.0 |
| bitflags | 2.13.2 | MIT OR Apache-2.0 |
| block-buffer | 0.10.4 | MIT OR Apache-2.0 |
| bytemuck | 1.25.2 | Zlib OR Apache-2.0 OR MIT |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 |
| cpufeatures | 0.2.17 | MIT OR Apache-2.0 |
| crc32fast | 1.5.2 | MIT OR Apache-2.0 |
| crypto-common | 0.1.7 | MIT OR Apache-2.0 |
| data-url | 0.3.2 | MIT OR Apache-2.0 |
| digest | 0.10.7 | MIT OR Apache-2.0 |
| dyn-clone | 1.0.20 | MIT OR Apache-2.0 |
| equivalent | 1.0.3 | Apache-2.0 OR MIT |
| euclid | 0.22.14 | MIT OR Apache-2.0 |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 |
| flate2 | 1.1.10 | MIT OR Apache-2.0 |
| float-cmp | 0.9.0 | MIT |
| foldhash | 0.2.0 | Zlib |
| generic-array | 0.14.7 | MIT |
| hashbrown | 0.16.1 | MIT OR Apache-2.0 |
| heck | 0.5.0 | MIT OR Apache-2.0 |
| imagesize | 0.14.0 | MIT |
| itoa | 1.0.18 | MIT OR Apache-2.0 |
| kurbo | 0.13.1 | Apache-2.0 OR MIT |
| libc | 0.2.190 | MIT OR Apache-2.0 |
| log | 0.4.34 | MIT OR Apache-2.0 |
| memchr | 2.8.3 | Unlicense OR MIT |
| miniz_oxide | 0.9.1 | MIT OR Zlib OR Apache-2.0 |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 |
| num-traits | 0.2.19 | MIT OR Apache-2.0 |
| pico-args | 0.5.0 | MIT |
| polycool | 0.4.0 | MIT OR Apache-2.0 |
| png | 0.18.1 | MIT OR Apache-2.0 |
| prettyplease | 0.2.37 | MIT OR Apache-2.0 |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| quick-xml | 0.42.0 | MIT |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| regress | 0.10.5 | MIT OR Apache-2.0 |
| regress | 0.12.0 | MIT OR Apache-2.0 |
| resvg | 0.47.0 | Apache-2.0 OR MIT |
| rgb | 0.8.53 | MIT |
| roxmltree | 0.21.1 | MIT OR Apache-2.0 |
| schemars | 0.8.22 | MIT |
| schemars_derive | 0.8.22 | MIT |
| semver | 1.0.28 | MIT OR Apache-2.0 |
| serde | 1.0.229 | MIT OR Apache-2.0 |
| serde_core | 1.0.229 | MIT OR Apache-2.0 |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 |
| serde_derive_internals | 0.29.1 | MIT OR Apache-2.0 |
| serde_json | 1.0.154 | MIT OR Apache-2.0 |
| sha2 | 0.10.9 | MIT OR Apache-2.0 |
| simd-adler32 | 0.3.10 | MIT |
| simplecss | 0.2.2 | Apache-2.0 OR MIT |
| siphasher | 1.0.4 | MIT OR Apache-2.0 |
| smallvec | 1.16.3 | MIT OR Apache-2.0 |
| strict-num | 0.1.1 | MIT |
| tiny-skia | 0.12.0 | BSD-3-Clause |
| svgtypes | 0.16.1 | Apache-2.0 OR MIT |
| syn | 2.0.119 | MIT OR Apache-2.0 |
| syn | 3.0.9 | MIT OR Apache-2.0 |
| thiserror | 2.0.21 | MIT OR Apache-2.0 |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 |
| tiny-skia-path | 0.12.0 | BSD-3-Clause |
| typenum | 1.20.1 | MIT OR Apache-2.0 |
| typify | 0.8.0 | Apache-2.0 |
| typify-impl | 0.8.0 | Apache-2.0 |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| usvg | 0.47.0 | Apache-2.0 OR MIT |
| version_check | 0.9.5 | MIT/Apache-2.0 |
| xmlwriter | 0.1.0 | MIT |
| zmij | 1.0.23 | MIT |
| zlib-rs | 0.6.8 | Zlib |

## Compatibilidad comprobada

- Windows, Rust 1.99.0; build de contratos y diez pruebas pasan sin GPU ni Python.
- Versiones de API/contratos iniciales: schema 1, reglas 1, motor 0.1.0.
- Tipos generados en OUT_DIR a partir de contracts/engine/v1; no se versionan los archivos generados.
- Configuration hash: SHA-256 de JSON de los tipos validados, con claves ordenadas (serde_json sin preserve_order), excluyendo count y startIndex.
- Hasta 4.5: suite CPU de 67 tests pasa; tres pruebas locales se omiten por defecto y la integrada con test.svg se ejecuta aparte. Máscara de cobertura CPU y PNG de diagnóstico comprobados con resvg 0.47.0 / tiny-skia 0.12.0. API de exportación PNG, presupuestos de salida, CLI y benchmark siguen pendientes. usvg no incluye fontdb/rustybuzz en este build; el importador rechaza imágenes y recursos externos. 4.5 no añade dependencias.
- Metadata --locked actual: 76 paquetes externos, todos con licencia declarada open source. Se añadieron ocho combinaciones paquete/versión respecto al registro anterior.
- 5.1 añade mosaicforge-cpu local (publish=false), dependiente de core; serde_json de tests ya existía. Ninguna versión externa nueva en Cargo.lock. Suite de ese checkpoint: 70 tests CPU; reserva base no implica cota de RAM total.
- 5.2 añade a cpu una referencia a resvg 0.47.0 ya fijado; sin paquetes/versiones externos nuevos. PNG CPU general implementado, 78 tests workspace y prueba local PNG pasan. El estimador de superficies depende del comportamiento de resvg 0.47.0: revisarlo al actualizar. No se midió RSS.

Fuente del generador: https://github.com/oxidecomputer/typify
