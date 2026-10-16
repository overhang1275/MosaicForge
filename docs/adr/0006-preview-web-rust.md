# ADR 0006 — Frontera local de previsualización web–Rust

Fecha: 2026-10-15. Estado: implementado y verificado en 2.1–2.3; editor pendiente de 2.4.

## Decisión

Next.js sirve una interfaz local y un proxy delgado hacia un proceso Rust en 127.0.0.1. Rust importa y valida SVG, genera y exporta con core/cpu existentes. Los mensajes de contracts/preview/v1 envuelven GenerationSpec y VariantResult; tipos generados para ambos lenguajes. No se duplica el motor en TypeScript.

Tres operaciones POST: inspect, preview PNG y export SVG/PNG. Una variante por petición, count=1, máximo 512 píxeles por eje. La futura UI solicita hasta 12 índices secuencialmente, sin redondear seed/índice u64 en JavaScript. Sin almacenar plantillas ni resultados entre peticiones; volver a importar tiene coste aceptado y pendiente de medir en el flujo real.

## Alternativas y coste

Ejecutar CLI desde Node exige gestionar procesos, temporales y cancelación en otra frontera. WebAssembly requiere validar compatibilidad y memoria del rasterizador en otro entorno. HTTP mantiene las responsabilidades actuales con un transporte nuevo y dos procesos locales; añade contratos, errores de red, configuración y pruebas de integración. No promete aceleraciones.

## Límites y controles implementados en 2.2/2.3

Un cálculo activo en Rust sin cola ilimitada; segunda petición devuelve 429. Cuerpo JSON hasta 32 MiB, SVG decodificado hasta 5 MiB UTF-8, spec hasta 1 MiB, respuesta hasta 16 MiB. Comprobar tamaño durante lectura antes de parsear y después de decodificar; no son límites de RSS.

Rust y Next.js solo en loopback. Token compartido obligatorio del lado servidor, fuera de Git/logs/bundles públicos. Dirección de upstream fija y validada, Host comprobado y sin CORS abierto. BFF exige Origin local configurado y JSON, sin rutas del host ni URLs de plantillas recibidas del cliente. PNG en previews; SVG solo descarga attachment y política de contenido adecuada. No incrustar SVG subido.

Espera BFF de 30 segundos; desconectar no garantiza interrumpir el cálculo ya iniciado. El permiso se retiene hasta terminar. requestId correlaciona respuestas, sin entrar en hashes de reproducción. Sin contenido SVG o secretos en mensajes/logs.

## Evolución y verificación

Fase 3 sustituye el upstream web directo por Java; este contrato local no sustituye la API pública ni autenticación/autorización multiusuario. Sin base de datos, broker, usuarios o proyectos en fase 2. No exponer a LAN/Internet.

2.1 verifica modelos, restricciones y fixtures cruzados. 2.2 verifica HTTP y paridad de bytes/hashes con CLI; 2.3 verifica proxy y desconexiones; 2.4–2.6 editor, respuestas obsoletas, accesibilidad y E2E real. Estos controles futuros no se consideran implementados por escribir este ADR. Diseño: ../superpowers/specs/2026-10-15-fase-2-web-design.md.

Evidencia ejecutada: ../avance-fase-2.2-2.3.md. Host HTTP exacto y Origin configurado se verifican en BFF; Next.js normaliza la URL interna loopback a localhost. Errores Rust previos al parseo usan UUID cero y el BFF publica correlación propia.
