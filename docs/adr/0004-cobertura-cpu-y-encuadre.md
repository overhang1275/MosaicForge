# ADR 0004 — Cobertura CPU y encuadre de la superficie semilla

Fecha: 2026-10-14. Estado: aplicado en submódulo 4.4 por solicitud del usuario de evitar esquinas/huecos vacíos y ampliar variación. La aclaración del usuario sobre una semilla más grande se refiere al tamaño/contenido del SVG, no al entero del PRNG.

## Decisión

Comprobar dos cosas independientes antes de aceptar un candidato:

1. Invertir su matriz para comprobar las cuatro esquinas del crop contra el viewBox de origen. Si hace falta, aumentar ambas escalas por el mismo factor mínimo, conservando rotación, posición y proporción entre escalas. Si supera cualquiera de los máximos configurados, rechazar ese intento. No reducir el arte ni repetirlo fuera de su superficie útil; no exceder los rangos pedidos.
2. Renderizar una máscara CPU transparente 256 × 256 con resvg/tiny-skia y medir alfa visible. Un bounding box no detecta agujeros ni clips ni opacidad cero. Esta máscara consume 262144 bytes de píxeles; esto no representa el uso total de RAM del pipeline.

La rasterización pequeña se adelanta desde tarea 5 porque permite validar lo que el usuario acaba de observar. resvg 0.47.0 y tiny-skia 0.12.0 son dependencias open source Rust, de la familia ya elegida; no hay nuevo lenguaje, servicio, transporte ni puerto. El SVG final permanece vectorial. La API de exportación PNG y sus presupuestos se mantienen en tarea 5.

## Regla inicial de cobertura

Sobre la máscara 256 × 256, un píxel cuenta con alfa >= 8/255. Se divide en 16 × 16 celdas de 16 × 16 píxeles; una celda cuenta si contiene al menos dos píxeles visibles. Se exigen al menos 512 píxeles visibles, 250 de 256 celdas ocupadas y las 16 celdas de las cuatro esquinas (2 × 2 cada una) ocupadas.

Son umbrales iniciales comprobados con fixtures analíticos y tres muestras del usuario; no equivalen a cobertura de pintura del 97.7%, ni a cero píxeles transparentes, ni a validación semántica universal. El espacio negativo propio del dibujo se conserva. Un dibujo disperso o una línea demasiado tenue puede rechazarse intencionadamente en este modo de encuadre completo. Más adelante, otros modos necesitan diseño y contrato explícitos; no introducir ahora opciones sin consumidores.

El ajuste geométrico solo conoce la superficie rectangular semilla; los recortes internos y el contenido real los comprueba la máscara. Cambiar unidades, width o viewBox sin añadir arte no crea más superficie útil. Un SVG más amplio con más motivos permite explorar más posiciones y escalas; si la semilla no alcanza, se rechaza en vez de inventar contenido.

## Determinismo y límites

Máximo ocho intentos por índice, con dominio PRNG propio por intento. Se devuelve el motivo del último intento fallido, sin desplazar el índice siguiente. Las escalas efectivas ajustadas quedan en CandidateDecision para trazabilidad; el hash de configuración sigue basándose en la configuración original.

La decisión de cobertura pertenece a la referencia CPU y al entorno fijado. No se promete paridad entre arquitecturas/rasterizadores sin medirla. Una futura GPU no sustituirá esta decisión discreta sin demostrar paridad. No se añade fondo a la máscara para disimular huecos.

Revisión de integración 4.5: la máscara usa width/height derivados del viewBox ya recortado, evitando medir letterboxing introducido por dimensiones originales de otra proporción; solo cambia el SVG transitorio de medición. La exportación final aún debe aplicar su política de dimensiones/fondo. La comparación del zoom calculado con su máximo admite hasta 8 × EPSILON relativa para redondeo de inversión/trigonometría y capa la escala efectiva al máximo; no se amplían los rangos. Tests reproducen giros 180°/270° y un exceso real rechazado.

## Fuentes técnicas

Actualización de cierre 6.6 (2026-10-15): la medición de producción ya no se limita al crop. Se materializa el viewBox implícito y se mide el mismo viewport/cover de SVG/PNG. Un crop mayormente ocupado podía dejar vacío un cover muy estrecho; regresión observada y corregida. PDF mide además la composición del viewport SVG con el cover físico a trim y la extensión real a sangrado/media, sin fondo añadido y reutilizando la transformación del exportador. Si no supera cobertura física aborta el documento, sin inventar rechazo de generación. Detalles vigentes en [validez-motor-cpu.md](../validez-motor-cpu.md) y avance de 6.6; lo anterior conserva el historial de 4.5.

[resvg render](https://docs.rs/resvg/0.47.0/resvg/fn.render.html) documenta el render sobre pixmap; [resvg 0.47.0](https://docs.rs/resvg/0.47.0/resvg/) expone tiny-skia y usvg. La implementación de encuadre, reglas y restricciones es propia.
