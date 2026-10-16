# Fase 2 — Dirección visual del editor

Fecha: 2026-10-15. Estado: propuesta inicial histórica. El usuario indicó después una dirección de estudio oscuro violeta/cian y tres áreas, registrada en [referencia-usuario.md](referencia-usuario.md). Esa referencia prevalece sobre la recomendación neutral de este documento. No adelanta 2.1–2.3 ni la landing comercial.

## Propósito y límites

El centro de la pantalla es comparar resultados reales de una plantilla propia y poder reproducirlos. La identidad nace de esa relación: una configuración explícita produce una serie de imágenes identificadas por su índice. Las imágenes del usuario aportan la variedad; la interfaz mantiene un marco estable y neutral.

La propuesta desarrolla la [especificación aprobada](../superpowers/specs/2026-10-15-fase-2-web-design.md) y respeta las pausas de [submódulos](../fase-2-submodulos.md). No añade cuentas, proyectos, presets guardados, gráficos de rendimiento, navegación comercial, PDF web, edición directa de nodos SVG ni lotes superiores a 12. Next.js y el BFF siguen siendo consumidores del motor, sin lógica generativa propia.

Composición original basada en las tareas y límites de MosaicForge. No se consultaron ni trasladaron pantallas, identidades o assets de productos propietarios. La colección local `awesome-design-md` queda como referencia de lectura crítica para una fase posterior, sin extraer layouts ni marcas.

## Tres direcciones posibles

| Dirección | Composición y carácter | Beneficio | Coste o riesgo |
| --- | --- | --- | --- |
| **Mesa de variantes — recomendada** | Inspector claro a la izquierda y galería abierta a la derecha; fondo gris neutro, superficies blancas y acción oscura. Índices y estados alineados bajo cada imagen. | Compara resultados sin que el color de la aplicación compita con ellos. Controles, estado y descarga conviven en una sola pantalla. | Requiere jerarquizar y plegar controles secundarios para evitar un formulario excesivamente largo. |
| Cuaderno de pruebas | Flujo vertical con importación, configuración y resultados en bandas; blanco cálido y tipografía sobria. | Orden fácil de entender en móvil y primera visita. | Las iteraciones obligan a desplazarse entre reglas y resultados; menos adecuada para uso repetido en escritorio. |
| Visor oscuro | Controles y marcos de gris profundo, con fondos de imagen claramente delimitados. | Distingue bien el área de trabajo del documento y funciona con imágenes luminosas. | Complica percibir transparencias y piezas oscuras; exige otra revisión de contraste y no justifica añadir selector de tema en esta fase. |

Se recomienda **Mesa de variantes**: una herramienta de trabajo contenida, con el mismo orden de lectura que el flujo real. La comparación de imágenes determina la cuadrícula; no se utilizan tarjetas decorativas, escaparates de funciones ni una portada previa al editor.

## Estructura de la pantalla recomendada

- Cabecera de aproximadamente 56 px: nombre MosaicForge y título «Editor de variantes». Sin navegación que conduzca a funciones inexistentes.
- Área de trabajo con margen de 24 px, separación de 24 px y dos columnas desde 1024 px: inspector de 320–360 px y galería que ocupa el resto. Las medidas son objetivos iniciales, sujetos a contraste con el concepto de 2.4.
- Inspector en flujo normal de documento, sin un segundo scroll interno obligatorio. Contiene «Plantilla», «Generación» y «Reglas», seguidos por «Generar variantes». El botón muestra la cantidad pedida cuando ayuda: «Generar 6 variantes».
- Galería con título «Variantes» y resumen real, por ejemplo «3 de 6 procesadas · 2 aceptadas · 1 rechazada». Los recuentos avanzan solo con resultados recibidos; no simular porcentaje de cálculo.
- Cada celda mantiene su índice, área de preview y pie de estado en el mismo sitio durante la generación. No reordenar al aceptar o rechazar. El rechazo ocupa su posición original y conserva su motivo.
- Imágenes completas con `contain`, sin recortes, sombras, inclinaciones ni filtros sobre el resultado. Mantener una altura visual comparable y mostrar la relación de aspecto real. Transparencia indicada mediante un damero neutro dentro del visor, nunca horneado en los bytes descargados.
- Una variante aceptada ofrece dos acciones explícitas: «Descargar SVG» y «Descargar PNG». No hace falta seleccionar globalmente un formato ni abrir un diálogo de exportación para una descarga individual.
- Detalle desplegable «Datos para reproducir» con seed, índice, hash normalizado y versiones disponibles del resultado. No llenar la cabecera con estos valores largos. Permitir seleccionarlos y copiarlos como texto completo.

