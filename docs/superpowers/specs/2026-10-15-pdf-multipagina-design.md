# Diseño propuesto — PDF vectorial único multipágina

Fecha: 2026-10-15. Estado: propuesta para revisión, no implementada. Base 1ba3c9c. Fase 1, submódulo 5.6.1.

## Objetivo y decisiones del usuario

Un PDF por trabajo completo, con una página por variante aceptada y orden ascendente del índice global. Los grupos N regulan procesamiento, no crean PDFs separados. Rechazos conservan índice/razón sin página. Reducir geometría oculta innecesaria para impresión manteniendo la apariencia y los vectores. Primera prueba elegida por el usuario: **100 × 100 mm, sin sangrado**; ambas medidas serán configurables.

Este diseño no promete PDF/X, conversión CMYK, perfiles ICC ni aceptación/rendimiento Fiery. No introduce servicios, lenguajes, GPU o Python de producto. El código y los tipos actuales v1 permanecen válidos.

## Alternativas

1. **Recomendación: contrato de documento nuevo que referencia GenerationSpec v1.** La generación produce candidatos y SVG finales como hoy; un ensamblador convierte cada variante en una página y finaliza un solo documento. Conserva las funciones probadas y distingue los hashes. Coste: especificar la colocación del viewport SVG en medidas físicas y mantener versiones del exportador PDF.
2. Nuevo GenerationSpec v2 con salidas por variante o documento. Permite unificar toda salida, pero exige migración de contratos y consumidores que el PDF inicial no necesita. No seleccionado.
3. PDFs separados por variante y unión posterior. Requiere gestionar muchos documentos intermedios y otra etapa de combinación; no seleccionado para este flujo.

## Contratos propuestos

Agregar después de aprobar el diseño un espacio nuevo contracts/engine/pdf/v1/, sin modificar schemas publicados de contracts/engine/v1/. Generar tipos Rust desde JSON Schema como en los contratos existentes; referencias locales resueltas en build, sin descargas de schemas. Cualquier necesidad posterior de romper v1 requerirá una propuesta y autorización distintas.

PdfJobSpec:

- schemaVersion: "1".
- generation: GenerationSpec v1 completo, validado con las reglas actuales; output.format debe ser "svg". Ese output describe el SVG intermedio: dimensiones lógicas, fondo y encuadre, sin inferir centímetros desde píxeles. Una solicitud PNG en este campo se rechaza, no se convierte silenciosamente.
- pdf: { exporterVersion: "1", trimWidthMm, trimHeightMm, bleedMm }. Sin valores implícitos para tamaño/sangrado: la primera configuración guarda 100, 100 y 0 expresamente. Dimensiones finitas positivas y sangrado finito no negativo. Política inicial propuesta: trim de 1 a 2000 mm por eje y bleed de 0 a 50 mm por lado; validar también sumas/conversión a puntos antes de generar. Estos límites son política, no capacidad medida de un RIP.

Los límites actuales de GenerationSpec permanecen, incluido count máximo 10000 por solicitud. Esta entrega no afirma admitir millones de páginas en una sola solicitud. Hilos, tamaño de grupo, rutas, cancelación y presupuesto de memoria son opciones operativas, fuera del hash de configuración reproducible.

PdfDocumentResult:

- Versiones de schema/exportador, templateHash, seed, startIndex, count y generationConfigurationHash.
- documentConfigurationHash: SHA-256 del PdfJobSpec validado/normalizado, incluyendo el intervalo completo y medidas PDF. Canonicalizar con los mismos tipos generados y claves ordenadas usados actualmente; sin timestamps ni opciones operativas. generationConfigurationHash mantiene la semántica v1, que excluye count/startIndex.
- Estado completed o no_pages; requestedCount, acceptedCount, rejectedCount y pageCount. Solo completed tiene documentHash SHA-256 de todos los bytes del PDF ya finalizado. no_pages tiene pageCount=0 y ningún PDF/hash de documento.
- El resultado de documento no contiene una lista enorme de todas las páginas. El mapa se emite como registros incrementales por índice: índice decimal u64, estado, razón si rechazo y pageNumber de base 1 si aceptada. Una página identifica también el artifactHash del SVG intermedio y sus metadatos VariantResult existentes. Un rechazo no tiene pageNumber ni artifactHash.

