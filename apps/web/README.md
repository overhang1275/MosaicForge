# Editor web local — fase 2.4

Next.js App Router/TypeScript estricto. Tres Route Handlers Node: POST /api/inspect, /api/preview y /api/export. Editor, controles, galería de hasta12muestras y descargas reales implementados en2.4. Dirección visual aprobada: ../../docs/diseno/DESIGN.md.

## Preparación y arranque

Node 24.19+ de la rama 24 y pnpm 11.25.0. Desde raíz:

```powershell
cargo build --manifest-path services/engine/Cargo.toml -p mosaicforge-cli -p mosaicforge-preview --locked
pnpm --dir apps/web install --frozen-lockfile --ignore-scripts
pnpm --dir apps/web build
node scripts/start-preview.mjs
```

Abrir http://127.0.0.1:3000. Launcher genera token criptográfico en memoria, inicia Rust 4771 y Next.js standalone 3000, ambos solo en loopback. Ctrl+C cierra los dos procesos propios. Requiere binarios debug y build web previos. MOSAICFORGE_ENGINE_PORT y PORT permiten puertos alternativos; no pueden coincidir. No imprime el token ni guarda plantillas. No iniciar el server.js generado directamente: su valor predeterminado de Host permite todas las interfaces; pnpm start usa el wrapper de loopback.

Para desarrollo con procesos gestionados por separado: pnpm dev escucha 127.0.0.1:3000. Rust y web deben recibir el mismo MOSAICFORGE_ENGINE_TOKEN privado mediante entorno; MOSAICFORGE_ENGINE_URL debe ser un origen HTTP canónico con host literal 127.0.0.1 (por defecto http://127.0.0.1:4771), y MOSAICFORGE_WEB_ORIGIN debe coincidir con el origen/puerto del navegador. No usar NEXT_PUBLIC ni guardar secretos en Git.

## Contratos y controles

- Tipos de contracts/generated/typescript/preview.ts y validadores standalone generados desde JSON Schema. La semántica compartida sigue scripts/contracts/validate.mjs. El motor conserva la autoridad geométrica.
- El BFF crea requestId y valida el sobre, sin convertir seed/índice u64 a Number. Solo destinos y rutas fijos; redirecciones prohibidas y token privado sustituye cualquier Authorization del cliente. Host HTTP y Origin exactos; Next.js normaliza su URL interna a localhost, por eso no se confía en esa URL para comprobar Host.
- JSON hasta 32 MiB durante lectura; SVG 5 MiB UTF-8, spec 1 MiB y respuesta 16 MiB. Espera total de 30 s y cancelación del fetch al desconectar. Rust puede seguir terminando su operación con permiso retenido. Sin persistencia ni caché.
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

QA del editor: ../../docs/qa/editor-2.4.md. test.svg original no pertenece al subconjunto SVG seguro; probar copia preparada según informe. Producción masiva de hasta1millón sigue planificada; no confundir límite de muestras con volumen del trabajo.
