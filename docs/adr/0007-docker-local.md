# ADR 0007 — Imagen local CPU con Rust y Next.js

Fecha: 2026-10-16. Estado: aceptado para pruebas locales.

## Decisión

Empaquetar el servidor Rust preview en release y Next.js standalone de producción en una imagen Linux multietapa. Docker Compose publica únicamente la web en 127.0.0.1 del host. Ambos procesos comparten contenedor para conservar la frontera HTTP privada en loopback definida en ADR 0006, sin relajar las validaciones de upstream o Host del motor. Next.js escucha en las interfaces del contenedor, bajo publicación exclusiva al loopback del host.

Un lanzador mínimo genera un token por arranque en memoria y termina ambos procesos si uno falla. Compose aporta init para recolección de procesos, healthcheck y reinicio. No hay secretos de build, montajes del código del host ni servicios nuevos de infraestructura. El origen externo usa el mismo puerto publicado, configurable para pruebas con una instancia host existente.

## Consecuencias

Una reconstrucción y recreación actualiza ambos componentes juntos. La imagen no proporciona despliegue multiusuario ni exposición a LAN/Internet. CPU sigue siendo la referencia; CUDA permanece opcional y fuera de esta imagen. La imagen final lleva solo Node, la salida standalone y el binario Rust, sin compiladores ni dependencias de desarrollo.

Dos contenedores con red compartida añadirían coordinación del ciclo de vida y del token sin aportar aislamiento de red en esta fase. Dos redes separadas exigirían cambiar la frontera privada actual. Se reconsiderará al integrar el plano de control Java.

## Verificación ejecutada

En Docker Desktop Linux: build Rust release y Next.js production con TypeScript, Compose saludable en el puerto alternativo 3001, web HTTP 200, inspección BFF 200 y exportación PDF de tres páginas con el fixture del contrato (200, application/pdf). Al terminar únicamente el proceso Rust con SIGTERM, el lanzador cerró Next.js y Compose reinició ambos; el contenedor volvió a estar saludable. Se comprobó usuario node y publicación exclusiva de la web en loopback. Estas pruebas de infraestructura no sustituyen la validación visual de una plantilla real.
