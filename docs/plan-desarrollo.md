# MosaicForge — Plan de desarrollo por fases

Fecha: 2026-10-14. Estado: planificación; implementación pendiente.

**Objetivo:** generar variantes originales de plantillas SVG, reproducibles y exportables, desde una interfaz web; completar primero el flujo CPU y añadir CUDA únicamente donde las mediciones demuestren beneficio.

**Fuentes:** `requisitos-fuente.txt`, `../AGENTS.md` y las aclaraciones del usuario registradas en `estado.md`. Las instrucciones del documento fuente se interpretan como requisitos del producto; no sustituyen las solicitudes del usuario.

**Método:** implementación incremental en esta conversación, con tareas pequeñas y verificables. Este documento es la hoja de ruta; cada subsistema necesita diseño y tareas concretas antes de implementarlo. No es una especificación técnica aprobada ni una promesa de rendimiento.

## Cómo usar la checklist

- Pendiente: `- [ ] Tarea`.
- Terminada y verificada: `- [x] ~~Tarea~~`; registrar evidencia en `estado.md`.
- Una fase termina cuando todos sus entregables y su criterio de salida se verifican.
- Una prueba omitida por falta de hardware no cuenta como aprobada.
- Antes de parar: actualizar estado, bloqueos y siguiente tarea. Al retomar: leer `../AGENTS.md`, `estado.md`, este plan y las decisiones aplicables.

## Decisiones y restricciones

| Componente | Tecnología | Justificación y frontera |
|---|---|---|
| Presentación | Next.js App Router, TypeScript estricto | UI y BFF delgado; la validación autoritativa pertenece a Java. |
| Control | Java | Autenticación, autorización, API, orquestación y puertos de infraestructura. Framework y build se deciden en fase 0. |
| Motor | Rust | Transformaciones, exportación, rasterización y paralelismo CPU; CUDA aislada tras feature. |
| Análisis opcional | Python | ML y análisis cuando aporten valor; su ausencia nunca bloquea generación/exportación. |
| Contratos | OpenAPI, Protobuf y JSON Schema | Clientes generados, mensajes versionados y eventos comunes. Evitar duplicar un contrato en varios formatos sin necesidad. |
| Persistencia propuesta | PostgreSQL | Metadatos y estado duradero, mediante adaptador. Pendiente de confirmación. |
| Caché/cola propuesta | Valkey y Streams | Coordinación, caché y trabajos por bloques; pendiente de confirmar e implementar recuperación de pendientes. |
| Artefactos | Adaptador local; API S3 propuesta para despliegue | Referencias en mensajes y metadatos; SVG/PNG/ZIP en almacenamiento de archivos. Proveedor por decidir. |
| Observabilidad | OpenTelemetry | Correlación de trazas y métricas entre servicios, junto con logs comunes. |

- Preferir librerías open source; documentar licencia, versión y motivo. CUDA es la excepción propietaria explícitamente permitida y siempre opcional.
- No fijar proveedor, base de datos ni broker en el dominio.
- No copiar productos, marcas, assets o algoritmos propietarios.
- No almacenar secretos ni contenido sensible en logs.
- No añadir ML, infraestructura distribuida o abstracciones sin una necesidad concreta.
- No modificar LICENSE, CI ni contratos publicados sin la autorización exigida en AGENTS.md.

## Estructura objetivo

```text
apps/web/                     Next.js y TypeScript
services/api/                 Java: dominio, aplicación y adaptadores
services/engine/               Rust: core, CPU, worker/CLI; CUDA al llegar a su fase
services/analysis/             Python opcional, al llegar a su fase
contracts/openapi/            API pública
contracts/proto/              Frontera de workers
contracts/events/             Esquema de logs y eventos
deploy/                       Contenedores y composición local
scripts/                      Generación y verificación
docs/                         Requisitos, diseños, decisiones, planes y avance
```

Crear cada carpeta cuando su entregable la necesite; no llenar el repositorio de esqueletos vacíos.

## Fase 0 — Diseño y preparación

- [x] ~~Leer los requisitos y las reglas del repositorio.~~
- [x] ~~Registrar la política de licencias y la excepción CUDA.~~
- [x] ~~Crear la hoja de ruta y el registro para retomar.~~
- [x] ~~Inventariar Java, Rust, Node, Python, herramientas de contenedores, GPU y driver disponibles.~~
- [x] ~~Confirmar los adaptadores PostgreSQL, Valkey y almacenamiento local inicial con interfaz S3 posterior.~~
- [x] ~~Decidir Spring Boot/Maven Wrapper y herramientas web; registrar alternativas descartadas.~~
- [ ] Elegir licencia del proyecto con aprobación explícita.
- [x] ~~Proponer el primer flujo CPU: cargar SVG → configurar → previsualizar → generar → descargar.~~
- [x] ~~Proponer límites de entrada, subconjunto SVG admitido, formatos y resolución iniciales.~~
- [x] ~~Proponer el modelo de dominio y la política de semillas, versiones y cancelación.~~
- [x] ~~Guardar el diseño propuesto en `docs/diseno-mvp.md`.~~
- [x] ~~Registrar decisiones aprobadas en `docs/adr/0001-arquitectura-inicial.md`.~~
- [x] ~~Revisar el diseño con el usuario y preparar el plan detallado del motor CPU.~~
- [ ] Revisar con el usuario el plan ejecutable antes de implementar.

