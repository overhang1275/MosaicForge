# Avance 6.2 — generación CLI y publicación

2026-10-15. Autorización del usuario incluye 6.2 y 6.3; continuar con 6.3 tras este checkpoint.

- [x] ~~Pruebas rojas: generate no reconocido, tres fallos de proceso esperados.~~
- [x] ~~CLI consume GenerationSpec generado; valida contrato/hash y lectura JSON hasta 1 MiB antes de crear salidas.~~
- [x] ~~SVG/PNG idénticos byte por byte al exportador CPU para índices globales 9..11; hashes y resultados coinciden.~~
- [x] ~~Publicación exclusiva por hard link desde temporal sincronizado/cerrado; colisiones y carrera real entre dos hilos preservan exactamente un archivo completo.~~
- [x] ~~Fallo de escritura inyectado: ningún final parcial ni temporal restante. Rechazos válidos: éxito, sin archivos.~~
- [x] ~~fmt, Clippy con -D warnings y cargo test --workspace --locked --quiet: 109 tests pasan, nueve locales omitidos, cero fallos.~~
- [x] ~~Prueba CLI real sobre out/test-prepared.svg: un SVG y un PNG aceptados, exit 0; originales intactos.~~

Artefactos locales: out/cli-6.2-svg/variant-0.svg y out/cli-6.2-png/variant-0.png. SHA-256 respectivos b034fb2c67793a19fc7ba0d89ec575a693a738d83b8702d1fdd5ba22c7762c52 y 79a61dd36b58ec4934fa136d2604b4de9f329a8ec43f884261438d6f0e020966.

No se añade dependencia externa: CLI referencia el crate CPU existente. Graphify consultado con presupuesto 300 (7 de 41 nodos, truncado); snapshot antiguo, interfaces verificadas leyendo código actual.

Límites: archivos completos anteriores permanecen ante fallo posterior; no transacción de todo el trabajo. Se requiere filesystem con hard links. La operación no protege contra un tercero con permisos que altere deliberadamente los directorios/temporales, ni garantiza durabilidad de metadatos ante pérdida de energía. stdout es flujo de resultados locales; manifiesto persistente/Ctrl+C corresponden a 6.4. No se midió rendimiento ni RSS en este paso.
