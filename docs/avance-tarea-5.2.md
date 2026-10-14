# Submódulo 5.2 — Render PNG CPU seguro

Fecha: 2026-10-14. BASE 143ecfb. Usuario autorizó 5.2; detenerse antes de 5.3.

- [x] ~~Implementar render_png y render_png_with_budget con resvg/tiny-skia.~~
- [x] ~~Preparar viewport seguro, dimensiones/fondo y encuadre centrado sin letterboxing.~~
- [x] ~~Planificar buffers temporales de capas aisladas y clips antes de crear el canvas.~~
- [x] ~~Decodificar PNG en tests y verificar píxeles, transparencia, clips y repetición.~~
- [x] ~~Generar y revisar visualmente tres PNG del archivo del usuario.~~
- [x] ~~Checks finales: 78 tests workspace, prueba local PNG, formato y Clippy pasan.~~
- [ ] Registrar hash del checkpoint antes de 5.3.

## API y encuadre

crate cpu::render expone render_png(svg, output) con presupuesto de 512 MiB, render_png_with_budget con disponible explícito y plan_png_render para inspeccionar reserva sin asignar pixmaps. OutputSpec generado se valida y debe pedir PNG. Se reimporta el SVG seguro antes del árbol raster y los resolvers de imágenes se desactivan explícitamente; usvg/resvg siguen sin features por defecto. Filtros, máscaras SVG, imágenes y texto fuera del subconjunto admitido se rechazan.

core::template::prepare_viewport valida SVG/salida y aplica width/height de output con preserveAspectRatio="xMidYMid slice": cubrir centrado, conservar proporción y recortar sobrante. No estirar ni añadir márgenes. Si faltaba viewBox, se materializa el rectángulo efectivo de origen antes de cambiar dimensiones. No añade fondo ni modifica el SVG de entrada.

Ruling: usar cover centrado — responde a la petición de evitar huecos/márgenes y no introduce una opción sin contrato. Coste si se necesita otro encuadre: diseñar/versionar esa política; este modo puede recortar motivos en los bordes cuando las proporciones difieren. El PNG respeta el fondo transparent/RGB solicitado. La exportación SVG final sigue en 5.3 y deberá compartir esta política.

Se añade EngineError::RenderFailed, código render_failed, para fallo de canvas/codificación; no confundirlo con SVG o configuración inválidos. No se forzó OOM real del equipo.

## Reserva de buffers

Se mantiene la reserva base de 5.1 y se añade un límite conservador de temporales que pueden coexistir según resvg 0.47.0 local (src/lib.rs, render.rs y clip.rs):

- Hermanos se renderizan secuencialmente: tomar máximo, no sumar todos.
- Grupo aislado conserva su RGBA mientras renderiza hijos o aplica clip: superficie propia + máximo de hijos/clip.
- Superficie aislada: bounding box transformado con margen AA conservador; limitado al máximo interno de cinco anchos por cinco altos del viewport.
- Cada nivel de clip conserva un RGBA antes del siguiente clip o de su máscara alfa; grupos internos con clip pueden requerir otro RGBA. Cachear costes por identidad del ClipPath, sin unsafe ni dependencias del orden del hash map.
- Aritmética comprobada y profundidad adicional limitada. Rechazar antes de reservar canvas si base + temporales supera disponible. Rechazar bounds que desborden las conversiones enteras conocidas de resvg, evitando capas omitidas silenciosamente.

Ruling: reserva conservadora, no pico medido — puede rechazar trabajos que una planificación más ajustada admitiría; coste: menor capacidad por solicitud, preferible a afirmar que solo el canvas determina memoria. Revisión obligatoria del estimador si cambia resvg fijado. Incluye superficies de píxeles y reserva de codificación; no es cota del RSS total ni medida de memoria de XML/árboles/curvas/allocator. No hay contador global de concurrencia: el caller debe coordinar presupuestos entre llamadas; la CLI de tarea 6 implementará esa coordinación. Resolución máxima admitida por contrato no implica que toda plantilla compleja quepa a esa resolución.

## Tests y depuración

RED inicial: nueve tests CPU, cinco fallan con stubs y cuatro pasan (incluidos tres de 5.1). El test de temporales detectó que SVG sin viewBox mantenía su tamaño de dibujo al cambiar width/height. Se inspeccionó árbol usvg: grupos aislados correctos pero transforms identidad y bounds de 100 frente a salida 256. Se corrigió materializando viewBox original; añadido caso de píxeles para tamaños 1×1 y 32×16.

El fixture de clips se amplió a tres niveles: dos niveles sí cabían bajo la política de 512 MiB a 4096², así que su expectativa inicial de rechazo era incorrecta. Tres niveles superan la reserva y se rechazan sin crear un pixmap de salida grande.

Regresión adicional RED→GREEN: capa con escala 1000000 y rectángulo 1000000² excedía enteros usados por resvg y era admitida por el plan. Ahora se rechaza explícitamente. Sin modificar límites del importador original ni prometer render de geometría no representable.

Tests verifican dimensiones/fondo por píxel, bytes PNG repetibles, cambios de color, crop de proporción distinta, opacidad y clipping sobre fondo, temporales anidados, frontera exacta de presupuesto, entrada insegura, mínimo 1×1, viewBox implícito, error render_failed y bounds extremos. Sin renderer propio ni librerías externas nuevas: cpu añade dependencia al resvg 0.47.0 ya fijado.

## Muestras locales

Prueba ignorada por defecto en cpu/tests/local_svg.rs usa out/test-4.4-{0,1,17}.svg. Salida 512 × 512, fondo #ffffff, PNG decodifica, alfa 255, contenido coloreado y repetición byte a byte. Salidas revisadas visualmente: mantienen el trazo fino y espacio negativo del arte original; no se alteró grosor para ocultar diferencias.

| Índice | Bytes PNG | Reserva base | Temporales calculados | Reserva total calculada | Salida |
|---|---|---|---|---|---|
| 0 | 156608 | 20971520 | 26778970 | 47750490 | out/test-5.2-0.png |
| 1 | 170369 | 20971520 | 21821368 | 42792888 | out/test-5.2-1.png |
| 17 | 167128 | 20971520 | 23450088 | 44421608 | out/test-5.2-17.png |

Las reservas son resultados del planificador, no mediciones de RAM. El PDF de referencia no se modificó ni se usaron sus assets. PNG no resuelve por sí solo la geometría oculta de exportaciones vectoriales; seguir requisitos-exportacion-impresion.md.

## Comandos

Desde services/engine, Cargo en PATH:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --quiet
cargo test -p mosaicforge-cpu --test local_svg --locked -- --ignored --nocapture
```

Graphify consultado con budget 300: 45 nodos, seis mostrados y resultado truncado. Snapshot antiguo; código y fuentes locales fijadas verificados directamente. Revisión local de este submódulo; independiente de toda tarea 5 prevista para 5.5.

Verificación final tras todas las correcciones: comandos anteriores con exit 0; 78 tests pasados (67 core + 11 cpu), cero fallos y cuatro tests locales omitidos por defecto. El test local CPU se ejecutó aparte y pasó sobre tres muestras, decodificando/reproduciendo PNG byte a byte. Los tres tests locales core no se repitieron en este cierre. Formato, Clippy --all-targets --locked -D warnings y revisión local del diff sin problemas pendientes.

Pendientes: SVG final 5.3, integración/artifactHash 5.4, revisión global 5.5 y CLI 6. No se midieron throughput/RSS, no se ejecutó GPU, PDF/Fiery ni PNG máximo de 4096²; ese tamaño se probó en planificación sin asignar el canvas grande.