**Salida:** diseño revisado, decisiones explícitas y una primera entrega acotada con pruebas y archivos concretos.

## Fase 1 — Contratos y motor CPU autónomo

- [ ] Definir plantilla, reglas, seed, índice de variante, salida y metadatos de reproducción.
- [ ] Definir contratos versionados y generación de tipos; probar serialización entre lenguajes.
- [ ] Implementar importación SVG segura: límites, rechazo de recursos externos y contenido activo, y errores claros.
- [ ] Implementar PRNG por contador; cada variante depende de seed e índice, sin estado compartido.
- [ ] Implementar traslación, rotación, escala, espejo, recorte y variaciones de color/opacidad.
- [ ] Añadir capas, composición, distorsión controlada y restricciones geométricas.
- [ ] Implementar exportación SVG y raster CPU PNG con límites de memoria.
- [ ] Implementar CLI CPU y generación por bloques con paralelismo acotado.
- [ ] Verificar determinismo, cambio del número de hilos, índices reanudados, SVG inválidos y límites.

**Salida:** una plantilla y configuración generan un lote reproducible desde CLI, sin Java, Python ni CUDA. La salida no depende del orden de ejecución de variantes.

## Fase 2 — API y trabajos duraderos

- [ ] Implementar API Java versionada, validación autoritativa y errores consistentes.
- [ ] Implementar autenticación y autorización por proyecto y artefacto.
- [ ] Implementar puertos de persistencia, cola, almacenamiento y caché con los adaptadores confirmados.
- [ ] Implementar estados: queued, running, succeeded, failed y cancelled; validar transiciones.
- [ ] Implementar worker Rust por contrato, con reporte de capacidades CPU y estado.
- [ ] Distribuir bloques de variantes; usar claves de idempotencia y publicar resultados de forma segura.
- [ ] Implementar recuperación tras caída, reintentos limitados y reconciliación entre persistencia y cola.
- [ ] Implementar progreso SSE con identificadores de evento y recuperación del estado tras reconexión.
- [ ] Implementar cancelación cooperativa, límites de concurrencia y limpieza de artefactos parciales.
- [ ] Probar aislamiento entre usuarios, entrega duplicada, caída del worker, desconexión de cola y reinicio de API.

**Salida:** por API se puede cargar, lanzar, consultar, cancelar y descargar un trabajo CPU; un reinicio no pierde el estado duradero ni publica resultados duplicados.

## Fase 3 — Interfaz y MVP CPU completo

- [ ] Generar cliente/tipos TypeScript desde OpenAPI y validar respuestas inesperadas en runtime.
- [ ] Crear flujo de proyecto, carga de plantilla y configuración de reglas con seed visible.
- [ ] Crear previsualización pequeña que use el mismo motor que el lote final.
- [ ] Implementar presets, lanzamiento del lote e historial de trabajos.
- [ ] Mostrar progreso, velocidad, backend y dispositivo; recuperar estado al reconectar.
- [ ] Crear galería paginada y virtualizada, filtros, selección y descarga.
- [ ] Aplicar Server Components a lectura y BFF delgado a sesión/proxy/eventos.
- [ ] Implementar carga diferida; usar Web Worker solo si una tarea local medida bloquea la UI.
- [ ] Verificar teclado, foco, contraste, errores y estados vacíos.
- [ ] Probar el flujo navegador → Java → Rust → descarga, con trace ID común.

**Salida:** un usuario completa el flujo de ocho pasos descrito en los requisitos sin GPU ni servicios Python. Es el primer MVP utilizable.

## Fase 4 — Operación y escala CPU

- [ ] Unificar logs JSON/texto y propagación W3C Trace Context; impedir datos sensibles.
- [ ] Instrumentar duración por etapa, profundidad de cola, memoria, errores y variantes por segundo.
- [ ] Añadir caché de previsualizaciones con clave de plantilla/configuración/versión y caducidad.
- [ ] Añadir rate limiting, cuotas y presión de retorno cuando la capacidad esté agotada.
- [ ] Completar formato, lint, tipado, contratos y pruebas relevantes en CI autorizada.
- [ ] Crear contenedores por servicio, health checks, límites y composición CPU portable.
- [ ] Verificar exportación por streaming y uso acotado de memoria/disco en lotes grandes.
- [ ] Medir CPU con resolución, complejidad SVG y tamaño de lote registrados; guardar comandos y hardware en `docs/benchmarks/`.
- [ ] Probar escalado de workers y recuperación bajo carga antes de anunciar capacidad para millones de variantes.

