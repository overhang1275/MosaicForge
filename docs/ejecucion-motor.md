# Ejecución del motor CPU

Actualizado: 2026-10-15, submódulos 6.1–6.5. La CLI expone **inspect**, **generate SVG/PNG**, **generate-pdf**, grupos, manifiesto incremental y cancelación Ctrl+C, --help y --version. SVG/PNG admiten uno/dos hilos; PDF procesa una página a la vez.

## Preparación

Rust/Cargo 1.99 y toolchain MSVC disponibles en esta sesión. En PowerShell, si Cargo no está en PATH:

```powershell
$env:PATH = 'C:\Users\init\.cargo\bin;' + $env:PATH
```

Desde la raíz del repositorio:

```powershell
cargo run --manifest-path services/engine/Cargo.toml -p mosaicforge-cli --locked -- --help
cargo run --manifest-path services/engine/Cargo.toml -p mosaicforge-cli --locked -- inspect --template "out/test-prepared.svg"
```

Para inspeccionar otro archivo, sustituir la ruta; usar comillas si tiene espacios. La CLI acepta rutas del sistema operativo, también Unicode. No escribe archivos ni altera la plantilla. stdout contiene un objeto JSON con templateHash, width, height, viewBox, layerIds y segmentCount, seguido de un salto de línea. El hash pertenece al SVG normalizado, no a los bytes originales del editor.

test.svg del usuario contiene CSS y metadatos no admitidos por el importador seguro. La copia out/test-prepared.svg se obtiene con scripts/prepare-test-svg.py, conversor específico de esta muestra, no importador general. El original se rechaza explícitamente; no se eliminan atributos automáticamente de cualquier entrada.

## Errores y límites

Generación desde raíz (spec conforme a contracts/engine/v1, templateHash obtenido de inspect):

```powershell
cargo run --manifest-path services/engine/Cargo.toml -p mosaicforge-cli --locked -- generate --template "out/test-prepared.svg" --spec "out/cli-6.2-svg.spec.json" --out "out/nuevo-trabajo" --batch-size 7 --threads 2
```

El padre de --out debe existir. Permite un directorio existente que no tenga manifest.jsonl, pero nunca reemplaza archivos: publica variant-{índice}.svg o .png según spec.output.format. El índice no depende de la posición en el grupo. stdout emite una línea JSON por resultado (kind=variant, result con el contrato VariantResult) y resumen final (kind=summary, status, accepted/rejected). El rechazo válido no escribe artefacto y devuelve 0. No se acumulan todos los artefactos del trabajo.

El JSON spec admite hasta 1 MiB; contrato/hash se comprueban antes de crear directorio. Temporales exclusivos se escriben/sincronizan/cierran antes de publicar por hard link. Se requiere filesystem compatible con hard links; falla con código 3 si no lo admite. No se usa rename con reemplazo. Ante error, archivos completos ya publicados permanecen; no equivale a transacción de todo el trabajo ni garantiza recuperación de energía. Un proceso terminado abruptamente puede dejar temporales.

--batch-size admite 1..10000, por defecto 100; --threads admite 1 o 2, por defecto 1. Cada grupo se procesa en ventanas de hasta --threads variantes, publicando/liberando cada artefacto en el trabajador. No se guardan N imágenes en RAM. N=1 limita el trabajo a una variante simultánea aunque se soliciten dos hilos. El grupo final puede ser incompleto. Pool Rayon local, con número explícito de hilos, sin depender de RAYON_NUM_THREADS.

Los trabajadores reservan el presupuesto estimado de su PNG antes de rasterizar; la suma de reservas no supera 512 MiB. Si dos imágenes no caben juntas, una espera hasta que la otra publique y libere los bytes. No se divide el límite por hilos, evitando rechazar una imagen que cabe sola. Esta cifra corresponde a buffers estimados por el planificador de resvg, **no a RSS total**: árboles, generación, asignador y runtime quedan fuera. SVG se acota por ventanas e importación. No hay afirmación de aceleración; perfil/medición de memoria quedan en 6.6.

