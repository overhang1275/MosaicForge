# Avance 2.2–2.3

Plan: superpowers/plans/2026-10-15-fase-2-2-3-http-bff.md. Base e9454fc. Usuario autorizó ambos submódulos juntos.

- [x] ~~Leer especificación, ADR y contratos existentes.~~
- [x] ~~2.2 HTTP y paridad CLI.~~
- [ ] 2.3 BFF, build y prueba real.
- [ ] Revisión y checkpoints finales.

Pre-flight: Task 1 produce envelopes preview v1 y Task 2 consume los mismos schemas/tipos. Sin cambio de contrato; requestId generado por BFF y devuelto por motor. Export cambia JSON del motor a attachment solo en BFF.

Ruling: mantener checkout feat/cpu-contracts — continuidad explícita del trabajo ya aprobado, sin crear otro checkout ni cambiar de rama — los checkpoints delimitan los cambios.

Ruling: scripts Bash del ejecutor pueden requerir adaptación Windows; conservar ledger permanente en docs y evidencias largas en out, sin borrar artefactos ajenos.

Graphify: snapshot anterior a 2.1; no es autoridad sobre módulos nuevos. La relación contrato→preview/core/cpu se comprueba en fuentes actuales. No reconstruir el grafo en este alcance.

## 2.2 — evidencia

RED: cinco pruebas HTTP fallaron con 404 antes de registrar endpoints (out/http-red.log). Tras implementación se ajustaron expectativas de prueba: JSON f64 frente a enteros, fixture con cobertura insuficiente y namespace SVG obligatorio; no se alteraron algoritmos para aceptar dibujos inválidos.

GREEN: 143 tests Rust pasan, diez locales ignorados; fmt y Clippy --workspace --all-targets --locked --offline -- -D warnings pasan. Preview añade siete pruebas (cinco HTTP y dos de permiso); doce en el crate contando contratos existentes. Baseline anterior: 136 tests pasan.

Prueba con procesos reales: out/preview-smoke-29fb360c-e646-4da7-8359-07d916111c42/result.json. Inspección, 429 mientras cuerpo chunked está abierto, liberación al desconectar, cuerpo chunked de 33 MiB rechazado 413 y rechazo de variante sin bytes. SVG/PNG HTTP = CLI real con seed e índice 18446744073709551615, salida 128².

| Formato | Bytes | SHA-256 |
|---|---:|---|
| SVG | 181 | d35a55134cd6bb40af7a59680807cbeba5c0ebddb71efc8c41bf7c27a7e877a2 |
| PNG | 375 | 8bc15ccb596a4958b563806476fb42ef8113f1618ed5aacb4489fd89854c3dab |

Ruling: errores antes del parseo usan UUID cero — no se consume un cuerpo potencialmente enorme para extraer correlación; BFF publica su propio requestId — clientes directos deben reconocer que aún no se conoce el sobre.

Ruling: herramientas Bash de Superpowers no pudieron resolver basename incluso con PATH de Git; se conserva el mismo plan/ledger y pruebas ejecutadas directamente en PowerShell. Sin perder ni borrar workspace de otras tareas.
