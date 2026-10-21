# Ejecución local con Docker

Requiere Docker Desktop con contenedores Linux y Docker Compose v2. Ejecutar desde la raíz del repositorio.

Construir y arrancar Rust CPU en release, Java y Next.js en producción, con
PostgreSQL y Valkey privados:

```powershell
node scripts/local-compose.mjs up -d --build --wait
```

Abrir http://127.0.0.1:3000. Se debe usar esta dirección exacta para que coincidan Host y Origin del BFF.

Actualizar después de cambiar código: volver a ejecutar el mismo comando. La imagen incorpora el código durante el build; no necesita Rust, Node ni pnpm instalados en el host. El primer build necesita descargar imágenes y dependencias; los siguientes reutilizan la caché de Docker.

```powershell
# Estado y logs
node scripts/local-compose.mjs ps
node scripts/local-compose.mjs logs --tail 100
# Reiniciar sin reconstruir
node scripts/local-compose.mjs restart
# Detener y retirar el contenedor
node scripts/local-compose.mjs down
```

Si 3000 está ocupado, usar otra terminal PowerShell:

```powershell
$env:MOSAICFORGE_HTTP_PORT = '3001'
node scripts/local-compose.mjs up -d --build --wait
```

Abrir entonces http://127.0.0.1:3001. Mantener esa variable en los comandos posteriores. El proyecto Compose tiene nombre fijo: cambiar el puerto reemplaza su contenedor, no crea una segunda instancia.

El contenedor de aplicación ejecuta tres procesos como usuario sin privilegios.
Rust escucha en127.0.0.1:4771 y Java en127.0.0.1:4772 dentro del contenedor;
solo se publica la web en loopback del host. Los tokens de servicio se generan
en memoria en cada arranque. El launcher entrega credenciales de DB/Valkey solo
a Java y la ruta del journal solo a Rust. El healthcheck comprueba web, API y
conexión Rust; no sustituye una prueba de generación y descarga.

El helper genera una sola vez `state/local-db.env` y `state/local-valkey.env`,
ignorados por Git. No imprimirlos, reemplazarlos ni ejecutar `compose config`
sin `--quiet`: pueden contener secretos. Se conservan los volúmenes de
PostgreSQL, Valkey y journal/artefactos Rust al reconstruir o reiniciar.
`down` conserva los volúmenes; no añadir `-v` para una actualización normal.

Los trabajos durables están habilitados mediante `MOSAICFORGE_JOBS_ENABLED`;
persisten proyecto/revisión, progreso y selección, y publican un PDF vectorial
completo más manifiesto. El piloto sigue limitado a12 aceptadas y un trabajador.
La UI de trabajos durables corresponde a3.5; las rutas BFF ya están disponibles.
No hay CUDA, login individual ni capacidad de un millón verificada.

Prueba de integración: `node scripts/durable-job-smoke.mjs submit` y luego
`node scripts/durable-job-smoke.mjs read`. Los resultados quedan en
`out/finalize/`. La inyección de fallo `durable-restart-probe.mjs` detiene un
proceso durante el trabajo QA y exige reiniciar la aplicación a continuación;
es una prueba deliberada, no una herramienta de operación habitual.
