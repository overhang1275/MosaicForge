# MosaicForge — Estado y punto de reanudación

Última actualización: 2026-10-14. Rama: feat/cpu-contracts.

## Leer al retomar

1. ../AGENTS.md.
2. Este archivo.
3. plan-desarrollo.md (orden vigente actualizado).
4. adr/0002-orden-incremental.md y diseno-mvp.md.
5. superpowers/plans/2026-10-14-motor-cpu.md y avance-tarea-1.md.

## Instrucciones vigentes del usuario

- Checklist visible, marcar y tachar solo lo completado.
- Planes, documentación y registros de continuidad en docs/.
- Usar Ponytail, Graphify y Superpowers.
- Preferir librerías open source; CUDA propietaria es excepción opcional permitida.
- Orden obligatorio: Rust CPU/CLI → frontend contra ese motor → API Java/trabajos → CUDA medida → Python solo si las reglas no bastan.
- Justificar cada frontera real entre lenguajes con ADR; no crear puertos/capas para usos hipotéticos.
- No pasar de fase 1 sin confirmación explícita del usuario.
- Usar DuckDB para JSON/TXT/CSV grandes cuando permita consultar/filtrar sin cargar todo; no imponerlo como dependencia del producto.
- Guardar commits cuando las implementaciones funcionen y pasen checks.
- Se permite investigar documentación pública HP SmartStream Mosaic/PrintOS si sirve para requisitos o flujos; registrar fuentes y mantener originales algoritmos/código/UI/assets.

## Avance comprobado

- [x] ~~Documentación, inventario y baseline Git.~~
- [x] ~~Diseño inicial aprobado y elecciones tecnológicas registradas.~~
- [x] ~~Tarea 1: contratos v1, tipos Rust generados, validación de entrada/resultados y hash.~~
- [x] ~~Revisión independiente y corrección de dos hallazgos P2.~~
- [x] ~~Plan revisado según las nuevas consideraciones, con criterios medibles.~~
- [ ] Usuario revisa el plan actualizado.
- [ ] Fase 1, tarea 2: PRNG determinista.
- [ ] Fase 1, tareas 3–6: SVG seguro, reglas, PNG y CLI.
- [ ] Evidencia de una variante SVG/PNG, determinismo y perfil CPU.
- [ ] Confirmación explícita para pasar a fase 2.

La tarea 1 no es un motor generativo funcional: no hay importador SVG, RNG, renderer, CLI, web, Java ni infraestructura implementados.

## Evidencia técnica

- RED inicial: cinco tests; dos fallan por seed fuera de u64 aceptada y hash vacío.
- GREEN inicial: cinco tests pasan.
- Ampliación: diez tests finales pasan, cero fallos/omitidos; incluyen overflow de resultados y opcionales null. Overflow de resultados tuvo RED observado antes de corregir.
- Hubo un error de sintaxis en una prueba y un warning collapsible_if de Clippy; corregidos, no pendientes.
- Checks finales con exit code 0: cargo fmt --all -- --check; cargo clippy --workspace --all-targets --locked -- -D warnings; cargo test --workspace --locked.
- Revisión independiente por lectura: dos P2 corregidos (null/schema y overflow de resultado); revisión posterior sin hallazgos materiales.
- Licencias declaradas de dependencias fijadas registradas en dependencias-motor.md; LICENSE del producto pendiente.
- No hay benchmarks, pruebas CUDA, tests de generación ni pruebas de fronteras Java/web ejecutados todavía.

## Git y recuperación

| Commit | Propósito | Verificación |
|---|---|---|
| b8cc6d8 | Baseline documental | diff --check y herramientas excluidas |
| 1c8f5ed | Registro de baseline | Commit documental |
| 2f1e6d4 | Contratos y validación Rust tarea 1 | Diez tests, fmt, Clippy, revisión corregida |

Autor técnico de commits: Codex <codex@localhost>, configuración por comando, sin cambiar identidad global. No se hizo push, merge, reset ni clean. No se versionan target/, dependencias descargadas ni resultados de usuarios.

## Decisiones / límites

- Java 21/Spring Boot/Maven Wrapper, PostgreSQL y Valkey aceptados para fase 3; no introducirlos antes.
- Web fase 2 requiere diseño y ADR del puente real al motor; HTTP local de preview es una propuesta, no implementada.
- Rust 1.99, Java 21, Node 24.19, pnpm 11.25 y Python 3.12 comprobados; algunas rutas no están en PATH.
- Equipo: i5-1235U, 10 núcleos/12 hilos, 7.69 GiB RAM, Intel UHD; NVIDIA no detectada. CUDA real necesita otro entorno.
- Contenedores no encontrados en las rutas comprobadas; despliegue todavía no probado.
- .graphify-cli/ es herramienta de sesión anterior, excluida del producto; no existe grafo del código aún.
- Una tolerancia de imagen y una aceleración solo se publican cuando se midan.

## Punto exacto para continuar

Presentar plan-desarrollo.md actualizado y esperar respuesta. No iniciar otras fases. Cuando el usuario autorice continuar fase 1, empezar tarea 2 desde su checklist; tarea 1 está terminada y no se repite. No volver a pedir aprobación de tecnologías ya aceptadas salvo que cambie la elección.
