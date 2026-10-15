# Contrato PDF agregado v1

job.schema.json referencia GenerationSpec v1 existente; generation.output.format debe ser svg. result.schema.json representa el documento completo y page.schema.json un registro incremental por índice. Tipos Rust generados en core/build.rs con resolución local de referencias y reutilización de GenerationSpec/VariantResult, sin descargas de schemas.

Validar semántica además de deserializar: core::pdf::{validate_pdf_job,validate_pdf_document_result,validate_pdf_page_record}. El schema describe tipos/rangos; relaciones como completed→hash/páginas y accepted→pageNumber/artifactHash se comprueban en estos validadores, invocados por el exportador.

El hash de documento cubre todos los bytes PDF, incluido xref/EOF. artifactHash de cada registro sigue siendo el hash del SVG intermedio, no del PDF ni de un stream de página. documentConfigurationHash normaliza todo PdfJobSpec, incluido intervalo; generationConfigurationHash conserva v1 e ignora count/startIndex. Opciones operativas N/ruta/hilos no forman parte del contrato reproducible.

Medidas explícitas en milímetros: corte 1–2000 mm por eje y sangrado 0–50 mm. Las cajas PDF se expresan en puntos (72/25.4 por mm). Sin aceptadas: no_pages, pageCount=0, sin PDF/hash. No hay formatos PDF nuevos dentro del enum SVG/PNG existente ni cambios incompatibles a ese contrato.

El caller debe usar un destino temporal vacío propiedad suya. generate_pdf devuelve error si falla writer/callback; ese destino parcial debe descartarse, nunca publicarse como éxito. Los registros emitidos antes de cerrar son provisionales. Publicación sin reemplazo, cancelación y manifiesto del usuario pertenecen a CLI tarea 6.
