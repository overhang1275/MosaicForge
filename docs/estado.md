# MosaicForge — Estado y punto de reanudación

Última actualización: 2026-10-14. Rama: feat/cpu-contracts.

## Leer al retomar

1. ../AGENTS.md.
2. Este archivo.
3. plan-desarrollo.md (orden vigente actualizado).
4. adr/0002-orden-incremental.md y diseno-mvp.md.
5. superpowers/plans/2026-10-14-motor-cpu.md y avance-tarea-1.md.
6. avance-tarea-2.md, adr/0003-determinismo.md y graphify.md.
7. avance-tarea-3.md y svg-admitido.md.
8. tarea-4-submodulos.md: checklist de ejecución y pausa antes de cada submódulo.
9. avance-tarea-4.4.md y adr/0004-cobertura-cpu-y-encuadre.md: cobertura, zoom y límites.

## Instrucciones vigentes del usuario

- Checklist visible, marcar y tachar solo lo completado.
- Planes, documentación y registros de continuidad en docs/.
- Usar Ponytail, Graphify y Superpowers.
- Preferir librerías open source; CUDA propietaria es excepción opcional permitida.
- Orden obligatorio: Rust CPU/CLI → frontend contra ese motor → API Java/trabajos → CUDA medida → Python solo si las reglas no bastan.
- Justificar cada frontera real entre lenguajes con ADR; no crear puertos/capas para usos hipotéticos.
- No pasar de fase 1 sin confirmación explícita del usuario.
- Tarea 4 se ejecuta por submódulos; preguntar antes de avanzar al siguiente para controlar tokens. Esta instrucción prevalece sobre ejecución continua del skill.
- Usar DuckDB para JSON/TXT/CSV grandes cuando permita consultar/filtrar sin cargar todo; no imponerlo como dependencia del producto.
- Guardar commits cuando las implementaciones funcionen y pasen checks.
- Evitar viewBox/esquinas vacíos y ampliar variación con zoom, posición y giro dentro de límites; semilla más grande se refiere al contenido/superficie SVG, no al número del PRNG.
- Implementar más adelante grupos configurables de N variantes para controlar memoria; incorporado a tarea 6, sin adelantar su código. Mantener reproducibilidad independiente de N y límites de buffers/concurrencia.
- Se permite investigar documentación pública HP SmartStream Mosaic/PrintOS si sirve para requisitos o flujos; registrar fuentes y mantener originales algoritmos/código/UI/assets.

## Avance comprobado

- [x] ~~Documentación, inventario y baseline Git.~~
- [x] ~~Diseño inicial aprobado y elecciones tecnológicas registradas.~~
- [x] ~~Tarea 1: contratos v1, tipos Rust generados, validación de entrada/resultados y hash.~~
- [x] ~~Revisión independiente y corrección de dos hallazgos P2.~~
- [x] ~~Plan revisado según las nuevas consideraciones, con criterios medibles.~~
- [x] ~~Usuario autorizó continuar fase 1 con tarea 2 después de Graphify.~~
- [x] ~~Generar grafo del snapshot de tarea 1 y registrar sus límites.~~
- [x] ~~Fase 1, tarea 2: PRNG determinista.~~
- [x] ~~Fase 1, tarea 3: importación SVG segura y normalización.~~
- [x] ~~Tarea 4, submódulo 4.1: muestreo geométrico determinista.~~
- [x] ~~Tarea 4, submódulo 4.2: aplicación geométrica segura al SVG.~~
- [x] ~~Tarea 4, submódulo 4.3: apariencia, orden de capas y recorte; pruebas con test.svg.~~
- [x] ~~Corrección de encuadres y muestras más variables con revisión visual.~~
- [x] ~~Tarea 4, submódulo 4.4: distorsión, encuadre/cobertura CPU y reintentos acotados.~~
- [ ] Confirmación del usuario para empezar 4.5: integración y determinismo completo.
- [ ] Fase 1, resto de tarea 4 y tareas 5–6: generación, PNG y CLI.
- [ ] Evidencia de una variante SVG/PNG, determinismo y perfil CPU.
- [ ] Confirmación explícita para pasar a fase 2.

Existen muestreo, geometría, apariencia/crop, distorsión local, ajuste de encuadre, cobertura CPU y búsqueda con ocho intentos, además de contratos, PRNG e importador seguro. Falta integración generate_variant/VariantResult, renderer de exportación, CLI, web, Java e infraestructura. El raster CPU pequeño se adelantó para validar cobertura y generar diagnósticos locales.

## Evidencia técnica

