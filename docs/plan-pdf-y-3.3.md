# PDF binario y fase 3.3

**Objetivo:** exportar un único PDF vectorial mayor de 12 MiB y después incorporar persistencia de usuarios/proyectos en PostgreSQL, ya elegido en ADR0001/0002.
**Autorización:** petición del usuario del 2026-10-19: arreglar PDF y continuar con 3.3. Ejecución incremental con revisión independiente; sin workers/cola de 3.4.

## PDF: diseño y criterios

Añadir `/v3/export-pdf-selection` con la solicitud v2 existente. La respuesta nueva es binaria `application/pdf`; `x-mosaicforge-result` contiene metadatos JSON validados por contrato nuevo (schemaVersion3, requestId, document, variantIndices). Los endpoints JSON anteriores permanecen compatibles. Rust genera en archivo temporal privado y solo publica el documento completo; Java verifica identidad, tamaño y hash antes de retransmitir; Next transmite el cuerpo sin base64. Archivos temporales se eliminan al finalizar/fallar/desconectar. Una cuota de disco explícita sustituye el tope de memoria; no se promete tamaño infinito.

- [x] ~~Contrato aditivo, generación de tipos/esquemas/clientes y tests.~~
- [x] ~~Rust: archivo temporal, límite de disco 1 GiB, lectura acotada, limpieza y rechazo antes de publicar.~~
- [x] ~~Java: recepción a archivo, verificación SHA256 y metadatos, transmisión y limpieza.~~
- [x] ~~Web: validar cabecera/identidad y retransmitir; conservar controles de origen, token y timeout.~~
- [x] ~~Tests de errores, cancelación y paridad; PDF real de test.svg de 12 páginas >12 MiB, vectorial y completo.~~
- [x] ~~Checkpoint Git y Docker actualizado, solo puerto3000 publicado.~~

## 3.3: diseño y criterios

Crear dominio Java pequeño de usuarios/proyectos, puerto de repositorio y adaptador JDBC PostgreSQL con migración inicial versionada. API privada/local autenticada por el mecanismo actual; no publicar registro ni simular autenticación multiusuario. Contratos versionados para crear/listar/consultar proyectos y guardar configuraciones de plantilla como revisiones inmutables. PostgreSQL en Compose sin puerto host y con volumen duradero. Credenciales por entorno y separadas de procesos web/motor.

- [ ] Leer alcance vigente y registrar ADR de persistencia y modelo de pertenencia local.
- [ ] Contratos antes de Java; validación de propietario, UUID, longitudes y configuración.
- [ ] Implementar repositorio transaccional y endpoints; sin dependencia JDBC en dominio.
- [ ] Probar creación/lectura, aislamiento de propietario, revisiones, conflictos y persistencia tras reinicio real de API/BD.
- [ ] Verificar formato, compilación, tests, migración repetible y ausencia de puertos extra publicados.
- [ ] Documentar límites y comandos, checkpoint Git final.

## Riesgos a verificar

PDF fallido no debe devolver200 ni archivo parcial; upstream incompleto no debe guardarse como PDF; las cuotas deben operar sin cargar el documento completo; reinicio no debe borrar proyectos; las revisiones deben conservar configuración/seed sin reinterpretación.

## Aclaración del usuario: un millón de páginas

Eliminar base64 no reduce el peso intrínseco de los vectores. La prueba real de9 páginas dio9731218 bytes; extrapolar linealmente da aproximadamente1,08TB por millón, únicamente una estimación. El writer actual ya comprime streams con Flate y limita el número de objetos a1000000 y los bytes a9999999999; usa xref clásico, por lo que no es el exporter final para esa escala. Antes de anunciar un millón hay que medir1000→10000→100000→1000000, evaluar recursos reutilizables sin restaurar geometría oculta costosa, resolver índices/estructura PDF de gran tamaño, disco, recuperación y lectores reales. Se conserva el requisito de PDF único y vectorial. El arreglo local de este plan no certifica esa capacidad.

El cliente interactivo todavía recibe un Blob antes de guardar: el streaming nuevo acota la memoria de los servicios, no garantiza memoria del navegador independiente del tamaño. La descarga masiva deberá usar un artefacto durable completado y descarga nativa/reanudable mediante URL autorizada, sin Blob JavaScript del archivo entero. Ese flujo pertenece al trabajo durable y almacenamiento de3.4, con cuotas/preflight y cancelación verificadas, no a subir el número de previews.
