# PDF vectorial multipágina — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task, with one independent final review.

**Goal:** Un único PDF vectorial determinista por trabajo, con geometría recortada y páginas físicas configurables.
**Architecture:** Contratos PDF nuevos referencian generación v1. CPU procesa una página a la vez; pdf-writer serializa objetos individuales a un writer y un cierre xref propio evita acumular el PDF completo. Geometría usvg/tiny-skia se convierte a contornos e intersecta con i_overlay; curvas se aproximan con kurbo y precisión documentada tras medición.
**Tech Stack:** Rust, tipos typify, usvg/resvg 0.47, pdf-writer 0.15, i_overlay 9, kurbo 0.13; fijar versiones al resolver y revisar licencias. Poppler/pypdf/Pillow para QA independiente, no dependencias de producto.
**Spec:** docs/superpowers/specs/2026-10-15-pdf-multipagina-design.md, aprobada por usuario con «sii terminalo». Ejecución inline continuada autorizada por esa instrucción; no volver a pedir aprobación entre pasos.

## Global Constraints

- 100 × 100 mm, sangrado 0 para la primera muestra; tamaños configurables y sin inferir DPI.
- Un solo PDF, una página por aceptada, índice ascendente, sin PDF de éxito vacío.
- Sin cambios incompatibles v1, raster fallback, servicios/FFI nuevos, LICENSE o CI.
- Hash de SVG intermedio distinto del hash de documento; N no influye en bytes.
- Recorte geométrico real, incluidos trazos y clips; error explícito al exceder límites.
- No afirmar tolerancias, rendimiento o RSS antes de medir. Fiery sigue pendiente.

## Review Focus

1. Clips con agujeros/anidamiento: intersección/union correctas y sin geometría exterior.
2. Trazos bajo escala desigual y miters: contornear antes de transformar/recortar.
3. Opacidad de grupo con figuras solapadas: componer en Form de transparencia, no multiplicar cada figura.
4. Error parcial de writer y cero aceptadas: no devolver completed/hash de éxito.
5. Complejidad expandida y xref: limitar puntos/objetos/bytes, offsets comprobados.

### 5.6.2: contratos y primitivas físicas

**Files:** contracts/engine/pdf/v1/{job,result,page}.schema.json; core/build.rs, core/src/{contract,pdf}.rs, core/tests/pdf_contract.rs.
**Interfaces:** PdfJobSpec, PdfDocumentResult, PdfPageRecord generados; validate_pdf_job(&PdfJobSpec), pdf_configuration_hash(&PdfJobSpec) y page_boxes(&PdfJobSpec) -> Result<PageBoxes, EngineError>. PageBoxes tiene media/trim [f32;4].

- [ ] Tests RED: job referencia generación v1; 100 mm equivale a 283.46456 pt; sangrado 3 mm; límites/no finitos/PNG rechazados; hash incluye intervalo y excluye opciones operativas inexistentes.
- [ ] Implementar resolución local del schema en build, usando tipo GenerationSpec existente; no duplicarlo a mano.
- [ ] GREEN: cargo test -p mosaicforge-core --test pdf_contract --locked; fmt/Clippy y commit.

### 5.6.3: geometría y página

**Files:** cpu/src/pdf/{mod,geometry,writer}.rs; cpu/tests/pdf_geometry.rs.
**Interfaces:** optimize_page(svg, job) -> Result<OptimizedPage, EngineError>; OptimizedPage contiene árbol de grupos/contornos pintados, métricas y geometría física en puntos. PdfWriter<W: Write> emite objetos por página, conserva solo offsets/IDs, y finaliza xref sin un Vec del documento entero.

- [ ] RED fixtures de bordes/trazos/agujeros/clips/grupos; todos los puntos emitidos dentro de MediaBox, ningún Image XObject, opacidad de grupo conservada, presupuesto de geometría rechazado.
- [ ] Contornear trazos con tiny-skia antes de transforms; flatten con kurbo; operaciones booleanas con fill-rule explícita y unión de hijos de clips. Límites de puntos y profundidad en cada expansión.
- [ ] Usar pdf-writer Chunk por objeto y Content por grupo; Forms solo para aislamiento/opacidad; recursos locales referenciados, sin defs heredadas sin uso. Comprimir streams con miniz_oxide existente.
- [ ] GREEN tests; render independiente frente a resvg a 300 DPI, medir diferencias antes de congelar precisión; commit.

### 5.6.4: documento integrado

**Files:** cpu/src/pdf/mod.rs, cpu/tests/pdf_document.rs.
**Interfaces:** generate_pdf(template, job, batch_size, writer, records_callback) -> Result<PdfDocumentResult, EngineError>; callback recibe PdfPageRecord y puede fallar; error aborta. Destino propiedad del caller, sin publicación desde librería. Solo primera aceptada escribe cabecera; no_pages deja destino vacío.

- [ ] RED una/múltiples páginas, índices hasta u64::MAX, cero aceptadas, error de writer/callback, N=0 inválido; N=1/7/100 sobre 201 índices incluido grupo final parcial, mismos bytes/hash/mapa. Fixture de rechazo intermedio o ensamblado de decisiones secuenciales comprobadas.
- [ ] Procesar índices secuencialmente, generar/exportar SVG final por índice y optimizar; emitir mapa incremental sin acumularlo. Asignación determinista de objetos, sin fechas/IDs aleatorios. Validar counters y resultados antes de completed; hash del writer cubre EOF.
- [ ] GREEN tests, fmt/Clippy y commit. CLI/publicación/cancelación siguen en tarea 6.

### 5.6.5: muestras, mediciones y revisión

**Files:** cpu/tests/local_pdf.rs; scripts/verify-pdf-export.py; docs/avance-tarea-5.6.md, docs/adr/0005-pdf-vectorial.md, docs/dependencias-motor.md y checklists.

- [ ] Generar un PDF de tres páginas desde test.svg preparado, índices 0/1/17 mediante trabajo de rango continuo con mapa o muestras separadas de validación del ensamblador; nunca declarar tres índices no contiguos como si fueran un GenerationSpec continuo. Primera muestra física 100 × 100 mm sin sangrado. Incluir prueba de sangrado positivo y opacidad/clip.
- [ ] pypdf inspecciona integridad, orden/cajas, referencias y ausencia de imágenes; Poppler rasteriza y Pillow mide diferencia frente a referencia. Registrar tamaño/objetos/segmentos y pico de memoria de proceso por separado con método y limitaciones.
- [ ] Formato, Clippy, suite CPU completa; revisión independiente global de tarea 5. Corregir hallazgos con regresiones; commit verificado y estado honesto.

Pre-flight: GenerationSpec.output=svg se conserva como artefacto intermedio; PDF aplica transformación física independiente. Writer devuelve hash solo tras xref/EOF; mapa callback no publica documento. Inicialmente secuencial: N se prueba como agrupación, no se introduce pool de threads antes de CLI. La ejecución autorizada continúa por estos pasos; si una elección técnica cambia el diseño se registra en el ledger.
