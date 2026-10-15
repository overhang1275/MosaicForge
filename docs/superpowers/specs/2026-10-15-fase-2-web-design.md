# Fase 2 — Editor web conectado al motor CPU

Fecha: 2026-10-15. Estado: diseño aprobado explícitamente por el usuario el 2026-10-15; plan de 2.1 escrito, implementación no iniciada. Base funcional: e99fa75.

## Objetivo y autorización

El usuario confirmó iniciar la fase 2 por submódulos tras cerrar la fase 1. El propósito ya está definido: cargar un SVG propio, ajustar reglas y seed, ver variantes reales reproducibles y descargar una variante SVG o PNG. El usuario aprobó después esta especificación escrita. Plan ejecutable de 2.1: ../plans/2026-10-15-fase-2-1-contratos.md, pendiente de revisión antes de implementar.

Éxito: navegador y CLI producen los mismos bytes para la misma plantilla, GenerationSpec, índice y entorno del motor. Rechazos visibles, solicitudes antiguas sin sustituir resultados nuevos, funcionamiento local sin Java, base de datos, broker, CUDA o ML.

## Enfoques

1. **Recomendado: HTTP local Rust y BFF Next.js.** Reutiliza importación, validación, generación y exportación existentes sin arrancar procesos por petición. Añade una frontera documentada y dos procesos locales.
2. **CLI invocada por Next.js.** Reutiliza el binario pero introduce gestión de subprocesos, temporales, permisos y cancelación desde Node. No se elige para el editor interactivo.
3. **Motor WebAssembly en navegador.** Podría evitar un proceso local, pero requiere comprobar compatibilidad, memoria y rasterización del motor en otro entorno. Se difiere; no es una implementación adicional ahora.

La comparación es de diseño, sin benchmarks ni afirmaciones de rendimiento de transportes todavía no implementados.

## Arquitectura y archivos

- Nuevo crate services/engine/crates/preview: transporte HTTP local que llama a core y cpu, sin depender de módulos internos de la CLI. Conserva CLI y contratos v1 existentes.
- Nueva aplicación apps/web: Next.js, TypeScript estricto, UI y Route Handlers como proxy delgado. Sin generación, validación geométrica ni almacenamiento de proyectos en TypeScript.
- Nuevos contratos de transporte bajo contracts/preview/v1. Referencian los esquemas GenerationSpec y VariantResult existentes; no duplicar sus modelos a mano. Tipos Rust y TypeScript generados desde contratos; comprobar regeneración sin diferencias.
- ADR 0006 registra la frontera, alternativas, límites y retirada del acceso directo al motor cuando Java entre en fase 3. Se crea como decisión propuesta hasta aprobación.
- Dependencias concretas y versiones se comprueban al preparar el plan ejecutable. Todas deben tener licencia open source; no modificar LICENSE ni CI en esta fase sin autorización específica.

Flujo: navegador → rutas del mismo origen Next.js → dirección Rust fija en loopback → import_svg/generate_and_export → respuesta tipada → PNG de previsualización. No servir ni insertar el SVG original como HTML.

## Transporte propuesto

Tres operaciones POST versionadas, sin persistencia:

- /v1/inspect: recibe templateSvg como texto UTF-8 y devuelve hash normalizado, dimensiones, viewBox, capas, segmentos y versiones disponibles. El hash es del importador Rust, no del archivo original.
- /v1/preview: recibe templateSvg y GenerationSpec con count=1, startIndex del índice solicitado y formato PNG. Devuelve VariantResult y, solo al aceptar, PNG codificado en base64 con tipo MIME. Los rechazos tienen resultado y motivo, sin artefacto.
- /v1/export: mismo sobre y una variante, formato SVG o PNG. Devuelve resultado y bytes base64 del artefacto final; BFF los entrega como descarga attachment con MIME fijo y nombre derivado únicamente del índice validado. Hash calculado por Rust, sin transformar bytes.

Cada respuesta incluye requestId para correlación. La definición exacta de campos, estados y fixtures se fija en contratos antes de implementar consumidores. Mantener seed/startIndex como cadenas decimales u64: no convertir a Number en JavaScript. Verificar overflow al construir los hasta 12 índices consecutivos.

HTTP: 200 para aceptación o rechazo válido; 400 para contrato/entrada inválidos; 413 para cuerpo excesivo; 429 si el motor está ocupado; 500 para error interno sin detalles sensibles. El BFF distingue motor inaccesible (502) y espera agotada (504). Ningún error incluye contenido SVG o secretos.

Límites iniciales, no capacidad medida:

- SVG UTF-8 hasta 5 MiB, mismos límites estructurales del importador; spec hasta 1 MiB. Cuerpo JSON de transporte máximo 32 MiB para admitir escapes, comprobado antes de parsear y también durante lectura de cuerpos sin Content-Length. Comprobar además el tamaño real de campos tras decodificar.
- Preview y descarga de esta fase: 1..512 píxeles por eje. No ampliar silenciosamente a 4096 ni a lotes CLI desde HTTP.
- Una operación de cómputo activa por proceso Rust, sin cola ilimitada. Segunda solicitud devuelve 429; la web puede reintentar manualmente. Reserva PNG estimada reutilizada, sin prometer límite de RSS del proceso.
- Una variante por respuesta, respuesta JSON hasta 16 MiB; errores explícitos si supera el límite. La web conserva como máximo 12 previews y libera Blob URLs al sustituirlas.
- BFF con espera máxima de 30 segundos; abortar fetch no promete interrumpir inmediatamente el cálculo Rust ya iniciado. El permiso de cómputo se conserva hasta que finalice, incluso si el cliente se desconecta. No lanzar trabajo CPU bloqueante en el ejecutor asíncrono de red.

