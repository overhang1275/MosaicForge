# MosaicForge — Plan de desarrollo por fases

Actualizado: 2026-10-15. Este orden reemplaza el plan anterior por instrucción directa del usuario. La aprobación anterior de tecnologías permanece; su incorporación se difiere a la fase que la necesita.

Fiery excluido del alcance por instrucción del usuario del 2026-10-15; no condiciona el cierre de fase 1. Tarea 6 completada por submódulos según tarea-6-submodulos.md; fase 2 autorizada por el usuario, diseño detallado aprobado; 2.1–2.3 implementados y verificados; 2.4 pendiente de confirmación.

## Alcance y punto de control

Prioridad: una variante válida y reproducible de punta a punta antes de invertir en infraestructura. Desarrollar solo Rust CPU/CLI en fase 1. No pasar a fase 2 sin confirmación explícita del usuario después de presentar evidencia de fase 1.

Cada fase tiene un documento ejecutable propio y termina con checks, evidencia en docs/estado.md y commits recuperables. Una casilla completada se marca [x] y se tacha. Lo omitido por falta de hardware queda pendiente.

## Preparación ya realizada

- [x] ~~Leer requisitos y reglas del repositorio.~~
- [x] ~~Inventariar herramientas y hardware.~~
- [x] ~~Registrar preferencia open source y excepción CUDA opcional.~~
- [x] ~~Guardar diseño inicial, checklist, documentación de continuidad y baseline Git.~~
- [x] ~~Aceptar la corrección de orden: Rust → web → Java → CUDA → Python condicional.~~
- [x] ~~Usuario autorizó continuar la fase 1 con tarea 2 después de Graphify.~~ El paso a fase 2 fue aprobado y 2.1–2.3 se completaron.
- [ ] Elegir LICENSE con aprobación explícita antes de distribución.

## Fase 1 — Motor Rust CPU y CLI

**Objetivo:** generar una variante SVG y PNG desde una plantilla original, después comprobar que los mismos índices producen lo mismo al repetir o dividir el lote. No desplegar web, Java, base de datos, broker ni Python.

**Entrega:** contratos v1 y tipos generados, SVG seguro, PRNG por contador, transformaciones/validación, raster CPU y CLI. Plan concreto: superpowers/plans/2026-10-14-motor-cpu.md.

- [x] ~~Tarea 1: contratos y validación de entrada/resultados; hash de configuración independiente del bloque.~~ Commit 2f1e6d4; diez tests, formato y Clippy pasan.
- [x] ~~Tarea 2: PRNG y dominio (seed, índice, regla, intento, muestra) con vectores de referencia.~~ Commit 30ac241; siete pruebas RNG y diez de contratos pasan.
- [x] ~~Tarea 3: importación segura, normalización y límites SVG.~~ Commit cf8c819; quince tests SVG, suite total de 32 y checks pasan. Subconjunto y límites en svg-admitido.md.
- [x] ~~Tarea 4: geometría, color, capas, recorte, distorsión, rechazo por índice e integración.~~
  4.1–4.5 terminados: 67 tests CPU, prueba local integrada con test.svg, formato y Clippy pasan. Revisión independiente: dos P2 corregidos con regresiones. Cobertura raster CPU pequeña adelantada, ADR 0004; exportación general y hash de artefacto final siguen en tarea 5. Evidencia y límites en avance-tarea-4.5.md.
- [x] ~~Tarea 5: SVG/PNG y PDF vectorial multipágina, presupuesto de buffers y recorte real.~~
  5.1–5.6 completados: 95 tests normales, tres nuevos locales PDF aparte, formato/Clippy y revisión independiente corregida pasan. PDF único con orden, tamaño/sangrado configurables, cero imágenes raster y recursos verificados. Checkpoint funcional 3b51db2; avance-tarea-5.6.md y ADR 0005. La memoria medida del test no es presupuesto global ni benchmark release.
