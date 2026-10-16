# Ejecución local con Docker

Requiere Docker Desktop con contenedores Linux y Docker Compose v2. Ejecutar desde la raíz del repositorio.

Construir y arrancar Rust CPU en release y Next.js en producción:

```powershell
docker compose -f deploy/compose/local.yml up -d --build --wait
```

Abrir http://127.0.0.1:3000. Se debe usar esta dirección exacta para que coincidan Host y Origin del BFF.

Actualizar después de cambiar código: volver a ejecutar el mismo comando. La imagen incorpora el código durante el build; no necesita Rust, Node ni pnpm instalados en el host. El primer build necesita descargar imágenes y dependencias; los siguientes reutilizan la caché de Docker.

```powershell
# Estado y logs
docker compose -f deploy/compose/local.yml ps
docker compose -f deploy/compose/local.yml logs --tail 100
# Reiniciar sin reconstruir
docker compose -f deploy/compose/local.yml restart
# Detener y retirar el contenedor
docker compose -f deploy/compose/local.yml down
```

Si 3000 está ocupado, usar otra terminal PowerShell:

```powershell
$env:MOSAICFORGE_HTTP_PORT = '3001'
docker compose -f deploy/compose/local.yml up -d --build --wait
```

Abrir entonces http://127.0.0.1:3001. Mantener esa variable en los comandos posteriores. El proyecto Compose tiene nombre fijo: cambiar el puerto reemplaza su contenedor, no crea una segunda instancia.

El contenedor ejecuta ambos procesos como usuario sin privilegios. Rust escucha en 127.0.0.1:4771 dentro del contenedor; solo se publica la web y solo en loopback del host. El token privado se genera en memoria en cada arranque y nunca se guarda en Compose ni en la imagen. Se conserva el flujo HTTP y la validación de contratos. El healthcheck comprueba la respuesta web y que el proceso Rust acepte conexiones; no sustituye una prueba de importar, generar y exportar desde el navegador.

Este despliegue es local, sin CUDA, Java, base de datos ni volúmenes persistentes. Los SVG y resultados se procesan en memoria, con los límites actuales de la aplicación.
