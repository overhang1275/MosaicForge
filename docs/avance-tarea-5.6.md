# Submódulo 5.6 — PDF único multipágina

Fecha: 2026-10-15. Base de implementación c2ccd9a. Diseño aprobado; usuario pidió «sii terminalo», autorizando ejecución continua de 5.6. Implementación funcional y checks normales verificados; QA final y memoria en curso al guardar este checkpoint.

- [x] ~~Registrar medida inicial elegida: 100 × 100 mm, sangrado 0.~~
- [x] ~~Leer requisitos y contratos existentes, consultar Graphify y fuentes primarias de candidatos.~~
- [x] ~~Guardar propuesta revisable de contrato agregado, hashes, página, geometría y memoria.~~
- [x] ~~Usuario aprueba diseño y pide terminar 5.6 sin más pausas.~~
- [x] ~~Plan de implementación guardado en superpowers/plans/2026-10-15-pdf-multipagina.md.~~
- [x] ~~5.6.2: contratos nuevos, tipos y selección técnica comprobada.~~
- [x] ~~5.6.3: página vectorial con recorte real y limpieza; QA de precisión ejecutado.~~
- [x] ~~5.6.4: documento multipágina, índices, hashes y trabajo acotado.~~
- [ ] 5.6.5: muestras reales, métricas y revisión independiente global.

Graphify en diseño: presupuesto 300, 67 nodos encontrados/ocho mostrados, truncado. Snapshot anterior a tarea 2; fuentes actuales contrastadas directamente. Investigación y decisión posterior en ADR 0005; ocho paquetes Rust open source nuevos, sin FFI/servicios/lenguajes de producto nuevos. Python es herramienta local de QA.

Implementación: pdf-writer serializa un objeto por Chunk hacia Write; cierre xref propio y hash de todos los bytes, sin buffer de documento completo. Tipos nuevos generados reutilizan GenerationSpec/VariantResult. Validadores semánticos comprueban estados/conteos/hash/páginas. usvg/tiny-skia/kurbo/i_overlay recortan rellenos y contornos de trazos, conservan reglas de relleno, clips, transforms y opacidad de grupo. Se eliminan grupos vacíos y recursos no referenciados. No hay CLI/publicación ni PDF/X/CMYK.

## Tests y revisión

RED observado: dos pruebas de contrato con stubs; dos de geometría; dos de documento; regresión adicional de autointersección. GREEN posterior. Suite normal verificada: 95 tests (73 core + 22 cpu), nueve locales omitidos, formato y Clippy pasan. La geometría adversarial entra como fixture pequeño para evitar provocar OOM real.

Una expectativa inicial no_pages usaba un rectángulo sin grupo: las reglas actuales de opacidad actúan sobre capas g y por tanto ese fixture no se volvía invisible. Corregido el fixture añadiendo g, sin cambiar semántica del motor.

Pruebas: N=1/7/100 sobre 201 índices, mismo PDF/hash/mapa y grupo final incompleto; rechazos intermedios sin huecos de página, índice u64::MAX; error de writer/callback, N=0 rechazado, cero aceptadas sin PDF/hash. Corte 100 mm y sangrado 3 mm, dimensiones finitas/límites; metadata inconsistente rechazada. Geometría de trazo que cruza borde, agujero, clip y grupo opaco/translúcido.

Revisión independiente global de tarea 5 detectó un P1: normalización con clip vacío no presupuestaba autointersecciones; el grafo podía expandirse antes del control de salida. Se reprodujo RED con estrella de 129 puntos. Corregido con barrido previo de cajas para autointersecciones e intersecciones, conversión fija i64 comprobada, regresión final con estrella de 1025 puntos y error antes del grafo. El límite inicial de 5000 candidatos rechazó test.svg legítimo: política ajustada a 50000, manteniendo límites de comparaciones y puntos; muestra real vuelve a pasar. No se provocó OOM ni se afirma cota RSS global.

Un intento de suite simultánea con el test local produjo LNK1104 de Windows por binario en ejecución; se esperó y repitió secuencialmente, con exit 0. Primer intento de descargar crates bloqueado por red del sandbox; descarga autorizada con elevación funcionó. Sin cambios LICENSE/CI o v1 publicado.

## QA independiente ya ejecutado

Primer PDF de tres páginas con rango continuo 0..2 (no índice 17 fingido), 100 × 100 mm sin sangrado. Pypdf estricto: 14 objetos alcanzables, tres Forms, cero imágenes; sin recursos sin uso ni referencias perdidas. Malla y todos los puntos de paths dentro de página. Arte original conservado; se usa la misma copia preparada y reglas de 4.4. Muestreo final actualizado pendiente al checkpoint.

Poppler a 300 DPI frente a resvg: MAE por canal 0.975–1.188/255, RMSE 5.020–5.455, máximo 198/255 en borde/antialias. No son píxeles idénticos. Cero tinta ausente/extra a más de dos píxeles dentro del área interior; 6–8 píxeles de diferencia de presencia en fila/columna exterior por píxel físico fraccional. Script verify-pdf-export.py usa límites de regresión derivados de estas muestras: MAE<=1.3 y RMSE<=6, sin convertirlos en promesa universal de paridad entre lectores.

Cuatro fixtures PDF independientes: opacidad de grupo con solapamientos, trazo curvo bajo transformación no uniforme, clips anidados con agujero/unión y sangrado positivo. Cajas físicas correctas, cero imágenes/recursos huérfanos y diferencias medidas dentro del umbral. Revisión visual final de las salidas actualizadas pendiente al checkpoint.

Baseline controlado de una página: mismo serializador, normalización, transforms y clips internos, reteniendo geometría exterior al MediaBox frente a recorte físico real. Primera medición: 2284331 → 824466 bytes (63.91% menos); 267606 → 96195 puntos emitidos; 108 → 50 operaciones de pintura no vacías. Ambos son PDFs propios, no comparación de assets/algoritmos HP. Diferencia visual de los dos PDFs a 300 DPI: MAE 0.0100, RMSE 1.1274, 219 píxeles cambiados (seis interiores; principalmente borde). Medición validada final se registra al cerrar; sin Fiery ni aceleración medida.

No confundir comandos SVG originales (22244 segmentos simplificados en el artefacto) con puntos de contornos poligonales: contornear/aproximar curvas puede aumentar puntos. Límites de precisión/complejidad y decisión de dependencias en ADR 0005. Memoria se medirá con contadores OS del proceso completo de test, separada del presupuesto PNG y del tamaño PDF.
