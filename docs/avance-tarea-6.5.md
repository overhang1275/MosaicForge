# Avance 6.5 — PDF único desde CLI

2026-10-15. Base 04a5359. Usuario autoriza 6.5; detenerse antes de 6.6. Se aplican Superpowers/Ponytail y skill PDF para verificar el documento real.

## Pausa anterior y reanudación

Usuario pidió parar porque quedaba 8% de uso; después autorizó explícitamente «continua, con el punto 6.5 que quedo inconcluso». Reanudado desde los cambios guardados, sin repetir implementación. Último checkpoint verificado al retomar: 04a5359 (6.4). No adelantar 6.6.

Implementado sin cierre de verificación: subcomando generate-pdf, módulo CLI pdf.rs, PendingFile reutilizable para PDF incremental, helpers compartidos de lectura/manifiesto/token y generate_pdf_cancellable en CPU con API generate_pdf anterior conservada. Nuevos tests de proceso en cli/tests/generate.rs: documento único de 201 variantes para N=1/7/100, cero aceptadas y protección ante archivo existente/entrada inválida. Se observaron tres fallos rojos iniciales por subcomando ausente; al pausar, cargo test -p mosaicforge-cli --locked --offline --test generate pdf_cli terminó con **3 tests pasan, cero fallos**, nueve filtrados. No equivale a suite completa ni a QA del PDF real.

Al retomar quedaban tests de cancelación/fallos, checks y PDF real/QA. Ya se ejecutaron, con evidencia abajo. Marcador del skill PDF ejecutado una vez antes de la pausa para esta operación; no se repitió.

- [x] ~~Tres tests rojos iniciales por subcomando ausente, seguidos de tres tests de proceso verdes antes de la pausa.~~
- [x] ~~Un mosaic.pdf por trabajo; 201 variantes en el mismo documento para N=1/7/100 e índices globales. PDF secuencial, sin flag threads ignorado.~~
- [x] ~~PendingFile reutilizable: temporal incremental, cierre/sincronización y hard_link exclusivo. Cero aceptadas sin PDF vacío.~~
- [x] ~~Manifiesto PDF: cabecera, páginas provisionales, documento/hash tras publicación y resumen.~~
- [x] ~~Token antes/entre/después de páginas y antes de publicar. Tests de cancelación previa, durante el trabajo y tras la última página pasan sin PDF/temporales supervivientes. API generate_pdf anterior conservada.~~
- [x] ~~Tests de grupos/índices/hash, rechazos, cero páginas, colisión, cancelación y fallos inyectados. fmt, Clippy -D warnings y suite workspace: 129 tests pasan, nueve locales omitidos, cero fallos.~~
- [x] ~~PDF real 100×100 mm sin sangrado; pypdf/Poppler, comparación 300 DPI y revisión visual de sus tres páginas pasan. Documentación de uso y continuidad actualizada para checkpoint verificado.~~

Ruling: nuevo subcomando `generate-pdf --template FILE --spec PDF_JOB --out DIR --batch-size N`, porque PdfJobSpec agrega dimensiones físicas/exportador y GenerationSpec conserva formatos SVG/PNG publicados. No duplicar modelos ni añadir flags ambiguos al contrato. PDF se procesa secuencialmente con buffers de una página, respetando el algoritmo existente y sin prometer aceleración. La concurrencia PDF requeriría medir y ordenar otro pipeline, fuera de 6.5.

Ruling: el manifiesto persiste los registros de página como provisionales: no representan un documento utilizable hasta que exista un registro pdf_document con hash/publicación completada. Ante error de manifiesto después de publicar puede quedar un PDF completo sin registro final, como cualquier publicación+manifiesto sin transacción; nunca PDF parcial publicado. Publicación existente se protege con la misma primitiva exclusiva, incluso si aparece por carrera.

## Evidencia al terminar

