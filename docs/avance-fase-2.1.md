# Avance 2.1 — Contratos de preview

Plan: docs/superpowers/plans/2026-10-15-fase-2-1-contratos.md. Base ec86514. Usuario aprobó plan y ejecución directa en este chat.

- [x] ~~Leer plan, spec, reglas y detectar rama/checkout.~~
- [ ] A: ADR y contrato documentado, checkpoint.
- [ ] B: esquemas, tipos y validación Rust con RED/GREEN.
- [ ] C: tipos TypeScript, fixtures compartidos y regeneración comprobada.
- [ ] Revisión, checks finales, commits y pausa antes de 2.2.

Pre-flight: A define los nombres de envelopes que consumen B/C; coinciden con spec. B/C comparten fixtures y deberán comprobar las mismas reglas semánticas; el servidor conserva además la validación real del motor. Generación de tipos no se confunde con validación.

Ruling: continuar en el checkout/ramal de trabajo actual feat/cpu-contracts — el usuario eligió ejecución directa aquí y pide continuidad/commits recuperables; no crear otro checkout ni mover artefactos — coste si se necesita aislamiento adicional: crear un worktree antes del siguiente submódulo.

Ruling: este documento en docs/ es el ledger persistente — instrucción explícita del usuario sobre continuidad en docs/ prevalece sobre scratch temporal de Superpowers — coste: conservar una página documental adicional.

Ruling: validación Node del sobre incluye reglas semánticas de contrato para comprobar paridad; no implementa transformación, rasterización ni decisiones generativas — es la frontera real de mensajes, no otro motor — coste: mantener pruebas cruzadas al cambiar contratos.

Instalación frontend-app-builder ya realizada en la configuración local del asistente; no es dependencia del producto. No se usa para contratos: se aplicará al diseñar/implementar UI.
