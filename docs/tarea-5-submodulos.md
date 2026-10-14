# Tarea 5 — Exportación CPU por submódulos

Fecha: 2026-10-14. Usuario autorizó tarea 5 dividida en submódulos. 5.1–5.2 terminados; pedir autorización antes de cada siguiente, como en tarea 4. Mantener fase 1; sin CLI, web, Java, GPU o PDF todavía.

- [x] ~~**5.1 Validación y presupuesto base:** OutputSpec generado, validación común y plan base PNG con aritmética comprobada; sin reservar imágenes.~~
- [x] ~~**5.2 PNG CPU:** render_png seguro, dimensiones/fondo, temporales de grupos/clips, PNG decodificado y encuadre sin letterboxing.~~
- [ ] **5.3 SVG final:** aplicar dimensiones/fondo y política de encuadre al SVG exportado, manteniendo vectores y seguridad. Registrar límites de geometría oculta; no declarar optimización geométrica/PDF implementada si solo hay viewBox/clip.
- [ ] **5.4 Integración:** exportar candidatos aceptados, calcular artifactHash de bytes finales y comprobar determinismo, rechazo y ausencia de artefactos para candidatos rechazados. E/S y publicación incremental siguen en tarea 6.
- [ ] **5.5 Cierre:** pruebas con test.svg, revisión independiente de toda tarea 5, correcciones verificadas, documentación y checkpoint antes de CLI.

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

5.1–5.2 terminados: 78 tests workspace, formato y Clippy pasan. Prueba local CPU aparte produce/repite tres PNG; revisión visual realizada. Evidencia en avance-tarea-5.2.md. No comenzar 5.3 sin confirmación. Tarea 4 ya completada; no repetirla. Consultar requisitos-exportacion-impresion.md y analisis-ejemplo-export.md al diseñar exportación, sin reutilizar assets propietarios.
