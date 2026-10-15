# Exportación para impresión: geometría útil y coste de RIP

Registrado: 2026-10-14. Requisito aportado por el usuario durante la pausa; no autoriza reanudar implementación ni adelantar PDF.

## Observación de origen

El usuario observa PDFs pesados y procesamiento lento en Fiery al exportar con la herramienta de HP, incluso con páginas de 10 × 10 cm. Su hipótesis es que se conserva mucha geometría oculta por máscaras de recorte. No se inspeccionó un PDF ni se midió ese flujo; no presentar esa causa como diagnóstico confirmado.

Actualización: se inspeccionó docs/ejemplo_export.pdf suministrado después. 158.52 MB, 250 páginas de aproximadamente 6 × 6 cm; cuatro páginas muestreadas contienen Forms invocados completamente fuera de página. Esto confirma geometría retenida en esa muestra, pero no mide ni explica por sí solo el coste Fiery. Evidencia y límites en analisis-ejemplo-export.md.

## Requisito para MosaicForge

La exportación final para impresión debe evitar arrastrar toda la plantilla detrás de un recorte pequeño. Conservar la plantilla original para reproducibilidad y ajustes; generar un artefacto final con la geometría necesaria para representar la página y su sangrado configurado.

- [x] ~~Implementar descarte/recorte real de rellenos y contornos de trazos en PDF, con curvas, transforms, fill-rule y clips; fixture de bordes y agujeros probado.~~
- [x] ~~Emitir solo grupos/pinturas útiles y recursos referenciados; auditoría pypdf de alcanzabilidad y uso pasa.~~
- [x] ~~Medir diferencia visual entre recorte y baseline, además de referencia resvg; umbrales de regresión a 300 DPI fijados con evidencia en avance-tarea-5.6.md.~~
- [x] ~~Medir bytes, objetos y puntos de contornos en baseline/optimizado del mismo diseño: 2284331→824466 bytes, seis objetos en ambos y 267606→96195 puntos.~~ No equiparar puntos poligonales con comandos Bézier SVG.
- [x] ~~Probar cajas físicas, corte de 100 mm y sangrado positivo de 3 mm; cero Image XObjects y comparación visual independiente.~~
- [ ] Comparar PDF con recorte simple frente a exportación optimizada: mismo contenido visible, tamaño, tiempo de generación y procesamiento RIP, RAM máxima y errores, por páginas y lotes.
- [ ] Registrar modelo/versión/configuración del Fiery, resolución y trabajo de prueba cuando se disponga de acceso. Sin ese entorno, declarar validación Fiery pendiente; no prometer aceleración.

Actualización 2026-10-15: comparación de peso/complejidad e imagen ya realizada en 5.6; proceso completo de prueba medido con PSAPI (74.62 MiB de pico residente). El punto de comparación completa de rendimiento/RIP permanece pendiente: falta perfil por etapa y Fiery real, no se publicó una aceleración.

No imponer ahora una meta de tamaño o velocidad sin baseline. Elegir objetivos medibles después de obtener muestras y ejecutar el perfil. Compresión y limpieza de recursos pueden formar parte del diseño, pero su beneficio debe medirse.

## Encaje en el plan

Actualización aprobada: PDF se incorpora explícitamente al MVP CPU como submódulo 5.6; un único PDF multipágina por trabajo con la totalidad de variantes generadas, una página por variante aceptada en orden de índice. Los grupos N son unidades de procesamiento/memoria, no archivos PDF separados. Rechazos se registran en manifiesto; no se inventan imágenes ni se publica un PDF vacío como éxito.

No confundir la máscara de validación CPU con optimización del artefacto final. El crop/exportador SVG actual conserva paths; el PDF implementado en 5.6 sí aplica recorte geométrico real y limpieza con precisión documentada en ADR 0005. Contrato agregado nuevo en contracts/engine/pdf/v1; hashes de variante/documento distintos. Escritura incremental/publicación/cancelación de CLI se integra con tarea 6. No prometer PDF/X, CMYK o prestaciones de Fiery sin diseño/pruebas; Fiery sigue pendiente.
