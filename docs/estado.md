# MosaicForge — Estado y punto de reanudación

Última actualización: 2026-10-15. Rama: feat/cpu-contracts.

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
- 2026-10-15: Fiery queda fuera del alcance por instrucción explícita; no es una validación pendiente ni condición para cerrar fase 1. Tarea 6 autorizada, ejecutada por submódulos.
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
- PDF aprobado como 5.6: un solo PDF multipágina con todas las variantes generadas, una página por variante aceptada en orden de índice. Grupos N no producen PDFs separados. Revisión final de tarea 5 pasa al final de 5.6; diseñar contrato agregado y tamaño físico/sangrado antes de implementar. Aprobación del plan no inicia 5.3–5.6.
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
- [x] ~~Usuario autorizó retomar 4.5.~~
- [x] ~~Tarea 4.5 y cierre de tarea 4: integración, determinismo y revisión independiente con dos P2 corregidos.~~
- [x] ~~Usuario autorizó tarea 5 dividida en submódulos.~~
- [x] ~~5.1: validación de salida y presupuesto base; 70 tests, formato y Clippy pasan.~~
- [x] ~~5.2: PNG CPU seguro, viewport/fondo, presupuesto de temporales y pruebas reales.~~
- [x] ~~5.3 autorizado y terminado: SVG final, fondo/encuadre y paridad visual CPU.~~
- [x] ~~5.4 autorizado y terminado: integración SVG/PNG y hash de bytes finales.~~
- [x] ~~5.5 autorizado y terminado: flujo integrado validado con test.svg preparado, seis artefactos y rechazo.~~
- [x] ~~Inicio de diseño 5.6 autorizado; primera prueba elegida de 100 × 100 mm, sin sangrado.~~
- [x] ~~5.6 y tarea 5 completos: PDF vectorial único, recorte real, contratos, métricas y revisión independiente.~~
- [x] ~~Fase 1, tarea 6: CLI incremental, publicación sin sobrescritura y cancelación.~~
- [x] ~~6.1: binario mosaicforge e inspect, lectura acotada y metadata JSON; 102 tests y checks pasan.~~
- [x] ~~Usuario autorizó 6.2 y 6.3 juntos. 6.2 completo: generación SVG/PNG y publicación segura, 109 tests y checks pasan.~~
- [x] ~~6.3 completo: grupos N, 1|2 hilos y presupuesto PNG compartido; 201 variantes reproducibles, 115 tests y checks pasan, revisión sin bloqueos.~~
- [x] ~~6.4 autorizado y terminado: manifiesto persistente exclusivo, resultados de toda la ventana y Ctrl+C cooperativo; 122 tests y checks pasan, señal real Windows exit 130.~~
- [x] ~~6.5 autorizado y terminado: generate-pdf, un PDF por trabajo, cancelación/publicación segura, 129 tests y QA real de tres páginas pasan.~~
- [x] ~~6.6 autorizado y terminado: perfil release por etapa y memoria, revisión global, encuadre final corregido y cierre funcional de fase 1. 131 tests pasan.~~
- [x] ~~Evidencia real SVG/PNG/PDF, determinismo y perfil CPU registrados en perfil-cpu-fase1.md y mediciones/.~~
- [ ] Confirmación explícita para pasar a fase 2.

