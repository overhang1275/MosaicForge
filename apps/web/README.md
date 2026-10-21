# Editor web local — fase 2.4

Next.js App Router/TypeScript estricto. Cuatro Route Handlers Node: POST /api/inspect, /api/preview, /api/export y /api/export-pdf. Editor, controles y galería de hasta 12 muestras implementados en 2.4. PDF vectorial único de toda la serie como descarga principal; SVG/PNG secundarios.

## Preparación y arranque

Node 24.19+ de la rama 24 y pnpm 11.25.0. Desde raíz:

```powershell
cargo build --manifest-path services/engine/Cargo.toml -p mosaicforge-cli -p mosaicforge-preview --locked
pnpm --dir apps/web install --frozen-lockfile --ignore-scripts
pnpm --dir apps/web build
node scripts/start-preview.mjs
```

Abrir http://127.0.0.1:3000. Launcher genera token criptográfico en memoria, inicia Rust 4771 y Next.js standalone 3000, ambos solo en loopback. Ctrl+C cierra los dos procesos propios. Requiere binarios debug y build web previos. MOSAICFORGE_ENGINE_PORT y PORT permiten puertos alternativos; no pueden coincidir. No imprime el token ni guarda plantillas. No iniciar el server.js generado directamente: su valor predeterminado de Host permite todas las interfaces; pnpm start usa el wrapper de loopback.

Para desarrollo con procesos gestionados por separado: pnpm dev escucha 127.0.0.1:3000. Rust y web deben recibir el mismo MOSAICFORGE_ENGINE_TOKEN privado mediante entorno; MOSAICFORGE_ENGINE_URL debe ser un origen HTTP canónico con host literal 127.0.0.1 (por defecto http://127.0.0.1:4771). MOSAICFORGE_WEB_ORIGIN define el puerto de la web; el proxy acepta ese origen con host 127.0.0.1 y también localhost en el mismo puerto, siempre que el encabezado Host coincida con Origin. No usar NEXT_PUBLIC ni guardar secretos en Git.

## Contratos y controles

- Tipos de contracts/generated/typescript/preview.ts y validadores standalone generados desde JSON Schema. La semántica compartida sigue scripts/contracts/validate.mjs. El motor conserva la autoridad geométrica.
- El BFF crea requestId y valida el sobre, sin convertir seed/índice u64 a Number. Solo destinos y rutas fijos; redirecciones prohibidas y token privado sustituye cualquier Authorization del cliente. Host HTTP y Origin exactos; Next.js normaliza su URL interna a localhost, por eso no se confía en esa URL para comprobar Host.
- JSON hasta 32 MiB durante lectura; SVG 5 MiB UTF-8, spec 1 MiB y respuesta 16 MiB. Espera total de 30 s para inspección/preview/export SVG-PNG y 120 s para PDF vectorial; cancelación del fetch al desconectar. Rust puede seguir terminando su operación con permiso retenido. Sin persistencia ni caché.
- Errores del upstream se validan y sustituyen por mensajes propios; códigos y requestId correlacionados. Motor ausente/protocolo inválido: 502; timeout: 504; ocupado: 429.
- Preview devuelve JSON PNG. Export accepted devuelve bytes intactos como attachment, nombre solo del índice validado, MIME fijo y CSP sandbox/default-src none. Export rejected devuelve 200 JSON con motivo, sin bytes. No insertar SVG del usuario en HTML.

## Comprobaciones

```powershell
pnpm --dir apps/web format
pnpm --dir apps/web lint
pnpm --dir apps/web typecheck
pnpm --dir apps/web test
pnpm --dir apps/web build
pnpm --dir scripts/contracts check
node scripts/preview-smoke.mjs
```

build usa webpack y copia .next/static al paquete standalone. No hay public/ en esta base; si se añade, deberá incluirse en empaquetado y smoke. El smoke real comprueba un script referenciado por el HTML y ejercita Next.js/Rust/CLI, incluidos límites chunked y descargas. Las pruebas Node del proxy usan un upstream HTTP local para respuestas adversas; no reemplazan el motor real en el smoke.

Dependencias fijadas: Next.js 16.4.2, React 19.3.1, TypeScript 6.0.3. TypeScript 7.0.2 fue rechazado por typescript-eslint y se sustituyó en esta app; herramientas de contratos existentes conservan su versión. Inventario de licencias declaradas: ../../docs/dependencias-http-web.json. pnpm registró excepciones locales de antigüedad para estas versiones Next.js en pnpm-workspace.yaml; sin cambiar configuración global.

Fuentes de implementación: [Next.js standalone](https://nextjs.org/docs/app/api-reference/config/next-config-js/output), [Route Handlers](https://nextjs.org/docs/app/getting-started/route-handlers), [Axum](https://docs.rs/axum/0.8.9/axum/), [Ajv standalone](https://ajv.js.org/standalone.html).

## PDF y Docker

PDF toma la serie completa guardada, aunque después se editen los controles; todas las muestras deben estar aceptadas. El tamaño predeterminado es 100 × 100 mm, sin sangrado. La ruta exige generación SVG, invoca el exportador vectorial Rust y nunca usa los PNG de previsualización para construir páginas. Fallos o rechazos no descargan un documento parcial.

Docker ejecuta ambos procesos con compilaciones de producción: [instrucciones y puertos](../../docs/docker-local.md). No recompilar sobre el standalone que un proceso local está sirviendo en Windows; detener primero o usar el despliegue Docker para actualizar la imagen.

## Proxies de proyectos y trabajos durables (3.4.5)

`/api/projects` permite GET (cursor UUID y limit 1–100) y POST; `/api/projects/me` consulta `/v1/me`. Las rutas con UUID permiten consultar proyectos, guardar revisiones y leer una revisión positiva. `/api/jobs` permite POST con `Idempotency-Key` estable de 16–128 caracteres ASCII visibles; `/api/jobs/{uuid}` consulta estado y `/cancel` permite POST sin cuerpo.

Estas rutas usan exclusivamente `MOSAICFORGE_API_URL` (por defecto `http://127.0.0.1:4772`), `MOSAICFORGE_API_TOKEN` privado y `MOSAICFORGE_WEB_ORIGIN`. Las escrituras requieren Origin coincidente con Host. Las lecturas nativas GET/HEAD pueden omitir Origin si Sec-Fetch-Site es same-origin o none. Las rutas y consultas están cerradas; ningún parámetro elige un destino arbitrario. Los validadores generados verifican JSON de entrada y salida; el cuerpo original se conserva para que Java detecte claves duplicadas.

`/api/jobs/{uuid}/artifacts/pdf` y `/manifest` sirven descargas nativas GET/HEAD de hasta 1 GiB mediante streams con contrapresión. Admiten un rango de bytes y If-Range acotados. Se verifican MIME, ETag, tamaño e intervalo; un GET completo comprueba SHA-256 y retiene el último byte hasta EOF. Un cuerpo incompleto o corrupto interrumpe la descarga. La desconexión cancela el stream upstream y el plazo máximo es 120 s. Los errores JSON se validan y sustituyen por mensajes locales; un 416 conserva Content-Range. La interfaz de trabajos y SSE quedan para 3.5.
