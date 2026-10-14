# Registro de ejecución — tarea 2: PRNG por contador

Plan: superpowers/plans/2026-10-14-motor-cpu.md. BASE: 2f43459. Rama: feat/cpu-contracts.

- [x] ~~Terminar Graphify antes de iniciar tarea 2, como pidió el usuario.~~
- [x] ~~Consultar directamente tres vectores de referencia Random123.~~
- [x] ~~Escribir pruebas y observar RED.~~
- [x] ~~Implementar Philox, dominio seed/índice y conversión a intervalos.~~
- [x] ~~Comprobar GREEN, formato, Clippy y revisión.~~
- [x] ~~Guardar commit funcional y registrar hash.~~

## Evidencia

RED: siete tests RNG; cuatro fallan con stub vacío (vectores, dominio, intentos y rangos). Tres tests de invariancia/composición pasan con stub constante, por lo que los vectores de referencia y separación de dominios aportan la verificación independiente que esos tests solos no dan.

GREEN: siete tests RNG y diez tests de contrato pasan (17 en total, cero fallos/omitidos). Formato y Clippy --locked -D warnings pasan, exit code 0. Revisión independiente por lectura sin hallazgos materiales. Commit funcional 30ac241. Sin mediciones de rendimiento o paridad GPU. Sin nuevas dependencias.

## Resoluciones

- Ejecución limitada a tarea 2 explícitamente autorizada. Tarea 3 pendiente; no pasar de fase 1 sin confirmación.
- Mapeo de u32 a intervalos con endpoints ponderados para evitar overflow al restar extremos f64 opuestos; fórmula fijada en ADR 0003.
- No instalar Rayon aún: pruebas de muestreo multihilo usan std::thread; el procesador de batches se implementa en tarea 6.
- El grafo actual refleja el snapshot anterior a tarea 2; no presentarlo como un mapa automáticamente actualizado del código nuevo.
