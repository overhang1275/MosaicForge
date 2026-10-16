# Fase 2.1 — Plan de implementación de contratos de preview

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans or superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Fijar y verificar la frontera local web–Rust sin implementar todavía HTTP ni Next.js.

**Architecture:** Los esquemas nuevos envuelven los contratos existentes del motor. Un crate Rust de preview empieza solo con tipos y validación de frontera; el servidor se añade en 2.2. Un paquete de herramientas Node genera los tipos TypeScript y comprueba fixtures compartidos sin crear una aplicación web.

**Tech Stack:** Rust 1.99, typify y serde existentes; Node disponible, pnpm, json-schema-to-typescript, TypeScript y Ajv draft-07 como herramientas de contratos. Fijar versiones exactas y lockfile durante la implementación, verificando compatibilidad y licencias; no usar dependencias globales o npx con descarga implícita.

**Spec:** ../specs/2026-10-15-fase-2-web-design.md, aprobado por el usuario el 2026-10-15. Plan aprobado y ejecutado directamente; evidencia en ../../avance-fase-2.1.md.

## Restricciones globales

- Nuevos contratos en contracts/preview/v1; no modificar los esquemas engine/v1 ni PDF publicados, LICENSE, CI o secretos.
- Preview/export de una variante: count=1, salida de 1..512 píxeles por eje. Preview solo PNG; export SVG o PNG.
- SVG UTF-8 hasta 5 MiB; spec hasta 1 MiB; cuerpo JSON hasta 32 MiB; respuesta JSON hasta 16 MiB.
- Hasta 12 previews en la futura UI, secuenciales. En 2.2: un permiso de cómputo, sin cola ilimitada; no implementar ese servidor aquí.
- Seed e índice siguen siendo cadenas decimales u64 canónicas; cero conversiones a Number en TypeScript.
- Generar modelos desde contratos; la generación de tipos no sustituye validación de ejecución.
- No instalar Next.js, transporte HTTP, persistencia ni servicios en este submódulo.
- Fixtures propios pequeños; no incluir test.svg del usuario ni resultados out/ en Git.

## Foco de revisión

1. Una cadena de 20 dígitos puede superar u64 aunque pase el patrón: rechazar 18446744073709551616 (tareas B/C).
2. Un SVG Unicode puede cumplir longitud en caracteres y exceder bytes UTF-8: comprobar bytes (B/C); JSON escapado se limita además en transporte 2.2.
3. Un accepted con artefacto ausente o un rejected con bytes no es una respuesta válida: comprobar relación, MIME y hash (B/C).
4. Una referencia de esquema podría leer fuera del repositorio o resolver red: permitir solo archivos dentro de contracts/ y fallar ante otras referencias (B/C).
5. Un generador puede producir tipos válidos pero distintos entre ejecuciones o de los versionados: generar dos veces, comparar y exigir --check sin diferencias (B/C).

## Archivos e interfaces

Contratos nuevos en contracts/preview/v1/:

- common.schema.json: definiciones RequestId (UUID canónico minúsculo mediante patrón), Artifact y ErrorCode.
- inspect-request.schema.json: InspectRequest { requestId, templateSvg }.
- inspect-response.schema.json: InspectResponse { requestId, templateHash, width, height, viewBox, layerIds, segmentCount, engineVersion, rulesVersion }. viewBox contiene exactamente cuatro números; dimensiones positivas finitas; versiones del motor vigente.
- generation-request.schema.json: GenerationRequest { requestId, templateSvg, generation }, donde generation referencia ../../engine/v1/generation.schema.json.
- generation-response.schema.json: GenerationResponse { requestId, variant, artifact? }, donde variant referencia ../../engine/v1/result.schema.json. Artifact { mimeType, base64 }; MIME solo image/png o image/svg+xml. Ausente al rechazar; presente al aceptar.
- error.schema.json: PreviewError { requestId, code, message }, mensaje de 1..256 caracteres sin fuente ni detalles internos.
- README.md: operaciones /v1/inspect, /v1/preview, /v1/export, códigos HTTP, tamaños en bytes y validaciones semánticas.
- fixtures/: solicitudes/respuestas válidas e inválidas y un index.json con archivo, esquema, operación y resultado esperado. Reutilizar basic.svg como contenido, sin hash cero cuando se pruebe generación real.