Los resultados de stdout/manifiesto salen en orden global, independientemente del orden de publicación de los trabajadores. Se drena toda la ventana, incluso si un trabajador falla o se cancela. No existe reanudación automática.

## Manifiesto incremental y Ctrl+C (6.4)

Cada generate crea --out/manifest.jsonl con create_new; un manifiesto existente provoca exit 3 antes de generar variantes. La cabecera kind=job guarda el GenerationSpec normalizado, configurationHash y backend. A continuación persiste cada resultado/rechazo y, al terminar, un resumen status=completed|cancelled|failed con accepted/rejected reales. No hay timestamps ni se modifica el contrato v1. Los números de la spec se serializan según los tipos Rust (por ejemplo 1.0).

Cada línea se escribe completa con write_all y flush antes de enviarla a stdout. Un rechazo válido no es cancelación ni fallo. Una variante que se cancela antes de publicar no cuenta como rechazada. Ante stdout roto, se sigue registrando el resto de publicaciones de la ventana y un resumen failed en manifiesto; exit 3. Si solo falla stdout al enviar el resumen final, el manifiesto puede estar completed porque los artefactos y sus registros sí terminaron; exit 3 sigue señalando la entrega incompleta a stdout.

Ante fallo del manifiesto se detiene con exit 3, sin emitir resumen completed a stdout: puede quedar la última línea incompleta y algún artefacto completo sin registro, porque publicación y manifiesto no forman una transacción. Solo consumir líneas completas. Ausencia de resumen completed significa que el trabajo no está confirmado como completo. No se promete durabilidad ante pérdida de energía ni recuperación automática.

Ctrl+C activa AtomicBool; se comprueba antes/después de generar, al reservar buffers, después de exportar y antes de cada ventana. La espera de presupuesto revisa el token cada 50 ms. Las llamadas individuales de generación/rasterización/escritura en curso no se interrumpen a mitad: puede tardar en responder hasta que termine la etapa. Una publicación ya iniciada termina de forma íntegra y se registra. Se esperan los trabajadores de la ventana, se escribe resumen cancelled y se devuelve 130. Si todas las variantes ya terminaron antes de observar la cancelación, el trabajo puede finalizar normalmente.

Prueba local Windows: scripts/verify-cli-cancellation-windows.py abre una consola oculta propia y comprueba que solo participan el verificador y su hijo antes de enviar CTRL_C_EVENT. No afecta la consola del usuario. Limpia la opción heredable de ignorar Ctrl+C en el verificador antes de crear el hijo; un host que ignore esa señal también puede transmitir esa opción a procesos hijos. Crea out/cli-6.4-cancel-signal exclusivamente y requiere que esa ruta no exista al ejecutar la prueba.

| Código | Estado |
|---|---|
| 0 | inspect/generate completado, ayuda o versión |
| 2 | Argumentos/contrato inválidos, hash incorrecto, SVG inseguro/malformado o >5 MiB, spec >1 MiB |
| 3 | Fallo al abrir/leer/escribir/publicar/renderizar, colisión o ruta no regular |
| 130 | Cancelación cooperativa observada; artefactos completos y resultados previos conservados |

stderr usa JSON con error.code y error.message. No incluye el contenido SVG. --help/--version producen texto en stdout. Un fallo del destino stdout devuelve 3; el consumidor debe comprobar el código de salida, porque un stream roto puede haber recibido datos parciales.

Se comprueba archivo regular antes de abrir y de nuevo sobre el handle abierto. La lectura se limita a 5 MiB + un byte, incluso si el archivo crece tras consultar sus metadatos; el byte extra permite detectar exceso antes de importar. No equivale a una sandbox del sistema de archivos ni impone un timeout a almacenamiento remoto.

