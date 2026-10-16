# Avance 2.1 — Contratos de preview

Plan: docs/superpowers/plans/2026-10-15-fase-2-1-contratos.md. Base ec86514. Usuario aprobó plan y ejecución directa en este chat.

- [x] ~~Leer plan, spec, reglas y detectar rama/checkout.~~
- [x] ~~A: ADR y contrato documentado, checkpoint cab8fd8.~~
- [x] ~~B: esquemas, tipos y validación Rust con RED/GREEN, commit 03c1323.~~
- [x] ~~C: tipos TypeScript, 52 fixtures compartidos y regeneración comprobada, commit 5d9776d.~~
- [x] ~~Revisión independiente y checks finales; checkpoints guardados.~~
- [ ] Confirmación del usuario antes de 2.2.

Pre-flight: A define los nombres de envelopes que consumen B/C; coinciden con spec. B/C comparten fixtures y deberán comprobar las mismas reglas semánticas; el servidor conserva además la validación real del motor. Generación de tipos no se confunde con validación.

Ruling: continuar en el checkout/ramal de trabajo actual feat/cpu-contracts — el usuario eligió ejecución directa aquí y pide continuidad/commits recuperables; no crear otro checkout ni mover artefactos — coste si se necesita aislamiento adicional: crear un worktree antes del siguiente submódulo.

Ruling: este documento en docs/ es el ledger persistente — instrucción explícita del usuario sobre continuidad en docs/ prevalece sobre scratch temporal de Superpowers — coste: conservar una página documental adicional.

Ruling: validación Node del sobre incluye reglas semánticas de contrato para comprobar paridad; no implementa transformación, rasterización ni decisiones generativas — es la frontera real de mensajes, no otro motor — coste: mantener pruebas cruzadas al cambiar contratos.

Instalación frontend-app-builder ya realizada en la configuración local del asistente; no es dependencia del producto. No se usa para contratos: se aplicará al diseñar/implementar UI.

## Evidencia durante ejecución

Baseline: cargo test --workspace --locked --offline --quiet pasó con 131 tests y diez locales ignorados. Task A: complete (ec86514..cab8fd8, git diff --check exit 0).

B RED: primero faltaban módulos; después de generar modelos y usar stubs se observaron tres fallos de comportamiento (count, Unicode en bytes y NaN). Se detectó además que typify usa claves de definitions, no title, para reemplazos: bundler conserva títulos únicos para reutilizar GenerationSpec/VariantResult del core. GREEN inicial: cinco tests Rust, con 50 fixtures compartidos. Clippy detectó module_inception del módulo error generado; renombrado el módulo contenedor preview_error. Clippy volvió a pasar.

C RED: faltaba check.mjs y, tras stubs, dos pruebas fallaban de comportamiento (count inválido y exceso UTF-8). GREEN inicial: seis tests Node, generado estable y tsc pasan. Versiones instaladas: json-schema-to-typescript 16.0.0, Ajv 8.20.0, TypeScript 7.0.2; Node bundled 24.19.0 y pnpm 11.25.0. El PATH de la ejecución elevada también contiene otro Node 24.22.0 y pnpm allí no funciona; usar rutas explícitas del runtime bundled para contratos.

Ruling: artifact null significa ausencia, como los opcionales v1 del motor — serde Option ya lo representa así; se explicita en el schema nuevo y en Node para evitar divergencia con Rust, con regresión compartida observada fallar en Node antes de corregir — coste: clientes deben distinguir null/ausente de un artefacto real; accepted nunca permite ninguno de ellos.

Ruling: schema_bundle.rs separa resolución local del build para poder probar referencias sin ejecutar otro compilador — misma frontera real de contratos, sin capa de producto adicional — coste: mantener dos resolutores de herramientas (Rust/Node) con tests equivalentes.

## Revisión independiente y alcance

review_preview_contracts (contexto nuevo, solo lectura) no encontró P1/P2 concretos. Ejecutó seis tests Node, incluidos 52 fixtures; no ejecutó Cargo. Revisión de esquemas, validadores, tipos generados y resolutores. El coordinador ejecuta los checks Rust.

