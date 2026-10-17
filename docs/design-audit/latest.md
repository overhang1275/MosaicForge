# Auditoría de diseño — 2026-10-16

## Alcance

Editor real de MosaicForge en http://127.0.0.1:3000, con fixture pública `docs/fixtures/editor-seed.svg`; la fixture es un SVG de entrada, no una maqueta de la aplicación. Se conserva la dirección visual existente. Auditoría basada en las [guías vigentes de Vercel](https://raw.githubusercontent.com/vercel-labs/web-interface-guidelines/main/command.md), recuperadas el 2026-10-16.

Sesión independiente `audit-design`, Playwright CLI 0.1.22 y Edge. Ventanas revisadas: 1280×800, 1440×900 y 390×844. Se abrieron y revisaron las capturas indicadas.

## Checklist persistente

- [x] ~~Leer instrucciones, dirección existente y componentes.~~
- [x] ~~Reproducir importación, generación y defectos antes de editar.~~
- [x] ~~Capturar y revisar imágenes de escritorio y móvil.~~
- [x] ~~Implementar correcciones acotadas, sin alterar controlador, contratos o motor.~~
- [x] ~~Ejecutar formato, lint, tipado y tests web.~~
- [x] ~~Verificar correcciones en aplicación reconstruida y exportación PDF.~~
- [x] ~~Repetir teclado, estados críticos y consola/red tras reconstrucción.~~
- [x] ~~Comprobar el último ajuste del contorno de foco tras la segunda reconstrucción.~~

## Hallazgos y cambios — dos rondas de implementación

| Prioridad | Ubicación | Hallazgo reproducido | Corrección |
|---|---|---|---|
| P2 | `apps/web/src/components/Editor.tsx:12` | Error de semilla a y=1078 en viewport de 800 px, con scroll del panel en 0; la acción parecía no responder. El mensaje sí tenía role alert. | Error y estado al principio del área de trabajo; foco programático en errores nuevos para revelarlos también tras hacer scroll. |
| P2 | `apps/web/src/components/Results.tsx:114` | Metadatos abiertos consumían 292 px en móvil; los ajustes comenzaban en y=1031. | Disclosure inicialmente cerrado, conservando todos los datos de reproducción. |
| P2 | `apps/web/src/components/Results.tsx:93` y `rejection-copy.ts:2` | Un rechazo mostraba únicamente `frame_exceeds_scale_limits`, sin recuperación legible. | Mensaje en español con acción para reintentar. Se conserva el código original en Información de la variante. Códigos futuros reciben un fallback seguro. |
| P2 | `apps/web/src/components/PdfExport.tsx:33` | Una serie rechazada mostraba «1 página · serie completa guardada» pese al bloqueo correcto del PDF. | El resumen cuenta variantes hasta que todas estén aceptadas. |

Reproducción del error: importar fixture, generar 9 variantes, escribir `invalida` en Semilla, pulsar Generar variantes. Reproducción del rechazo: fixture pública, seed 2026, count 1, rotación 45..45, escala X/Y 1..1; resto de ajustes por defecto. No se modificó la validación para obtener aceptación.

## Evidencia antes

- `screenshots/before-empty-1280.png` — estado vacío real.
- `screenshots/before-error-1280.png` y `before-error-1440.png` — el error existe fuera del viewport.
- `screenshots/before-results-390.png` — resultado móvil con metadatos abiertos.
- `screenshots/before-rejected-1440.png` — rechazo técnico y resumen PDF incorrecto.
- `screenshots/before-keyboard-1440.png` — foco visible del enlace de salto activado por teclado.

## Comprobaciones ejecutadas

Desde la raíz se empleó `./scripts/playwright.ps1 -s=audit-design` con los comandos `open`, `resize`, `click`, `upload`, `fill`, `snapshot`, `screenshot`, `eval`, `run-code`, `console warning` y `requests`. El Chrome predeterminado no estaba instalado; se usó `--browser=msedge` sin instalar paquetes.

La importación retornó POST `/api/inspect` 200; nueve POST `/api/preview` 200 produjeron nueve tarjetas Aceptada. En móvil, `scrollWidth=390` e `innerWidth=390`; tampoco hubo desbordamiento global a 1440×900. Consola inicial: 0 errores y 0 avisos.

Teclado: Tab → Enter en «Ir a la vista previa» dejó `activeElement.id=preview-title`, con contorno cian sólido de 3 px visible. Los controles conservan etiquetas, botones nativos y disclosures operables por teclado.

