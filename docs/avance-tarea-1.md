# Registro de ejecución — tarea 1: contratos CPU

Plan: `superpowers/plans/2026-10-14-motor-cpu.md`. Diseño: `diseno-mvp.md`.

Fecha: 2026-10-14. BASE: `1c8f5ed`. Rama: `feat/cpu-contracts`.

- [x] ~~Usuario confirmó iniciar la tarea 1.~~
- [x] ~~Crear contratos generation/result v1, fixture identidad y pruebas.~~
- [x] ~~Comprobar RED por validación semántica y hash faltantes.~~
- [x] ~~Implementar generación de tipos y validación completa de esta frontera.~~
- [x] ~~Comprobar GREEN, formato y Clippy.~~
- [x] ~~Documentar versiones/licencias transitivas y hacer commit.~~

## Resoluciones

- Ruling: ejecutar solamente tarea 1 — es el alcance explícito del usuario en esta solicitud; no extender automáticamente a las tareas 2–6.
- Ruling: trabajar en rama dedicada en el checkout actual — conserva documentos y contexto compartido, evita modificar main; no se creó worktree adicional ni se pidió otra autorización para una acción reversible.
- Ruling: el registro persistente está en docs/, como pidió el usuario — no depender de scratch ignorado que se pierda al cambiar de sesión.
- Ruling: crop se valida como rectángulo positivo en esta tarea; su pertenencia al viewBox se valida al importar/generar la plantilla en tareas 3–4 porque GenerationSpec no contiene el viewBox.
- Ruling: count/startIndex no participan en configurationHash — no deben modificar los hashes de variantes al reanudar un lote. Seed, versiones, plantilla, reglas y output sí participan.
- Ruling: una palette vacía equivale a omitir la paleta — typify representa esa propiedad opcional como Vec con default vacío; declarar minItems 0 evita desacuerdo entre schema y tipos. No impide limitar a 256 colores.
- El usuario indicó usar DuckDB para consultar JSON/TXT/CSV grandes; aplicar filtros/agregaciones cuando su tamaño lo justifique. No añadir DuckDB al dominio ni instalarlo para estos contratos pequeños.
- Ruling: opcionales crop/artifactHash/reason admiten null explícitamente y se normalizan a ausencia; mantener coherencia entre schema y serde/typify.
- Ruling: añadir validate_result obligatorio para metadata — tipos generados no aplican todos los límites semánticos; protege seed/índice u64, hashes y longitudes de resultado.
- Cambio de orden por usuario documentado en ADR 0002: web antes de Java; Python solo si reglas insuficientes. No pasar de fase 1 sin confirmación.

## Preflight de interfaces

| Tareas | Relación | Resultado |
|---|---|---|
| 1 → 2 | seed/startIndex como decimal u64 | Parsear sin pérdida de precisión antes del RNG |
| 1 → 3–4 | hash de plantilla, reglas y output | Tipos generados exportados desde core; sin modelos públicos manuales |
| 1 → 5 | OutputSpec | Exportar el tipo generado para reutilizarlo en CPU |
| 1 → 6 | validación y hash | La CLI deberá validar antes de importar/generar; hash independiente del bloque |

## Evidencia

RED comprobado: 5 tests, 3 pasan, 2 fallan por causas previstas: seed superior a u64 aceptada por el stub y configurationHash vacío. Tipos generados compilaron correctamente. Duración de primera compilación: 52.44 s. No marcar tarea terminada hasta leer resultados reales de tests/fmt/clippy.

GREEN final: diez tests pasan, cero fallos/omitidos. fmt --check y Clippy --locked -D warnings pasan. Revisión independiente encontró dos P2 y fueron corregidos con pruebas; seguimiento sin hallazgos materiales. Commit funcional 2f1e6d4. No se midió rendimiento ni se implementaron tareas posteriores.
