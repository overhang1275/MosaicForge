# ADR 0003 — Dominio Philox y muestreo independiente del lote

Fecha: 2026-10-14. Estado: implementación de tarea 2; verificaciones registradas en avance-tarea-2.md.

## Decisión

Usar Philox4x32-10 con aritmética entera definida: productos u32 se calculan en u64; el aumento de clave se hace módulo 2^32. Sin RNG global, reloj, GPU, FFI ni dependencia nueva.

Dominio de sample_u32(seed: u64, index: u64, rule: u16, attempt: u8, draw: u32):

- key = [seed_low32, seed_high32].
- counter = [index_low32, index_high32, draw / 4, rule | (attempt << 16)].
- lane = draw % 4.
- attempt admite 0–7; fuera de rango se devuelve invalid_spec.
- Se admite todo el rango u64 de seed/índice y u32 de draw, sin sumar ni truncar el índice global.

Rule ID permanece estable aunque cambie el orden de configuración. Cada regla nueva debe asignar un ID explícito sin reutilizar el de otra regla de la misma versión. La distribución de IDs se fija al implementar reglas (tarea 4). No añadir batch size, número de hilos o startIndex al dominio. Los reintentos de una variante no consumen la secuencia de ninguna otra.

Este mapeo pertenece a rulesVersion 1; cambiarlo exige una versión nueva. Philox se usa para variación reproducible, no para credenciales o secretos. No se promete que diferentes inputs nunca colisionen en una sola salida u32.

Submódulo 4.1 congela IDs geométricos: translateX=1, translateY=2, rotationDegrees=3, scaleX=4, scaleY=5; draw=0 por campo. MirrorX/Y se conservan como flags explícitos sin PRNG. Estos valores forman parte de rulesVersion 1; futuras reglas usarán IDs nuevos sin desplazar los existentes.

## Conversión a intervalos

map_u32_to_range(value, min, max) exige endpoints finitos y ordenados. u = value / 2^32 se representa exactamente en f64 y está en [0,1). Para endpoints diferentes, usar (1-u)*min + u*max, sin FMA explícita; evita overflow de max-min cuando los extremos son opuestos. min==max devuelve min. El resultado se limita a [min,max] para proteger redondeos; por redondeo puede alcanzar max. Un resultado no finito es error.

No usar esta conversión flotante como evidencia de paridad CUDA todavía. Las decisiones discretas deben usar enteros u32 y reglas comunes; la compatibilidad de render CPU/GPU se prueba más adelante con tolerancia medida y entorno fijado.

## Referencias y pruebas

Se consultaron directamente [vectores Random123](https://raw.githubusercontent.com/DEShawResearch/random123/main/tests/kat_vectors) (líneas 27–29 en vista numerada, 2026-10-14) y [definición pública de Philox](https://raw.githubusercontent.com/DEShawResearch/random123/main/include/Random123/philox.h). Tres fixtures: cero, todos los bits activos y dígitos de pi. No se copió código de curand ni material de HP.

Los tests comprueban vectores independientes, separación de dominio, las cuatro lanes, límites de intentos, índices extremos, reordenamiento, rangos aislados y grupos 1/7/100 sobre 201 índices con un último grupo parcial. Esto verifica el muestreo, no implementa todavía el procesamiento/exportación de batches de la CLI.