**No llamar artifactHash al hash del PDF completo.** Tampoco afirmar que el hash SVG representa bytes PDF de una página: compresión y referencias de objetos son parte del documento. Los registros se ordenan por índice, con páginas 1..acceptedCount sin huecos; acceptedCount + rejectedCount = requestedCount y pageCount = acceptedCount.

Fallos de conversión, presupuesto o E/S son errores de trabajo, distintos de rechazo de validez. No emitir completed ni convertir un fallo de exportación en rechazo para terminar aparentemente bien. Estados operativos de cancelación/error y publicación pertenecen a la CLI de tarea 6; el ensamblador no publica un documento parcial como éxito.

## Página y sangrado

Convertir explícitamente puntos = milímetros × 72 / 25.4; el tamaño no depende de DPI ni de output.width/height. Esas dimensiones del SVG intermedio definen su viewport lógico y su encuadre. Colocarlo con escala uniforme y cover centrado sobre el rectángulo de corte. Si las proporciones difieren, recortar sobrante, conservando proporción. La primera prueba cuadrada no añade recorte respecto al SVG cuadrado actual.

Con ancho W, alto H y sangrado B: MediaBox y BleedBox abarcan (W+2B) × (H+2B); TrimBox es [B,B,B+W,B+H]. CropBox coincide con MediaBox para inspeccionar el sangrado. Origen PDF inferior izquierdo. Con B=0 las cuatro cajas coinciden. Fondo RGB cubre MediaBox; transparente no introduce relleno blanco obligatorio. Sin marcas de corte en esta entrega.

El sangrado positivo extiende la misma transformación del arte más allá del corte usando la geometría semilla disponible: no estira el diseño ni duplica píxeles del borde. Solo se amplía el límite exterior de página; los clips internos del diseño siguen vigentes. Si la plantilla no pinta esa zona, no inventar contenido ni garantizar cobertura de sangrado. Se debe medir/reportar esa cobertura; no confundir el fondo con tinta del diseño. Caso positivo con fixture que sí cubre sangrado obligatorio aunque la primera muestra del usuario use cero.

## Vector y limpieza geométrica

Primero validar y conservar originales; optimizar solo la representación de impresión por página:

1. Resolver transforms y geometría de pintura, conservando composición/opacidades de grupo y orden de dibujo.
2. Descartar objetos totalmente exteriores al área MediaBox/sangrado usando límites conservadores que incluyan trazos. No clasificar exterior por centro o puntos de control aislados.
3. Intersectar geometría que cruza el límite y resolver clips internos. Para trazos, considerar contorno, caps, joins y miters en el espacio adecuado antes de intersectar; no cortar el eje del trazo y crear extremos artificiales.
4. Mantener rellenos, agujeros y fill-rule/clip-rule; no aplanar opacidades de grupo como si fueran opacidades independientes.
5. Eliminar recursos/definiciones ya no referenciados después del recorte; comprobar que no queden referencias colgantes. Un clip PDF, una BBox o un viewBox por sí solos no cumplen estos pasos.

No se promete intersección analítica exacta de toda curva. Si una biblioteca exige aproximar curvas por segmentos vectoriales, definir el error geométrico en unidades físicas y medir después la diferencia de imagen. No fijar una tolerancia aceptable inventada: presentar la evidencia antes de congelar esa política. Limitar segmentos/nodos nuevos durante el recorte, no solo en la entrada; detenerse con error explícito si excede presupuesto o no se puede conservar una característica admitida. Prohibido rasterizar silenciosamente como fallback.

## Ensamblado y memoria

Una variante/página se transforma y escribe en un destino temporal secuencial antes de liberar su geometría. El número N no obliga a retener N SVG/PNG o páginas. El orden de emisión y asignación de objetos se determina por índice global, aunque el cálculo futuro use varios workers; cola de reordenamiento acotada.

El escritor debe separar cuerpos de páginas del cierre del documento: usar destino temporal y escritura/finalización controladas. También crecen xref, árbol de páginas y mapa índice→página; presupuestarlos o externalizarlos, sin anunciar RAM constante solo porque se libera el SVG. La primera versión respeta el límite de 10000 índices del contrato existente.

No seleccionar como solución de memoria un API que guarda todo el documento en un Vec y devuelve esos bytes al final. Ni acumular un SVG o un PDF por página para unirlos después. La evaluación de bibliotecas debe demostrar escritura de cuerpos de objetos hacia un writer/spool y finalización de un PDF válido con referencias correctas.

