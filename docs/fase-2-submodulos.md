# Fase 2 — Checklist y punto de reanudación

2026-10-15. Usuario confirmó comenzar fase 2 por submódulos. Base CPU: e99fa75.

- [x] ~~Explorar estado, contratos y funciones actuales del motor; sin app web existente.~~
- [x] ~~Comparar HTTP local, subproceso CLI y WebAssembly; preparar propuesta escrita.~~
- [x] ~~Usuario aprobó el diseño: superpowers/specs/2026-10-15-fase-2-web-design.md.~~
- [x] ~~Preparar plan ejecutable de 2.1: superpowers/plans/2026-10-15-fase-2-1-contratos.md.~~
- [ ] Revisar plan y confirmar método de ejecución; se recomienda ejecución directa en este chat.
- [ ] 2.1: plan ejecutable, ADR de frontera y contratos/tipos/fixtures verificados.
- [ ] 2.2: transporte HTTP Rust y paridad con CLI.
- [ ] 2.3: Next.js/TypeScript y BFF.
- [ ] 2.4: editor, galería y descarga SVG/PNG.
- [ ] 2.5: robustez y accesibilidad.
- [ ] 2.6: E2E real, perfil local, revisión y cierre.

Pausa antes de cada submódulo. Fase 2 y especificación aprobadas. Plan ejecutable de 2.1 escrito y auto-revisado; pendiente revisión del usuario y método de ejecución según writing-plans. Sin código de producto ni dependencias nuevas en el repositorio.

Graphify: intento de CLI impedido por PermissionError al leer .graphify-cli/graphify/__init__.py. Fallback de recorrido del JSON existente: 143 nodos, 248 relaciones; snapshot anterior a tarea 2, no actualizado. Nodos encontrados: contratos como frontera (AGENTS.md L60-L63), fase 2 frontend (docs/plan-desarrollo.md L53-L72), validación (tests de contratos). Ubicaciones del snapshot, no números actuales. Se contrastaron directamente generation.schema.json, core/build.rs y cpu/src/export.rs; sin atribuir relaciones nuevas al grafo antiguo.

Verificación de este paso: revisión documental de coherencia y git diff --check. No se ejecutaron tests de producto: ningún código cambió. Los 131 tests y métricas de fase 1 pertenecen al checkpoint e99fa75, no al transporte propuesto.

Skill solicitada: frontend-skill no existe con ese nombre en el catálogo curado. El usuario facilitó después openai/plugins, carpeta plugins/build-web-apps/skills/frontend-app-builder. Instalada mediante skill-installer en C:/Users/init/.codex/skills/frontend-app-builder; disponible en el siguiente turno. Es configuración local del asistente, no dependencia de MosaicForge.
