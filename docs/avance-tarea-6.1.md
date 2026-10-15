# Submódulo 6.1 — Binario CLI e inspect

Fecha: 2026-10-15. Base 3c7139c. Usuario inicia tarea 6; Fiery queda fuera. Plan/checklist en tarea-6-submodulos.md.

- [x] ~~Tests RED del binario y frontera de archivos/argumentos.~~
- [x] ~~Implementar inspect con JSON estable e importación acotada.~~
- [x] ~~Verificar errores, rutas y checks del workspace.~~
- [x] ~~Inspección real y documentación de ejecución.~~
- [ ] Commit y pausa antes de 6.2.

## Evidencia

RED: cuatro pruebas de proceso fallan con binario stub (ayuda, metadatos, errores de archivo y rechazo SVG). GREEN dirigido: cuatro tests pasan, más dos unitarios de lectura acotada y fallo de stdout. Se añade quinto test de proceso para el límite exacto de 5 MiB: válido y mismo hash normalizado que la entrada corta. La lectura de un reader sin EOF se detiene exactamente en 5 MiB + un byte.

clap 4.6.7 con features mínimas resuelve argumentos/rutas OS; cuatro paquetes nuevos MIT/Apache, 88 externos en lock. Se comparte core::template::MAX_SVG_BYTES con el importador en vez de duplicar el límite. No cambia la semántica de SVG/contratos ni se añaden dependencias CPU al inspector.

Prueba real: inspect out/test-prepared.svg exit 0, JSON estable de dimensiones 144 × 144, viewBox [0,0,144,144], capa Page_1, 22239 segmentos y hash e8cfd87f3c8684427494222a03db48bbc8f2bfa1e5c6170ede7a556046474cac. Ayuda exit 0. Sin modificar archivos ni devolver SVG fuente. Uso en ejecucion-motor.md.

Fuentes primarias del parser: https://docs.rs/clap/latest/clap/struct.Command.html (consultado 2026-10-15). Graphify encontró cuatro nodos, sin truncado, presupuesto 300; snapshot antiguo contrastado con código actual. Revisión local de este paso pequeño; revisión independiente global prevista al cerrar 6.6.

Verificación final con exit 0: cargo fmt --all -- --check; cargo clippy --workspace --all-targets --locked -- -D warnings; cargo test --workspace --locked --quiet. **102 tests normales** (73 core + 22 cpu + siete CLI), cero fallos; nueve tests locales omitidos por defecto y no repetidos en este paso de inspector. Cargo emitió el aviso conocido de canonicalización de C:\Users\init. Sin benchmark/RSS nuevo, GPU o generación por CLI ejecutados. Fiery excluido.

La extracción del límite MAX_SVG_BYTES es una reutilización de la misma regla existente, sin ampliar el subconjunto ni cambiar su valor. Revisión local: el parser rechaza flags duplicados/desconocidos, las rutas se conservan como PathBuf, la lectura no confía en tamaño estático y los errores no incorporan el contenido fuente. Pruebas de proceso usan directorios temporales propios y limpieza no recursiva.