Existen generador, reglas/validación, SVG/PNG integrados y PDF vectorial único con dimensiones físicas, sangrado, recorte real y hashes separados. Validado con copia preparada de test.svg, Poppler/pypdf y revisión independiente. CLI/publicación/cancelación y perfil por etapas completos. Web, Java e infraestructura corresponden a fases posteriores. La máscara pequeña sigue siendo solo validación de cobertura.

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
- Submódulo 4.5: cinco pruebas de integración/determinismo y dos regresiones de revisión nuevas; RED observado y suite final 67 tests CPU pasa, fmt/Clippy pasan. Tres pruebas locales omitidas por defecto; la integrada se ejecuta aparte y reproduce muestras anteriores. Revisión independiente de tarea 4: dos P2 corregidos y verificados por tests, sin otros hallazgos reportados. Decisiones y límites en avance-tarea-4.5.md.
- Submódulo 5.1: crate CPU local, validate_output común y plan de reserva base PNG; RED dos de tres tests, GREEN 70 tests workspace (67 core + tres cpu), fmt/Clippy pasan. Tres tests locales omitidos por defecto y no repetidos en este paso sin render de salida. Sin librerías externas nuevas ni RAM medida; temporales resvg pendientes de 5.2. Evidencia en avance-tarea-5.1.md.
- Submódulo 5.2: render_png, preparación de viewport cover y estimador de temporales; RED de render vacío, viewBox implícito y bounds excesivos; suite final 78 tests pasa, formato/Clippy pasan. Cuatro tests locales omitidos por defecto; test local CPU ejecutado aparte con tres PNG 512² decodificados y repetibles, revisión visual hecha. Fuente SVG intacta. Reservas calculadas, no RSS medido. Evidencia y límites en avance-tarea-5.2.md.
- Submódulo 5.3: export_svg seguro, fondo independiente de efectos de raíz y cover compartido; RED observado, 82 tests pasan, fmt/Clippy pasan. Cinco tests locales omitidos por defecto; nuevo test CPU ejecutado aparte sobre tres muestras con bytes repetibles, píxeles idénticos a PNG y revisión visual. Sin dependencias nuevas ni recorte geométrico real. Evidencia en avance-tarea-5.3.md.
- Submódulo 5.4: cpu::export::generate_and_export devuelve candidato y bytes finales opcionales; SHA-256 tras exportación completa, rechazos sin bytes/hash y errores propagados. RED tres fallos observado, GREEN 86 tests workspace, fmt/Clippy pasan; cinco locales omitidos, no repetidos. Orden/subrangos/tres hilos comprobados con SVG/PNG. Sin nuevos paquetes externos ni schemas. Evidencia en avance-tarea-5.4.md.
- Submódulo 5.5: nuevo test local integra generación/exportación/hash sobre la copia preparada de test.svg; seis artefactos aceptados y repetidos por subrangos, bytes iguales a 5.3, SHA-256 verificado además con Python, píxeles SVG/PNG idénticos y revisión visual de tres PNG. Rechazo empty_viewport tras ocho intentos sin artefacto; fuentes intactas. Suite normal 86 tests, seis locales omitidos; el nuevo local se ejecutó aparte y pasó. Formato/Clippy pasan. Solo tests/docs, sin cambios de producto. Evidencia en avance-tarea-5.5.md.
- Submódulo 5.6: contratos PDF nuevos, tipos/validadores, recorte real y writer por objetos. 95 tests normales (73 core + 22 cpu), nueve locales omitidos; tres PDF locales ejecutados aparte, formato/Clippy pasan. PDF final 3 páginas de 100 mm, 2730275 bytes; cero imágenes raster, hash/mapa/referencias y siete páginas visuales comprobados. Baseline propio: reducción de peso 63.91% en una muestra, sin garantía general ni de Fiery. Pico OS del proceso de test: 74.62 MiB residente y 149.47 MiB commit privado. Revisión independiente: un P1 de autointersecciones corregido con regresión, revisión posterior sin hallazgos importantes. Evidencia en avance-tarea-5.6.md y ADR 0005.
- 6.6: perfil release por etapa y ocho procesos CLI medidos; 131 tests normales pasan y diez locales ignorados por defecto. Nuevo perfil local ejecutado aparte; QA PDF real pasa. Revisión global sin bloqueos P1/P2 tras corregir cobertura del encuadre final con regresiones SVG/PNG y PDF. Detalles en avance-tarea-6.6.md. No hay pruebas CUDA ni de fronteras Java/web ejecutadas todavía. La medición de 5.6 incluye el proceso de test completo, no solo el exportador. El benchmark de Graphify estima tokens de navegación, no rendimiento del motor.

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
| 8031f75 | Integración 4.5 y cierre de tarea 4 | 67 tests CPU, prueba local integrada, fmt/Clippy, dos P2 de revisión corregidos |
| 8013a89 | Validación de salida y reserva base PNG 5.1 | 70 tests workspace, fmt/Clippy, revisión local |
| 24b46db | Render PNG CPU y presupuesto de temporales 5.2 | 78 tests workspace, prueba local PNG, fmt/Clippy, revisión visual |
| 4d7a1b9 | SVG final con fondo independiente y encuadre cover 5.3 | 82 tests, prueba local de tres SVG/PNG, fmt/Clippy, revisión visual |
| d72883f | Integración SVG/PNG y hash de bytes finales 5.4 | 86 tests, fmt/Clippy, determinismo por subrangos/orden/tres hilos |
| c5ef9c0 | Validación integrada con SVG del usuario 5.5 | 86 tests normales, nuevo local aparte, seis artefactos, hashes, rechazo y revisión visual |
| 3b51db2 | PDF vectorial multipágina, contratos y recorte real 5.6 | 95 tests, formato/Clippy, QA Poppler/pypdf, revisión P1 corregida |
| 22b82ae | CLI inspect seguro y exclusión de Fiery 6.1 | 102 tests, formato/Clippy, inspección real de SVG preparado |
| c8094ac | CLI generate SVG/PNG y publicación exclusiva 6.2 | 109 tests, formato/Clippy, carrera/escritura fallida y CLI real |
| 3088cc7 | CLI grupos y concurrencia CPU acotada 6.3 | 115 tests, formato/Clippy, 201 variantes reproducibles y prueba real |
| 04a5359 | Manifiesto incremental y cancelación CLI 6.4 | 122 tests, formato/Clippy, Ctrl+C real Windows exit 130 |
| 1b9499f | CLI PDF único por trabajo 6.5 | 129 tests, formato/Clippy, cancelación real y QA PDF de tres páginas |

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
- PDF de referencia inspeccionado durante la pausa: docs/ejemplo_export.pdf, 158.52 MB / 250 páginas; análisis de cuatro páginas y render de la primera confirma Forms invocados con BBox exterior. Fiery no medido. Informe en analisis-ejemplo-export.md; PDF original intacto, no incluido en commit documental.

