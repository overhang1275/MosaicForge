# Transporte local de preview v1

Estado: contratos, HTTP Rust y BFF local implementados en 2.1–2.3 y verificados en Rust/Node y con procesos reales. ADR: ../../../docs/adr/0006-preview-web-rust.md. Editor 2.4 verificado; ampliación aditiva PDF vectorial el 2026-10-16.

Objetos cerrados, JSON Schema draft-07. Tipos generados; no copiar GenerationSpec/VariantResult del motor. Mensajes:

| POST | Entrada | Salida 200 |
|---|---|---|
| /v1/inspect | InspectRequest {requestId, templateSvg} | InspectResponse {requestId, templateHash, width, height, viewBox, layerIds, segmentCount, engineVersion, rulesVersion} |
| /v1/preview | GenerationRequest {requestId, templateSvg, generation} | GenerationResponse {requestId, variant, artifact?} |
| /v1/export | GenerationRequest | GenerationResponse |

requestId: UUID canónico minúsculo creado por BFF y devuelto por Rust; no participa en hashes. Artifact {mimeType, base64}, MIME image/png o image/svg+xml. Seed/startIndex/index: cadenas u64 canónicas, nunca Number. Un accepted requiere artefacto, SHA-256 coincidente y ningún motivo de rechazo; rejected requiere motivo y no lleva bytes/hash. artifact omitido o null representa ausencia, igual que los opcionales del contrato del motor; accepted requiere un Artifact real. El formato solicitado se comprueba en la frontera con la petición; Operation::Preview exige PNG. Export acepta ambos formatos.

Importar SVG es responsabilidad del motor, no del validador de sobres. templateHash corresponde al SVG normalizado de Rust; el hash de configuración sigue el contrato engine/v1 y artifactHash corresponde a bytes exportados. Los identificadores de transporte no cambian esos hashes.

## Límites y validación

- SVG hasta 5 MiB en UTF-8; spec hasta 1 MiB serializada. El patrón decimal no prueba el rango u64: comprobarlo semánticamente.
- count=1; dimensiones 1..512 por eje; preview solo PNG. Generación conserva las validaciones semánticas del motor.
- InspectResponse: dimensiones positivas finitas, viewBox de cuatro números finitos con ancho/alto positivos, capas y segmentos acotados a límites del importador.
- Artefacto base64 canónico; hash, MIME y aceptación/rechazo coherentes. Rechazos válidos retornan 200 sin imagen. Este contrato no comprueba seguridad SVG ni decodificación real PNG del artefacto: solo el motor puede producirlos para publicar.
- Servidor y BFF limitan cuerpo JSON a 32 MiB durante lectura y respuesta a 16 MiB. Validar tipos no sustituye esas restricciones de transporte.

PreviewError {requestId, code, message}: mensaje de 1..256 caracteres, sin fuente ni secretos. Códigos y HTTP:

| code | HTTP |
|---|---:|
| invalid_request / invalid_template | 400 |
| unauthorized | 401 |
| forbidden | 403 |
| payload_too_large | 413 |
| unsupported_media_type | 415 |
| busy | 429 |
| internal_error | 500 |
| engine_unavailable (BFF) | 502 |
| timeout (BFF) | 504 |

Método incorrecto: 405. No reflejar texto de parsers o contenido de entrada en errores. Los campos desconocidos se rechazan también en objetos referenciados. Ajv y Rust deben coincidir en fixtures/index.json y casos de borde; generar tipos no ejecuta por sí mismo todo JSON Schema.

## Exportación principal PDF vectorial

POST /v1/export-pdf recibe PdfExportRequest {requestId, templateSvg, job}, con referencia al PdfJobSpec publicado de engine/pdf/v1. Responde PdfExportResponse {requestId, document: PdfDocumentResult, artifact: {mimeType: application/pdf, base64}}. El BFF /api/export-pdf devuelve los mismos bytes como attachment mosaic.pdf. No amplía Artifact de SVG/PNG ni cambia sus mensajes publicados.

En este transporte local, generation.count es 1..12, dimensiones lógicas 1..512 y formato svg obligatorio; seed e índices u64 exactos, sin desbordamiento. Tamaño físico 1..2000 mm por eje y sangrado 0..50 mm, según contrato PDF existente. El PDF reúne toda la serie: un rechazo aborta la respuesta y nunca publica páginas parciales. Resultado completado exige requestedCount = count = acceptedCount = pageCount, rejectedCount = 0, base64 canónico, firma PDF y SHA-256 coincidente.

Se reutiliza mosaicforge_cpu::pdf: geometría vectorial recortada/optimizada, nunca páginas construidas desde PNG. Lote interno de una página; buffer binario acotado a 12 MiB menos 64 KiB para mantener el JSON final bajo 16 MiB. No representa una ruta de producción para un millón de variantes; esa entrega necesita trabajos asíncronos por bloques en fase 3.