La publicación sin reemplazar archivos, temporal en el destino apropiado, cancelación y manifiesto atómico se integran en tarea 6. Pruebas de 5.6 usan un destino temporal controlado y leen/hash el documento completo solo una vez finalizado; eso no prueba publicación segura de CLI.

## Determinismo

PDF idéntico byte a byte para misma plantilla, PdfJobSpec, backend CPU y versión/entorno fijados. Asignación de referencias, compresión, orden de recursos y metadatos estables; sin fecha actual ni IDs aleatorios. Cambios del exportador o del recorte que alteren bytes deben registrar su versión. N/hilos no afectan contenido, decisiones discretas ni numeración de páginas.

## Evaluación técnica pendiente, sin dependencias instaladas

Se consultaron fuentes primarias. svg2pdf ofrece conversión a Form XObject/Chunk y licencia MIT/Apache-2.0; el repositorio consultado figura archivado y recomienda krilla/krilla-svg como alternativa mantenida a evaluar. El Cargo.toml de main leído directamente declara usvg/resvg 0.46 y pdf-writer 0.14, distintos del usvg 0.47 actual. El índice de documentación de la versión publicada presenta dependencias anteriores: fijar y revisar el paquete concreto antes de decidir, no asumir que main coincide con el release. No está seleccionado ni se promete compatibilidad directa. pdf-writer 0.15 documenta un buffer global en Pdf; su API de Chunks requiere estudiar finalización/offsets antes de alegar streaming. i_overlay se evalúa para intersecciones de contornos, sin asumir que por sí solo resuelve curvas o semántica SVG.

Decisión del siguiente paso: verificar código/licencias/versiones de candidatos para cubrir subconjunto seguro, vector, compatibilidad de árbol, recorte y escritura acotada. Preferir Rust y bibliotecas existentes; una frontera FFI/lenguaje nueva exige ADR antes de aceptarse. Si ninguna satisface memoria/fidelidad, presentar el coste de un escritor limitado a nuestro subconjunto; no incorporar un serializer PDF completo sin justificación.

Fuentes consultadas 2026-10-15:

- https://docs.rs/svg2pdf/latest/svg2pdf/ y https://github.com/typst/svg2pdf
- https://raw.githubusercontent.com/typst/svg2pdf/main/Cargo.toml (manifiesto main leído; la apertura directa del manifiesto de docs.rs falló)
- https://docs.rs/pdf-writer/latest/pdf_writer/ y https://docs.rs/pdf-writer/latest/pdf_writer/struct.Chunk.html
- https://docs.rs/i_overlay/latest/i_overlay/

## Criterios de terminado y pasos de 5.6

- **5.6.1 Diseño:** aprobar este documento, medidas iniciales registradas y contratos/hash/mapa/cajas claros. Después redactar plan de implementación; no equivale a tener PDF implementado.
- **5.6.2 Contratos y selección técnica:** schemas nuevos y tipos generados, tests de entrada/resultados, elección fijada con licencias revisadas y evidencia de escritura por página. Sin cambiar v1 publicado.
- **5.6.3 Página vectorial y optimización:** una página segura, recorte real y recursos limpios; fixtures con bordes, agujeros, clips anidados, grupos opacos/translúcidos y trazos. Medir diferencias frente a referencia CPU y complejidad; fijar política de precisión con los resultados.
- **5.6.4 Documento completo:** una/múltiples páginas, orden por índice, rechazo intermedio, sin páginas y error de writer; grupos 1/7/100 sobre 201 índices y último grupo incompleto, mismos bytes/hash, conteos y recursos correctos. Grupos de esta prueba no sustituyen el coordinador CLI de tarea 6.
- **5.6.5 Validación y cierre:** test.svg preparado, PDF 100 × 100 mm sin sangrado y fixture con sangrado positivo, render por un lector PDF independiente, revisión visual, bytes/objetos/segmentos y memoria medida por separado; formato/lint/tests y revisión independiente global de toda tarea 5. Fiery pendiente si no hay equipo.

Pedir autorización antes del siguiente paso como solicitó el usuario. No marcar tarea 5 completa hasta cumplir 5.6; no pasar a fase 2 sin confirmar.
