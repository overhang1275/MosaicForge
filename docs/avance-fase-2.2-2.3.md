# Avance 2.2–2.3

Plan: superpowers/plans/2026-10-15-fase-2-2-3-http-bff.md. Base e9454fc. Usuario autorizó ambos submódulos juntos.

- [x] ~~Leer especificación, ADR y contratos existentes.~~
- [x] ~~2.2 HTTP y paridad CLI.~~
- [x] ~~2.3 BFF, build y prueba real.~~
- [x] ~~Revisión y checkpoints finales.~~

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

## 2.3 — evidencia y revisión

RED: cinco tests del proxy fallaron inicialmente (stub devolvía 501). GREEN: siete pruebas BFF pasan con upstream HTTP local real; cubren configuración/destino, Origin/Host, JSON, streaming, correlación, u64, descarga, errores adversos, timeout y desconexión. Seis tests de contratos y 52 fixtures pasan usando los validadores standalone nuevos; check de regeneración, formato, ESLint y tipado pasan.

Build Next.js standalone con webpack y copia de recursos estáticos pasa. install --frozen-lockfile --ignore-scripts pasa. TypeScript 7.0.2 fue rechazado por typescript-eslint (error reproducido); app fijada a 6.0.3, manteniendo TypeScript de las herramientas existentes.

Ruling: validar Host HTTP literal en BFF, junto a Origin exacto — NextURL normaliza 127.0.0.1 a localhost y fallaba el primer smoke real con 403; se verificó el código instalado de Next.js, se reprodujo RED de Host ajeno y URL normalizada, y ambos pasan tras corregir. No se aceptan destinos DNS localhost para upstream.

Revisión independiente review_http_bff: un P2, recursos estáticos ausentes del paquete standalone. Reproducido en out/standalone-assets-red.log con 404; corregido mediante package-standalone.mjs y smoke que solicita un script del HTML. Sin otros P1/P2 concretos. Reviewer ejecutó siete BFF y doce preview; no modificó archivos.

Final: Ruling: editor 2.4, RSS/rendimiento bajo carga y paridad multiplataforma excluidos del juicio de revisión — son trabajo posterior/no medido, no se anuncia cobertura de esos ámbitos. Documentación final se completó tras revisar; criterios funcionales contrastados con pruebas y guía de uso.

Prueba final con launcher Rust + Next standalone + CLI: out/preview-smoke-3d9f4b32-98df-4f56-94af-a39b9ada4e3e/result.json. HTML y su script 200; inspección BFF 200; Origin ajeno 403; cuerpo BFF chunked de 33 MiB 413; bytes/hash de descargas SVG/PNG idénticos a CLI y HTTP (mismos valores de tabla). Variante rechazada sin bytes. Token aleatorio solo en memoria/entorno; búsqueda en .next/static no encontró nombre de variable privada ni secretos de fixture. Launcher y wrappers fuerzan loopback; smoke cierra procesos propios.

Licencias declaradas: dependencias-http-web.json, 129 paquetes Rust y 333 paquetes web instalados, ninguno sin licencia declarada. Hay licencias copyleft y permisivas; no afirmar que todas son MIT ni que esto sea auditoría legal. cargo metadata offline inicial intentó resolver dependencias de otra plataforma; se limitó correctamente a x86_64-pc-windows-msvc, sin descargar esas dependencias.

No se modificaron algoritmos CPU, contratos engine publicados, LICENSE ni CI. Sin Java, CUDA, Redis/Valkey, Python/ML, persistencia o landing. No se ejecutó QA visual del editor ni E2E de navegador, ni carga masiva/medición RSS. El umbral de respuesta Rust 16 MiB está implementado pero el motor real de la fixture produce archivos pequeños; la respuesta upstream excesiva sí fue ejercitada en pruebas BFF.

Siguiente: esperar autorización para 2.4 (editor/galería con DESIGN.md vigente).

Comprobación adicional de cierre: out/preview-smoke-36da1600-87fa-4ee7-a40d-ebab4f5254cf/result.json, mismo smoke con aserciones de salida efectiva de ambos procesos; exit 0. typecheck genera tipos Next.js antes de tsc para no depender de un build previo.
