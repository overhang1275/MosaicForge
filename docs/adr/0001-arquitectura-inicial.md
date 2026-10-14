# ADR 0001 — Arquitectura e infraestructura inicial

Fecha: 2026-10-14. Estado: aceptado por el usuario mediante «ok continua» después de presentar `docs/diseno-mvp.md`.

## Decisión

Construir primero el motor CPU autónomo, luego API/trabajos y web. Usar Java 21 con Spring Boot/Spring Security y Maven Wrapper para control, Rust para generación, Next.js/TypeScript para presentación y Python opcional para análisis.

PostgreSQL será el adaptador inicial de persistencia; Valkey Streams/caché será el de coordinación. Archivos locales al principio y un adaptador compatible con S3 después, sin elegir proveedor todavía. OpenTelemetry correlacionará las operaciones. Todos detrás de fronteras reales, sin dependencias de infraestructura en el dominio.

Favorecer dependencias open source y registrar licencias; CUDA propietaria está permitida como excepción opcional. CUDA solo tras medir CPU y disponer de hardware NVIDIA para verificar paridad y rendimiento.

## Alternativas y consecuencias

- Se descarta arrancar todos los servicios juntos: retrasa la validación del motor y aumenta dependencias operativas.
- No usar Kafka inicialmente: no existe necesidad demostrada que justifique su operación adicional.
- No generar variantes en TypeScript: crearía un segundo motor y debilitaría reproducción y contratos.
- No escribir un rasterizador CPU propio: usar bibliotecas existentes y dedicar código propio al modelo generativo.
- El equipo local permite desarrollo CPU; no se detectó NVIDIA y no se prometen pruebas CUDA aquí.
- Frameworks y adaptadores están aceptados; versiones, proveedor S3 y LICENSE siguen requiriendo su verificación/decisión correspondiente.

## Recuperación con Git

El usuario pidió guardar puntos recuperables cuando funcionen las implementaciones. Inicializar Git y guardar documentación como baseline. Cada tarea de producto termina en commit después de formato, lint y tests pertinentes. Registrar comando, resultado y hash en `docs/estado.md`.

No hacer commits con fallos conocidos sin distinguirlos explícitamente como trabajo incompleto. Para investigar regresiones preferir `git diff`, `git show`, ramas y `git revert`; no descartar trabajo mediante reset/clean sin autorización. Nunca versionar secretos, dependencias descargadas ni resultados de usuario.
