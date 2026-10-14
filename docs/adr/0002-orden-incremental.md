# ADR 0002 — Orden incremental y fronteras diferidas

Fecha: 2026-10-14. Estado: instrucción directa del usuario; plan actualizado pendiente de revisión.

## Cambio respecto al ADR 0001

El orden obligatorio es Rust CPU + CLI → frontend Next.js contra ese motor → Java y trabajos → CUDA donde se mida mejora → Python solo si las reglas de validez no bastan. No introducir servicios/lenguajes ni infraestructura antes de su fase. Las elecciones Java/PostgreSQL/Valkey permanecen aceptadas para la fase 3, no como dependencias de fase 1.

Priorizar seed/índice y restricciones de validez. Empezar con funciones/módulos sencillos; abstraer solo una frontera real o dos implementaciones. Un ADR nuevo justifica cada frontera entre lenguajes cuando se incorpore.

## CUDA y formatos

SVG y futura exportación PDF de impresión son vectoriales; no rasterizar por defecto. Perfilar el pipeline completo, incluida codificación/E/S; evaluar resvg/tiny-skia antes de GPU. Color por píxel, composición, miniaturas y hashes son candidatos, no promesas de aceleración. Registrar hardware, drivers, entradas, comandos y resultados.

Identidad GPU depende de backend/arquitectura/versiones fijadas, incluido driver. Paridad CPU/GPU conserva decisiones discretas; tolerancia de píxel medida antes de activar una etapa. No usar reducciones flotantes no deterministas para aceptación.

## Consecuencias

- El puente web → motor Rust de fase 2 necesita un ADR propio al diseñarlo; transporte HTTP local de previsualización es la propuesta mínima. No ejecutar subprocesos entre lenguajes sin justificación explícita.
- Python entra únicamente tras comprobar insuficiencia de validación basada en reglas.
- Plan por fases con criterios medibles en `../plan-desarrollo.md`.
- No pasar de fase 1 sin confirmación explícita del usuario.
- Tarea 1 ya iniciada sigue siendo válida. Cerrar su revisión/checks y guardar checkpoint; no adelantar otras fases.
- DuckDB se usará como herramienta de consulta para JSON/TXT/CSV grandes cuando aporte utilidad; no sustituye la persistencia acordada.