- RED inicial: cinco tests; dos fallan por seed fuera de u64 aceptada y hash vacío.
- GREEN inicial: cinco tests pasan.
- Ampliación: diez tests finales pasan, cero fallos/omitidos; incluyen overflow de resultados y opcionales null. Overflow de resultados tuvo RED observado antes de corregir.
- Hubo un error de sintaxis en una prueba y un warning collapsible_if de Clippy; corregidos, no pendientes.
- Checks finales con exit code 0: cargo fmt --all -- --check; cargo clippy --workspace --all-targets --locked -- -D warnings; cargo test --workspace --locked.
- Revisión independiente por lectura: dos P2 corregidos (null/schema y overflow de resultado); revisión posterior sin hallazgos materiales.
- Licencias declaradas de dependencias fijadas registradas en dependencias-motor.md; LICENSE del producto pendiente.
- Tarea 2: siete tests RNG añadidos; RED observado antes de implementación, GREEN total 17 tests; fmt/Clippy también pasan. Revisión independiente por lectura sin hallazgos materiales.
- Tarea 3: quince tests SVG nuevos; RED inicial y reproducciones de revisión comprobadas, GREEN total 32 tests. fmt/Clippy --locked -D warnings pasan. Revisión por lectura: todos los hallazgos originales corregidos, último P2 local/acumulado confirmado resuelto. Sin medir RAM o throughput.
- Submódulo 4.1: cinco pruebas transforms añadidas, RED observado y GREEN total 37 tests, cero fallos/omitidos; fmt y Clippy pasan. Revisión local, sin subagente por este paso pequeño. Graphify consultado con budget 800 y resultado truncado; se verificó código actual directamente por snapshot antiguo.
- Submódulo 4.2: siete pruebas geométricas añadidas; RED observado y GREEN total 44 tests, cero fallos/omitidos; fmt, Clippy y diff --check pasan. Identidad, orden, centro del viewBox, transform previo, clips/capas, repetición y límites comprobados. Revisión local; detalle en tarea-4-submodulos.md.
- Submódulo 4.3: ocho pruebas añadidas, RED observado y GREEN total 52 tests; fmt/Clippy pasan. Una prueba local omitida en suite normal, ejecutada aparte con test.svg y pasó. Original rechazado explícitamente; copia preparada y tres SVG generados con conservación de paths y repetición por índice. Evidencia y comandos en avance-tarea-4.3.md.
- Submódulo 4.4: ocho pruebas añadidas, RED observado y GREEN total 60 tests CPU; fmt/Clippy pasan. Tres pruebas locales omitidas en suite normal, las tres ejecutadas aparte y pasan. Se revisaron seis PNG de diagnóstico; originales conservados, nuevas muestras más variables y sin celdas vacías de esquina. Evidencia en avance-tarea-4.4.md y ADR 0004. Revisión local; revisión global independiente pendiente de 4.5.
- No hay benchmarks de render, pruebas CUDA, tests del generador final ni pruebas de fronteras Java/web ejecutados todavía. El benchmark de Graphify estima tokens de navegación, no rendimiento del motor.

## Git y recuperación

| Commit | Propósito | Verificación |
|---|---|---|
| b8cc6d8 | Baseline documental | diff --check y herramientas excluidas |
| 1c8f5ed | Registro de baseline | Commit documental |
| 2f1e6d4 | Contratos y validación Rust tarea 1 | Diez tests, fmt, Clippy, revisión corregida |
| 30ac241 | Philox y dominio de muestreo tarea 2 | 17 tests, fmt, Clippy, revisión sin hallazgos |
| cf8c819 | Importador SVG seguro y normalización tarea 3 | 32 tests, fmt, Clippy, hallazgos originales resueltos |
| ff7df4f | Muestreo geométrico submódulo 4.1 | 37 tests, fmt, Clippy, revisión local |
| b4b346b | Aplicación geométrica submódulo 4.2 | 44 tests, fmt, Clippy, revisión local |
| 9eeaa95 | Apariencia y recorte submódulo 4.3 | 52 tests CPU + prueba local test.svg, fmt, Clippy |
| 95da985 | Encuadre, cobertura, distorsión y reintentos 4.4 | 60 tests CPU + tres pruebas locales, fmt, Clippy, revisión visual |

Autor técnico de commits: Codex <codex@localhost>, configuración por comando, sin cambiar identidad global. No se hizo push, merge, reset ni clean. No se versionan target/, dependencias descargadas ni resultados de usuarios.

## Decisiones / límites

- Java 21/Spring Boot/Maven Wrapper, PostgreSQL y Valkey aceptados para fase 3; no introducirlos antes.
- Web fase 2 requiere diseño y ADR del puente real al motor; HTTP local de preview es una propuesta, no implementada.
- Rust 1.99, Java 21, Node 24.19, pnpm 11.25 y Python 3.12 comprobados; algunas rutas no están en PATH.
- Equipo: i5-1235U, 10 núcleos/12 hilos, 7.69 GiB RAM, Intel UHD; NVIDIA no detectada. CUDA real necesita otro entorno.
- Contenedores no encontrados en las rutas comprobadas; despliegue todavía no probado.
- .graphify-cli/ es herramienta de sesión anterior, excluida del producto. Grafo en graphify-out/: 143 nodos, 248 relaciones, diez comunidades; snapshot anterior a tarea 2. Límites/estimaciones en graphify.md; actualizar antes de consultar relaciones nuevas.
- Una tolerancia de imagen y una aceleración solo se publican cuando se midan.
- Requisito de impresión registrado durante la pausa: evitar geometría oculta innecesaria en exportación final, medir complejidad/tamaño y coste RIP/Fiery. Ver requisitos-exportacion-impresion.md. La causa atribuida al PDF de HP es hipótesis del usuario, no diagnóstico medido; crop actual conserva paths y todavía no es optimización final.

## Punto exacto para continuar

Trabajo pausado por petición del usuario; la información de impresión añadida después de la pausa solo actualiza documentación. Submódulo 4.4 terminado, commit 95da985. Esperar permiso explícito para retomar 4.5: generate_variant, resultados versionados y determinismo por índices/rangos/hilos/grupos, con revisión independiente global. Consultar tarea-4-submodulos.md, avance-tarea-4.4.md, ADR 0004 y requisitos-exportacion-impresion.md. No repetir módulos completados ni pasar a fase 2 sin confirmación. El grafo sigue siendo snapshot anterior a tarea 2.
