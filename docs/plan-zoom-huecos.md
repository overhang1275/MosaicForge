# Zoom y espacios originales del SVG — 2026-10-20

El usuario aprobó respetar los huecos originales siempre que la página esté
cubierta geométricamente por el SVG transformado y se mantengan proporciones.
Diagnóstico: con test.svg, seed 2026, escala 1..4, los nueve resultados aceptados
quedaron entre 1.22 y 1.66. Nueve pruebas fijas 2/3/4 fueron rechazadas por
empty_corners/sparse_viewport: el filtro confunde espacio negativo con fallo.

Introducir rulesVersion 3 con candado explícito como 2. Se conserva el mínimo
de 512 píxeles pintados en la máscara 256² para rechazar salidas vacías; los
huecos/esquinas sin tinta dejan de ser rechazo por sí solos. El ajuste geométrico
de cobertura sigue obligatorio. El PDF también comprobará el marco transformado
contra la página física completa, incluido sangrado, antes de aceptar huecos.
Versiones 1/2 mantienen exactamente su validación y sus bytes reproducibles.

- [x] ~~Contratos 3 aditivos y validadores Node/Java/Rust; pruebas de presencia del candado.~~
- [x] ~~RED de SVG con espacio negativo; aceptar 3 y preservar rechazo 2, vacío y marco descubierto.~~
- [x] ~~Aplicar criterio común a PNG/SVG/PDF con comprobación geométrica física.~~
- [x] ~~Probar test.svg, 12 variantes con 1..4, medir escalas y PDF vectorial.~~
- [x] ~~Verificar legado, formato/lint/tests, actualizar Docker y checkpoint Git.~~

No se promete legibilidad estética universal; no se cambia el PRNG ni se relajan
los límites geométricos para inventar variantes completas.
