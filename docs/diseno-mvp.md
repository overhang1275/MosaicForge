# MosaicForge — Diseño propuesto del MVP CPU

Fecha: 2026-10-14. Estado: diseño aprobado y ajustado por instrucciones posteriores del usuario. Orden vigente: Rust CPU/CLI → frontend contra motor → Java/trabajos → CUDA medida → Python condicionado a insuficiencia de reglas. Ver `plan-desarrollo.md` y `adr/0002-orden-incremental.md`. Tarea 1 CPU autorizada; no pasar de fase 1 sin confirmación.

## Propósito y alcance

Una persona carga una plantilla SVG propia, ajusta reglas, ve una previsualización reproducible, guarda un preset, genera un lote y descarga sus resultados. El primer incremento es un motor Rust autónomo probado mediante CLI; el MVP termina cuando ese mismo motor está integrado con Java y la web. CUDA, análisis Python y múltiples GPUs son etapas posteriores, sin bloquear el flujo CPU.

Requisitos y contexto: `requisitos-fuente.txt`, `plan-desarrollo.md`, `inventario-entorno.md` y `../AGENTS.md`. La preferencia por librerías open source y la excepción CUDA proceden de las aclaraciones directas del usuario.

## Enfoques considerados

1. **Elegido: motor CPU vertical primero.** CLI reproducible → interfaz de previsualización conectada directamente al motor → API Java/trabajos. Permite validar seguridad SVG, calidad de salida y determinismo sin infraestructura, antes de integrar los servicios requeridos.
2. **Todos los servicios desde el inicio.** Hace visibles pronto las fronteras, pero exige cola, base de datos y frontend antes de verificar que el producto genera resultados correctos.
3. **Demostrador web sin motor compartido.** Facilita una demo rápida, pero duplicaría generación en TypeScript y Rust o trasladaría negocio a Next.js; se descarta por las restricciones del proyecto.

El primer enfoque conserva la arquitectura final y reduce el número de problemas abiertos en cada entrega.

## Matriz técnica propuesta

| Responsabilidad | Elección | Motivo | Alternativa descartada para el MVP |
|---|---|---|---|
| API y negocio | Java 21, Spring Boot, Spring Security, Maven Wrapper | JDK ya disponible; ecosistema de seguridad y servicios. Dominio independiente de Spring. | Quarkus: viable, pero no hay requisito de imagen nativa que justifique otra decisión ahora. |
| Motor | Rust, Rayon, usvg/resvg y tiny-skia | Memoria segura, tareas CPU paralelas, render SVG existente. | Rasterizador propio inicial: aumenta mucho la superficie de errores. |
| Aleatoriedad | Philox4x32-10 con vectores de referencia | Flujo por contador y reproducción posterior CPU/CUDA. | PRNG global mutable: resultados dependientes del orden de ejecución. |
| Interfaz | Next.js App Router, TypeScript estricto, CSS Modules | Requisitos establecidos y estilos sin dependencia extra inicial. | Segundo backend en Next.js: contradice la frontera acordada. |
| Tipos web | openapi-typescript y validación runtime en frontera | Contrato Java como fuente de verdad. | Modelos HTTP escritos manualmente en TypeScript. |
| Metadatos | Adaptador PostgreSQL, aceptado | Transacciones para estados, pertenencia y progreso confirmado. | Una base adicional especializada antes de medir necesidades. |
| Cola/caché | Adaptador Valkey Streams, aceptado | Bloques de generación, consumidores y caché; licenciamiento abierto. | Kafka: complejidad operativa no justificada para la primera entrega. |
| Archivos | Adaptador local primero; S3 posteriormente | Desarrollo sin proveedor y ruta hacia almacenamiento remoto. | Binarios dentro de la cola o un proveedor cloud obligatorio. |
| Trazabilidad | OpenTelemetry, esquema JSON común | Correlación entre web, API y worker. | Formatos independientes por servicio. |
| Análisis | Python opcional, selección de librerías posterior | Solo incorporarlo con una capacidad que aporte utilidad medida. | Servicio vacío o ML obligatorio. |

