# Tarea 6 — CLI y ejecución incremental

Fecha: 2026-10-15. Base 3c7139c. Usuario autorizó iniciar tarea 6 y **excluyó Fiery del alcance**. Mantener ejecución por submódulos con pausa al terminar cada uno; la autorización previa de terminar 5.6 no adelanta fase 2 ni el resto de tareas.

- [x] ~~**6.1 Binario e inspect:** crate CLI, argumentos/help, lectura acotada, metadata JSON estable y errores/códigos comprobados; 102 tests workspace pasan.~~
- [ ] **6.2 Generación y publicación segura:** comando generate SVG/PNG, temporales y publicación sin reemplazo; pruebas de colisión, carrera y fallo de escritura. Nada de comprobar exists y luego sobrescribir.
- [ ] **6.3 Grupos y concurrencia:** --batch-size N y --threads 1|2, Rayon con memoria/buffers acotados; índices globales, N=1/7/100, subrangos y grupo incompleto reproducibles. SVG/PNG se liberan al publicarse.
- [ ] **6.4 Manifiesto y cancelación:** JSONL incremental con resultados/rechazos/resumen, Ctrl+C/AtomicBool cooperativo; pruebas con writer/token inyectados. Códigos 0 éxito, 2 entrada, 3 E/S/render y 130 cancelación. No confundir rechazo válido con fallo de proceso.
- [ ] **6.5 Integración PDF:** PdfJobSpec agregado, un solo documento final para todo el trabajo, mapa de páginas/hash y publicación después de cerrar; error/cancelación sin PDF truncado publicado ni sobreescritura. Cero aceptadas sin PDF vacío.
- [ ] **6.6 Validación y cierre de fase 1:** corridas reales SVG/PNG/PDF mediante CLI, perfil por etapa y memoria separados, README de ejecución, revisión independiente global. Presentar evidencia y esperar confirmación antes de fase 2. **Fiery excluido**, no pendiente para cerrar.

## 6.1 — Diseño acotado que ejecuta el plan aprobado

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
