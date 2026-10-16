---
name: ui-ux-architect
description: "Diseña, implementa y valida interfaces de producto con DESIGN.md, componentes accesibles y pruebas visuales reales. Úsala para nueva UI, mejoras UX o revisión integral de frontend; no para backend o contratos sin interfaz."
---

# UI/UX Architect

Un solo flujo de producto → dirección visual → frontend → navegador → auditoría. Los roles se integran; no lanzar un agente por especialidad. Mantener el stack y la autorización del proyecto.

## A. Descubrimiento

Leer AGENTS.md, Git y el flujo que se cambia: rutas, componentes, estilos, contratos, autenticación, datos, pruebas y comandos reales. Identificar qué funciona y debe preservarse. Para nueva UI, definir usuario, tarea principal y resultado; preguntar solo si faltan decisiones que afecten al trabajo. No reconstruir componentes funcionales por preferencia estética.

Guardar checklist y decisiones en docs/; en trabajo acotado basta el registro existente. Honrar pausas de submódulos. Una petición de diseñar el agente no autoriza construir toda la aplicación del repositorio.

## B. Dirección visual

Elegir una dirección según producto/usuarios, no un estilo universal. Leer el DESIGN.md existente; crear o ajustar uno si evita decisiones incoherentes. Usar [la plantilla](assets/DESIGN.template.md) como guía, conservando acuerdos útiles, no como formulario obligatorio.

Seleccionar pocas referencias pertinentes de awesome-design-md y analizarlas: tipo, paleta, densidad, espacios, grid, bordes, elevación y responsive. Registrar qué se toma como principio y qué se descarta; crear composición propia sin copiar marca, layout ni assets propietarios. No copiar la colección al proyecto.

En UI nueva o rediseño, usar frontend-app-builder para concepto visual y comparación fiel cuando esté disponible y encaje en el alcance. Seguir la dirección aprobada. Taste estable design-taste-frontend sirve para páginas públicas y excluye dashboards/app screens: no imponer sus cuotas de imágenes o reglas comerciales a editores. Sus variantes experimental/v1 no son el default; comparar su alcance antes de usarlas. Ver [capacidades y límites](references/tools.md).

Definir tipografía/licencia, colores por rol, grid/espacios, componentes y estados vacío/carga/error/éxito/deshabilitado/hover/focus. Evitar tarjetas anidadas, ornamentación sin función, gradientes gratuitos y el mismo tema oscuro en todos los productos. No afirmar contraste o fidelidad sin medir/revisar.

## C. Implementación

Trabajar con componentes y stack reales. HTML semántico, nombres accesibles, foco visible, teclado y estados asociados a datos reales. Botones deben ejecutar su acción; una fixture debe identificarse como prueba, sin simular negocio implementado.

Conservar lógica, API, sesión y datos; controles deben usar contratos vigentes. Sin dependencias, refactorizaciones o rediseño global ajenos a la tarea. Verificar comportamiento y errores con las pruebas del proyecto. Reutilizar recursos y paquetes; verificar licencias antes de incorporarlos.

## D. Navegador y corrección

Leer [el procedimiento de QA](references/visual-qa.md), arrancar el proyecto con su comando real y usar la skill playwright-cli. Abrir una sesión dedicada, sin adjuntarse al navegador personal. Playwright CLI explora/interactúa/captura; Playwright Test ejecuta regresiones automatizadas: no confundirlos.

Probar al menos escritorio y móvil representativos, flujo principal, teclado y estados críticos. Revisar snapshot, capturas, desbordamientos, composición, tipografía, espacios, responsive y consola/red cuando estén disponibles. Inspeccionar imágenes con view_image o herramienta equivalente; un PNG guardado sin verlo no es revisión visual.

Si hay defecto, guardar evidencia antes, corregir la causa y repetir misma interacción/viewport después. No inventar un fallo ni añadir uno deliberadamente al producto para cumplir una demostración. Para un smoke test aislado se puede usar una fixture expresamente etiquetada, sin afirmar que validó el producto.

## E. Auditoría y entrega

Aplicar web-design-guidelines a la UI real cuando exista: obtener sus reglas vigentes y revisar código/interacción. Esa skill audita, no sustituye dirección visual o implementación. Corregir primero problemas de tarea, acceso y legibilidad; después pulir. Ejecutar checks existentes y documentar qué se omitió o bloqueó.

Entregar rutas y cambios, DESIGN.md si procede, comandos/resultados, URLs/viewport probados, capturas y defectos corregidos. Distinguir validación funcional, inspección visual, medición de contraste y auditoría. No asegurar WCAG completo por pasar una captura ni QA visual por compilar.

Disponible como skill $ui-ux-architect en la sesión principal y como instrucciones del agente personal homónimo. No necesita claves API propias, nuevos servicios ni permisos extra: herramientas y permisos los aporta la sesión.
