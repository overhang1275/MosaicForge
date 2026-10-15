# Ejecución del motor CPU

Actualizado: 2026-10-15, submódulo 6.2. La CLI expone **inspect**, **generate SVG/PNG**, --help y --version. PDF CLI pendiente de 6.5; las funciones Rust PDF ya existen.

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
cargo run --manifest-path services/engine/Cargo.toml -p mosaicforge-cli --locked -- generate --template "out/test-prepared.svg" --spec "out/cli-6.2-svg.spec.json" --out "out/nuevo-trabajo"
```

El padre de --out debe existir. Permite un directorio existente, pero nunca reemplaza archivos: publica variant-{índice}.svg o .png según spec.output.format. El índice no depende de la posición en el grupo. stdout emite una línea JSON por resultado (kind=variant, result con el contrato VariantResult) y resumen final (kind=summary, accepted/rejected). El rechazo válido no escribe artefacto y devuelve 0. No se acumulan todos los artefactos del trabajo.

El JSON spec admite hasta 1 MiB; contrato/hash se comprueban antes de crear directorio. Temporales exclusivos se escriben/sincronizan/cierran antes de publicar por hard link. Se requiere filesystem compatible con hard links; falla con código 3 si no lo admite. No se usa rename con reemplazo. Ante error, archivos completos ya publicados permanecen; no equivale a transacción de todo el trabajo ni garantiza recuperación de energía. Un proceso terminado abruptamente puede dejar temporales. Manifiesto persistente y cancelación cooperativa llegan en 6.4.

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
