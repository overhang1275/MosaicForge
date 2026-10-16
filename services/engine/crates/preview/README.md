# Preview HTTP local

Crate CPU que reutiliza core/cpu; no depende de la CLI. Ejecutable: mosaicforge-preview.

Desde services/engine:

```powershell
cargo build -p mosaicforge-preview -p mosaicforge-cli --locked
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Escucha exclusivamente 127.0.0.1, puerto MOSAICFORGE_ENGINE_PORT (4771 por defecto). MOSAICFORGE_ENGINE_TOKEN es obligatorio, entre 32 y 256 bytes ASCII gráficos. No imprimirlo, guardarlo en Git ni usar NEXT_PUBLIC. El launcher local scripts/start-preview.mjs genera el token en memoria y lo comparte con Next.js.

POST /v1/inspect, /v1/preview, /v1/export y /v1/export-pdf siguen contracts/preview/v1. Host debe coincidir exactamente con la dirección/puerto de escucha; Authorization: Bearer con token, JSON obligatorio y ningún Origin permitido (el cliente es el BFF). Sin CORS, URLs remotas, rutas de archivos ni persistencia. Método incorrecto: 405.

Un permiso exclusivo cubre lectura y cómputo. Segunda operación: 429, sin cola. Cálculo/importación/parseo en spawn_blocking; desconexión no libera su permiso hasta terminar. El timeout de lectura es 30 s; no hay promesa de interrupción inmediata del render. Límite streaming JSON 32 MiB; campos SVG 5 MiB UTF-8/spec 1 MiB; salida serializada 16 MiB. No son cotas de RSS.

requestId válido se devuelve desde el contrato parseado. Los errores anteriores al parseo (auth, cuerpo, ocupación) usan UUID cero, que el BFF sustituye por su correlación propia; nunca se refleja contenido del error interno. Los rechazos de variantes son 200 sin artefacto. Errores internos/panic se convierten en 500, con liberación del permiso.

Prueba real desde raíz: node scripts/preview-smoke.mjs --engine-only. Requiere binarios debug compilados; usa puertos efímeros y token temporal en memoria, conserva resultados en out y cierra sus procesos. Incluye límite chunked, 429/desconexión, rechazo y bytes/hashes SVG/PNG iguales a CLI, con seed/índice u64 máximo.
