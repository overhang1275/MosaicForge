# Avance 6.4 — manifiesto y cancelación

2026-10-15. Base 3088cc7. Usuario autoriza solo 6.4; detenerse antes de 6.5. Se conserva la implementación en la rama actual y el contrato generado v1.

- [x] ~~Prueba roja: manifiesto ausente. Nuevo test también comprueba no reemplazo y conserva bytes al reintentar.~~
- [x] ~~manifest.jsonl exclusivo: cabecera con spec/metadatos reproducibles, resultados/rechazos ordenados y resumen completed/cancelled/failed. Cada registro se guarda antes de stdout.~~
- [x] ~~Ctrl+C establece AtomicBool; comprobaciones entre etapas/ventanas y espera de presupuesto interrumpible. Código 130 observado en proceso Windows real.~~
- [x] ~~Se drenan todos los resultados de la ventana incluso si otro falla/se cancela, registrando las publicaciones completas.~~
- [x] ~~Tests con writer/token: cancelación previa, entre grupos/ventanas, hilos 1|2, reserva en espera, error parcial/flush de manifiesto, stdout roto y colisión de un trabajador.~~
- [x] ~~fmt, Clippy y cargo test --workspace --locked --offline --quiet: 122 tests pasan, nueve locales omitidos, cero fallos; corridas CLI reales e inventario registrados.~~
- [x] ~~Implementación/documentación preparadas para checkpoint Git de 6.4; se detiene antes de 6.5.~~

Ruling: manifiesto incremental visible durante ejecución, creado con create_new y nunca reemplazado; no temporal con publicación al final, porque se perdería el seguimiento incremental. Ante interrupción abrupta/E/S puede quedar una última línea incompleta: solo las líneas completas previas son válidas, ausencia de resumen completed significa trabajo incompleto. No se promete transacción ni recuperación automática. Un artefacto completo puede quedar sin registro si falla el almacenamiento del manifiesto después de publicarlo; error 3 explícito y sin resumen de éxito.

Ruling: se usa ctrlc, librería open source para la frontera real con señales Windows/Unix, no un manejador nativo propio. Registro una vez por proceso generate. El handler solo activa el token. Referencia primaria: [ctrlc set_handler](https://docs.rs/ctrlc/latest/ctrlc/fn.set_handler.html).

El manifiesto contiene una cabecera kind=job con spec completo y configurationHash/backend, seguida de los registros de stdout. Resumen incluye status y recuentos aceptados/rechazados reales. Una variante cancelada no se cuenta como rechazo válido. La última ventana termina y drena sus publicaciones; generación/rasterización/escritura individual en curso no se aborta dentro de la llamada del motor. El perfil de latencia de cancelación corresponde a 6.6.

Revisión independiente global al terminar 6.6 según executing-plans; no subagente por este submódulo.

## Evidencia y depuración

La primera corrida CLI después de implementar tuvo un fallo en manifest_contains_job_then_exact_stdout_records_and_never_gets_replaced: el test comparaba JSON de entrada con enteros contra la serialización del contrato, cuyos rangos son f64 (1 frente a 1.0). Se confirmó en el diff completo del assert; se corrigió la expectativa convirtiendo también la entrada al GenerationSpec generado. No se cambió la semántica del motor. El test aislado pasó después.

Prueba de señal real: scripts/verify-cli-cancellation-windows.py, consola nueva oculta, solo helper e hijo en GetConsoleProcessList. El primer intento en out/cli-6.4-cancel agotó el timeout de 90 segundos y el verificador terminó únicamente su propio hijo; heredaba la opción de ignorar CTRL_C_EVENT del host. Se verificó el comportamiento documentado por Windows y se corrigió solo el harness, limpiando ese flag heredable antes de crear el hijo. No se atribuye ese intento a cancelación correcta. Sus archivos locales se conservan como diagnóstico, sin resumen completed.

La repetición corregida en out/cli-6.4-cancel-signal terminó con exit 130: dos PNG aceptados, cero rechazados, cuatro registros de manifiesto (cabecera, dos variantes y resumen cancelled), cero artefactos sin registro y cero temporales. Se compararon los hashes SHA-256 con cada registro y stdout coincide con el manifiesto sin cabecera. Evidencia: out/cli-6.4-cancel-signal/verification.json, stdout.jsonl, stderr.json y artifacts/manifest.jsonl. El registro de espec/datos sigue las reglas CPU del usuario, sin timestamps en hashes.

Prueba real completada: out/cli-6.4-svg/manifest.jsonl, tres registros (job, variante 0, resumen completed, accepted=1/rejected=0), exit 0. SVG SHA-256 b034fb2c67793a19fc7ba0d89ec575a693a738d83b8702d1fdd5ba22c7762c52, idéntico al de 6.2/6.3.

Referencias primarias consultadas: [GenerateConsoleCtrlEvent](https://learn.microsoft.com/en-us/windows/console/generateconsolectrlevent), [AttachConsole](https://learn.microsoft.com/en-us/windows/console/attachconsole). No se envió señal a la consola del usuario.

Dependencias: ctrlc 3.5.2 y ocho transitivas del lock, 103 paquetes externos con licencias open source declaradas; inventario detallado en dependencias-motor.md. Solo Windows ejecutado; no se probó Linux/macOS. No se midió latencia de cancelación ni RSS/rendimiento. Revisar condiciones heredadas de consola al automatizar señales.
