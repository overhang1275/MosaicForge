# MosaicForge

Motor local Rust CPU para generar variantes vectoriales reproducibles desde una plantilla SVG. Exporta SVG, PNG y **un PDF multipágina por trabajo**. Incluye CLI, reglas geométricas, validación de cobertura, manifiesto incremental y cancelación cooperativa.

La fase 1 no incorpora web, API Java, base de datos, colas, CUDA ni Python/ML. Las dependencias del producto tienen licencias open source registradas en [docs/dependencias-motor.md](docs/dependencias-motor.md). La licencia de distribución del repositorio sigue pendiente de elección; todavía no es una publicación del producto.

## Compilar y ejecutar

Se ha probado Windows x64 con Rust/Cargo 1.99 y MSVC. Desde la raíz:

```powershell
cargo build --manifest-path services/engine/Cargo.toml --release -p mosaicforge-cli --locked
services/engine/target/release/mosaicforge.exe --help
services/engine/target/release/mosaicforge.exe inspect --template semilla.svg
services/engine/target/release/mosaicforge.exe generate --template semilla.svg --spec variantes.json --out out/trabajo-svg --batch-size 7 --threads 2
services/engine/target/release/mosaicforge.exe generate-pdf --template semilla.svg --spec trabajo-pdf.json --out out/trabajo-pdf --batch-size 7
```

El padre de `--out` debe existir. Usar un directorio nuevo para cada trabajo; nunca se reemplazan artefactos ni manifiestos existentes. El filesystem debe permitir hard links (por ejemplo NTFS). No hace falta Python ni CUDA para compilar o ejecutar el motor. Python/Poppler se usan solo en las herramientas locales de QA.

`inspect` devuelve el hash del SVG **normalizado**. Usarlo como `templateHash` en la configuración; no es el hash de los bytes originales. El importador admite un subconjunto vectorial estricto de SVG: no carga fuentes, imágenes ni recursos externos; rechaza scripts, CSS y texto. Preparar el arte antes de importar, sin eliminar contenido automáticamente. Ver [SVG admitido](docs/svg-admitido.md).

`generate` recibe [GenerationSpec v1](contracts/engine/v1/generation.schema.json); el fixture [identity.json](services/engine/tests/fixtures/identity.json) sirve de estructura inicial, pero requiere el hash real de la plantilla. Seed e índice inicial son cadenas decimales u64. Cada variante depende de seed/índice global, no de su grupo ni del orden de ejecución. N admite 1..10000; threads admite 1 o 2. Los archivos se llaman `variant-{índice}.svg` o `.png` según `output.format`.

`generate-pdf` recibe [PdfJobSpec v1](contracts/engine/pdf/v1/job.schema.json): incluye `generation` con salida SVG y `pdf` con `exporterVersion="1"`, `trimWidthMm`, `trimHeightMm` y `bleedMm`. La prueba inicial usa **100×100 mm y 0 mm de sangrado**. Produce únicamente `mosaic.pdf`, con una página por variante aceptada en orden global. N no divide el PDF y no se acepta `--threads`: el pipeline PDF procesa una página a la vez. Cero aceptadas produce `no_pages` sin PDF vacío.

## Resultados y límites

`manifest.jsonl` guarda configuración, resultados/rechazos y resumen incremental; stdout entrega los resultados como JSONL y stderr los errores. Las páginas PDF son provisionales hasta el registro `pdf_document` posterior al cierre y publicación. El hash de variante identifica el SVG previo al PDF; `documentHash` identifica todos los bytes finales del documento. Los detalles de reproducción y formato están en [ejecucion-motor.md](docs/ejecucion-motor.md).

| Salida | Significado |
|---|---|
| 0 | Trabajo evaluado completo; un rechazo válido no es fallo |
| 2 | Argumentos, SVG o configuración inválidos |
| 3 | E/S, publicación o exportación fallida |
| 130 | Cancelación cooperativa observada |

Ctrl+C espera a que termine la etapa en curso. Una publicación ya iniciada puede terminar íntegra. Archivo y manifiesto no son una transacción: un fallo del manifiesto después de publicar puede dejar un archivo completo sin registro final. No hay reanudación automática. Los temporales se eliminan en errores/cancelación normales; una terminación forzada o pérdida de energía puede dejarlos.

La validez incluye rangos, límites de complejidad y cobertura del viewport final; PDF comprueba también su encuadre físico, sin contar el fondo añadido. Un encuadre físico insuficiente aborta el documento con error de exportación para evitar publicar páginas vacías. No se garantiza legibilidad, contraste ni calidad estética arbitraria. Ver [validez y limitaciones](docs/validez-motor-cpu.md).

El presupuesto compartido PNG es una **estimación de buffers de 512 MiB**, no un máximo de RAM del proceso. SVG/PDF conservan metadata y límites propios; no se promete capacidad de millones de variantes/páginas. PDF es vectorial RGB, sin PDF/X ni conversión CMYK. Fiery queda fuera del alcance. No se afirman aceleraciones GPU.

## Verificar

Desde `services/engine`:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Los tests locales ignorados necesitan los archivos de QA indicados en sus mensajes; no son dependencias del producto. El perfil release, las mediciones de memoria y la revisión final se registran en [avance-tarea-6.6.md](docs/avance-tarea-6.6.md). Linux/macOS y CUDA no se han ejecutado en este entorno. Fase 2 autorizada; submódulos 2.1–2.3 completos.

## Web local (fase 2.3)

La base HTTP Rust y el BFF Next.js están disponibles. Preparación, arranque con token temporal en memoria y comandos de comprobación: [apps/web/README.md](apps/web/README.md). El editor visual se implementará en 2.4.