- [x] ~~5.6: muestra única de tres páginas, 100 × 100 mm sin sangrado, hashes y mapa verificados; N=1/7/100 produce el mismo documento en tests. Recorte real medido frente a baseline propio y pico de memoria medido.~~ Fiery excluido; sin promesa de PDF/X/CMYK.
- [x] ~~Tarea 6: CLI incremental, cancelación y errores de E/S sin sobrescritura ni artefactos finales incompletos.~~
- [x] ~~6.1–6.5: CLI inspect/generate SVG/PNG/PDF, grupos, manifiesto, cancelación y publicación exclusiva; 129 tests y PDF real pasan. 6.6 completa el cierre: 131 tests, perfil release y revisión global corregida; avance-tarea-6.6.md.~~
- [x] ~~6.3: grupos N y 1|2 hilos implementados, ventanas acotadas y presupuesto PNG compartido; 201 variantes SVG/PNG reproducibles con N=1/7/100 y subrangos.~~
- [x] ~~Guardar corridas reales SVG/PNG/PDF y sus manifiestos; hashes comprobados.~~
- [x] ~~Medir por separado normalización, muestreo/reglas, validación, serialización SVG, raster, codificación PNG y escritura; perfil-cpu-fase1.md.~~
- [x] ~~Documentar las restricciones de validez y su cobertura de tests; validez-motor-cpu.md.~~
- [x] ~~Preparar y presentar evidencia del cierre de fase 1.~~
- [x] ~~Obtener confirmación explícita para pasar a fase 2 por submódulos.~~

**Terminado medible:**

1. Fixture SVG original → variante SVG y PNG; PNG decodifica con dimensiones correctas. PDF 5.6: documento único multipágina con variantes aceptadas en orden, conteo y tamaño físico correctos; sin PDF por variante/grupo.
2. Pruebas comparan hashes/bytes de índices 0–199 con 1 y 2 hilos y con bloques reordenados; índices 100–199 aislados coinciden con el rango del lote completo.
3. Tests rechazan SVG con recursos externos/scripts/entidades/ciclos y entradas que superen los límites aprobados; pruebas en el límite y justo por encima.
4. Restricciones prueban rangos finitos, límites geométricos, recorte, escalas positivas y aceptación/rechazo reproducibles. Fallar explícitamente después de ocho intentos; no desplazar los índices siguientes.
5. Cancelación y errores de escritura probados, sin sobrescribir archivos existentes.
6. cargo fmt, cargo clippy y cargo test pasan; CPU ejecuta sin CUDA ni Python.
7. Perfil inicial contiene comandos, fixture, versiones, equipo, tamaños, tiempos de etapas y memoria observada. Es una línea base medida; no una promesa de capacidad.

**Validez:** diseñar cobertura visible, área fuera del encuadre, área mínima de elementos, solapamiento y contraste de colores adyacentes relevantes. Implementar primero métricas deterministas comprobables con fixtures analíticos. No prometer detectar texto ilegible o buena estética en arte arbitrario; texto viene convertido a paths y las limitaciones se documentan.

## Fase 2 — Frontend conectado al motor CPU

Inicio autorizado. Checklist: fase-2-submodulos.md. Diseño aprobado en superpowers/specs/2026-10-15-fase-2-web-design.md; plan de 2.1 escrito en superpowers/plans/2026-10-15-fase-2-1-contratos.md, aprobado y completado con evidencia en avance-fase-2.1.md.

**Objetivo:** cargar una plantilla, ajustar reglas/seed y obtener previsualizaciones reales del motor, todavía sin usuarios, proyectos persistentes o cola de producción.

**Frontera mínima propuesta:** añadir transporte HTTP de previsualización al binario Rust, ligado a loopback en desarrollo, para inspect y generación pequeña. Next.js solo sirve UI/BFF proxy. Los contratos JSON Schema de fase 1 generan los tipos web; no implementar una segunda generación en TypeScript. Proponer y aprobar la interfaz exacta en el diseño de esta fase.

