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

Los endpoints JSON v1/v2 conservan su límite de salida. POST /v3/export-pdf-selection recibe la solicitud v2 y devuelve application/pdf con Content-Length y metadatos JSON ASCII en x-mosaicforge-result (contrato preview/v3). Genera primero el documento completo en un archivo temporal privado, con cuota de 1 GiB; exceso devuelve 413 antes de publicar. Transmite bloques de hasta 64 KiB mediante un canal acotado. El permiso exclusivo se conserva durante generación y transmisión; éxito, error y desconexión eliminan el temporal. La desconexión durante generación permite terminar el cálculo en curso antes de liberar recursos. No hay base64 ni documento completo en memoria en este transporte.

## Bloques CPU duraderos (worker/v1)

`POST /internal/v1/blocks` solo existe cuando el proceso recibe
`MOSAICFORGE_JOURNAL_DIR` explícito y no vacío. El token, Host exacto, ausencia de
Origin, lectura limitada a 32 MiB/30 s y permiso exclusivo son los mismos de
preview. El cuerpo sigue los tipos generados desde `contracts/worker/v1`.
Requiere un directorio privado (0700 en Unix), sin enlaces simbólicos, un único
propietario del journal mediante bloqueo del SO y un sistema de archivos que
soporte hard links exclusivos y `sync_all` de archivos/directorios. Falla cerrado
al iniciar si no puede proporcionar esas garantías; el despliegue CPU Linux es
el entorno verificado. Los IDs UUID nunca se interpretan como rutas del cliente.

Cada bloque admite 1–10000 candidatos y 1–10000 aceptadas como máximo, con
`job.generation.count=1`. Valida hash real de plantilla e intervalo u64 antes de
trabajar; termina inmediatamente al alcanzar acceptedLimit. El índice u64 máximo
se procesa sin calcular su sucesor. Las decisiones físicas comparten la función
del exportador PDF, incluidas reglas 3 y media/sangrado. Un candidato inválido
produce un registro rechazado; un error de cómputo/transporte no cuenta como rechazo.
No publica un PDF final.

Cuotas predeterminadas: 256 MiB de contenido en journal, 4096 entradas incluyendo
la entrada temporal de publicación (hasta 4095 bloques confirmados), 4 MiB por
registro persistido y presupuesto cooperativo de 30 s comprobado entre candidatos
y antes de publicar. No interrumpe a mitad de una etapa CPU. La memoria retiene
hasta 10000 registros de decisión y una sola variante geométrica por vez. Son
límites de trabajo/estructuras, no una medición ni promesa de RSS. `JournalLimits`
permite límites menores al integrar/probar el adaptador; no hay borrado automático.
Temporales huérfanos tras caída consumen la cuota y nunca se promueven en replay.

El archivo inmutable contiene respuesta y checksum SHA-256 sobre su JSON compacto
con claves ordenadas. Se escribe un temporal exclusivo 0600, se sincroniza,
se publica mediante hard link exclusivo y se sincroniza el directorio antes de
confirmar. Repetir devuelve exactamente los bytes de respuesta persistida. El
mismo blockId con otro cuerpo es 409; corrupción detectada es 500 y conserva el
archivo para diagnóstico, sin reemplazar datos confirmados. Cuota agotada es 507;
presupuesto de ejecución agotado es 503.

Protocolo de hashes: `requestHash` es SHA-256 de los **bytes UTF-8 exactos del
cuerpo HTTP**. El cliente serializa una vez y conserva esos bytes en cada reintento;
cambiar espacios también es conflicto. `recordsSha256` es SHA-256 del array
`records` serializado como JSON compacto con claves de objetos ordenadas
lexicográficamente (`serde_json::to_value` seguido de `to_vec`, sin preserve_order).
Cada registro siempre incluye `accepted`, `index` decimal string y `reason` string
o null; no intervienen floats. `recordsId` es un UUID v4 opaco creado por Rust.