Comandos desde services/engine: cargo fmt --all -- --check; cargo clippy --workspace --all-targets --locked --offline -- -D warnings; cargo test --workspace --locked --offline --quiet. Pasan 129 tests: 73 core, 22 CPU y 34 CLI. Nueve locales ignorados por defecto; QA real se ejecutó aparte. Sin paquetes externos nuevos: lock conserva el inventario de 103 registrado en 6.4.

Se añadió una prueba mixta de rechazos/numeración que falló inicialmente porque solo variaba scaleX, sin producir ambos estados en la muestra elegida. Se comparó con el fixture CPU existente y se añadió el rango scaleY faltante. El assert exige aceptadas y rechazadas; pasó aislado y después dentro de la suite completa. Ningún cambio de algoritmo para acomodar la prueba.

Tests inyectados PDF: cancelación antes de comenzar, después de primera página y tras última página; todos devuelven 130 sin archivo final ni temporal. Error/short write/flush de manifiesto y stdout roto antes de publicar: exit 3 y temporal eliminado. Error de manifiesto después de publicar: únicamente sobrevive un PDF completo con EOF, sin falso registro de documento completo. La carrera de publicación sigue cubierta por los tests de la primitiva compartida PendingFile/hard_link.

Prueba CLI real:

```powershell
services/engine/target/debug/mosaicforge.exe generate-pdf --template out/test-prepared.svg --spec out/pdf-5.6-release/job.json --out out/cli-6.5-pdf --batch-size 7
```

Exit 0. Un solo out/cli-6.5-pdf/mosaic.pdf, tres páginas para índices 0..2, 100×100 mm sin sangrado, 2 730 275 bytes. SHA-256 dd1208f9fea7fa7150e4811bf56e33879ba2c99237aa847167b95508789cfa01, idéntico byte por byte al PDF de 5.6 para esa misma spec. Manifiesto con cabecera, tres páginas, resultado de documento y resumen completed; artifactHash de variantes corresponde al SVG previo, documentHash al PDF completo.

QA: scripts/verify-pdf-export.py out/cli-6.5-pdf --pdf mosaic.pdf --pages 3 --pdftoppm <Poppler del runtime>. pypdf strict: 14 objetos alcanzables, tres Forms, cero imágenes raster, sin recursos sin uso; cajas físicas y paths acotados comprobados. Poppler 300 DPI: MAE 1.0011/1.1880/0.9750, RMSE 5.3344/5.4545/5.0195 frente a referencias resvg de 5.6 (misma spec, bytes finales sin cambio). Cero píxeles de tinta faltante/extra lejos del borde en el interior; 6/7/8 de tinta faltante incluyendo el borde, diferencia conocida de rasterizadores. No se afirma identidad de píxeles ni tolerancia CPU/GPU. Se inspeccionaron visualmente qa-page-1.png, qa-page-2.png y qa-page-3.png: composición, trazos y recorte correctos. Evidencia completa en out/cli-6.5-pdf/qa-summary.json; resultados locales fuera de Git.

Se extendió el verificador Windows existente con --pdf, sin dependencias nuevas. Corrida real CTRL_C_EVENT, consola oculta aislada, out/cli-6.5-pdf-cancel-signal/verification.json: exit 130, una candidata aceptada provisional, tres registros de manifiesto (job, página, resumen cancelled), ningún PDF final, cero temporales/artefactos sin registrar. La variante provisional no se presenta como PDF exportado. El modo PNG del helper no se volvió a ejecutar; el flujo PNG Rust sí pasó todos sus tests de regresión.

## Continuidad y límites

6.5 terminado; esperar confirmación antes de 6.6. Pendiente 6.6: perfil por etapa/RAM, revisión independiente global y cierre de fase 1. No se midió aceleración, RSS ni latencia máxima de cancelación aquí. Linux/macOS no ejecutados. Fiery excluido; sin PDF/X/CMYK ni promesas de millones de páginas. Cancelación cooperativa espera la etapa en curso y la publicación iniciada puede terminar íntegra. Si falla el registro posterior a publicar puede quedar el PDF completo sin manifiesto final; comprobar salida/manifest antes de reintentar. No hay reemplazo ni reanudación automática.
