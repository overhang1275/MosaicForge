# Tarea 6 — CLI y ejecución incremental

Fecha: 2026-10-15. Base 3c7139c. Usuario autorizó iniciar tarea 6 y **excluyó Fiery del alcance**. Mantener ejecución por submódulos con pausa al terminar cada uno; la autorización previa de terminar 5.6 no adelanta fase 2 ni el resto de tareas.

- [x] ~~**6.1 Binario e inspect:** crate CLI, argumentos/help, lectura acotada, metadata JSON estable y errores/códigos comprobados; 102 tests workspace pasan.~~
- [x] ~~**6.2 Generación y publicación segura:** generate SVG/PNG, temporales y publicación sin reemplazo; colisión, carrera y fallo de escritura comprobados. 109 tests y checks pasan.~~
- [x] ~~**6.3 Grupos y concurrencia:** --batch-size N y --threads 1|2; pool Rayon, ventanas acotadas y presupuesto PNG compartido. 201 variantes SVG/PNG reproducibles con N=1/7/100, subrangos y grupo incompleto; 115 tests y checks pasan.~~
- [ ] **6.4 Manifiesto y cancelación:** JSONL incremental con resultados/rechazos/resumen, Ctrl+C/AtomicBool cooperativo; pruebas con writer/token inyectados. Códigos 0 éxito, 2 entrada, 3 E/S/render y 130 cancelación. No confundir rechazo válido con fallo de proceso.
- [ ] **6.5 Integración PDF:** PdfJobSpec agregado, un solo documento final para todo el trabajo, mapa de páginas/hash y publicación después de cerrar; error/cancelación sin PDF truncado publicado ni sobreescritura. Cero aceptadas sin PDF vacío.
- [ ] **6.6 Validación y cierre de fase 1:** corridas reales SVG/PNG/PDF mediante CLI, perfil por etapa y memoria separados, README de ejecución, revisión independiente global. Presentar evidencia y esperar confirmación antes de fase 2. **Fiery excluido**, no pendiente para cerrar.

## 6.1 — Diseño acotado que ejecuta el plan aprobado

## Autorización y diseño 6.2–6.3 (2026-10-15)

El usuario autoriza ambos submódulos juntos; detenerse antes de 6.4. Se conservan los contratos v1 y la lógica CPU existente.

- [x] ~~6.2: generate, JSON acotado a 1 MiB, contrato y hash; archivos variant-{índice}.svg|png, resultados/stdout y resumen. Rechazos sin artefacto y exit 0.~~
- [x] ~~Temporales exclusivos, sincronizados y cerrados; publicación hard_link sin reemplazo y limpieza. No hay fallback de reemplazo ni transacción de trabajo completo.~~
- [x] ~~Igualdad con motor, colisión, carrera y fallo de escritura comprobados.~~
- [x] ~~6.3: grupos 1..10000, hilos 1|2 y pool local Rayon; ventanas <=hilos, sin acumular N imágenes ni reiniciar índices.~~
- [x] ~~Presupuesto compartido PNG 512 MiB de buffers estimados por candidato, liberado tras publicación/bytes; no se divide entre hilos y no implica límite de RSS.~~
- [x] ~~201 variantes SVG/PNG con N=1/7/100 y ambos hilos, grupo incompleto, subrangos y coordinación/liberación de reserva comprobados.~~

Referencias primarias: [Rust hard_link](https://doc.rust-lang.org/std/fs/fn.hard_link.html), [Rayon ThreadPoolBuilder](https://docs.rs/rayon/latest/rayon/struct.ThreadPoolBuilder.html). Manifiesto persistente, Ctrl+C y PDF CLI permanecen para 6.4/6.5.

## Diseño 6.1 (histórico)

Crear crates/cli con binario mosaicforge. Consume core::template::import_svg y devuelve hash normalizado, dimensiones, viewBox, layerIds y segmentCount; no devuelve el contenido SVG ni escribe archivos. Estos son metadatos locales del comando, no un nuevo contrato entre servicios. No añadir modelos de GenerationSpec/PdfJobSpec duplicados ni cambiar contratos publicados.

Usar clap para la frontera real de argumentos/rutas OS, ayuda y flags: se reutilizará al añadir generate en 6.2. No construir un parser propio que luego haya que sustituir. Dependencias open source fijadas y licencias registradas. Sin lenguajes/servicios nuevos ni capas de puertos para una sola implementación.

Leer como máximo 5 MiB + un byte antes de importar, con Read::take; no usar fs::read sin límite ni depender únicamente de metadata.len por crecimiento del archivo. Solo archivos regulares: rechazar directorios y rutas no regulares para evitar leer de dispositivos. El importador conserva la validación SVG estricta.

## Terminado medible 6.1

- inspect --template FILE produce JSON repetible y hash coincidente con importación del motor; funciona con espacios y caracteres Unicode en la ruta.
- SVG malformado, contenido activo y archivo >5 MiB se rechazan con exit 2; ruta inexistente/fallo E/S con exit 3. Sin SVG fuente en los mensajes de error ni salida de éxito parcial.
- --help y --version con exit 0; argumentos ausentes/desconocidos/duplicados rechazados con exit 2.
- Errores de escritura de stdout propagados con exit 3; el binario usa stdout solo para datos/ayuda y stderr para errores.
- Tests del proceso CLI, formato, Clippy y suite pasan; probar inspect real sobre out/test-prepared.svg y registrar commit/docs. Pausar antes de 6.2.

Graphify consultado con budget 300: cuatro nodos encontrados y mostrados. Snapshot antiguo: interfaces reales contrastadas por lectura directa. Revisión independiente global al final de 6.6, no subagente por cada submódulo pequeño.
