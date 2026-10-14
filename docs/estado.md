# MosaicForge — Estado y punto de reanudación

Última actualización: 2026-10-14.

## Leer al retomar

1. `../AGENTS.md`.
2. Este archivo.
3. `plan-desarrollo.md`.
4. `requisitos-fuente.txt` y las decisiones de `adr/` cuando existan.

## Solicitudes y aclaraciones del usuario

- Mantener checklist visible y tachar lo completado conforme avance.
- Usar Ponytail, Graphify y Superpowers.
- Priorizar librerías open source; CUDA es una excepción propietaria permitida, opcional.
- Preparar el desarrollo por fases.
- Guardar planes, checklists y documentación de continuidad en `docs/`.
- Diseño e infraestructura aprobados mediante «ok continua» después de presentar el diseño escrito. LICENSE sigue pendiente.
- Usar Git para guardar puntos recuperables cuando cada implementación funcione y pase sus checks.

## Estado actual

- [x] ~~Leer el documento fuente y AGENTS.md.~~
- [x] ~~Preparar el plan por fases y el registro de continuidad.~~
- [x] ~~Inventariar herramientas y GPU disponibles.~~
- [x] ~~Guardar el diseño propuesto del MVP CPU.~~
- [x] ~~Confirmar decisiones de fase 0 y revisar el diseño CPU.~~
- [x] ~~Registrar ADR y crear el plan ejecutable de seis tareas CPU.~~
- [ ] Usuario revisa el plan ejecutable y confirma ejecución directa.
- [ ] Iniciar implementación después del diseño y plan del subsistema.

El repositorio no contiene código del producto. Antes de esta planificación contenía AGENTS.md y `.graphify-cli/`, un directorio de herramientas instalado en una sesión anterior. Este directorio no forma parte del producto y debe excluirse de análisis del código, imágenes y entregas. No existe `graphify-out/graph.json`.

## Evidencia de esta sesión

- Se leyó `C:/Users/init/Desktop/promt.txt` y se guardó una copia local para continuidad.
- Se leyó AGENTS.md completo y se comprobó la ausencia de grafo Graphify.
- Se crearon `plan-desarrollo.md` y este registro; se verificó su existencia y las fases del plan.
- Se ejecutaron comandos de versión y consultas de hardware; detalles en `inventario-entorno.md`.
- Se escribió y revisó `diseno-mvp.md`: límites, SVG seguro, determinismo, contratos e integración.
- No se ejecutaron pruebas del producto: todavía no existe implementación.

## Decisiones aceptadas y pendientes

- PostgreSQL: metadatos/estado duradero mediante adaptador.
- Valkey: caché, coordinación y cola Streams mediante adaptador.
- Archivos locales para desarrollo; interfaz S3 para almacenamiento desplegado, proveedor pendiente.
- OpenTelemetry para trazabilidad/observabilidad.
- Next.js/TypeScript, Java, Rust y Python opcional corresponden a los requisitos de partida.

## Punto exacto para continuar

Revisar `superpowers/plans/2026-10-14-motor-cpu.md` con el usuario; ejecución directa recomendada por el tamaño y dependencia secuencial de las tareas. Luego comenzar tarea 1: contratos y tipos generados. Diseño y adaptadores no requieren pedir aprobación nuevamente. Todavía no se han instalado dependencias de producto ni escrito implementación.

## Política Git

Inicializar repositorio y guardar documentación baseline, sin incluir `.graphify-cli/`, resultados o secretos. No existía configuración de autor Git consultable; usar identidad técnica `Codex <codex@localhost>` solo para cada commit, sin cambiar configuración global. Cada implementación obtiene un commit cuando pasan formato, lint y tests. Registrar hash y evidencia en el siguiente checkpoint documental; no usar reset/clean para descartar cambios sin autorización.

## Bloqueos y límites

- Sin blockers comprobados para la planificación.
- Java 21, Rust 1.99, Node 24.19, pnpm 11.25 y Python 3.12 comprobados mediante ruta absoluta. Algunas herramientas no están configuradas en PATH.
- Equipo: i5-1235U, 10 núcleos/12 hilos, 7.69 GiB RAM, Intel UHD. NVIDIA no detectada; pruebas CUDA reales necesitan otro equipo/runner.
- Herramientas de contenedores no encontradas en PATH/ruta habitual; despliegue local todavía no probado.
- CUDA requiere mediciones y pruebas en hardware real; no afirmar aceleración antes de ejecutarlas.
- Antes de tocar LICENSE, CI o contratos publicados, seguir las autorizaciones indicadas por AGENTS.md.

## Registro de cambios

| Fecha | Cambio | Verificación |
|---|---|---|
| 2026-10-14 | Hoja de ruta por fases y documentos de continuidad | Lectura de archivos, fases y checklist; sin cambios de producto |
| 2026-10-14 | Inventario y propuesta de diseño CPU | Versiones/hardware comprobados; diseño revisado para índices, límites y recuperación; pendiente aprobación |
| 2026-10-14 | Diseño aprobado, ADR y plan ejecutable CPU | Seis tareas con archivos, interfaces, pruebas y commits; implementación pendiente |