El inspector conserva todos los límites/restricciones de docs/svg-admitido.md. No instala fuentes ni permite recursos externos. Los metadatos del comando son una salida local; todavía no representan API HTTP ni contrato entre lenguajes.

## PDF único desde CLI (6.5)

Desde la raíz:

```powershell
cargo run --manifest-path services/engine/Cargo.toml -p mosaicforge-cli --locked -- generate-pdf --template "archivo.svg" --spec "trabajo-pdf.json" --out "out/pdf-nuevo" --batch-size 7
```

trabajo-pdf.json usa el contrato existente contracts/engine/pdf/v1/job.schema.json: schemaVersion="1", generation con el GenerationSpec habitual (output.format="svg") y pdf con exporterVersion="1", trimWidthMm, trimHeightMm y bleedMm. Primera muestra: 100, 100 y 0 mm respectivamente. Esas dimensiones físicas son independientes del tamaño del viewport de generación. templateHash debe coincidir con inspect.

El comando escribe **un solo mosaic.pdf para todo el trabajo**, una página por variante aceptada en orden de índice global. Un grupo N no crea otro documento; el grupo final puede estar incompleto. --batch-size admite 1..10000, por defecto 100. El pipeline PDF es secuencial, con una página/artefacto de variante a la vez y metadata de objetos/páginas; no acepta --threads. No se prometen mejoras de velocidad ni memoria de proceso por cambiar N.

No publica SVG/PNG individuales. Escribe el PDF por objetos en un temporal exclusivo del directorio final, cierra el documento/xref, sincroniza y publica mediante el mismo hard_link sin reemplazo usado por SVG/PNG. Si no hay aceptadas, devuelve resultado no_pages y no crea PDF vacío. Un manifiesto o PDF existente no se sobrescribe. El contenido conserva el exportador vectorial con recorte real de 5.6; no es PDF/X ni conversión CMYK.

manifest.jsonl empieza con kind=job, format=pdf y PdfJobSpec normalizado. Después emite kind=pdf_page con record del contrato PdfPageRecord: variant y pageNumber (base 1, solo aceptadas, sin huecos). artifactHash de cada variante corresponde al SVG previo al exportador PDF; **no es el hash de la página ni del documento**. Estas páginas son provisionales hasta el cierre y publicación.

Solo después de publicar aparece kind=pdf_document con result del contrato PdfDocumentResult y artifact="mosaic.pdf"; documentHash es SHA-256 de todos los bytes finales. Con no_pages, artifact es null y documentHash ausente/null. El resumen completed/cancelled/failed conserva los recuentos de variantes procesadas. stdout replica los registros de progreso/documento/resumen; la cabecera solo queda en manifiesto.

Ctrl+C se comprueba entre generación, optimización, páginas y antes de cierre/publicación: salida 130 y eliminación del temporal. Las páginas ya registradas quedan como progreso provisional, sin documento/hash final. No se interrumpe una operación individual en curso. Si la publicación ya comenzó al llegar la señal, puede finalizar con el PDF completo.

Errores de manifiesto/stdout previos a publicar descartan el temporal; salida 3, sin PDF truncado publicado. Después de la publicación atómica, un fallo al registrar el documento puede dejar un PDF **completo** sin registro final, porque archivo y manifiesto no son una transacción. No interpretar las páginas provisionales ni un archivo preexistente como confirmación del nuevo trabajo. Nunca hay reemplazo ni reanudación automática.

Prueba local de señal PDF Windows: scripts/verify-cli-cancellation-windows.py --pdf. Usa el mismo aislamiento de consola de 6.4 y un directorio exclusivo out/cli-6.5-pdf-cancel-signal; requiere la plantilla preparada y la spec de la prueba local previa.

## Comandos de verificación

Desde services/engine:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --quiet
cargo test -p mosaicforge-cli --locked
```

Continuidad y pruebas en avance-tarea-6.1.md; pasos siguientes en tarea-6-submodulos.md. Fiery está fuera del alcance por decisión del usuario, no es requisito de cierre de fase 1.
