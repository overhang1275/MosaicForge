# Verificación del agente y herramientas

Fecha local del usuario: 2026-10-15. Resultado: pruebas técnicamente disponibles ejecutadas; no se atribuyen a la UI de MosaicForge, todavía inexistente.

## Instalación y configuración

- [x] ~~Rol nativo personal ui-ux-architect.toml y skill homónima instalados.~~
- [x] ~~SKILL.md validado por quick_validate.py; TOML y metadatos YAML parseados, UTF-8 comprobado.~~
- [x] ~~Fuentes y cinco archivos de la skill instalados coinciden; no se sobreescribieron archivos ajenos.~~
- [x] ~~config.toml no cambió: SHA-256 antes/después de instalación coincidente. Sin modelos, permisos, proveedores ni MCP nuevos.~~
- [x] ~~Invocación nativa agent_type=ui-ux-architect aceptada y ejecutada: el agente confirmó sus instrucciones y su flujo de cinco fases.~~
- [x] ~~Plantilla DESIGN.md, criterios de QA, routing de skills y guía de reutilización/actualización creados.~~

Codex CLI observado: 0.162.0-alpha.17.2. Directorios y campos contrastados con documentación oficial antes de crear el rol. Taste estable y experimental evaluadas por su alcance; experimental no adoptada como default. Las instalaciones de terceros y referencias están en herramientas-diseno.md.

## Navegador real

Playwright CLI 0.1.22; sesión dedicada uiux-architect-resume, Edge headless reportado Edg/155.0.0.0 en Windows. URL inicial http://127.0.0.1:5179/smoke.html; query QA evita reutilizar el HTML antiguo al recargar. Sin perfil personal, datos de terceros ni servicios externos en esta prueba.

- [x] ~~Fixture abierta y snapshot real con textbox, botón, link y status.~~
- [x] ~~Formulario rellenado y enviado por teclado; status esperado en ambas dimensiones.~~
- [x] ~~Skip link es primer Tab y Enter mueve foco al main; foco del botón visible.~~
- [x] ~~1366×900: scrollWidth 1366; 390×844: scrollWidth 390. Sin desbordamiento horizontal.~~
- [x] ~~Enlace navega a #criterios.~~
- [x] ~~Capturas desktop-resume.png y mobile-resume.png revisadas con view_image tras la corrección; la móvil incluye todo el contenido vertical.~~
- [x] ~~Contraste medido en DOM: texto principal 14.5921:1, enlace 6.7276:1. Solo esos dos pares, no una auditoría completa WCAG.~~
- [x] ~~Consola de sesión: cero mensajes/errores/avisos; script sin pageerrors, requestfailed ni respuestas >=400.~~

Capturas locales en out/uiux-browser, fuera de Git. Manifiesto resultado.json registra hashes de fixture/script/capturas. Comandos del flujo en exercise.js y README del paquete; node --check del script pasó.

## Auditoría y corrección

Se obtuvieron las [directrices Vercel vigentes](https://raw.githubusercontent.com/vercel-labs/web-interface-guidelines/main/command.md). Revisión acotada a smoke.html: semántica, etiquetas, teclado, foco, formulario, contenido, responsive y estados. Sin imágenes de producto, hidratación, listas extensas, autenticación, animaciones ni gestos en esta fixture.

Hallazgos antes de corregir (líneas históricas de la fixture):

- smoke.html:37 — input sin política explícita de autocomplete y name poco descriptivo.
- smoke.html:24 — falta atajo de teclado al contenido principal.
- smoke.html:4 — metadato de color del navegador ausente; mejora secundaria.

Se añadió prueba antes de corregir: falló realmente con Test field lacks explicit autofill policy or meaningful name (exit 1). Se añadieron autocomplete=off, name=testName y skip link probado; también theme-color, balance de títulos y margen para ancla. La misma prueba completa volvió a exit 0, con valores arriba. No se inventó un defecto visual ni se alteró el producto para provocar uno.

Durante la corrección, el navegador seguía mostrando HTML anterior mientras archivo/HTTP con query contenían el nuevo. Se aisló la caché y se añadió query solo al test, sin cambiar la lógica del producto. Dos fallos de invocación previos fueron de herramienta: comillas en .cmd y punto y coma tras función para run-code; corregidos con --filename y formato esperado. No quedan fallos de la prueba pendientes.

La prueba del agente temporal se interrumpió por falta de créditos durante una pausa del usuario. Sus capturas previas se revisaron, pero el cierre se basa en la ejecución completa al retomar y en la invocación nativa posterior confirmada, no en inferir éxito del intento interrumpido.

## Límites y continuidad

El agente puede aplicar su flujo como skill en la sesión principal o como subagente nativo personal, sin cambiar sandbox. La fixture prueba implementación/interacción pequeñas y QA real. El editor completo de MosaicForge requiere 2.2–2.4; no está implementado ni visualmente validado. Su dirección indicada por el usuario es el estudio oscuro violeta/cian, no esta fixture de diagnóstico. No se ejecutó Playwright Test del producto, ni lectores de pantalla, dispositivos táctiles físicos o todos los pares de contraste.

## Cierre de la sesión

Sesión de navegador de QA cerrada. Comprobación final del puerto local 5179 sin listener; el servidor temporal ya no está activo.
