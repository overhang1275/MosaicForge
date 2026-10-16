# Concepto e implementación del estudio — 2.4

Dirección aprobada: [DESIGN.md](DESIGN.md) y [referencia-usuario.md](referencia-usuario.md). Alcance: editor local de variantes; no landing. Concepto propio generado el 2026-10-15 con ImageGen integrado, inspeccionado con `view_image` antes de implementar. Imagen local no versionada: `out/diseno/concepto-editor-2.4.png` (1544 × 1019). No es una captura del producto ni demuestra generación del motor.

## Especificación visual extraída

- Fondo navy `#080C18`, paneles `#0E1526`, campos `#141D32`, bordes `#354663`, texto `#F5F7FF` y secundario `#B9C4DA`. El acento principal combina violeta claro y cian con texto navy. Sin tintes sobre arte real.
- Sans del sistema (Arial/Helvetica), sin fuente remota ni dependencia añadida. Controles 13–14 px, encabezados 16 px, marca 26 px, mensaje central 20 px. Información secundaria 12 px; hashes monoespaciados completos con salto de línea.
- Cabecera 68 px con marca y «Estudio de variantes». Escritorio: inspector 300 px, espacio central flexible, galería 288 px; gutters 16 px, radios 6–8 px. Inspector con scroll propio y acción inferior fija en el panel. Canvas con PNG `object-fit: contain` y patrón discreto para transparencia.
- Secciones con `details` nativo, etiquetas persistentes y valores editables; botones secundarios sobrios, acción Generar con acento. Iconos SVG originales de documento/imagen, trazo uniforme; no identidad de la referencia.
- Móvil: vista previa primero, después inspector y galería. Sin columnas comprimidas. El orden DOM conserva acceso temprano a carga/controles y un enlace permite saltar a la vista previa.

## Adaptaciones funcionales al concepto

El concepto sirve a la dirección ya aprobada. Se corrigen valores ilustrativos para respetar contratos: salida máxima 512 por eje (el dibujo decía 768), seed inicial 2026, nueve variantes, fondo transparente. Seed e índice usan texto decimal para conservar u64; no spinbuttons que conviertan a Number. Se añade fondo al inspector y se incluyen todos los controles contratados dentro de las secciones avanzadas. La UI no promete arrastrar archivos, zoom o pantalla completa: el dibujo incluía esos elementos sin necesidad funcional acordada. Importar usa estilo secundario y Generar conserva la jerarquía principal. No se crean previews, métricas ni arte de demostración.

Reproducción muestra semilla/índice/hash/versiones del resultado seleccionado; las descargas pertenecen a su snapshot aunque cambien los ajustes. Las tarjetas preservan su sitio al generar/rechazar/fallar; la miniatura y selección son botones reales y las descargas están fuera del botón de selección.

## Inventario de componentes

- `Editor.tsx`: shell, mensajes anunciados y error accesible.
- `Inspector.tsx`: carga explícita, resumen de plantilla, reglas completas y generación/cancelación.
- `Results.tsx`: preview PNG real, metadatos, galería y descargas.
- `Fields.tsx`: campos asociados a su etiqueta, secciones e icono de vacío.
- `app/globals.css`: tokens, layout, estados y responsive. Sin dependencias nuevas.

## Checklist y evidencia

- [x] Referencia real abierta; dirección existente leída y conservada.
- [x] Concepto ImageGen generado e inspeccionado antes de código.
- [x] Estado vacío sin datos simulados, estados por tarjeta y mensajes de carga/error.
- [x] UI conectada al hook/controlador del producto; sin SVG subido insertado en DOM.
- [x] Foco visible, etiquetas, selección con `aria-pressed`, regiones anunciadas y movimiento reducido.
- [x] Formato de los componentes y CSS con Prettier local; ESLint y typecheck (ver ejecución integrada para resultados finales).
- [ ] Comparación concepto/captura del producto en 1366 × 900 y 390 × 844.
- [ ] Flujo real de importación, generación y descargas, teclado y estados críticos en navegador.
- [ ] Contraste medido, desbordamiento, consola/red y auditoría web.

La verificación visual del producto la realiza la sesión principal sobre esta misma UI; su evidencia y resultados se registran en el avance 2.4. Los checkboxes pendientes aquí no acreditan QA por compilación.

## Comparación que debe completar QA

1. Copia: cabecera, Vista previa, Variantes, secciones y acción principal conservan el concepto. Extras funcionales descritos arriba.
2. Layout: tres zonas y canvas dominante en escritorio; scroll de inspector comprobable.
3. Tipografía: sans legible, jerarquía compacta del chrome, sin título comercial.
4. Paleta: superficies navy constantes, acento solo en acción/foco/selección; PNG sin filtros.
5. Contenedores: paneles únicos con divisores, campos contenidos, imágenes completas sin recortes.
6. Responsive: preview primero, controles legibles y galería de dos columnas en móvil; hashes e índices largos sin overflow.

No se afirma una conformidad WCAG completa ni una fidelidad visual verificada antes de revisar las capturas reales.