No se promete una vista previa del SVG recién seleccionado: antes de generar se muestra el resumen de inspección. Todas las imágenes del editor son PNG del motor; nunca se inserta el SVG recibido como HTML.

## Importación y controles

«Seleccionar SVG» abre el selector accesible; mostrar nombre y tamaño local antes de «Importar SVG». Seleccionar no envía el archivo. La ayuda indica «SVG UTF-8 · máximo 5 MiB». Tras inspeccionar, mostrar dimensiones, viewBox y recuentos entregados por el motor. «Cambiar archivo» permite volver al selector.

Los controles deben reflejar exactamente los contratos disponibles cuando se implemente 2.4:

| Grupo | Presentación propuesta |
| --- | --- |
| Generación | Seed e índice inicial como campos de texto decimal, cantidad de 1 a 12; etiquetas persistentes y ayuda breve sobre reproducibilidad. Nunca truncar ni convertir u64 a Number. |
| Transformación | Traslación, rotación, escala y opacidad con mínimos/máximos emparejados, unidades visibles y campos numéricos editables. Evitar que un deslizador sea la única forma de dar un valor exacto. |
| Composición | Espejos y mezcla de capas mediante controles con etiqueta; distorsión según parámetros efectivos del contrato. |
| Color y encuadre | Paleta opcional, crop opcional y fondo con activación explícita; mostrar sus campos cuando se activan y explicar las unidades del motor. |

«Generación» y «Transformación» están abiertas inicialmente; grupos secundarios usan desplegables semánticos. Los valores predeterminados son válidos, visibles y proceden del modelo acordado, no de ejemplos de una maqueta. El motor conserva la autoridad de validación; la web señala errores de entrada sin recrear sus reglas geométricas.

Editar controles no genera automáticamente. Al modificar una configuración se indica «Configuración modificada. Genera una nueva serie». La serie anterior deja de ser el resultado vigente; si se mantiene temporalmente visible, queda identificada como anterior y sus descargas siguen ligadas a su copia inmutable. Una respuesta antigua nunca sustituye la serie actual. El tratamiento preciso de retirada visual debe mantenerse consistente con el modelo de revisiones implementado.

## Color, tipo y geometría

Paleta candidata neutral; los códigos son decisiones de diseño propuestas, no una certificación de contraste. Medir todos los pares y estados en 2.4–2.5.

| Rol | Candidato | Uso |
| --- | --- | --- |
| Fondo de trabajo | `#F3F4F4` | Marco de la aplicación; no modifica el color de los artefactos. |
| Superficie | `#FFFFFF` | Inspector, campos e información de resultados. |
| Texto principal / acción primaria | `#192323` | Texto y botón principal con etiqueta blanca. |
| Texto secundario | `#526060` | Ayuda y metadatos; nunca información esencial atenuada hasta resultar ilegible. |
| Borde de control | `#788787` | Identificar campos y controles; separar de los divisores puramente decorativos. |
| Foco / selección | `#005F67` | Contorno de foco visible y estado seleccionado, acompañados por forma o texto. |
| Aceptada | `#24583B` sobre `#EAF3EC` | Palabra «Aceptada» y detalle breve. |
| Rechazada | `#805000` sobre `#FFF3DD` | Palabra «Rechazada» y motivo; distinguir validación de fallo técnico. |
| Error | `#9E2424` sobre `#FCEDED` | Fallos de importación, transporte o exportación. |

