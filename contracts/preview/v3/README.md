# Binary PDF download

POST `/v3/export-pdf-selection` accepts the unchanged selected-PDF v2 request.
On success it returns `application/pdf`, a decimal Content-Length and
`x-mosaicforge-result`: ASCII JSON matching `pdf-download-response.schema.json`.
The metadata identifies request, template, seed, selected indices, complete
page count and SHA256 of the raw body (`document.documentHash`). Receivers
must verify identity, actual length and digest; incomplete streams are failures.
Errors before publication use the existing JSON error contract. No base64 PDF
body is used. v1/v2 remain available unchanged.