Versiones exactas de paquetes y compatibilidad se fijan en el plan ejecutable y lockfiles después de verificarlas. No elegir automáticamente la última versión sin comprobar compatibilidad con Java 21/Node y licencias transitivas.

Fuentes de evaluación: [Spring Boot](https://docs.spring.io/spring-boot/installing.html), [licencia Spring](https://github.com/spring-projects/spring-boot/blob/main/CONTRIBUTING.adoc), [resvg](https://github.com/linebender/resvg), [cambio de licencia resvg](https://github.com/linebender/resvg/blob/main/CHANGELOG.md), [Random123](https://github.com/DEShawResearch/random123), [Valkey](https://valkey.io/topics/introduction/), [PostgreSQL](https://www.postgresql.org/docs/18/tutorial-transactions.html), [OpenTelemetry](https://opentelemetry.io/docs/).

## Primer contrato del motor

Propuesta de `GenerationSpec` versionado:

- `schemaVersion`: versión explícita del contrato.
- `templateHash`: SHA-256 del SVG normalizado y admitido por el motor.
- `seed`: entero sin signo de 64 bits representado como cadena decimal en JSON para evitar pérdida de precisión en JavaScript.
- `startIndex`, `count`: intervalo de índices globales de variantes, enteros no negativos con límites.
- `rules`: rangos finitos para transformación, color, capas y restricciones.
- `output`: formato SVG/PNG, dimensiones y fondo.
- `engineVersion`, `rulesVersion`: identificadores que participan en reproducción y caché.

El motor valida seguridad, geometría y sus límites operativos, incluso si Java ya validó la solicitud; esto protege la CLI y la frontera del worker sin duplicar reglas de negocio de proyectos/cuotas.

Cada resultado contiene índice, seed, hashes de plantilla/configuración/artefacto, versiones, backend, dispositivo y motivo de rechazo o fallback. Una variante rechazada conserva su índice. Los reintentos de reglas tienen contador propio y un límite; no desplazan la secuencia de variantes siguientes.

## SVG admitido y límites iniciales propuestos

Estos valores son límites de arranque sujetos a pruebas, no capacidad medida:

| Límite | Valor inicial |
|---|---|
| SVG de entrada | 5 MiB |
| Nodos XML / profundidad | 20,000 / 64 |
| Segmentos de paths normalizados | 100,000 |
| Dimensiones de salida | 1–4096 px por eje; máximo 16,777,216 píxeles |
| Previsualización | Hasta 12 variantes, 512 × 512 px |
| Lote inicial | Hasta 10,000 variantes; exportación incremental |
| Bloque en cola | Hasta 100 variantes; ajustar con memoria/resolución medidas |
| Render CPU concurrente en este equipo | 2 inicialmente; reducir a 1 en salidas máximas según presupuesto de memoria |
| Presupuesto de buffers de render por worker | 512 MiB; rechazar/adaptar antes de reservar buffers excesivos |
| Reintentos de generación por variante | Máximo 8; rechazo explícito al agotarlos |

Subconjunto inicial: paths, figuras básicas, grupos, transforms, fills/strokes sólidos, opacidad y recorte con referencias internas acíclicas acotadas. Las capas se identifican con grupos de primer nivel. Texto debe venir convertido a paths para evitar depender de fuentes del sistema. Se rechazan scripts, eventos, foreignObject, DTD/entidades, URLs externas, imágenes embebidas, referencias cíclicas, animaciones, filtros y otros elementos no admitidos; no se eliminan silenciosamente.

Mostrar el SVG original subido no es seguro: las previsualizaciones web serán PNG generados por el motor. Las descargas SVG se servirán con políticas de contenido apropiadas y sin recursos externos. El parser nunca resolverá red o archivos del host.

Distorsión controlada de paths tendrá límites de desplazamiento y expansión geométrica; preservar curvas válidas y rechazar no finitos. No se promete validación de legibilidad semántica para arte arbitrario.

## Determinismo y reproducción

Philox usa seed como clave y variante, identificador de regla, intento y contador de muestra como dominio del contador. Congelar ese mapeo con vectores de prueba; cambios requieren nueva versión de reglas. Los parámetros discretos y aceptación se calculan de manera común antes del render CPU/GPU.

Las variantes SVG y los parámetros deben coincidir al cambiar número de hilos, orden de bloques o reanudación. Para bytes PNG, fijar motor, dependencias, encoder y entorno de ejecución; no prometer identidad entre versiones o arquitecturas sin tests. Guardar la huella del entorno para repetir exactamente el render.

La tolerancia CPU/GPU se definirá antes de activar CUDA, con referencias de imagen y decisiones discretas iguales. Identidad bit a bit GPU solo dentro del mismo backend, arquitectura, versión de motor/dependencias/driver/toolchain fijados. Evitar reducciones flotantes no deterministas que afecten aceptación. No inventar ahora un umbral de diferencias que aún no se ha medido.

## Integración posterior del MVP

Entidades Java: User, Project, Template, Preset, Job y Artifact. El estado duradero vive en persistencia. Un Job referencia plantilla/configuración inmutables, seed y rango; estados queued/running/succeeded/failed/cancelled. Backend preferido auto/cpu/gpu; el MVP solo declara capacidad CPU y responde claramente si se exige GPU inexistente.

API propuesta `/api/v1`: proyectos, subida/consulta de plantillas, presets, previsualizaciones, creación/consulta/cancelación de trabajos, eventos SSE y artefactos paginados. Los permisos se comprueban también al descargar. Autenticación de sesión, cookies HttpOnly y protección CSRF; decidir flujo de alta y administración antes de implementar acceso público.

Java registra Job y evento outbox en una transacción. Un dispatcher publica bloques con idempotency key. Los workers escriben artefactos temporales y publican resultados por contrato; Java confirma de forma idempotente, de modo que cola y persistencia puedan reconciliarse tras una caída. Streams permite entrega repetida: nunca asumir exactly-once.

Java controla trabajos y permisos; Rust realiza generación y exportación. El worker consume la cola mediante un adaptador separado del núcleo generativo. Web usa clientes generados y se conecta al progreso vía BFF/SSE con recuperación del snapshot del trabajo. El progreso distingue procesadas, aceptadas, rechazadas y fallidas.

## Verificación del primer incremento

- Vectores de referencia Philox y muestras por índice/regla/intent.
- Misma seed y configuración producen SVG idéntico con 1 y 2 hilos y bloques reordenados.
- Generar índices 100–199 por separado coincide con ese intervalo de una corrida completa.
- SVG con URL, entidad XML, scripts, ciclos o límites excedidos falla sin E/S externa ni artefacto parcial válido.
- Escala/rotación/color y recorte se prueban con fixtures propios pequeños y restricciones geométricas conocidas.
- PNG de fixtures se decodifica, cumple dimensiones y cambia cuando cambian parámetros; hashes reproducibles dentro del entorno fijado.
- Rechazos agotados conservan el índice; cancelación libera recursos y evita publicar resultados incompletos.
- CPU compila y prueba sin CUDA ni Python.

## Criterio de aprobación y continuidad

Revisar alcance, límites y elecciones propuestas antes de escribir producto. La confirmación de infraestructura se solicita por la regla de AGENTS.md; la revisión del diseño se exige por Superpowers brainstorming. Tras aprobación: crear el plan ejecutable de contratos y motor CPU, con archivos, interfaces y pruebas concretas; luego implementarlo.

CUDA necesitará un entorno NVIDIA distinto al equipo observado. Este requisito no bloquea el motor CPU ni el MVP web.
