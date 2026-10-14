# Graphify — Snapshot del proyecto

Fecha: 2026-10-14. Snapshot tomado después de tarea 1 y antes de implementar tarea 2. No es un grafo actualizado automáticamente al modificar el código.

- [x] ~~Detectar corpus y excluir dependencias descargadas, target, .git y graphify-out.~~
- [x] ~~Extraer AST y documentación, construir comunidades y diagnosticar integridad.~~
- [x] ~~Exportar HTML, JSON, informe y manifest/cache de actualización.~~
- [x] ~~Verificar existencia de los artefactos y limpiar sidecars temporales.~~

## Resultados

Corpus: 23 archivos, 14,594 palabras estimadas por el detector; 10 código y 13 documentación. No se omitieron archivos sensibles. Sin PDFs/imágenes/video.

AST: 33 nodos, 78 relaciones. Semántica: 99 nodos, 175 relaciones y tres hyperedges. Grafo exportado: 143 nodos, 248 relaciones, diez comunidades (incluye referencias materializadas por el constructor).

Outputs locales excluidos de Git:

- ../graphify-out/graph.html: visualización interactiva, 149,502 bytes al verificar.
- ../graphify-out/graph.json: grafo, 138,878 bytes al verificar.
- ../graphify-out/GRAPH_REPORT.md: informe, 6,793 bytes al verificar.
- ../graphify-out/diagnostics.json: diagnóstico de extracción.

## Hallazgos e interpretación

- identity() es el nodo AST más conectado por servir como fixture de pruebas; no significa que sea una abstracción de negocio central.
- Los tipos desde JSON Schema enlazan contratos, generación build, licencias y planificación del motor. El grafo conecta evidencia implementada con requisitos, sin convertir los requisitos en código existente.
- Reproducibilidad por variante conecta con rangos/hilos/grupos independientes; versiones de contratos y dominio Philox tienen una relación conceptual inferida, etiquetada como tal.
- Los nodos semánticos distinguen status implementado/planeado. Una comunidad amplia de documentación no justifica por sí misma dividir módulos de código.

## Límites de integridad y costes

El diagnóstico de la extracción original encuentra 18 extremos sin nodo (imports/reexports y dependencias) y una pareja colapsada al construir el grafo no dirigido. Algunos se materializan como referencias en el grafo final. No se inventó implementación para cubrirlos. contract.rs, lib.rs y Cargo.toml no aportaron símbolos AST propios. El grafo sirve para navegar, pero no prueba que las referencias sean completas ni sustituye compilación/tests.

Tokens de extracción semántica: 22,664 entrada y 29,661 salida estimados por caracteres/4; no disponibles como telemetría real en esta herramienta. El reporte/cost.json los etiqueta como estimaciones. No se usó un proveedor LLM externo con una clave de API.

Benchmark Graphify de navegación: corpus considerado por ese benchmark 7,150 palabras (~9,533 tokens), consulta media ~1,631 tokens, reducción estimada 5.8x. Usa un conteo distinto al detector general; no son tiempos reales, consumo medido de sesión ni aceleración del motor MosaicForge.

## Continuidad

La tarea 2 fue autorizada para comenzar después de terminar este grafo. Sus nuevas funciones/tests no pertenecen al snapshot; actualizar el grafo al volver a consultar relaciones nuevas. Para usar la instalación local, establecer PYTHONPATH a .graphify-cli y ejecutar el Python bundled; .graphify_python registra su ruta. No instalar una nueva dependencia por esta sesión.
