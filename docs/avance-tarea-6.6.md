# 6.6 — Perfil, revisión global y cierre de fase 1

2026-10-15. Base 1b9499f. Usuario autoriza 6.6; no fase 2. Fiery excluido.

- [x] ~~Perfil por etapas en test local ignorado release: importación, muestreo, transformaciones, cobertura, SVG, preparación PNG, raster, codificación, hash y escritura. Sin instrumentar el producto ni cambiar contratos.~~
- [x] ~~Ocho procesos CLI release medidos con contadores Windows: SVG/PNG de 12 índices, N=1/7/100, uno o dos hilos; PDF de tres páginas. Memoria separada del compilador y de la repetición por etapas.~~
- [x] ~~Hashes y bytes coinciden entre N/hilos y con baselines. Equipo, comandos, tamaños, versiones y límites guardados en perfil-cpu-fase1.md y mediciones/.~~
- [x] ~~Revisión independiente global: P2 de cobertura final confirmado y corregido con dos regresiones; revisor confirma solución por lectura.~~
- [x] ~~README nuevo, validez-motor-cpu.md, nota ADR 0004 y ejecución actualizados con límites explícitos.~~
- [x] ~~Formato, Clippy -D warnings y suite completa: 131 tests pasan, diez locales ignorados. Build release CPU sin CUDA y perfil local adicional pasan.~~
- [ ] Esperar aprobación explícita antes de fase 2.

Decisión: usar un test local de perfil con acceso al preparador PNG privado en lugar de añadir flags o instrumentación a la CLI. Repite por separado las etapas de la candidata aceptada y comprueba igualdad con la generación completa. Esta última incluye reintentos y reimportaciones, y se mide aparte. Los tiempos desglosados no se suman a la duración total de la CLI. La memoria corresponde a procesos CLI separados; el presupuesto estimado PNG de 512 MiB no es una cota de RSS.

## Revisión y correcciones

Revisor independiente con contexto nuevo, lectura global de contratos, Rust, tests, scripts y README, sin ejecutar Cargo para no interferir con binarios Windows. Detectó P2: la máscara del crop completo podía aceptar un arte que el encuadre final dejaba vacío. SVG de 100 × 100 con dos rectángulos x=0..49 y x=51..100, salida 1×4096/32×4096, reprodujo el fallo antes de corregirlo. La medición nueva usa dimensiones finales, viewBox implícito materializado y xMidYMid slice, sin fondo añadido; comparte los umbrales de Coverage.

También se verificó la composición PDF: generación de 4096×32 seguida de página de 100×100 podía volver a seleccionar un hueco. La primera corrección directa al tamaño media no modelaba esa composición. Se amplió la regresión, volvió a fallar y se reutilizó exactamente cover_transform del exportador PDF sobre el viewport SVG transitorio sin fondo. Se mide la máscara de media incluyendo extensión a sangrado. Si falla, RenderFailed aborta el documento antes de publicar, sin inventar un rechazo de GenerationSpec. Ambas regresiones y la suite completa pasaron después.

No se cambió el contrato ni una versión publicada: el prototipo tiene publish=false y todavía no se ha publicado. Los casos afectados cambian su decisión respecto al snapshot previo. Los baselines reales cuadrados conservaron exactamente sus bytes. Mantener el commit junto con seed, configuración, backend y versión al reproducir snapshots de desarrollo. Detalles en validez-motor-cpu.md. El revisor confirmó la corrección y ausencia de otros bloqueos P1/P2 por lectura; los tests los ejecutó el agente implementador.

Clippy detectó una aserción constante en el guard de release del nuevo test de perfil. Se permitió ese lint solo en ese test: debe compilar en la suite debug e impedir mediciones accidentales en debug al ejecutarlo. Después Clippy pasó. Comandos iniciales de lectura PowerShell con expansión de llaves no admitida y un aviso de tipo al leer el registro CPU no afectaron pruebas; se obtuvieron modelo CPU, toolchain y OS.

## Verificación y observaciones

- cargo fmt --all; cargo clippy --workspace --all-targets --locked --offline -- -D warnings; cargo test --workspace --locked --offline --quiet: exit 0, 131 tests (74 core, 23 CPU, 34 CLI), diez ignorados (nueve previos y perfil nuevo).
- cargo build --release -p mosaicforge-cli --locked --offline y cargo test --release -p mosaicforge-cpu --lib --no-run --locked --offline pasan. Se compila antes de medir, sin compilador concurrente.
- scripts/profile-cli-phase1.py y test release render::profile::supplied_artwork_profiles_stages_in_release ejecutado directamente: exit 0, una prueba local adicional pasa. Los otros nueve ignorados no se repitieron; QA real adicional documentada.
- Matriz de punta a punta y repetición por etapas: test.svg original intacto; 12 SVG/PNG por caso, los tres primeros idénticos a baselines; PDF único de tres páginas idéntico a 5.6. Pico working set PNG con un hilo: 27.05–27.51 MiB; con dos: 41.04 MiB. Commit privado: 33.40–34.14 y 57.31 MiB respectivamente. Una corrida por caso, sin cota de RAM ni generalización de velocidad.
- Perfil release de tres candidatas: raster 31.573–37.907 ms, codificación 12.339–14.214 ms; generación inclusiva 208.671–277.678 ms y repetición separada. No sumar valores ni concluir que codificación/E/S siempre domina; estos datos no justifican CUDA.
- PDF nuevo revalidado con pypdf/Poppler a 300 DPI: tres páginas de 100×100 mm, 14 objetos alcanzables, tres Forms, cero imágenes raster y cero recursos sin uso. SHA-256 dd1208f9fea7fa7150e4811bf56e33879ba2c99237aa847167b95508789cfa01. PDF y tres PNG del render idénticos al baseline revisado visualmente; primera página vista otra vez. QA JSON preservado en mediciones/.

## Cierre y continuidad

Fase 1 funcional CPU/CLI terminada, pendiente confirmación del usuario para fase 2. Sin publicación, merge ni push. La licencia del repositorio sigue pendiente de elección, protegida por AGENTS.md; queda antes de distribución pública. Sin CUDA, ML, Java ni servicios nuevos; Python se usa solo para QA. Sin Fiery, PDF/X/CMYK, OCR, garantías estéticas arbitrarias, Linux/macOS, benchmark GPU, OOM real, latencia máxima de cancelación ni millones de variantes validados.

README y perfil-cpu-fase1.md son la entrada de ejecución y evidencia; docs/estado.md mantiene el punto de reanudación. El grafo Graphify previo sigue siendo un snapshot antiguo y no se reconstruyó. La revisión contrastó fuentes directamente.
