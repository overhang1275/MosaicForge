# Exportación para impresión: geometría útil y coste de RIP

Registrado: 2026-10-14. Requisito aportado por el usuario durante la pausa; no autoriza reanudar implementación ni adelantar PDF.

## Observación de origen

El usuario observa PDFs pesados y procesamiento lento en Fiery al exportar con la herramienta de HP, incluso con páginas de 10 × 10 cm. Su hipótesis es que se conserva mucha geometría oculta por máscaras de recorte. No se inspeccionó un PDF ni se midió ese flujo; no presentar esa causa como diagnóstico confirmado.

Actualización: se inspeccionó docs/ejemplo_export.pdf suministrado después. 158.52 MB, 250 páginas de aproximadamente 6 × 6 cm; cuatro páginas muestreadas contienen Forms invocados completamente fuera de página. Esto confirma geometría retenida en esa muestra, pero no mide ni explica por sí solo el coste Fiery. Evidencia y límites en analisis-ejemplo-export.md.

## Requisito para MosaicForge

La exportación final para impresión debe evitar arrastrar toda la plantilla detrás de un recorte pequeño. Conservar la plantilla original para reproducibilidad y ajustes; generar un artefacto final con la geometría necesaria para representar la página y su sangrado configurado.

- [ ] Diseñar descarte de objetos completamente exteriores y recorte geométrico de los que cruzan el límite. Comprobar resultado visible, trazos, uniones, curvas, transformaciones, reglas de relleno y clips internos; no eliminar objetos solo por sus puntos de control o por el centro de su bounding box.
- [ ] Retirar definiciones y recursos sin referencias después de optimizar; conservar los recortes que aún sean necesarios para reproducir el resultado.
- [ ] Probar que el recorte real no cambia el resultado visible, con diferencias medidas; añadir casos de trazos que cruzan el borde, agujeros y clips anidados. La tolerancia debe fijarse con evidencia.
- [ ] Medir bytes, objetos, segmentos y recursos antes/después sobre un mismo diseño. Una página de dimensiones pequeñas no es por sí sola un criterio de complejidad vectorial.
- [ ] Al implementar PDF, probar tamaño físico, cajas de página/sangrado y requisitos de impresión definidos para ese flujo. No rasterizar por defecto para reducir tamaño.
- [ ] Comparar PDF con recorte simple frente a exportación optimizada: mismo contenido visible, tamaño, tiempo de generación y procesamiento RIP, RAM máxima y errores, por páginas y lotes.
- [ ] Registrar modelo/versión/configuración del Fiery, resolución y trabajo de prueba cuando se disponga de acceso. Sin ese entorno, declarar validación Fiery pendiente; no prometer aceleración.

No imponer ahora una meta de tamaño o velocidad sin baseline. Elegir objetivos medibles después de obtener muestras y ejecutar el perfil. Compresión y limpieza de recursos pueden formar parte del diseño, pero su beneficio debe medirse.

## Encaje en el plan

Actualización aprobada: PDF se incorpora explícitamente al MVP CPU como submódulo 5.6; un único PDF multipágina por trabajo con la totalidad de variantes generadas, una página por variante aceptada en orden de índice. Los grupos N son unidades de procesamiento/memoria, no archivos PDF separados. Rechazos se registran en manifiesto; no se inventan imágenes ni se publica un PDF vacío como éxito.

No confundir la máscara de validación CPU con optimización del artefacto final. El crop actual cambia el viewBox y conserva paths: todavía no cumple el recorte geométrico final aquí solicitado. Antes de implementar 5.6, diseñar unidades físicas/sangrado y contrato de documento agregado, sin confundir hashes por variante con el del PDF completo. Escritura incremental/publicación/cancelación se integra con tarea 6. No prometer PDF/X, CMYK o prestaciones de Fiery sin diseño/pruebas; Fiery seguirá pendiente si no hay equipo disponible.
