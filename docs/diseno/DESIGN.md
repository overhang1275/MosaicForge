# MosaicForge — Sistema visual del editor

Dirección indicada por el usuario el 2026-10-15. Fuente visual: referencia-usuario.md. Este documento guía el frontend implementado en2.4; evidencia y límites en ../qa/editor-2.4.md. La landing comercial se difiere al cierre del producto.

## Identidad y tarea

Estudio creativo oscuro para explorar variaciones reproducibles de un SVG propio. La pieza y sus variantes son protagonistas; los controles ayudan a comparar y reproducir. Sin portada comercial dentro del editor local.

## Composición

En escritorio, tres áreas: inspector de plantilla/reglas, preview seleccionado grande y galería de hasta 12 variantes. Bordes finos, superficies diferenciadas y alineación estable. Cada resultado conserva índice, estado y acceso a sus descargas. No recortar/deformar imágenes para llenar tarjetas.

En móvil, preview y acciones principales primero, controles accesibles mediante secciones compactas y galería debajo. Reorganizar el flujo; no reducir tres columnas hasta volverlas ilegibles.

## Roles de color propuestos

| Rol | Punto de partida |
|---|---|
| Fondo | Azul casi negro, #080C18 |
| Panel | #0E1526 |
| Superficie elevada/campo | #141D32 |
| Texto principal | #F5F7FF |
| Texto secundario | #B9C4DA |
| Bordes | #354663 |
| Acento violeta | #A736FF |
| Acento cian | #00D3EA |
| Foco | Cian claro visible, con contorno y separación |

Son decisiones originales de partida, no colores extraídos ni contraste certificado. Medir cada par/estado antes de implementar. Gradiente violeta→cian reservado para acción/identidad, con texto que conserve legibilidad; superficies de trabajo sobrias. Éxito, rechazo y fallo técnico usan texto/icono además de color. No aplicar filtros ni tonos de UI al PNG real.

## Tipo, espacio y componentes

Sans serif legible con licencia abierta y fallback del sistema; escoger archivo/familia al implementar. Texto operativo 14–16 px, seed/índice completos y cifras tabulares. Encabezados de sección discretos; el gran título y la navegación comercial pertenecen a la futura landing.

Escala de espacios 4/8/12/16/24/32 px; radios contenidos de 6–12 px y sombras solo para jerarquía real. Inspector compacto con etiquetas persistentes, valores numéricos editables y controles de rango cuando aporten. Sin tarjetas dentro de tarjetas ni decoración que compita con el arte.

## Interacción y datos

Importación explícita; seed/índices decimales u64. Generación por botón y cola secuencial; hasta 12 previews reales. Mostrar estado pendiente/generando/aceptada/rechazada/error sin cambiar posiciones. Descargar SVG/PNG con los parámetros guardados de cada resultado; invalidar respuestas antiguas al editar. No recrear reglas del motor en TypeScript.

Vacío con siguiente acción clara; carga sin imágenes inventadas; errores con causa/recuperación; controles deshabilitados solo cuando corresponde. Foco visible, teclado y mensajes anunciados. Microinteracciones breves, opcionales y respetuosas con reduced-motion.

## Validación 2.4 y continuación 2.5

Concepto visual fiel a esta dirección, implementación con componentes reales y comparación de capturas. Revisar 1366×900, 390×844, contenido largo, zoom 200 %, teclado, errores y resultados rechazados. Medir contraste y revisar consola/red. Evidencia del motor y paridad CLI separadas de similitud visual. La fixture técnica del agente no es el concepto visual del producto.

2.4: comprobados1366×900 y390×844, teclado/errores, capturas y pares de contraste representativos. Bordes de controles#657B9B; paneles#354663. Zoom200% nativo, lectores de pantalla y robustez ampliada siguen pendientes de2.5.
