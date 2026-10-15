# Ejecución del motor CPU

Actualizado: 2026-10-15, submódulos 6.2–6.3. La CLI expone **inspect**, **generate SVG/PNG**, grupos y uno/dos hilos, --help y --version. PDF CLI pendiente de 6.5; las funciones Rust PDF ya existen.

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

El padre de --out debe existir. Permite un directorio existente, pero nunca reemplaza archivos: publica variant-{índice}.svg o .png según spec.output.format. El índice no depende de la posición en el grupo. stdout emite una línea JSON por resultado (kind=variant, result con el contrato VariantResult) y resumen final (kind=summary, accepted/rejected). El rechazo válido no escribe artefacto y devuelve 0. No se acumulan todos los artefactos del trabajo.

El JSON spec admite hasta 1 MiB; contrato/hash se comprueban antes de crear directorio. Temporales exclusivos se escriben/sincronizan/cierran antes de publicar por hard link. Se requiere filesystem compatible con hard links; falla con código 3 si no lo admite. No se usa rename con reemplazo. Ante error, archivos completos ya publicados permanecen; no equivale a transacción de todo el trabajo ni garantiza recuperación de energía. Un proceso terminado abruptamente puede dejar temporales. Manifiesto persistente y cancelación cooperativa llegan en 6.4.

--batch-size admite 1..10000, por defecto 100; --threads admite 1 o 2, por defecto 1. Cada grupo se procesa en ventanas de hasta --threads variantes, publicando/liberando cada artefacto en el trabajador. No se guardan N imágenes en RAM. N=1 limita el trabajo a una variante simultánea aunque se soliciten dos hilos. El grupo final puede ser incompleto. Pool Rayon local, con número explícito de hilos, sin depender de RAYON_NUM_THREADS.

Los trabajadores reservan el presupuesto estimado de su PNG antes de rasterizar; la suma de reservas no supera 512 MiB. Si dos imágenes no caben juntas, una espera hasta que la otra publique y libere los bytes. No se divide el límite por hilos, evitando rechazar una imagen que cabe sola. Esta cifra corresponde a buffers estimados por el planificador de resvg, **no a RSS total**: árboles, generación, asignador y runtime quedan fuera. SVG se acota por ventanas e importación. No hay afirmación de aceleración; perfil/medición de memoria quedan en 6.6.

Los resultados de stdout salen en orden global, independientemente del orden de publicación de los trabajadores. Ante error, otro trabajador de la misma ventana puede haber publicado un archivo completo cuyo resultado todavía no se emitió; comprobar siempre exit code y contenido del directorio antes de reintentar. No existe todavía reanudación automática ni manifiesto persistente.

| Código | Estado |
|---|---|
| 0 | inspect/generate completado, ayuda o versión |
| 2 | Argumentos/contrato inválidos, hash incorrecto, SVG inseguro/malformado o >5 MiB, spec >1 MiB |
| 3 | Fallo al abrir/leer/escribir/publicar/renderizar, colisión o ruta no regular |

stderr usa JSON con error.code y error.message. No incluye el contenido SVG. --help/--version producen texto en stdout. Un fallo del destino stdout devuelve 3; el consumidor debe comprobar el código de salida, porque un stream roto puede haber recibido datos parciales.

Se comprueba archivo regular antes de abrir y de nuevo sobre el handle abierto. La lectura se limita a 5 MiB + un byte, incluso si el archivo crece tras consultar sus metadatos; el byte extra permite detectar exceso antes de importar. No equivale a una sandbox del sistema de archivos ni impone un timeout a almacenamiento remoto.

El inspector conserva todos los límites/restricciones de docs/svg-admitido.md. No instala fuentes ni permite recursos externos. Los metadatos del comando son una salida local; todavía no representan API HTTP ni contrato entre lenguajes.

## Verificación

Desde services/engine:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --quiet
cargo test -p mosaicforge-cli --locked
```

Continuidad y pruebas en avance-tarea-6.1.md; pasos siguientes en tarea-6-submodulos.md. Fiery está fuera del alcance por decisión del usuario, no es requisito de cierre de fase 1.