Una sola familia sans serif de licencia abierta: **Geist Sans**, alojada localmente si se incorpora y tras verificar licencia. Fallback nativo del sistema. Texto de lectura y campos a 16 px; etiquetas y metadatos de 14 px como mínimo; títulos de grupo 18 px y título de pantalla 24–28 px. Interlineado 1.45–1.6, cifras tabulares para índices y dimensiones. No introducir tipografía propietaria, serif decorativa ni fuentes externas obligatorias.

Escala de espacios de 4/8/12/16/24/32 px. Radio de 6 px para campos y botones; imágenes con esquinas rectas o radio mínimo consistente. Superficies planas, sin tarjetas anidadas, cristal, gradientes, texturas sobre imágenes ni sombra permanente. Iconos solo cuando aclaren una acción, siempre acompañados por texto en importación, generación y descarga. Priorizar separación y alineación sobre ornamentación.

## Estados y mensajes

| Estado | Presentación y siguiente acción |
| --- | --- |
| Sin plantilla | «Importa un SVG para generar variantes». Selector e importación visibles; galería vacía sin resultados ficticios. |
| Archivo seleccionado / importando | Nombre, tamaño y botón explícito; durante inspección, «Importando SVG…». Conservar datos editados y anunciar el resultado sin mover el foco arbitrariamente. |
| Importación inválida | Mensaje junto al archivo y acción «Elegir otro SVG». Tamaño o formato indicados con claridad; no mostrar contenido SVG ni detalles internos. |
| Pendiente | Índice ya visible y texto «Pendiente» en el área reservada; sin miniatura inventada. |
| Generando | Texto «Generando» en la celda activa; solicitudes secuenciales. Mantener geometría estable y anunciar progreso agregado de manera moderada. |
| Aceptada | PNG real, índice, texto «Aceptada», descargas y datos de reproducción. |
| Rechazada | «Rechazada» y motivo del motor presentado de forma segura y legible; sin PNG ni descarga. No atribuirle un juicio estético que el motor no haya emitido. |
| Error / motor ocupado | Mensaje específico y reintento manual; «El motor está ocupado» para 429, diferenciado de rechazo válido. Conservar la configuración. |
| Motor ausente / espera agotada | «No se pudo conectar con el motor local» o «La solicitud superó el tiempo de espera». Reintento manual sin prometer que el cálculo previo ya se detuvo. |
| Descargando / fallo de exportación | Estado junto a la acción correspondiente, sin bloquear las otras celdas; permitir reintentar con los mismos datos originales. |

Los motivos largos pueden envolver varias líneas y revelar detalle sin tooltip obligatorio. No indicar «Guardado» ni «Descarga completada» sin evidencia del navegador; la aplicación puede anunciar que preparó o inició la descarga.

## Escritorio, móvil y accesibilidad

En 1366×768, importación y controles de generación deben ser alcanzables con poca lectura, y al menos una fila de resultados quedar visible una vez importado. Galería de dos o tres columnas según ancho disponible, procurando previews de al menos unos 220 px; el máximo de 12 no obliga a encajar toda la serie en una pantalla. No reducir imágenes y texto para conseguirlo.

Por debajo de 1024 px se pasa a una columna: plantilla, configuración, acción y resultados. En móvil de 390 px, margen de 16 px, campos mínimo/máximo apilados cuando sea necesario y galería de una columna. Un enlace «Ir a variantes» permite saltar el formulario. Grupos secundarios plegables reducen recorrido sin ocultar seed, índice o cantidad. Evitar barras flotantes que tapen errores o compitan con el teclado virtual.

