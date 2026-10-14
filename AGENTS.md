# AGENTS.md ? MosaicForge

Guía para agentes de IA y colaboradores que trabajan en este repositorio. Léela completa antes de tocar código. Si una instrucción de una subcarpeta (`services/*/AGENTS.md`, `apps/web/AGENTS.md`) contradice a este archivo, manda la más específica.

## 1. Qué es MosaicForge

Aplicación web open source que genera miles o millones de variantes visuales únicas, deterministas y reproducibles, a partir de plantillas vectoriales semilla (SVG como formato principal). Aceleración opcional con GPU NVIDIA (CUDA) donde las mediciones demuestren beneficio.

## 2. Reglas inviolables

1. **Originalidad:** no copiar ni aproximar código, assets, nombres, algoritmos patentados, UI ni documentación de HP SmartStream Mosaic ni de otro producto propietario. Todo algoritmo es propio: transformaciones paramétricas, ruido controlado, reglas de composición y validación estética.
2. **Determinismo:** misma plantilla + configuración + seed + backend + versión del motor ? mismas variantes. Cada variante deriva su aleatoriedad solo de `(seed, índice de variante)` (PRNG basado en contador). Entre CPU y GPU: mismas decisiones discretas y diferencias de píxel dentro de la tolerancia documentada.
3. **CUDA es opcional:** el proyecto debe compilar, probarse y ejecutarse completo sin CUDA. El backend CPU es la implementación de referencia y la fuente de verdad en tests. No se mueve una etapa a GPU sin medir antes que supera a CPU.
4. **Sin dependencias propietarias obligatorias.** Python/ML también es opcional para el flujo principal.
5. **Sin infraestructura hardcodeada:** nada de base de datos, broker, proveedor cloud o formato de almacenamiento fijado en el dominio. Todo va tras puertos (interfaces) con adaptadores sustituibles.
6. **Sin secretos en el repositorio** ni en logs. Configuración por variables de entorno o archivos de configuración.
7. **Sin código muerto ni abstracciones sin uso real.** Una abstracción se justifica con al menos dos implementaciones o una frontera real.
8. **No inventar resultados:** no afirmar aceleraciones, tolerancias ni cobertura de tests que no se hayan medido o ejecutado.

## 3. Arquitectura y estructura del repositorio

Monorepo con un directorio por servicio/lenguaje y contratos compartidos.

```
mosaicforge/
??? AGENTS.md                  # este archivo
??? CLAUDE.md                  # importa AGENTS.md (ver sección 9)
??? README.md
??? LICENSE
??? contracts/                 # fuente de verdad entre servicios
?   ??? openapi/               # API HTTP (Java ? web)
?   ??? proto/                 # mensajes/servicios (Java ? Rust ? Python)
?   ??? events/                # esquema común de eventos de log (JSON Schema)
??? apps/
?   ??? web/                   # Next.js + TypeScript (AGENTS.md propio)
??? services/
?   ??? api/                   # Java: plano de control (AGENTS.md propio)
?   ??? engine/                # Rust: motor generativo (AGENTS.md propio)
?   ?   ??? crates/            # core, cpu, cuda (feature), worker/cli
?   ??? analysis/              # Python: ML y análisis, opcional (AGENTS.md propio)
??? deploy/
?   ??? docker/                # Dockerfiles por servicio (CPU y CUDA)
?   ??? compose/               # composición local, con y sin GPU
??? docs/
?   ??? adr/                   # decisiones de arquitectura
??? scripts/                   # utilidades de desarrollo y CI
```

Responsabilidades:

| Componente | Lenguaje | Rol |
|---|---|---|
| `services/api` | Java | API pública versionada, autenticación/autorización, orquestación de trabajos, reglas de negocio, puertos de persistencia/cola/almacenamiento/cache |
| `services/engine` | Rust | Transformaciones, rasterización, validación geométrica, exportación, backends CPU/CUDA |
| `services/analysis` | Python | Validación estética con modelos, similitud semántica, paletas y presets sugeridos, reportes |
| `apps/web` | TypeScript (Next.js) | UI, BFF delgado (sesión, proxy, streaming de progreso) |

Fronteras:

