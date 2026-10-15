# Submódulo 5.6 — PDF único multipágina

Fecha: 2026-10-15. Base 1ba3c9c. Usuario autorizó empezar por contrato, tamaño físico y sangrado. No hay código PDF implementado.

- [x] ~~Registrar medida inicial elegida: 100 × 100 mm, sangrado 0.~~
- [x] ~~Leer requisitos y contratos existentes, consultar Graphify y fuentes primarias de candidatos.~~
- [x] ~~Guardar propuesta revisable de contrato agregado, hashes, página, geometría y memoria.~~
- [ ] Usuario revisa y aprueba docs/superpowers/specs/2026-10-15-pdf-multipagina-design.md.
- [ ] Plan de implementación con pasos 5.6.2–5.6.5 y verificaciones concretas.
- [ ] 5.6.2: contratos nuevos, tipos y selección técnica comprobada.
- [ ] 5.6.3: página vectorial con recorte real y limpieza.
- [ ] 5.6.4: documento multipágina, índices, hashes y trabajo acotado.
- [ ] 5.6.5: muestras reales, métricas y revisión independiente global.

Graphify: presupuesto 300, 67 nodos encontrados/ocho mostrados, truncado. Snapshot anterior a tarea 2; fuentes actuales contrastadas directamente. Investigación documental solamente, sin instalar librerías ni ejecutar código de terceros. No se midieron memoria, tolerancias, peso PDF o rendimiento.

Autorrevisión del diseño: distingue estado de candidato/artefacto/documento, hashes por variante y documento, generación v1 y contrato agregado nuevo, medidas físicas y viewport, límite actual de 10000 índices y aspiración futura de millones. Incluye casos sin aceptadas, error de conversión, crecimiento de xref y recursos, sangrado positivo y límite de complejidad del recorte. Dependencias aún son candidatos, no decisiones aprobadas ni resultados ejecutados.

Superpowers brainstorming clasifica PDF como cambio de arquitectura; primero se presenta esta especificación para aprobación, después se redacta el plan. La documentación y los checkpoints se mantienen en docs/. No se cambian contratos publicados ni se adelanta implementación con la aprobación genérica del alcance.

Verificación documental: git diff --check pasa. Sin código/dependencias/contratos modificados, no se repite la suite; evidencia anterior conservada: 86 tests normales y prueba local integrada pasan en 5.5. La investigación detectó que svg2pdf está archivado y que la documentación indexada y main declaran versiones diferentes: se registró la lectura directa de main y se exige comprobar el paquete que se elija. Sin elección de librería ni prueba de streaming todavía.
