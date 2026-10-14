# Análisis de referencia: ejemplo_export.pdf

Fecha: 2026-10-14. Inspección de solo lectura durante la pausa. No se optimizó ni modificó el PDF; no se retomó implementación. Documento propietario usado únicamente para examinar estructura y coste potencial de exportación, no para copiar algoritmos, assets o diseño.

## Archivo e inspección

- Fuente local: docs/ejemplo_export.pdf.
- SHA-256: a7235e10cd57ecaf7403e2e1da39457b0b18ee46b4c899b11d6a19c1a5695591.
- Tamaño: 158519772 bytes, 158.52 MB decimales / aproximadamente 151.18 MiB.
- 250 páginas; PDF 1.4, no cifrado. Primera página: MediaBox/CropBox 0 0 170.079 170.079 puntos, aproximadamente 6 × 6 cm; no 10 × 10 cm en esta muestra.
- Metadatos: Creator HP Designer-ID(MAC)16.0.10; Producer Adobe PDF Library 16.0. pdfinfo indica Optimized yes; esa etiqueta no demuestra eliminación de geometría exterior.
- Herramientas: Poppler pdfinfo/pdftoppm y pypdf del runtime local. Estructura muestreada en páginas 1, 2, 125 y 250. Página 1 renderizada a PNG de 700 px e inspeccionada: motivos vectoriales muy claros sobre blanco.
- Diagnósticos locales en out/pdf-reference/: page-1.png y summary.json. No se incluye el PDF de 158 MB en el commit documental.

## Resultados estructurales medidos

Las cuatro páginas invocan 55 XObjects de tipo Form cada una mediante Do; no invocan Image XObjects directamente. Cada conjunto contiene 54030 operadores PDF, de los que 51201 son operaciones de construcción de trazados (m/l/c/v/y/re/h), y 55 operaciones de clip. No confundir operadores con número de objetos visibles, paths únicos o curvas efectivamente pintadas.

| Página | Forms invocados | Forms con BBox completamente exterior | Bytes comprimidos de todos los Forms | Bytes comprimidos de Forms exteriores | Operadores de trazado en Forms exteriores |
|---|---|---|---|---|---|
| 1 | 55 | 52 | 615066 | 212196 | 17410 |
| 2 | 55 | 43 | 605086 | 167832 | 14015 |
| 125 | 55 | 35 | 608035 | 138012 | 11450 |
| 250 | 55 | 37 | 599710 | 137630 | 11486 |

Clasificación exterior: normalizar extremos de cada BBox, comprobar ausencia de intersección con MediaBox y verificar Matrix identidad. Ninguno de los cuatro streams de página contiene operaciones cm; no se omitió una transformación de colocación de página en esta clasificación. Los Forms incluyen sus propios clips y transforms internos. La tabla cuenta objetos invocados, no solo recursos presentes sin uso.

En página 1, los 55 streams de Form suman 1559392 bytes después de descompresión; los 52 Forms exteriores suman 530085 bytes. Además, los dos Forms principales tienen cajas mayores que la página y contienen 23694 y 10040 operadores respectivamente: puede existir más geometría oculta dentro de ellos, pero no se cuantificó con una intersección geométrica exacta.

No hay referencias de Form compartidas entre página 1 y cada una de las otras tres páginas muestreadas. Esto indica recursos distintos en esa muestra; no demuestra ausencia de reutilización de todos los recursos en las 250 páginas. Se contaron 34081 entradas de xref, no se auditaron individualmente todos los objetos.

## Interpretación y límites

La muestra confirma que se retienen e invocan estructuras vectoriales con cajas completamente fuera de la página, además de clipping y Forms extensos. Esto respalda la preocupación del usuario y justifica descartar geometría exterior en nuestra exportación final.

No se midió el RIP Fiery, ni RAM, ni tiempo de procesamiento, ni reducción de tamaño tras optimización. No se atribuye toda la lentitud a estos objetos: un RIP podría descartar algunos por sus cajas. Los bytes anteriores son tamaños de streams muestreados, no ahorro garantizado del archivo final. No se interpretaron todos los clips ni se contaron exactamente todos los segmentos visibles/invisibles dentro de los Forms que intersectan la página.

Aplicar requisitos-exportacion-impresion.md: descarte/recorte geométrico, limpieza de recursos, comparación visual y medición de artefacto/RIP. La implementación continúa pausada y PDF sigue como entrega posterior con diseño propio.