- Los servicios se comunican **solo** mediante los contratos de `contracts/` (HTTP/gRPC/mensajes sobre la cola abstracta). Nada de memoria compartida ni dependencias de rutas internas entre lenguajes.
- Java es el plano de control y **no ejecuta CUDA**. La GPU se usa solo desde Rust (y Python si aporta).
- Next.js **no** contiene lógica de negocio, persistencia ni cómputo generativo.
- Los tipos/clientes se **generan** desde `contracts/`; no se duplican modelos a mano.

## 4. Comandos

Actualiza esta sección en cuanto se elijan las herramientas concretas (framework Java, gestor de paquetes, etc.). Hasta entonces, usa el tooling estándar de cada ecosistema.

```bash
# Rust (services/engine)
cargo fmt --check && cargo clippy --all-targets -- -D warnings
cargo test                          # CPU, sin CUDA
cargo test --features cuda          # solo con GPU disponible
cargo bench                         # benchmarks CPU vs GPU

# Java (services/api)
# <build + test + lint: definir al elegir Maven o Gradle>

# Python (services/analysis)
# <lint + tipado estático + pytest: definir al elegir el gestor de entorno>

# Web (apps/web)
# <instalar, lint, tsc --noEmit, tests de componentes, e2e>
```

Antes de dar una tarea por terminada: formateo, lint, tipado y tests del componente tocado deben pasar. Si no puedes ejecutarlos, dilo explícitamente.

## 5. Convenciones por lenguaje

- **Rust:** `unsafe` solo en un módulo mínimo y documentado (especialmente CUDA), con tests. Kernels CUDA en su propio directorio con documentación de grid/block, memoria y supuestos. Abstracción `ComputeBackend` con `CpuBackend` (por defecto) y `CudaBackend` (feature `cuda`).
- **Java:** arquitectura hexagonal; el dominio no depende del framework. Validación de entrada en la frontera. Errores con formato consistente.
- **Python:** tipado estático, entorno aislado, sin dependencias obligatorias para el flujo principal.
- **TypeScript:** modo estricto, sin `any` injustificado, componentes reutilizables y accesibles (WCAG razonable).

## 6. Logging y trazabilidad

- JSON en producción, texto legible en desarrollo; niveles trace/debug/info/warn/error configurables.
- Mismo esquema de eventos (`contracts/events/`) en Java, Rust, Python y web.
- Contexto: request ID, job ID, user ID, plantilla, seed, duración, resultado; en cómputo: backend (cpu/cuda), dispositivo, tamaño de lote, tiempos por etapa, VRAM, fallbacks/OOM.
- Propaga el trace ID (por ejemplo, W3C Trace Context) en cada llamada y mensaje de la cola.
- **Nunca** registrar secretos, tokens, contenido sensible ni datos personales innecesarios.

## 7. Tests

- Motor: determinismo, límites, validaciones y transformaciones; paridad CPU?GPU (se omite limpiamente sin GPU); determinismo en GPU.
- Integración: API, carga de plantillas, generación de lotes, exportación.
- Contrato: en cada frontera entre lenguajes; un e2e Java ? Rust (? Python si está habilitado) con propagación de trace ID.
- Fallback: sin GPU, error de inicialización CUDA, OOM de VRAM, servicios Python caídos.
- Robustez: archivos inválidos, plantillas corruptas, recursos excesivos, límites de tamaño, variantes, memoria (RAM y VRAM), tiempo y concurrencia.

## 8. Cómo trabajar aquí (para agentes)

1. **Plan primero:** antes de cambios grandes, propón estructura, contratos y decisiones; registra las importantes en `docs/adr/`.
2. **Incremental:** implementa primero el backend CPU completo y correcto; después añade CUDA etapa por etapa, verificando paridad y rendimiento en cada paso.
3. **Contratos antes que código:** cambia `contracts/` primero, regenera tipos/clientes y luego implementa.
4. **Cambios pequeños y verificables:** una responsabilidad por cambio, con sus tests.
5. **No tocar sin pedir:** `LICENSE`, `contracts/` publicados (cambios incompatibles requieren versión nueva), configuración de CI y secretos.
6. **Ante la duda, pregunta** en lugar de asumir decisiones de infraestructura (base de datos, broker, almacenamiento, cloud).
7. **Sé honesto con el estado:** indica qué se ejecutó, qué falló y qué no se pudo verificar.

## 9. Compatibilidad con otras herramientas

`AGENTS.md` es el archivo común para agentes. Si usas codex, crea en la raíz un `CODEX.md` que lo importe:

```md
@AGENTS.md
```

Añade debajo solo instrucciones específicas de Claude Code, si las hay.