**Salida:** despliegue CPU reproducible, límites medidos, recuperación probada y evidencia de capacidad. No se promete un volumen que no haya sido ensayado.

## Fase 5 — CUDA opcional y medida

- [ ] Evaluar bindings mantenidos, incluidos cudarc y Rust-CUDA; fijar versiones y licencias en un ADR.
- [ ] Definir ComputeBackend con implementación CPU real y backend CUDA real, bajo feature `cuda`.
- [ ] Detectar dispositivo, VRAM y compatibilidad; conservar compilación/ejecución CPU sin toolkit.
- [ ] Definir tolerancia de píxeles, decisiones discretas compartidas y contrato de reproducción por versión/arquitectura GPU.
- [ ] Implementar y comparar PRNG por contador y transformación de color por píxel.
- [ ] Implementar y comparar composición de capas y rasterización básica para el subconjunto SVG admitido.
- [ ] Medir transferencia, kernels y posproceso; activar una etapa GPU solo cuando mejore el tiempo completo en lotes realistas.
- [ ] Implementar buffers reutilizables, lotes adaptativos, streams y solapamiento según el perfil medido.
- [ ] Implementar OOM → reducción de lote → reintento → fallback CPU según política; registrar backend por bloque/artefacto.
- [ ] Probar cancelación, liberación de recursos, falta de GPU, error de inicialización y OOM simulado.
- [ ] Ejecutar paridad y determinismo en GPU real; registrar los checks omitidos si falta hardware.
- [ ] Añadir contenedor CUDA y guía de runner GPU; conservar el pipeline CPU principal.

**Salida:** GPU opcional con resultados reproducibles, paridad comprobada y mejoras medidas. Si el hardware no está disponible, esta fase queda pendiente de validación real.

## Fase 6 — Análisis Python opcional

- [ ] Seleccionar la primera capacidad útil: paletas o detección de similitud, según necesidades observadas.
- [ ] Versionar contratos y generar clientes; separar análisis del flujo obligatorio.
- [ ] Implementar análisis con dependencias/modelos de licencia documentada y versiones reproducibles.
- [ ] Definir límites, timeouts, cancelación y reparto de VRAM si comparte dispositivo con Rust.
- [ ] Verificar ausencia, caída y timeout del servicio: generación y exportación CPU continúan.
- [ ] Medir utilidad/coste antes de incorporar embeddings, modelos estéticos o sugerencias de presets adicionales.

**Salida:** una mejora de análisis funciona y su indisponibilidad provoca degradación documentada, sin bloquear el producto.

## Fase 7 — Escala avanzada y entrega

- [ ] Añadir múltiples GPUs, cola por dispositivo y planificación de VRAM a partir de mediciones de una GPU.
- [ ] Probar galería y metadatos con decenas de miles de resultados; medir lotes mayores progresivamente.
- [ ] Revisar licencias, dependencias, vulnerabilidades y aislamiento de usuarios.
- [ ] Completar README, ejemplos originales, API, instalación CPU/CUDA y guía de contribución.
- [ ] Elegir y añadir LICENSE con aprobación explícita.
- [ ] Construir/actualizar Graphify excluyendo dependencias, binarios y artefactos generados; revisar fronteras entre servicios.
- [ ] Ejecutar verificación final y registrar resultados, limitaciones y tareas diferidas.

**Salida:** entrega reproducible con documentación, pruebas y benchmarks verificables; capacidad y soporte GPU descritos según evidencia.

## Orden y dependencias

Fases 0 → 1 → 2 → 3 → 4 → 5. La fase 6 puede comenzar después de estabilizar el MVP CPU y sus contratos; la fase 7 integra el resultado final. No retrasar el MVP por ML o múltiples GPUs.

## Riesgos que deben tener pruebas

| Condición | Comportamiento esperado | Fase |
|---|---|---|
| SVG malicioso, corrupto o excesivo | Rechazo seguro con límites y error comprensible | 1–3 |
| Seed idéntica con distinto paralelismo o reanudación | Mismos parámetros y resultados dentro del contrato de determinismo | 1, 5 |
| Mensaje duplicado o caída entre escritura y confirmación | Recuperación idempotente sin publicar dos resultados | 2 |
| Cancelación durante exportación o render | Liberación de recursos y artefactos parciales identificados/limpiados | 2, 5 |
| Resultado ajeno solicitado por identificador | Acceso denegado tanto a metadatos como a descarga | 2–3 |
| CUDA o Python ausentes, OOM o caída | CPU disponible y degradación/fallback visible | 5–6 |

## Próximo paso

Revisar `superpowers/plans/2026-10-14-motor-cpu.md` y ejecutar la tarea 1 tras confirmación. Ninguna fase de implementación se ha iniciado. LICENSE permanece pendiente sin bloquear el desarrollo del motor.