No se cachean ni guardan plantillas en el servidor. Cada operación vuelve a suministrar el SVG; aceptar ese coste para evitar una sesión/almacén prematuros. Los límites y la repetición de importación deberán medirse en la prueba real.

## Seguridad del uso local

Rust escucha solo en 127.0.0.1. El BFF usa una URL configurada del lado servidor, validada como loopback y sin aceptar destinos/rutas arbitrarios del navegador. El browser nunca recibe la credencial del motor.

Cada arranque local usa un token compartido entre Rust y BFF mediante configuración fuera del repositorio, sin valor por defecto. Rust rechaza peticiones sin token válido y hosts inesperados; sin CORS abierto. El launcher/documentación deben permitir crear el token sin imprimirlo en logs ni incluirlo en bundles NEXT_PUBLIC.

Next.js se ejecuta también en loopback para esta fase. Sus POST requieren JSON y Origin coincidente con el origen local configurado; rechazar orígenes ajenos. Sin fetch de URLs de plantillas ni rutas de archivos del host recibidas por HTTP. CSP y Content-Disposition apropiados para descargas SVG; UI muestra PNG del motor mediante Blob, nunca SVG subido incrustado.

Sin autenticación multiusuario ni exposición a LAN/Internet: ese alcance necesita la API de fase 3. Ningún log registra SVG, cuerpo completo o tokens; requestId se propaga entre ambos procesos.

## Interfaz mínima

Una pantalla en español con selector de SVG, resumen de importación, controles y galería. Carga explícita, sin envío automático de archivos al seleccionarlos hasta accionar importar. Mensajes de formato no admitido y límite de tamaño legibles.

Controles iniciales: seed, índice inicial, cantidad 1..12, rangos de traslación/rotación/escala/opacidad, espejos, mezcla de capas, distorsión, paleta opcional, crop opcional y fondo. Valores iniciales válidos y visibles; conservar configuración al corregir un error. El motor es la autoridad de validación, sin copiar sus reglas de generación a la UI.

Botón Generar inicia solicitudes secuenciales. Editar configuración o importar otra plantilla invalida la serie anterior; AbortController y un identificador de revisión evitan publicar respuestas antiguas. No lanzar una solicitud por cada pulsación. Estados por tarjeta: pendiente, generando, aceptada, rechazada o error. Conservar índice/motivo en rechazos y mostrar seed/hash/versiones para reproducción.

Descarga por tarjeta usa una copia inmutable de plantilla/configuración de ese resultado; no combina una tarjeta vieja con controles actuales. Si se cambia el formato SVG/PNG se conserva el resto de parámetros y el índice; el hash de artefacto puede cambiar con el formato. Comparar bytes con CLI del formato correspondiente.

Etiquetas y controles accesibles por teclado, foco visible y mensajes anunciados. Sin cuentas, proyectos, guardado de presets, exportación PDF web ni lotes masivos aquí. El PDF único de la CLI sigue disponible.

## Verificación requerida

1. Contratos: fixtures válidos/invalidos, tipos generados reproducibles, u64 máximo preservado, campos desconocidos y límites de preview rechazados.
2. Rust HTTP: importación maliciosa, hash incorrecto, límites antes/después de decodificar, host/token inválidos, ocupación, fallo de cómputo, error sin filtración y liberación del permiso al desconectar. CLI previa conserva sus tests.
3. Paridad: bytes/hash PNG y SVG HTTP iguales a CLI para fixture, seed e índice fijados; rechazo coincide con motor directo.
4. BFF: destino fijo, Origin, tamaño, tiempo agotado, motor ausente, códigos y descarga sin cambiar bytes.
5. UI: reglas e índices enviados correctamente, u64 sin pérdida, respuesta obsoleta ignorada, cambio de plantilla, URLs liberadas, descarga asociada a la configuración original, teclado y errores.
6. E2E real: importar fixture, generar, cambiar una regla, observar preview real y descargar; comparar con CLI. Probar copia preparada de test.svg y documentar sus limitaciones de importación.
7. Formato/Clippy/tests CPU; typecheck/lint/tests web/E2E y build standalone. Registrar qué se ejecutó y qué no. Medir duración/memoria del flujo local sin prometer aceleraciones.

## Submódulos y pausas

- **2.1 Diseño y contratos:** revisar esta propuesta; después escribir plan ejecutable, ADR y contratos de transporte con fixtures/generación verificada. No instalar Next.js todavía.
- **2.2 Transporte Rust:** endpoints y límites con pruebas y paridad CLI.
- **2.3 Base Next.js y BFF:** tipos generados, proxy local, errores y build reproducible.
- **2.4 Editor y galería:** importación, controles, previews reales y descargas asociadas al resultado.
- **2.5 Robustez y accesibilidad:** respuestas obsoletas, desconexión, límites y teclado.
- **2.6 Cierre:** E2E real, métricas locales, revisión, documentación y checkpoint.

Cada submódulo termina con evidencia en docs/, checklist tachada y commit de implementación verificada; pedir confirmación antes del siguiente. No iniciar fase 3 automáticamente.