Todo el flujo debe funcionar con teclado: selector, importación, desplegables, controles, generar y descargar. Etiquetas asociadas, agrupaciones mediante fieldset/legend cuando proceda, errores descritos junto al campo y foco visible. No depender de color, hover, arrastre o tooltip. Objetivo de área táctil: 44×44 px para acciones. Mantener contenido y foco al actualizar resultados; usar anuncios `polite` para avance y un tratamiento más directo para errores que impiden continuar.

No se anima la imagen ni el desplazamiento de la galería. Como máximo, transición breve de foco/acción o aparición de estado sin alterar posición; con movimiento reducido, contenido inmediato y estable. Evitar destellos, entradas en cascada y esqueletos que sugieran imágenes reales.

## Criterios verificables para 2.4–2.5

- Importar requiere una acción explícita; el estado inicial no incluye variantes simuladas ni llamadas automáticas por pulsación.
- A 1366×768 y 390×844, y con zoom al 200 %, etiquetas, mensajes y acciones son utilizables; a 320 px no aparece scroll horizontal de toda la página.
- No hay texto operativo inferior a 14 px en el tamaño base, valores u64 cortados sin acceso al valor completo ni miniaturas deformadas o recortadas.
- Contraste medido: al menos 4.5:1 para texto normal, 3:1 para texto grande y 3:1 en límites/indicadores esenciales de controles y foco. Revisar también hover, error y selección.
- La secuencia de tabulación sigue el orden visual; no hay trampas de teclado y todos los estados se entienden sin distinguir colores.
- Una serie con aceptadas, rechazos y fallos conserva orden, índices y razones legibles. La galería muestra como máximo 12 previews.
- Cambiar reglas o plantilla mientras llega una respuesta no publica resultados obsoletos. Descargar una tarjeta utiliza sus datos originales.
- Comparar concepto aceptado y capturas reales en al menos cinco aspectos: composición, texto, tipografía, color, espacios/componentes; añadir revisión de móvil y estados.
- Probar importación, generación real y descarga SVG/PNG con el motor; la paridad de bytes/hash se verifica con CLI según la especificación, no por similitud visual.

## Skills consultadas y trabajo diferido

Se leyó **frontend-app-builder** como guía principal para diseñar primero toda la superficie, distinguir la tipografía del editor de la imagen y comprobar posteriormente fidelidad y funcionamiento. Esta tarea pide solo preparación escrita: no se ejecutaron generación de imágenes, implementación, instalación, navegador ni comparación de capturas. En 2.4 corresponderá generar un concepto legible de pantalla completa, detalles de controles/estados y vista móvil, someterlo a aceptación, extraer tokens/componentes y realizar el ciclo de implementación y comparación con `view_image`.

Se contrastaron **minimalist-ui** para economía visual y jerarquía, y **high-end-visual-design** para consistencia y revisión de composición. Sus recetas incompatibles entre sí —bordes obligatorios frente a bordes prohibidos, tarjetas dobles, grandes márgenes comerciales o animaciones obligatorias— no se acumulan: esta propuesta elige un editor compacto y neutral conforme al alcance del producto. No se afirma haber ejecutado sus protocolos de implementación.

Se leyó **design-taste-frontend** para conocer su alcance: excluye explícitamente pantallas de aplicaciones y dashboards. No se aplica su sorteo de layouts, cuotas de fotografías, prohibición de índices ni flujo de marketing al editor; aquí el índice es un dato esencial y las imágenes son salidas reales del motor. No se ejecutaron sus scripts.

Se leyó **web-design-guidelines** para preparar la revisión futura. No se efectuó una auditoría con esta skill: no existe UI implementada que revisar y el encargo excluye navegación. En 2.4–2.5 habrá que obtener sus directrices vigentes y contrastarlas con el código y la pantalla real, junto a teclado y lector de pantalla.

Verificación de este documento: coherencia manual con la especificación y revisión de whitespace mediante `git diff --check`. Sin código de producto, tests, contraste medido, capturas ni resultados de rendimiento en este encargo.