- [ ] ADR de frontera Next.js ↔ Rust: coste, despliegue, seguridad, contrato y retirada del proxy directo cuando entre Java.
- [ ] Incorporar Next.js/TypeScript estricto y cliente generado.
- [ ] Carga SVG, rangos, seed e índice visibles; máximo 12 previews de 512 × 512.
- [ ] Mostrar rechazos de reglas con motivo y mantener contexto al ajustar parámetros.
- [ ] Mostrar PNG seguro generado por Rust; descargar SVG/PNG de una variante.
- [ ] Probar foco/teclado, errores, cambios rápidos y desconexión del motor.
- [ ] Exportar Next.js standalone y documentar ejecución local de ambos procesos.

**Terminado medible:**

1. Prueba de navegador carga fixture, configura seed, pide preview, ajusta una regla y descarga.
2. La descarga/previsualización corresponde al mismo índice/seed/versión que produce CLI con el mismo contrato.
3. Respuestas antiguas de previews no sustituyen una solicitud más reciente; errores y rechazos visibles.
4. Typecheck, lint, tests de componentes y E2E pasan; ningún motor generativo está duplicado en Next.js.
5. No se exige Java, PostgreSQL, Valkey, CUDA ni Python para demostrar este flujo.
6. Contrato y ADR de la frontera real registrados; no crear puertos genéricos para transportes que aún no existen.

## Fase 3 — API Java, usuarios/proyectos y trabajos asíncronos

**Objetivo:** convertir el flujo demostrado en un producto con permisos, estado duradero, lotes, recuperación y progreso.

- [ ] ADR de la frontera Java ↔ Rust y del paso web → Java; justificar coste frente al flujo local existente.
- [ ] Introducir Java 21/Spring Boot/Spring Security y Maven Wrapper aquí, no antes.
- [ ] Diseñar API OpenAPI y trabajo/eventos versionados; generar clientes y probar fronteras.
- [ ] Autenticación, autorización por proyecto/artefacto, presets y metadatos.
- [ ] Incorporar PostgreSQL y Valkey aceptados como implementaciones de fronteras reales. No crear capas o puertos para responsabilidades inexistentes.
- [ ] Estado duradero, outbox/reconciliación, bloques idempotentes, recuperación de pendientes y reintentos limitados.
- [ ] SSE con reconexión/snapshot, cancelación y límites de concurrencia.
- [ ] Almacenamiento local real; añadir adaptador S3 cuando se necesite despliegue remoto, con ADR/proveedor por elegir.
- [ ] Galería paginada/virtualizada, selección e historial; retirar el acceso web directo al transporte de preview Rust.
- [ ] Integrar trace ID/logs y métricas de fronteras; contenedores CPU, CI y LICENSE con las autorizaciones necesarias.

**Terminado medible:**

1. E2E web → Java → Rust: usuario autorizado crea proyecto, carga, genera lote, consulta estado y descarga.
2. Usuario distinto no accede a metadatos ni archivos del primero.
3. Pruebas de mensaje duplicado, caída del worker/API y reinicio no duplican publicación ni pierden estado confirmado.
4. Cancelación libera recursos y no publica artefactos incompletos.
5. Progreso se recupera tras desconexión; misma plantilla/reglas/seed/índice sigue reproduciendo lo generado por CLI.
6. Tests de contratos, integración y E2E pasan; despliegue CPU reproducible sin CUDA/Python.
7. Lotes progresivos se miden con memoria/disco/tiempo; capacidad anunciada solo hasta lo ensayado.

## Fase 4 — CUDA condicionada a evidencia

**Objetivo:** acelerar solo etapas cuyo tiempo total medido mejore frente al CPU. SVG/PDF vectoriales evitan raster por defecto; PDF multipágina está incorporado a la fase 1, submódulo 5.6, con diseño/pruebas de impresión propios.

