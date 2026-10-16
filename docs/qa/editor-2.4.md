# Cierre QA del editor 2.4

2026-10-16. Windows x64, Edge mediante Playwright CLI 0.1.22, aplicación standalone real en 127.0.0.1:3000 y motor Rust CPU en 127.0.0.1:4771. No Playwright Test ni certificación WCAG. El IAB se había probado antes de la pausa; se usa sesión independiente para no interferir con la pestaña del usuario.

## Qué se ejecutó

- [x] ~~Formato, ESLint, tipado y 14 tests web (7 editor + 7 BFF), sin fallos.~~
- [x] ~~Build standalone final, recursos estáticos e icon.svg.~~
- [x] ~~Smoke existente CLI/Rust/BFF con procesos reales y cierre de sus procesos.~~
- [x] ~~Test del editor: carga explícita, tres variantes reales, descargas SVG/PNG después de editar la semilla, teclado y overflow u64.~~
- [x] ~~Escritorio 1366×900; móvil 390×844 sin scroll horizontal y orden preview→controles→galería. Layout equivalente 683×450 sin overflow.~~
- [x] ~~Rechazo real con opacidad cero: empty_viewport, sin imagen ni descargas.~~
- [x] ~~Capturas de escritorio/móvil/concepto abiertas y comparadas con view_image.~~

Scripts locales reproducibles: editor-2.4.browser.js, editor-2.4-contrast.js y editor-2.4-rejection.js, para run-code --filename de Playwright CLI desde raíz. Incluyen la ruta de este workspace Windows; adaptar root para otra máquina. Necesitan out/test-prepared.svg, generado únicamente para el SVG de prueba del usuario por scripts/prepare-test-svg.py. No son tests portables de CI.

Evidencia estructurada: [editor-2.4-result.json](editor-2.4-result.json). Capturas y logs locales ignorados en out/editor-qa/: desktop-final.png, mobile-final.png, empty-final.png, original-error-final.png, rejected-final.png, final-browser.log y contrast-final.log.

## Resultado de test.svg

El archivo original (757 135 bytes) se conserva: SHA-256 ceae058483698070aee16a8d1337715167142b198c47a28e626a0bb8683fbbf8. El importador lo rechaza por namespace XML fuera del subconjunto admitido; la UI muestra «SVG no admitido o inválido» y no habilita generación. No se amplió ni debilitó el importador para pasar la prueba.

La copia preparada existente (777 922 bytes; SHA-256 df642228575cce583adcc969da6dafacb17aa94954c2f6a41c5f3b2238b6e798) importa y genera índices 0, 1 y 2, seed 2026, PNG 256² y fondo transparente. Tras cambiar el campo seed a 2027, las descargas de la tarjeta 0 siguen usando seed 2026.

Comparación de archivos descargados desde el navegador frente a CLI ejecutada con los sobres reales capturados, misma plantilla preparada y configuración:

| Formato | Bytes | SHA-256 | Paridad |
|---|---:|---|---|
| SVG | 771 537 | d347e05f67c4c2cf11a1d1fb2ab61567ee869591b490e57012697ddd3736f817 | Bytes idénticos |
| PNG | 72 839 | a0a1c4310974732fe93f20414d204b3aa9f8db8d20410d792d6f001c9d8a1231 | Bytes idénticos |

La consola/red contiene únicamente el 400 esperado al importar el original; no se observaron errores JS ni otras respuestas fallidas en el flujo. El favicon 404 de la sesión anterior no aparece en este build.

## Corrección y medición de contraste

Una prueba sobre estilos computados falló al medir borde de campo 1,7639:1. Se separó el token de borde de controles (#657B9B) del borde decorativo del panel (#354663). La misma comprobación pasa después de recompilar:

| Par representativo habilitado | Ratio |
|---|---:|
| Texto de campo / fondo | 15,69:1 |
| Borde de campo / fondo | 3,88:1 |
| Etiqueta / panel | 10,37:1 |
| Texto principal / extremos del gradiente, mínimo | 5,76:1 |
| Texto de botón secundario / fondo | 14,34:1 |
| Borde de botón secundario / fondo | 3,55:1 |

No se midieron todos los estados, píxeles intermedios del gradiente ni controles nativos. No equivale a auditoría WCAG completa. Se conserva desktop-before-contrast.png como evidencia previa.

## Fidelidad y revisión visual

1. Cabecera/copia: misma marca, Estudio de variantes y jerarquía de Vista previa/Variantes. «Variantes de muestra» aclara el alcance 1–12; no crea una promesa de producción masiva.
2. Composición: tres áreas, inspector 300 px, preview flexible y galería 288 px; scroll interno mantiene accesible la acción inferior. El concepto se diseñó en 1544×1019, validación de implementación en 1366×900.
3. Tipo: Arial/Helvetica del sistema, cuerpo 14 px y título 16 px comprobados mediante CSS computado. Semilla decimal íntegra, hashes con salto de línea y sin overflow.
4. Paleta: navy y violeta/cian coherentes. Borde de controles más claro por el defecto medido; paneles conservan su borde discreto.
5. Arte: PNG real con contain y filter:none. Checker claro bajo resultados transparentes para distinguir trazos oscuros; no cambia ningún byte descargado. Los trazos del SVG de prueba son finos por su propio diseño y rasterización a 256².
6. Responsive: móvil apila preview, controles y galería; imágenes completas, etiquetas y hashes legibles. Captura full-page 390×2706; viewport 390×844.

Se conservaron las adaptaciones funcionales al concepto ya documentadas: sin zoom/fullscreen ni drag-and-drop prometidos, salida hasta512, importar secundario y generación principal. No se añadieron navegación comercial ni landing.

Auditoría acotada con las [directrices Vercel consultadas](https://raw.githubusercontent.com/vercel-labs/web-interface-guidelines/main/command.md): semántica, etiquetas/nombres, foco visible, skip link, imágenes con dimensiones/alt, anuncios asíncronos, color-scheme/theme-color, texto largo y reduced-motion comprobados en código/flujo. La profundidad de accesibilidad (foco de errores por campo, lector de pantalla, zoom nativo, orden de tabulación completo y dispositivos físicos) pertenece al siguiente submódulo 2.5.

## Límites y continuidad

Revisión independiente del código 2.4 realizada antes de la pausa, sin P1/P2; probes de snapshots, resuscripción y descarte de respuestas al desmontar pasaron. En esta continuación solo se modificaron bordes CSS, documentación y scripts de QA. React StrictMode real no se certificó; no se repite una auditoría completa del motor sin cambios.

No se ejecutó un millón de variantes; requisito de producción permanece en requisito-produccion-millon.md y fase3. La muestra de12 y el contrato actual máximo10000/ejecución no son esa capacidad. Tampoco se probaron Linux/macOS, lectores de pantalla o zoom nativo200%; 683×450 verifica reflow equivalente, no sustituye esa prueba.

2.4 terminado. Esperar autorización antes de 2.5; no iniciar Java ni producción masiva por cerrar el editor local.
