# ADR0010 — descarga binaria de PDF completado

Aceptado por la petición del usuario de arreglar el límite de PDF (2026-10-19).

El transporte JSON/base64 limita documentos a unos12 MiB y multiplica copias de memoria. Añadimos un endpoint v3 binario con metadatos de identidad en cabecera y mantenemos v1/v2. Rust prepara un archivo temporal completo antes de responder; Java verifica tamaño/hash e identidad en disco antes de publicar. Next transmite por bloques. Esto consume disco temporal y retrasa el primer byte, pero permite responder con error antes de publicar un PDF incompleto. El límite inicial es1 GiB; sigue siendo una vista local de hasta12 variantes, no el sistema masivo de trabajos.

No se altera el generador vectorial ni el determinismo. La frontera nueva es una representación HTTP de la frontera existente Rust→Java→web, no un servicio adicional. No se usa memoria compartida ni rutas de archivo entre servicios.