Todos los objetos son cerrados y todos los nombres anteriores son wire names exactos. requestId lo crea el BFF por operación y Rust lo devuelve, sin incluirlo en hash de configuración. common.schema.json no es un mensaje HTTP independiente.

ErrorCode: invalid_request, invalid_template, unauthorized, forbidden, payload_too_large, unsupported_media_type, busy, internal_error, engine_unavailable, timeout. HTTP respectivamente 400, 400, 401, 403, 413, 415, 429, 500, 502, 504; 502/504 son del BFF. Método incorrecto: 405. Códigos sin filtrar texto de errores del parser.

**Rust:** nuevo crate mosaicforge-preview, services/engine/crates/preview/{Cargo.toml,build.rs,src/lib.rs,src/contract.rs,src/validation.rs,tests/contract.rs}; registrarlo en services/engine/Cargo.toml y actualizar Cargo.lock si procede. Dependencias core/serde/serde_json/sha2 y build typify/syn/prettyplease como en el motor; base64 para verificar el artefacto, con versión/licencia fijadas al implementarlo. Sin Tokio/Axum todavía.

- contract expone InspectRequest, InspectResponse, GenerationRequest, GenerationResponse, PreviewError desde OUT_DIR. Referencias GenerationSpec y VariantResult usan reemplazos typify hacia mosaicforge_core::contract, sin nuevos modelos manuales.
- validation expone Operation { Preview, Export }, validate_inspect_request(&InspectRequest), validate_generation_request(&GenerationRequest, Operation) y validate_generation_response(&GenerationResponse, Operation); retornan Result<(), mosaicforge_core::EngineError>.
- Validar schema cerrado y restricciones semánticas; si el generador no aplica una restricción, el validador debe hacerlo. Reutilizar validate_spec/validate_result. Verificar bytes UTF-8/spec serializada, u64 y count/dimensiones/formato. Respuesta accepted requiere base64 canónico decodificable, MIME compatible, SHA-256 igual a variant.artifactHash; rejected no lleva artefacto/hash y conserva motivo válido.
- La comprobación de cuerpo/respuesta HTTP serializados y límites durante lectura pertenece a 2.2. Este crate no debe afirmar que serde por sí solo verifica esos límites de transporte.

**Node:** nuevo scripts/contracts/{package.json,pnpm-lock.yaml,generate.mjs,check.mjs,contract.test.mjs,tsconfig.json,typecheck.ts}; tipos generados en contracts/generated/typescript/preview.ts. --check compara en memoria sin sobrescribir. generate exporta generatePreviewTypes(): Promise<string>; check exporta validateFixture(value: unknown, schemaName: string, operation: string): boolean. El test runner es node --test; Ajv valida draft-07 y chequeos semánticos complementan byte lengths/u64/relaciones. Ambos leen solo esquemas locales; denegar resolución HTTP y referencias fuera de contracts/.

Una entrada de compilación TS alcanza todos los mensajes y referencias del motor, evitando copias manuales de GenerationSpec/VariantResult. El archivo typecheck.ts importa los nombres públicos y comprueba seed como string mediante @ts-expect-error para una asignación numérica. La generación no incorpora fechas, rutas absolutas ni orden dependiente del sistema de archivos.

## Tarea A — ADR de frontera y contrato documentado

- [x] Escribir docs/adr/0006-preview-web-rust.md con HTTP local/BFF, alternativas CLI/WASM, coste de importación repetida, token solo servidor, loopback, límites, ausencia de almacenamiento y sustitución por Java en fase 3. Diseño aprobado; implementación todavía pendiente.
- [x] Escribir README de transporte con las interfaces anteriores, estado HTTP y reglas semánticas; distinguir mensajes de transporte de hashes/modelos del motor.
- [x] Revisar coherencia con el diseño y git diff --check; commit documental explícito, sin marcar implementados controles futuros.

## Tarea B — Contratos y tipos Rust verificados