## Punto exacto para continuar

**Fase 1 funcional CPU/CLI completa; esperar confirmación explícita antes de fase 2.** Tareas 1–6 terminadas. README.md documenta inspect, generate SVG/PNG y generate-pdf; avance-tarea-6.6.md registra el cierre y perfil-cpu-fase1.md enlaza las mediciones conservadas. No iniciar frontend, transporte HTTP, Java ni infraestructura sin la siguiente autorización.

131 tests normales pasan (74 core, 23 CPU, 34 CLI); formato y Clippy sin warnings pasan, build release CPU pasa. Diez tests locales ignorados por defecto: el perfil nuevo se ejecutó aparte, los otros nueve no se repitieron en 6.6. Ocho corridas reales CLI y QA pypdf/Poppler del PDF pasan; tres páginas de 100 × 100 mm sin sangrado, 2 730 275 bytes, idénticas a 5.6/6.5. Hashes SVG/PNG conservados entre N/hilos y baselines. Revisión independiente global sin bloqueos P1/P2 tras dos regresiones de cobertura: se valida el encuadre visible final y la composición de encuadres PDF, sin contar el fondo añadido como arte. Los casos afectados pueden cambiar respecto al snapshot previo: conservar el commit para reproducir versiones de desarrollo.

Medición exploratoria PNG 512², 12 variantes: pico working set observado aproximadamente 27 MiB con un hilo y 41 MiB con dos. Una corrida por caso; no es una cota de RAM ni una aceleración general. SVG/PNG mantienen 1|2 hilos y reserva compartida estimada de buffers PNG de 512 MiB, distinta del consumo real. PDF secuencial y único por trabajo; páginas provisionales en manifiesto, documento/hash final solo tras publicar. No hay transacción archivo/manifiesto: un fallo del registro posterior a publicar puede dejar un archivo completo sin resultado final. Sin reemplazo ni reanudación automática.

Prototipo local todavía sin publicar; LICENSE pendiente de elección antes de distribución pública y sin modificar. Linux/macOS, GPU, OOM real, latencia máxima de cancelación y millones de variantes no medidos. Fiery excluido. PDF/X/CMYK no implementados o validados. Grafo Graphify antiguo sin reconstruir; revisión contrastada con fuentes actuales. No repetir tareas completas.
