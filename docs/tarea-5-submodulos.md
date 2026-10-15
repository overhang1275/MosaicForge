# Tarea 5 — Exportación CPU por submódulos

Actualizado: 2026-10-15. Usuario autorizó tarea 5 dividida en submódulos y después añadió PDF vectorial multipágina como 5.6. 5.1–5.3 terminados; pedir autorización antes de cada siguiente. Mantener fase 1; sin web, Java o GPU todavía. PDF autorizado en el plan, no implementado ni autorizado a empezar ahora.

- [x] ~~**5.1 Validación y presupuesto base:** OutputSpec generado, validación común y plan base PNG con aritmética comprobada; sin reservar imágenes.~~
- [x] ~~**5.2 PNG CPU:** render_png seguro, dimensiones/fondo, temporales de grupos/clips, PNG decodificado y encuadre sin letterboxing.~~
- [x] ~~**5.3 SVG final:** dimensiones/fondo y encuadre cover compartido con PNG, vectores y seguridad conservados; límites de geometría oculta documentados en avance-tarea-5.3.md.~~
- [ ] **5.4 Integración:** exportar candidatos aceptados, calcular artifactHash de bytes finales y comprobar determinismo, rechazo y ausencia de artefactos para candidatos rechazados. E/S y publicación incremental siguen en tarea 6.
- [ ] **5.5 Validación SVG/PNG:** casos reales con test.svg, pruebas de integración y documentación de resultados. La revisión independiente final de toda tarea 5 se traslada al final de 5.6 para incluir PDF.
- [ ] **5.6 PDF vectorial multipágina y cierre:** un único PDF con todas las variantes generadas, una página por variante y orden por índice; tamaño físico/sangrado, recorte geométrico real y limpieza de recursos. Procesar por grupos sin acumular todas las páginas rasterizadas/SVG en RAM; integrar hashes/manifiesto y publicación final con CLI en tarea 6. Revisión independiente final, comparación visual y métricas de peso/complejidad; Fiery pendiente hasta tener acceso.

## Requisitos aprobados para 5.6

- Un solo archivo PDF por trabajo/lote completo, no un PDF por imagen ni por grupo N. Cada variante aceptada ocupa una página, en orden de índice global, sin duplicados u omisiones al cambiar N. Rechazos no son imágenes generadas: se registran por índice en manifiesto, sin inventar páginas válidas.
- Conservar contenido vectorial; la palabra «imágenes» del requisito no implica rasterizar páginas por defecto. PDF con tamaño físico y sangrado explícitos, preservando apariencia.
- Antes de implementar, diseñar contrato de salida de documento/lote y mapa índice→página: artifactHash de una variante no debe confundirse con hash del PDF completo. No añadir simplemente pdf al enum por variante sin resolver esta diferencia. Cualquier cambio incompatible de contrato requiere versión nueva y autorización según AGENTS.md.
- Tamaño físico/sangrado requieren definición explícita de unidades/contrato; no inferir centímetros desde los píxeles actuales ni prometer PDF/X/CMYK/perfiles de color sin diseño y validación.
- Aplicar requisitos-exportacion-impresion.md: descarte/recorte real de vectores fuera de página/sangrado, limpieza de definiciones sin uso y pruebas de equivalencia visual. Un MediaBox/CropBox o clip no demuestra esa optimización.
- Ensayar una y múltiples páginas, grupos 1/7/100 y grupo final incompleto, orden y conteos, rechazo intermedio, determinismo, ausencia de recursos externos, integridad del PDF y tamaño físico correcto. Medir bytes/objetos/segmentos y memoria de proceso por separado antes de anunciar mejoras.
- La CLI publicará un solo PDF final tras completarlo; cancelación/error no debe publicar un PDF truncado ni sobrescribir archivos existentes. Diseño/seguridad de escritura incremental y manifiesto se integra en tarea 6.
- Si no hay variantes aceptadas, devolver resultado explícito de trabajo sin páginas; no publicar un PDF de éxito vacío. Validación en Fiery requiere el equipo real y no bloquea declarar únicamente lo comprobado localmente.

## Diseño acotado de 5.1

Crear crate mosaicforge-cpu, dependiente del core local. No hay nueva frontera entre lenguajes ni librerías externas nuevas. Compartir validate_output entre validate_spec y el planificador; no construir un GenerationSpec ficticio para validar dimensiones de render.

Plan base por variante: cuatro tamaños RGBA (canvas, copia/scanlines/codificación y salida con margen de política) más 16 MiB de reserva fija. Es una política inicial conservadora para buffers conocidos, no un pico medido ni cota completa del rasterizador. Tope por worker: 512 MiB; presupuesto disponible explícito permite que el futuro caller descuente buffers en vuelo. Cada cálculo usa checked_mul/checked_add.

Antes de render real, 5.2 debe añadir/validar el coste de superficies aisladas, clips y temporales del rasterizador; 5.1 por sí solo no autoriza asumir que el render cabe. En resvg 0.47.0 las capas aisladas pueden usar superficies mayores que el viewport y coexistir por anidamiento: no extrapolar memoria total desde ancho × alto × 4.

## Criterios de terminado 5.1

- Salidas de 1–4096 px por eje y fondos transparent/RGB válidos; inválidos rechazados en frontera.
- Solicitudes PNG admitidas solo si el plan cabe en presupuesto disponible <=512 MiB; probar frontera exacta y un byte menos, formato incorrecto y dimensiones inválidas.
- No asignar pixmaps ni afirmar RAM/throughput medidos.
- Suite del workspace, formato y Clippy pasan; registrar evidencia y commit en docs/estado.md.

## Continuidad

5.1–5.3 terminados. Quedan tres submódulos: 5.4–5.6. 82 tests workspace, formato y Clippy pasan; prueba local aparte reproduce tres SVG y compara sus píxeles con PNG. No comenzar 5.4 sin confirmación. PDF multipágina aprobado solo como ampliación del plan; no está implementado. Consultar avance-tarea-5.3.md, requisitos-exportacion-impresion.md y analisis-ejemplo-export.md, sin reutilizar assets propietarios.