- [x] Crear tests de fixtures con estas aserciones: request válido conserva seed="18446744073709551615"; desconocidos fallan; count=2, width=513, height=0 y PNG/SVG incorrecto para Preview fallan; u64 superior falla; Unicode que excede 5 MiB falla; accepted sin artefacto falla; rejected con artefacto falla; hash o MIME incorrecto falla; error con mensaje vacío falla.
- [x] Registrar el crate mínimo y ejecutar cargo test -p mosaicforge-preview --test contract --offline. Este primer comando puede actualizar el lockfile por el crate nuevo; si no hay dependencias disponibles no confundir ese bloqueo con RED de comportamiento. Observar fallo por interfaces/esquemas todavía ausentes; registrar RED real, no inferido.
- [x] Escribir esquemas y fixtures del manifiesto index.json. En build.rs resolver referencias locales con raíz canónica contracts/, sin red; referencias externas al árbol fallan con diagnóstico. Usar typify y reemplazos para modelos existentes, conservando nombres wire.
- [x] Implementar validación mínima y prueba build_rejects_reference_outside_contracts (con fixture de esquema de prueba), además de tamaños/relaciones anteriores. No duplicar generación, seguridad SVG o geometría: la importación real será 2.2.
- [x] Ejecutar cargo fmt --all -- --check, cargo clippy --workspace --all-targets --locked --offline -- -D warnings y cargo test --workspace --locked --offline. Exigir exit 0; informar pruebas locales ignoradas por separado. Guardar commit con archivos explícitos.

## Tarea C — Tipos TypeScript y mismos fixtures

- [x] Crear contract.test.mjs con pruebas shared_fixture_verdicts, u64_roundtrip, rejects_external_refs y generated_output_is_stable. Verificar que faltan generador/validadores al ejecutar node --test scripts/contracts/contract.test.mjs; registrar RED.
- [x] Crear paquete de herramientas privado y fijar dependencias verificadas con lockfile. json-schema-to-typescript genera declaraciones; Ajv valida draft-07 sin descargas y sin coerción/defaults/eliminación de campos. Registrar licencias y compatibilidad real de Node/pnpm. No instalar el frontend como efecto de este paquete.
- [x] Implementar generatePreviewTypes y validateFixture con las interfaces anteriores. Los fixtures inválidos tienen el mismo veredicto Rust/Node; si un caso depende de operación o semántica debe declararse en index.json y comprobarse en ambos.
- [x] Implementar --check que retorna exit 1 si el generado difiere o falta; no lo reescribe. Probar también que una modificación del esquema cambia salida o validación según corresponda, sin dejar cambios de prueba en el repositorio.
- [x] Ejecutar pnpm --dir scripts/contracts exec node --test contract.test.mjs, pnpm --dir scripts/contracts exec node generate.mjs --check y pnpm --dir scripts/contracts exec tsc --noEmit. Generar dos veces y comparar bytes; exigir exit 0 y ninguna modificación adicional. Guardar commit explícito.

## Cierre de 2.1 y siguiente paso

- [x] Revisión local cruzada de cada fixture, fuentes de schemas, equivalencia de referencias y nombres generados. Un cambio en engine/v1 queda fuera: corregir el envoltorio nuevo, no el contrato existente.
- [x] Actualizar docs/fase-2-submodulos.md, docs/estado.md y docs/avance-fase-2.1.md con comandos, resultados, fallos corregidos, licencias y commits. No presentar paridad HTTP ni seguridad de servidor como probadas todavía.
- [x] Detenerse y pedir confirmación antes de 2.2. Los planes ejecutables de 2.2–2.6 se detallan al llegar, partiendo de contratos ya comprobados; no suponer sus interfaces implementadas.

## Fuentes de herramientas y revisión del plan

Documentación primaria consultada el 2026-10-15: [json-schema-to-typescript](https://github.com/bcherny/json-schema-to-typescript) (generación, MIT y requisitos de Node) y [Ajv JSON Schema](https://ajv.js.org/json-schema.html) (draft-07). Confirmar versiones exactas antes de instalar; estas lecturas no son pruebas de integración.

Auto-revisión: alcance limitado a 2.1, nombres consistentes con envelopes, restricciones y los cinco focos asignados a B/C. Seguridad de sockets, streaming, credenciales, UI, BFF, mediciones y E2E quedan explícitamente en sus submódulos, sin marcarse probados. Ejecución recomendada: directa en este chat, por tareas, para conservar el coste de contexto; revisión global al cierre de fase 2. Usuario aprobó revisión y ejecución directa. Implementación completada; detenerse antes de 2.2.

Cierre: 136 tests Rust, seis Node, 52 fixtures compartidos, formato/Clippy/lint/tipado y regeneración pasan. Decisiones de ejecución y diferencias del plan (artifact:null como ausencia, scripts pnpm run, resolutores probados por separado) registradas en ../../avance-fase-2.1.md. Cada casilla describe trabajo ejecutado; las pausas y controles del servidor futuro no se consideran implementados.