- [ ] Perfilar SVG y PNG de extremo a extremo con varias complejidades, resoluciones y tamaños de lote.
- [ ] Separar transferencia, lanzamiento, kernels, codificación y E/S; comparar primero la referencia resvg/tiny-skia.
- [ ] Elegir operaciones candidatas por medición: color por píxel, composición, miniaturas y hashes; no priorizar raster de paths sin evidencia.
- [ ] ADR para CUDA/bindings/unsafe/versiones y contrato numérico; CPU sigue compilando sin toolkit.
- [ ] Incorporar ComputeBackend cuando existan CPU y CUDA reales; PRNG/discretos/aceptación comunes.
- [ ] OOM → lote menor → reintento → fallback según política; cancelación/liberación y metadatos visibles.
- [ ] Validar paridad/determinismo en equipo NVIDIA; streams/buffers/múltiples GPUs solo si el perfil lo exige.

**Contrato de determinismo establecido temprano:** identidad bit a bit dentro del mismo backend, arquitectura GPU, motor, dependencias y versión de driver/toolchain fijados. Entre CPU y GPU: mismas decisiones discretas y diferencias de píxel dentro de una tolerancia medida antes de activar la etapa. No usar reducciones flotantes de orden variable para decisiones de aceptación.

**Terminado medible:**

1. Cada etapa activada dispone de benchmark reproducible con baseline CPU y tiempo GPU total, incluidas transferencias.
2. Repetir al menos 10 veces una misma corrida en GPU produce hashes idénticos en el entorno fijado.
3. Dataset de paridad, métrica, umbral medido y peores diferencias documentados; ningún umbral inventado.
4. No GPU/init error/OOM/cancelación tienen tests; CPU sigue funcionando.
5. Si no hay mejora total, la etapa se conserva en CPU y se documenta. Si falta hardware, validación GPU pendiente; nunca marcarla como aprobada.
6. SVG/vector no paga costes de raster GPU innecesarios.

## Fase 5 — Python/ML solo con necesidad demostrada

**Entrada obligatoria:** casos originales documentados que pasan las reglas existentes pero siguen siendo inválidos o ilegibles; demostrar qué métrica basada en reglas resulta insuficiente.

- [ ] Evaluar ampliar reglas antes de añadir ML.
- [ ] ADR de frontera Java ↔ Python: beneficio, latencia, coste, despliegue, modelos/licencias y datos usados.
- [ ] Añadir una única capacidad útil inicialmente, con tipos/cliente generados y presupuesto de recursos.
- [ ] Comparar detección con/sin modelo sobre dataset etiquetado y versionado.
- [ ] Python opcional con timeouts/cancelación; generación/exportación siguen funcionando si falta o cae.
- [ ] Reservar VRAM compartida con Rust solo si ambos usan GPU.

**Terminado medible:**

1. Dataset y criterio de mejora acordados antes de implementar; informar verdaderos/falsos positivos y negativos.
2. Comparación reproducible demuestra beneficio útil sobre el baseline de reglas y mide latencia/memoria.
3. Caída/ausencia/timeout del servicio no rompe generación/exportación.
4. Modelo/dependencias/licencias/versiones registrados; contratos y pruebas pasan.
5. Si las reglas bastan, omitir esta fase y registrar el motivo; no desplegar Python por variedad.

## Git, evidencia y continuidad

- Commit por implementación verificada, con archivos explícitos; nunca incluir secretos, dependencias descargadas o resultados de usuario.
- Registrar hashes/comandos/resultados en docs/estado.md. Cambios documentales se distinguen de implementaciones funcionales.
- DuckDB para consultar JSON/TXT/CSV grandes cuando ayude a filtrar/agregar; no imponerlo como dependencia del producto ni instalarlo para archivos pequeños.
- Graphify cuando exista código suficiente: excluir herramientas descargadas/builds/artefactos y usar el grafo para revisar fronteras y dependencias; no confundir grafo con evidencia de tests.
- No afirmar aceleración, cobertura ni capacidad sin medición.
- Nunca reanudar una tarea marcada pendiente como si estuviera completa: leer estado, plan y git log.
