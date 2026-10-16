# Transporte local de preview v1

Estado: contratos de fase 2.1 implementados y verificados en Rust/Node. Sin servidor HTTP todavía. ADR: ../../../docs/adr/0006-preview-web-rust.md.

Objetos cerrados, JSON Schema draft-07. Tipos generados; no copiar GenerationSpec/VariantResult del motor. Mensajes:

| POST futuro | Entrada | Salida 200 |
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
- El servidor futuro debe limitar cuerpo JSON a 32 MiB durante lectura y respuesta a 16 MiB; el BFF también. Validar tipos no sustituye esas restricciones de transporte.

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
