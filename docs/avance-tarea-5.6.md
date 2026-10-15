# Submódulo 5.6 — PDF único multipágina

Fecha: 2026-10-15. Base de implementación c2ccd9a. Diseño aprobado; usuario pidió «sii terminalo», autorizando ejecución continua de 5.6. Implementación funcional 3b51db2, QA independiente y memoria verificados; tarea 5 completa. CLI/publicación siguen en tarea 6.

- [x] ~~Registrar medida inicial elegida: 100 × 100 mm, sangrado 0.~~
- [x] ~~Leer requisitos y contratos existentes, consultar Graphify y fuentes primarias de candidatos.~~
- [x] ~~Guardar propuesta revisable de contrato agregado, hashes, página, geometría y memoria.~~
- [x] ~~Usuario aprueba diseño y pide terminar 5.6 sin más pausas.~~
- [x] ~~Plan de implementación guardado en superpowers/plans/2026-10-15-pdf-multipagina.md.~~
- [x] ~~5.6.2: contratos nuevos, tipos y selección técnica comprobada.~~
- [x] ~~5.6.3: página vectorial con recorte real y limpieza; QA de precisión ejecutado.~~
- [x] ~~5.6.4: documento multipágina, índices, hashes y trabajo acotado.~~
- [x] ~~5.6.5: muestras reales, métricas, revisión visual y revisión independiente global.~~

Graphify en diseño: presupuesto 300, 67 nodos encontrados/ocho mostrados, truncado. Snapshot anterior a tarea 2; fuentes actuales contrastadas directamente. Investigación y decisión posterior en ADR 0005; ocho paquetes Rust open source nuevos, sin FFI/servicios/lenguajes de producto nuevos. Python es herramienta local de QA.

Implementación: pdf-writer serializa un objeto por Chunk hacia Write; cierre xref propio y hash de todos los bytes, sin buffer de documento completo. Tipos nuevos generados reutilizan GenerationSpec/VariantResult. Validadores semánticos comprueban estados/conteos/hash/páginas. usvg/tiny-skia/kurbo/i_overlay recortan rellenos y contornos de trazos, conservan reglas de relleno, clips, transforms y opacidad de grupo. Se eliminan grupos vacíos y recursos no referenciados. No hay CLI/publicación ni PDF/X/CMYK.

## Tests y revisión

RED observado: dos pruebas de contrato con stubs; dos de geometría; dos de documento; regresión adicional de autointersección. GREEN posterior. Suite normal verificada: 95 tests (73 core + 22 cpu), nueve locales omitidos, formato y Clippy pasan. La geometría adversarial entra como fixture pequeño para evitar provocar OOM real.

Una expectativa inicial no_pages usaba un rectángulo sin grupo: las reglas actuales de opacidad actúan sobre capas g y por tanto ese fixture no se volvía invisible. Corregido el fixture añadiendo g, sin cambiar semántica del motor.

Pruebas: N=1/7/100 sobre 201 índices, mismo PDF/hash/mapa y grupo final incompleto; rechazos intermedios sin huecos de página, índice u64::MAX; error de writer/callback, N=0 rechazado, cero aceptadas sin PDF/hash. Corte 100 mm y sangrado 3 mm, dimensiones finitas/límites; metadata inconsistente rechazada. Geometría de trazo que cruza borde, agujero, clip y grupo opaco/translúcido.

Revisión independiente global de tarea 5 detectó un P1: normalización con clip vacío no presupuestaba autointersecciones; el grafo podía expandirse antes del control de salida. Se reprodujo RED con estrella de 129 puntos. Corregido con barrido previo de cajas para autointersecciones e intersecciones, conversión fija i64 comprobada, regresión final con estrella de 1025 puntos y error antes del grafo. El límite inicial de 5000 candidatos rechazó test.svg legítimo: política ajustada a 50000, manteniendo límites de comparaciones y puntos; muestra real vuelve a pasar. No se provocó OOM ni se afirma cota RSS global.

Un intento de suite simultánea con el test local produjo LNK1104 de Windows por binario en ejecución; se esperó y repitió secuencialmente, con exit 0. Primer intento de descargar crates bloqueado por red del sandbox; descarga autorizada con elevación funcionó. Sin cambios LICENSE/CI o v1 publicado.

## QA independiente ya ejecutado

PDF final de tres páginas con rango continuo 0..2 (no índice 17 fingido), 100 × 100 mm sin sangrado. Pypdf estricto: 14 objetos alcanzables, tres Forms, cero imágenes; sin recursos sin uso ni referencias perdidas. Malla y todos los puntos de paths dentro de página. Arte original conservado; se usa la misma copia preparada y reglas de 4.4. Artefacto final out/pdf-5.6-release/mosaic-100mm.pdf: **2730275 bytes**, SHA-256 **dd1208f9fea7fa7150e4811bf56e33879ba2c99237aa847167b95508789cfa01**. Hash y mapa contrastados con resultados guardados por el motor.