Contraste sRGB de pares reales de CSS: texto/panel 17,01:1; secundario/panel 10,37:1; texto de error/fondo de error 11,80:1; borde de control/campo 3,88:1; foco/panel 13,01:1; texto de botón frente a extremos violeta/cian 5,76:1 y 11,14:1. Son pares representativos, sin certificar todos los puntos del gradiente o estados deshabilitados.

Desde `apps/web`, tras las correcciones de componentes y cliente: `pnpm format`, `pnpm lint`, `pnpm typecheck` y `pnpm test` completaron con exit 0. Pasaron 26 tests, incluidos dos de recuperación de rechazo y fallback para códigos futuros y nombres de propiedades de objetos. No existe una suite Playwright Test configurada en el proyecto; la exploración de navegador se hizo con CLI.

## Límites de la validación

La compilación de producción se reconstruyó y se revisó en navegador. El ajuste final del contorno también se comprobó en la aplicación reconstruida. La validación móvil usa viewport de Edge de escritorio, no hardware táctil. No se ejecutaron lector de pantalla ni zoom nativo al 200 %. No se afirma conformidad WCAG completa. La traducción de los cinco motivos se revisó contra los códigos existentes del motor; solo `frame_exceeds_scale_limits` se reprodujo específicamente en esta sesión.


## Resultado posterior en la aplicación real

Todas las capturas siguientes se abrieron y revisaron. Los cambios conservan la composición de tres áreas en escritorio, y la pieza y los controles se leen sin desbordamiento horizontal en móvil.

- [Vacío 1280×800](screenshots/after-empty-1280.png) y [vacío 390×844](screenshots/after-empty-390.png).
- [Generación real en curso](screenshots/after-generating-1280.png): nueve posiciones estables; PDF deshabilitado y acción Cancelar generación disponible.
- [Exportación real en curso](screenshots/after-exporting-1280.png): botón y estado indican Preparando PDF vectorial.
- [Resultados 1280×800](screenshots/after-results-1280.png), [1440×900](screenshots/after-results-1440.png) y [390×844](screenshots/after-results-390.png): nueve variantes aceptadas. Metadatos cerrados al inicio; en móvil el inspector empieza en y=790, frente a y=1031 antes, y scrollWidth=innerWidth=390.
- [Rechazo 1440×900](screenshots/after-rejected-1440.png) y [390×844](screenshots/after-rejected-390.png): explicación en español; resumen de una variante y exportación deshabilitada. Al abrir Información de la variante mediante Enter aparece el código original `frame_exceeds_scale_limits`.
- [Teclado 1280×800](screenshots/after-keyboard-1280.png): Tab → Enter en el enlace de salto enfoca Vista previa con contorno cian de 3 px.
- Error de semilla tras abrir metadatos y desplazar el panel hasta abajo: el error nuevo recibe foco y vuelve a y=84, frente a y=1078 antes. Comprobación final en [1280×800](screenshots/after-error-1280.png) y [1440×900](screenshots/after-error-1440.png).

Descarga pública comprobada: `POST /api/export-pdf` 200, un `mosaic.pdf`, `download.failure()=null`; mensaje final «PDF vectorial completo preparado para descargar». `pdfinfo` confirmó 9 páginas, tamaño 283,465×283,465 pt (100×100 mm), PDF 1.7 y 293011 bytes. La entrada fue exclusivamente la fixture pública con ajustes predeterminados, seed 2026 e índices 0..8. La revisión de exportación complementaria confirmó nueve páginas de 100 mm, trazados y cero Image XObjects; renderizó las nueve páginas y revisó visualmente las páginas 1, 5 y 9.

Consola tras importación, generación, PDF y rechazo: 0 errores y 0 avisos. Nueve previews y el PDF retornaron 200. Las validaciones locales no enviaron una solicitud de generación. Los metadatos siguen siendo operables por teclado y conservan los hashes y el código técnico.

## Segunda ronda: contorno de foco

`apps/web/app/globals.css:501` — en el primer después, el contorno exterior del error era recortado por el scroll del panel. Se conserva el hallazgo en [before-error-focus-clipped-1280.png](screenshots/before-error-focus-clipped-1280.png). Se cambió únicamente `outline-offset` a `-3px` para dibujarlo dentro del mensaje. La segunda reconstrucción confirmó el contorno cian completo, sin recortes, en 1280×800, 1440×900 y [390×844](screenshots/after-error-390.png); capturas abiertas y revisadas. Foco activo, offset calculado -3 px y error a y=84 en escritorio y y=90 en móvil. `pnpm format` pasó de nuevo tras este cambio CSS. No se añadieron dependencias ni se cambió la lógica de datos.
