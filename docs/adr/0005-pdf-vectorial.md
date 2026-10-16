# ADR 0005 — Documento PDF vectorial y escritura por objetos

Fecha: 2026-10-15. Decisión implementada en el motor CPU; pruebas de contrato, documento y geometría PDF en services/engine/crates.

## Contrato y fronteras

Agregar contracts/engine/pdf/v1 como contrato de documento, reutilizando GenerationSpec/VariantResult v1. No añadir pdf como formato de una variante: un PDF contiene todas las aceptadas del trabajo, ordenadas por índice. Un hash de SVG identifica el artefacto intermedio; documentHash identifica el PDF completo. El contrato nuevo y tipos generados no introducen una frontera entre lenguajes, servicio o FFI. CPU/Rust sigue siendo suficiente.

Primera prueba solicitada: corte 100 × 100 mm, sin sangrado. Tamaño y sangrado son parámetros explícitos. MediaBox/CropBox/BleedBox incluyen sangrado y TrimBox define corte. Escala uniforme cover centrada; no inferir tamaño físico desde píxeles. Con sangrado se extiende la transformación y se conserva geometría disponible, sin inventar arte. No se promete PDF/X, CMYK, ICC ni cobertura suficiente de una plantilla arbitraria.

## Bibliotecas y escritor

Seleccionar pdf-writer 0.15.0 para serializar Content y un Chunk por objeto, nunca Pdf con buffer global. Un escritor pequeño propio mantiene offsets/IDs y emite cabecera, cada objeto y cierre xref clásico/trailer a Write. No implementa un parser PDF ni un conversor SVG completo. El crecimiento de offsets y árbol de páginas sigue existiendo: máximo un millón de objetos y 10000 páginas, con bytes/offsets comprobados. El presupuesto PNG no es cota RSS de PDF.

svg2pdf quedó descartado: repositorio archivado y dependencias de usvg anteriores a la versión fijada; su conversión general no elimina por sí sola geometría oculta. Reescribir el cierre xref acotado permite reutilizar pdf-writer sin acumular todo el documento ni introducir otro servicio. Comprobar xref y referencias con pypdf/Poppler, no solo con tests del escritor.

## Recorte real

usvg 0.47 resuelve formas/transforms/clips; tiny-skia contornea trazos antes de aplicar transforms de escala desigual. kurbo 0.13.1 permite subdividir Béziers con presupuesto/profundidad explícitos; i_overlay 9.0.1 normaliza fill-rule e intersecta contornos, incluidos clips internos y rectángulo de página. Conservar composición/opacidad de grupos con Forms de transparencia aislados; no multiplicar opacidad individual de figuras superpuestas. Quitar grupos vacíos y emitir solo recursos referenciados.

Precisión geométrica de esta versión: subdivisión de curva hasta 0.001 puntos PDF (aprox. 0.000353 mm), malla de intersección 0.000001 puntos con conversión i64 comprobada. Son parámetros del exportador, no una garantía de diferencia máxima por píxel. La versión 1 congela esos parámetros tras medir las muestras; diferencias de render independientes registradas en el informe.

Límites: dos millones de puntos subdivididos y dos millones emitidos por página; hasta 32768 puntos por operación booleana. Antes de construir el grafo, barrido de cajas de segmentos para presupuestar también autointersecciones: hasta 50000 parejas candidatas y dos millones de comparaciones. Cajas ampliadas por 0.00001 puntos para cubrir cuantización. Política conservadora: puede rechazar diseños legítimos complejos y no equivale a una cota demostrada del RSS completo. Expansión, profundidad o coordenadas incompatibles devuelven error explícito, sin fallback raster ni éxito parcial.

## Alternativas y coste

Mantener toda la semilla detrás de clips deja carga geométrica innecesaria. Rasterizar páginas oculta el problema a costa de perder vectores. Uniones de PDFs por variante añaden archivos/etapa y retención. El enfoque elegido conserva vectores como contornos; puede aumentar puntos al aproximar curvas/contornear trazos. Medir peso frente a baseline con mismo pipeline y geometría exterior retenida, no equiparar número de comandos SVG con puntos de polígonos ni prometer un ahorro universal.

## Determinismo y fallos

Asignación de objetos por índice global, compresión/recursos estables y sin fechas/IDs aleatorios. N agrupa procesamiento secuencial y no cambia bytes/hash/mapa. Sin aceptadas, no se escribe cabecera ni PDF de éxito vacío. Error de writer/callback deja un temporal incompleto sin resultado completed; el caller no lo publica. Publicación/cancelación global pertenecen a tarea 6. Fiery real sigue pendiente.