Final: Ruling: HTTP, autenticación, límites durante lectura, concurrencia y cancelación quedan en 2.2/2.3 — el alcance aprobado es contratos; no hay servidor que exponer todavía — coste si se omite después: frontera insegura o consumo sin límite; no dar por probados esos controles ahora.

Final: Ruling: seguridad SVG y decodificación real PNG quedan a cargo del motor — este sobre verifica forma/base64/hash, no legitima bytes de terceros — coste si se confunde: aceptar contenido arbitrario como salida segura; 2.2 debe importar/generar con core/cpu antes de responder.

Final: Ruling: comprobar el formato de export contra la petición concreta requiere contexto en 2.2 — Operation::Export permite SVG/PNG, y el sobre aislado no conoce el formato pedido — coste si se omite: descarga de formato equivocado; añadir prueba de correlación al plan de 2.2.

Final: Ruling: UI/BFF/dirección visual siguen en sus submódulos — el agente de diseño solo preparó una propuesta original por petición del usuario — coste: ninguna validación visual/E2E todavía; no presentar el documento como UI implementada.

Sin menores diferidos reportados por el revisor. No se pidió una segunda revisión.

Formato/lint adicionales: ESLint 10.12.0 y Prettier 3.8.1 fijados porque AGENTS.md exige checks del componente. La primera invocación de formato falló por package.json recién editado sin formatear y pnpm exec del entorno no localizó prettier; se ejecutó el binario local para formatear. Después todos los scripts run format/lint/test/check/typecheck pasaron. Un comando Python previo usó ruta relativa al cwd incorrecto; no cambió archivos y se corrigió con cwd raíz. No hay fallo pendiente de código.

Ruling: los comandos pnpm run sustituyen a exec del plan en este runtime — ejecutan los mismos binarios locales y tienen PATH correcto; exec no localizó prettier — coste: mantener aliases explícitos en package.json, documentados en scripts/contracts/README.md.

CLI de generación: una prueba deliberada de drift observó exit 1, comprobó que --check no escribió y restauró bytes originales en finally; después --check volvió a exit 0. Instalación con --frozen-lockfile --ignore-scripts pasó sin cambios. Licencias declaradas de 97 paquetes instalados en dependencias-preview.json; no equivale a auditoría legal ni comprobación de todas las variantes opcionales de otros sistemas.

Instalaciones de diseño pedidas durante ejecución: 14 skills Taste, web-design-guidelines y playwright-cli, más colección awesome-design-md fuera del repo. CLI global @playwright/cli 0.1.22 verificada por ruta absoluta; no en PATH de esta sesión. Detalle en herramientas-diseno.md. design_frontend creó únicamente diseno/fase-2-direccion-visual.md, sin implementar UI. Las 17 SKILL.md (incluida frontend-app-builder previa) existen.

## Cierre verificado

Task B: complete (cab8fd8..03c1323): cargo fmt --all -- --check, cargo clippy --workspace --all-targets --locked --offline -- -D warnings y cargo test --workspace --locked --offline --quiet finalizaron exit 0. Total 136 tests (74 core + 23 CPU + 34 CLI + 5 preview); diez locales ignorados, sin repetirse en 2.1. El warning conocido de canonicalize C:/Users/init no es fallo de Cargo. Los 52 casos compartidos se ejecutan dentro del test de fixtures.

Task C: complete (03c1323..5d9776d): pnpm run format/lint/test/check/typecheck desde scripts/contracts, exit 0; seis pruebas Node y 52 fixtures. Instalación frozen con scripts desactivados pasó. Inspección Git: contracts/engine sin diferencias. No se modificaron LICENSE, CI, secretos o herramientas de runtime del producto. Un texto del ledger sufrió doble codificación en lectura Python sin encoding explícito; se reparó y verificó UTF-8 antes del cierre.

Siguiente punto: 2.2 servidor HTTP Rust, pendiente confirmación y detalle del plan sobre los contratos ya comprobados. Por solicitud nueva del usuario se prepara mientras tanto un agente reutilizable ui-ux-architect; no reemplaza ni adelanta 2.2–2.4. Esta sesión no ha abierto ni validado la UI de MosaicForge: todavía no existe.