Poppler a 300 DPI frente a resvg: MAE por canal 0.975–1.188/255, RMSE 5.020–5.455, máximo 198/255 en borde/antialias. No son píxeles idénticos. Cero tinta ausente/extra a más de dos píxeles dentro del área interior; 6–8 píxeles de diferencia de presencia en fila/columna exterior por píxel físico fraccional. Script verify-pdf-export.py usa límites de regresión derivados de estas muestras: MAE<=1.3 y RMSE<=6, sin convertirlos en promesa universal de paridad entre lectores.

Cuatro fixtures PDF independientes: opacidad de grupo con solapamientos, trazo curvo bajo transformación no uniforme, clips anidados con agujero/unión y sangrado positivo de 3 mm. Cajas físicas correctas, cero imágenes/recursos huérfanos y diferencias medidas dentro del umbral. Revisión visual de las siete páginas rasterizadas hecha: sin defectos de encuadre/composición nuevos; espacio negativo del arte conservado. Fixture de sangrado cubre su zona extra con el arte extendido; no se promete cobertura de cualquier plantilla arbitraria.

Baseline controlado de una página: mismo serializador, normalización, transforms y clips internos, reteniendo geometría exterior al MediaBox frente a recorte físico real. Medición repetida con código final: 2284331 → 824466 bytes (63.91% menos); 267606 → 96195 puntos emitidos; 108 → 50 operaciones de pintura no vacías. Ambos son PDFs propios, no comparación de assets/algoritmos HP. Diferencia visual de los dos PDFs a 300 DPI: MAE 0.0100, RMSE 1.1274, 219 píxeles cambiados (seis interiores; principalmente borde). Baseline/optimizado: out/pdf-5.6-{baseline,optimized}-validated.pdf; no se entregan como PDFs separados de variantes, solo controles de una página. Sin Fiery ni aceleración medida.

No confundir comandos SVG originales (22244 segmentos simplificados en el artefacto) con puntos de contornos poligonales: contornear/aproximar curvas puede aumentar puntos. Límites de precisión/complejidad y decisión de dependencias en ADR 0005. Memoria medida con contadores OS del proceso completo de test, separada del presupuesto PNG y del tamaño PDF.

## Memoria medida y cierre

scripts/measure-test-memory.py ejecutó el binario local_pdf ya compilado con --ignored --nocapture y consultó GetProcessMemoryInfo de Windows cada 50 ms. Se registraron picos del OS, no una extrapolación de buffers: working set **78241792 bytes (74.62 MiB)** y commit privado **156729344 bytes (149.47 MiB)**. 1857 consultas, 93.62 s, exit 0. Incluye generación, exportación PDF, relectura de diagnóstico y PNG de referencia; excluye compilador/monitor. No es un benchmark release del exportador ni una comparación de RAM contra el baseline. Valores/cmd/log en out/pdf-5.6-memory.{json,log}. psutil no estaba instalado; se usó ctypes estándar, sin instalar dependencias Python nuevas.

Se ejecutaron aparte los tres tests PDF locales nuevos: muestra real, baseline y fixtures; los seis locales anteriores no se repitieron en este cierre. Se revalidaron los artefactos finales con verify-pdf-export.py y se inspeccionaron las siete páginas. El original conserva SHA-256 ceae058483698070aee16a8d1337715167142b198c47a28e626a0bb8683fbbf8.

Revisión independiente posterior: sin nuevos hallazgos importantes; P1 confirmado corregido, validadores y poda de grupos revisados. El revisor ejecutó la regresión adversarial y pasó. Se reforzó después para exigir el error específico del preflight, evitando que una validación tardía de tamaño dé un falso positivo.

Comandos desde services/engine con Cargo en PATH:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --quiet
cargo test -p mosaicforge-cpu --test local_pdf --locked -- --ignored --nocapture
cargo test -p mosaicforge-cpu write_untrimmed_local_baseline --lib --locked -- --ignored --nocapture
cargo test -p mosaicforge-cpu write_pdf_visual_fixtures --lib --locked -- --ignored --nocapture
```

QA desde raíz con Python/Poppler del runtime: verify-pdf-export.py out/pdf-5.6-release --pages 3 --pdftoppm <ruta>; fixtures: directorio out/pdf-5.6-fixtures --pdf fixtures.pdf --pages 4 --bleed-last 3. Configuración, mapas, referencias y qa-summary.json quedan en out/, excluidos de Git. No se publicaron PDFs de usuarios ni se tocó ejemplo_export.pdf. El helper inicial de marcado se tomó del plugin documentos y rechazó formato PDF; se corrigió usando el helper del plugin PDF, que terminó exit 0. No afectó al motor ni a los archivos.

Fiery, PDF/X/CMYK, tolerancia CPU/GPU, perfil completo por etapa y CLI/publicación segura no se verificaron ni se dan por implementados. No se afirma soporte probado de millones de páginas ni RAM constante; count sigue limitado a 10000 y N agrupa trabajo secuencial.
