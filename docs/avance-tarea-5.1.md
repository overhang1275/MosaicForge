# Submódulo 5.1 — Validación y presupuesto base PNG

Fecha: 2026-10-14. BASE 47737df. Usuario autorizó tarea 5 por submódulos. Detenerse antes de 5.2 y preguntar.

- [x] ~~Definir cinco submódulos en tarea-5-submodulos.md.~~
- [x] ~~Extraer validate_output y reutilizarlo desde validate_spec.~~
- [x] ~~Crear crate CPU local y planificador de reserva base.~~
- [x] ~~Pruebas de fronteras, formatos y presupuesto.~~
- [x] ~~Suite completa y checks: 70 tests, fmt y Clippy pasan.~~
- [x] ~~Checkpoint funcional 8013a89 registrado.~~

## Implementación

core::validate_output usa OutputSpec generado, conserva validación de dimensiones hasta 4096 por eje / 16777216 píxeles y fondo transparent/RGB. validate_spec lo llama; no se duplica modelo ni se añade una configuración ficticia de generación al renderer.

Nuevo crate mosaicforge-cpu, dependencia del core local. plan_png_buffers(output, available_bytes) solo calcula: exige PNG, valida salida, calcula RGBA con operaciones checked y comprueba reserva base contra presupuesto disponible y tope de 512 MiB. Ningún pixmap, PNG o buffer de imagen se asigna aquí.

Reserva base = 4 × (width × height × 4 bytes) + 16 MiB. A 4096 × 4096: RGBA 64 MiB, reserva 272 MiB; descontarla de 512 MiB deja 240 MiB, insuficiente para una segunda reserva igual. Es prueba aritmética de presupuesto, no prueba de concurrencia real o medida de RAM.

Ruling: separar reserva base de temporales resvg — permite cerrar validación sin fingir que ancho × alto controla toda la memoria. Coste si se interpreta como cota completa: podría infravalorarse el render con grupos/clips; 5.2 debe contabilizar esos temporales antes de habilitar render. Política de cuatro RGBA + 16 MiB no es un pico medido ni límite RSS. No elevarla a garantía de seguridad global.

Se leyó resvg 0.47.0 local, src/render.rs: las capas aisladas crean pixmaps propios y pueden coexistir por anidamiento; clips añaden trabajo. No introducir todavía un renderer que use solo esta reserva base como si fuese presupuesto total.

## Evidencia

RED: tres tests del crate CPU, dos fallan con stub; prueba de rechazo pasa porque el stub rechaza todo. GREEN: tres pasan tras implementar. Fronteras: 1/4096 por eje, 4097/u64 máximo, fondo y cero inválidos al deserializar; formato SVG rechazado en planificador PNG; presupuesto exacto admitido y un byte menos rechazado; cero y más de 512 MiB rechazados.

Sin dependencias externas nuevas. Cargo.lock solo registra el crate local y dependencias ya existentes. Revisión local del diff confirma que extraer validate_output mantiene las validaciones anteriores. Revisión independiente global de tarea 5 prevista en 5.5, no en cada módulo pequeño.

Graphify query con budget 350 encontró 42 nodos, mostró ocho y truncó el resto; snapshot antiguo. Referencias de OutputSpec/buffers contrastadas con código y plan actuales directamente. Sin reconstruir grafo para este paso pequeño.

## Verificación final

Desde services/engine, Cargo en PATH: cargo fmt --all -- --check; cargo clippy --workspace --all-targets --locked -- -D warnings; cargo test --workspace --locked --quiet. Ejecutados con exit 0: 70 tests pasados (67 core + tres cpu), cero fallos; tres tests locales omitidos deliberadamente. Sin repetir pruebas locales de imágenes. No se añadieron librerías externas. Revisión local y git diff --check sin problemas pendientes.

## Pendiente de 5.2

5.2: render real, validación SVG, temporales, fondo/dimensiones y PNG decodificado. No se ejecutó raster de salida ni se midieron RAM/throughput en 5.1. Los tests locales de arte no se repiten porque aquí no se renderiza. PDF y limpieza vectorial siguen en su alcance documentado.
