# Registro de ejecución — tarea 3: importación SVG

Fecha: 2026-10-14. Plan: superpowers/plans/2026-10-14-motor-cpu.md. BASE: da6fa2a. Rama: feat/cpu-contracts.

- [x] ~~Leer estado/plan/reglas y comprobar callers; no existía import_svg.~~
- [x] ~~Crear fixtures y pruebas de seguridad/límites y observar RED.~~
- [x] ~~Implementar preflight, valores geométricos y límites de referencias.~~
- [x] ~~Normalizar representación, validar con usvg y calcular hash/capas.~~
- [x] ~~Comprobar quince tests SVG y suite total de 32 tests, formato y Clippy.~~
- [x] ~~Completar revisión independiente y corregir hallazgos.~~
- [x] ~~Guardar commit funcional y actualizar registro/checklist.~~

## Resoluciones

- Alcance limitado a tarea 3; no adelantar renderer, reglas generativas, CLI o servicios.
- Separar svg_values.rs (gramática/valores) y template.rs (XML/grafo/canonicalización) para que la frontera de seguridad tenga responsabilidades legibles.
- Canonicalizar XML admitido conservando capas; usar usvg para normalizar/validar geometría. Su writer podría colapsar grupos y no es la fuente de hash.
- Namespace SVG por defecto obligatorio; aliases/prefijos y unidades relativas no soportados explícitamente en esta primera entrega.
- Añadir magnitud máxima 1,000,000 y presupuesto de expansión de referencias, porque un input pequeño con grafo acíclico puede ser costoso o desbordar transforms al normalizar.
- IDs de capas anónimas layer:N evitan colisiones con IDs XML admitidos sin modificar el diseño fuente.
- Cero explícito en transform/opacidad conserva su significado SVG; visibilidad/estética se comprobarán en tarea 4. Se rechazan underflow involuntario y escalas no cero incompatibles con el backend.
- No usar DuckDB para estos fixtures pequeños o pruebas sintéticas en memoria. No instalar Python/ML.

## Evidencia

RED inicial: ocho tests fallan con importador stub. Primera compilación real expuso APIs UTF-8 nuevas de quick-xml0.42; se comprobaron y corrigieron las llamadas, sin bajar de versión para ocultar errores. GREEN inicial: ocho tests pasan.

RED adicional de XML malformado: declaraciones repetidas se normalizaban indebidamente a un SVG aceptado. Corrección: declaración única como primer evento, atributos explícitos y validación XML de caracteres; prueba pasa.

GREEN previo a revisión: once tests SVG pasan junto con los diez de contratos y siete RNG (28 total). Se corrigieron cuatro advertencias de estilo Clippy sin desactivarlas.

Revisión independiente encontró P2 de g dentro de clipPath, paths moveto-only, underflow f64→f32/unidades físicas/transforms locales y XML whitespace/orden. Se reprodujeron con tests RED y se corrigieron en las validaciones comunes. Última revisión por lectura: ningún hallazgo original pendiente. GREEN final: quince SVGtests + diez contratos + siete RNG = 32, cero fallos/omitidos; fmt/Clippy --locked -D warnings también pasan.

No se verificó aún raster/render/exportación; no se midió RAM/RSS ni throughput. No hay nuevas fronteras entre lenguajes, CUDA, web o infraestructura en esta tarea.

Commit funcional: cf8c819 (`feat(engine): validate and normalize safe SVG templates`). Tarea 4 pendiente; no adelantarla sin nueva instrucción.
