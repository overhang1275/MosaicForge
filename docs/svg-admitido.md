# SVG admitido por el importador CPU

Actualizado: 2026-10-14. Implementación: core::template::import_svg(bytes). No es todavía un renderer ni un generador de variantes.

## Entrada y errores

Solo UTF-8, XML 1.0 y namespace SVG por defecto declarado en la raíz. Si existe declaración XML, debe ser el primer evento, única, con version=1.0, encoding UTF-8 opcional y standalone yes/no opcional. Se rechazan nombres prefijados, declaraciones de namespace adicionales/reasignadas, DTD, entidades no predefinidas, instrucciones de procesamiento, CDATA y caracteres XML inválidos.

La raíz exige width+height o viewBox. viewBox tiene cuatro valores finitos y ancho/alto positivos. Longitudes absolutas admitidas: sin unidad, px, in, cm, mm, pt, pc, con DPI 96. Porcentajes/em/ex no están admitidos inicialmente. Magnitud máxima de números de origen, coordenadas/longitudes convertidas, componentes de transforms locales y acumulados: 1,000,000. Se rechazan números no cero que se redondearían a cero en f32, antes y después de convertir unidades. Transformaciones locales y acumuladas deben conservar escalas no cero superiores a f32::EPSILON según usvg; cero explícito es válido estructuralmente y su visibilidad se juzgará en tarea 4. Los errores usan código invalid_svg y mensajes estáticos; no incluyen contenido o rutas del usuario.

## Elementos y atributos

- svg (una raíz), g, defs, clipPath, path, rect, circle, ellipse, line, polyline, polygon.
- defs solo debajo de la raíz; clipPath solo debajo de defs y con figuras directas (sin grupos). Figuras no contienen otros elementos. Un path exige al menos un segmento LineTo/CurveTo/Quadratic, no solo MoveTo/cierre.
- Comunes: id, transform, fill/stroke sólidos o none, stroke-width, stroke-linecap, stroke-linejoin, stroke-miterlimit, fill-rule, clip-rule, fill-opacity, stroke-opacity, opacity, clip-path.
- Raíz: xmlns, width, height, viewBox, preserveAspectRatio.
- Rect: x/y/width/height/rx/ry; circle: cx/cy/r; ellipse: cx/cy/rx/ry; line: x1/y1/x2/y2; path: d; polyline/polygon: points; clipPath: clipPathUnits.
- Rangos de opacidad 0–1, miterlimit al menos 1, tamaños no negativos y tamaños principales de figuras positivos. Geometría de paths/points debe parsear completamente y conservar valores finitos acotados.
- IDs ASCII únicos, máximo 256 bytes; inicial letra/underscore, posteriores letras/dígitos/underscore/punto/guion.
- clip-path solo none o url(#id) hacia clipPath existente. Se resuelven IDs antes de normalizar; referencias externas o malformadas se rechazan después de decodificar entidades del atributo.

Todo elemento/atributo fuera de la lista se rechaza, incluyendo style/CSS, clases, imágenes, texto/fuentes, gradients/patterns, filtros, eventos, animation, foreignObject y use. El usuario debe convertir texto a paths. El whitespace interelemento/fuera de raíz se limita a los cuatro caracteres XML S (espacio, tab, CR, LF); la declaración conserva el orden version → encoding → standalone. No se eliminan silenciosamente características visuales no soportadas.

## Límites de recursos

| Recurso | Límite |
|---|---|
| Bytes de entrada | 5 MiB |
| Elementos XML | 20,000, incluida raíz |
| Profundidad | 64, incluida raíz y profundidad expandida por referencias |
| Segmentos | 100,000 en paths simplificados/estimación conservadora de figuras |
| Expansión de referencias | Máximos 20,000 elementos / 100,000 segmentos con reutilizaciones contabilizadas |

Comentarios y espacios se descartan sin conservar nodos; los limita el máximo de bytes. Ciclos, cadenas de recorte profundas y expansión combinatoria de referencias se rechazan mediante un DAG comprobado iterativamente antes de usvg. No se recorre recursivamente un grafo no validado.

## Representación y reproducción

Template expone width/height derivados de usvg, view_box de origen (o viewport), layer_ids, normalized_svg, hash y segment_count. Grupos directos de la raíz definen capas; sin id tienen identificadores lógicos layer:N, que no colisionan con IDs XML admitidos. Los demás grupos permanecen en el SVG.

normalized_svg conserva la geometría/capas de origen, ordena atributos, escapa valores, elimina comentarios/espacios interelemento/declaración y convierte tags vacíos a apertura+cierre. Hash SHA-256 de esos bytes. Roundtrip y cambio de orden de atributos/comentarios producen el mismo hash. No se promete mismo hash para SVG visualmente equivalentes con expresiones geométricas, unidades o colores distintos.

usvg construye y normaliza su geometría después del preflight; no se usa su writer como fuente canónica porque puede simplificar grupos/capas que necesita el generador. No se guardan rutas del host. usvg se compila sin sus features de texto/fuentes; ambos resolvers de imágenes retornan None. Se evita from_data (puede aceptar SVGZ); solo se usa from_str sobre XML ya validado.

Las restricciones de visibilidad, cobertura, recorte del contenido e integridad estética corresponden a tarea 4. Un SVG estructuralmente admitido no es una promesa de diseño legible o agradable.

## Verificación disponible

Tres fixtures originales (basic/layers/clip), XML hostil/malformado, atributos inesperados, referencias externas/escapadas/cíclicas, recursos excesivos, coordenadas inválidas, transforms acumulados, todas las figuras admitidas y unidades absolutas. Tests en cada límite y justo por encima, generando en memoria entradas grandes. Los resultados reales y el commit se registran en avance-tarea-3.md.

Fuentes técnicas: [quick-xml 0.42](https://docs.rs/quick-xml/0.42.0/quick_xml/), [usvg Options](https://docs.rs/usvg/0.47.0/usvg/struct.Options.html), [usvg Tree](https://docs.rs/usvg/0.47.0/usvg/struct.Tree.html), [resolvers](https://docs.rs/usvg/0.47.0/usvg/struct.ImageHrefResolver.html).
