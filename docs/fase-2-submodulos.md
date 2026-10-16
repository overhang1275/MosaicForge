# Fase 2 — Checklist y punto de reanudación

2026-10-15. Usuario confirmó comenzar fase 2 por submódulos. Base CPU: e99fa75.

- [x] ~~Explorar estado, contratos y funciones actuales del motor; sin app web existente.~~
- [x] ~~Comparar HTTP local, subproceso CLI y WebAssembly; preparar propuesta escrita.~~
- [x] ~~Usuario aprobó el diseño: superpowers/specs/2026-10-15-fase-2-web-design.md.~~
- [x] ~~Preparar plan ejecutable de 2.1: superpowers/plans/2026-10-15-fase-2-1-contratos.md.~~
- [x] ~~Usuario aprobó plan y ejecución directa en este chat.~~
- [x] ~~2.1: ADR, contratos y tipos generados Rust/TypeScript; 136 tests Rust y seis Node pasan, 52 fixtures compartidos. Revisión sin bloqueos P1/P2.~~
- [x] ~~2.2: transporte HTTP Rust y paridad con CLI; checkpoint 4c0f902.~~
- [x] ~~2.3: Next.js/TypeScript y BFF, build standalone y paridad real de descargas.~~
- [ ] 2.4: editor, galería y descarga SVG/PNG.
- [ ] 2.5: robustez y accesibilidad.
- [ ] 2.6: E2E real, perfil local, revisión y cierre.

Pausa antes de cada submódulo. Fase 2 y especificación aprobadas. 2.1 completo con commits 03c1323 y 5d9776d; evidencia en avance-fase-2.1.md. Usuario autorizó 2.2 y 2.3 juntos; ambos completos. Leer avance-fase-2.2-2.3.md. Esperar confirmación para 2.4; todavía no hay editor visual.

Graphify: intento de CLI impedido por PermissionError al leer .graphify-cli/graphify/__init__.py. Fallback de recorrido del JSON existente: 143 nodos, 248 relaciones; snapshot anterior a tarea 2, no actualizado. Nodos encontrados: contratos como frontera (AGENTS.md L60-L63), fase 2 frontend (docs/plan-desarrollo.md L53-L72), validación (tests de contratos). Ubicaciones del snapshot, no números actuales. Se contrastaron directamente generation.schema.json, core/build.rs y cpu/src/export.rs; sin atribuir relaciones nuevas al grafo antiguo.

Verificación actual: 143 tests Rust, siete BFF y seis Node de contratos; formato, lint, tipado, regeneración y build standalone pasan. Diez pruebas locales ignoradas no se repitieron. HTTP y BFF verificados con procesos reales en 2.2/2.3. UI espera 2.4. Los perfiles de fase 1 no son mediciones del transporte.

Skill solicitada: frontend-skill no existe con ese nombre en el catálogo curado. El usuario facilitó después openai/plugins, carpeta plugins/build-web-apps/skills/frontend-app-builder. Instalada mediante skill-installer en C:/Users/init/.codex/skills/frontend-app-builder; disponible en el siguiente turno. Es configuración local del asistente, no dependencia de MosaicForge.

Herramientas adicionales instaladas y agente temporal de diseño completado: herramientas-diseno.md y diseno/fase-2-direccion-visual.md. Solicitud nueva: crear ui-ux-architect reutilizable; trabajo separado del servidor/producto.

Pausa vigente: 2.4 autorizado e implementado parcialmente; cierre QA pendiente. Usuario solicitó parar por tokens. Ver avance-fase-2.4.md; continuar solo cuando lo pida.
