# Submódulo 5.3 — SVG final

Fecha: 2026-10-15. Base add1eff. Usuario autorizó continuar; detenerse antes de 5.4.

- [x] ~~Exportar SVG vectorial con dimensiones, fondo y encuadre compartido con PNG.~~
- [x] ~~Separar el fondo de transformaciones, opacidad y clips del dibujo.~~
- [x] ~~Revalidar la salida y conservar los límites del importador.~~
- [x] ~~Comprobar suite completa, tres muestras reales y revisión visual.~~
- [ ] Registrar commit y punto de continuación.

## Implementación y decisiones

core::export::export_svg recibe SVG y OutputSpec generado; devuelve bytes SVG normalizados. Reutiliza prepare_viewport: width/height solicitados, viewBox explícito y preserveAspectRatio="xMidYMid slice" (cubrir centrado, sin estirar). No introduce dependencias, formatos ni contratos nuevos. No modifica el archivo de origen.

Con fondo transparente no añade geometría ni grupos. Con fondo RGB añade un rectángulo del viewBox con fill-opacity=1 y stroke=none. Si la raíz tenía transform, opacity o clip-path, mueve esos efectos a un grupo del dibujo para que no afecten al fondo. Mantiene defs en la raíz y los atributos de pintura heredables en su contexto; conserva IDs y orden de pintura. El grupo adicional puede cambiar la lista de capas de primer nivel; no se promete preservar esa estructura editorial en el artefacto final.

Se reimporta la salida con las mismas restricciones de seguridad, profundidad, nodos y segmentos. Una entrada justo al límite puede rechazarse si añadir el grupo/fondo excede el presupuesto; no se relajan límites ni se eliminan objetos para conseguir exportarla. Repetir la exportación de la misma entrada/configuración produce los mismos bytes. No se promete idempotencia al volver a exportar un SVG ya exportado con fondo.

El encuadre conserva geometría fuera de la ventana. No hay recorte real de paths ni limpieza de definiciones sin uso: esos requisitos de impresión siguen en 5.6. Tampoco hay PDF, integración de artifactHash ni publicación de archivos de producto en este paso.

## Evidencia

RED observado con stub: fallaron los tres tests core y el test CPU de equivalencia. GREEN dirigido tras implementar: los cuatro pasan. Cubren dimensiones, offsets negativos, fondo normalizado, vectores/IDs, repetición, transparencia, rechazo de formato/entrada/tamaño y profundidad máxima. El test CPU compara píxeles RGBA exactos de cuatro combinaciones de dos fixtures y dos fondos, con giro, clip, opacidad, atributos heredados y viewBox implícito. La equivalencia comprobada usa resvg 0.47.0 fijado; no se extrapola a otros renderizadores.

Graphify: consulta local con presupuesto 300, 35 nodos encontrados y ocho mostrados (truncada). Primer intento denegado por permisos de lectura; segundo ejecutado con elevación y exit 0. Snapshot anterior a tarea 2: las decisiones se contrastaron con código actual directamente.

Verificación final: formato y Clippy pasan; suite workspace: 82 tests (70 core + 12 cpu), cero fallos, cinco locales omitidos por defecto. El nuevo test local CPU se ejecutó aparte y pasó con tres variantes derivadas de test.svg: bytes SVG repetibles y píxeles RGBA idénticos al PNG del candidato original. Los otros cuatro tests locales no se repitieron en este paso. Cargo emitió un aviso de canonicalización de C:\Users\init; todos los comandos terminaron con exit 0.

| Índice | Bytes SVG | Bytes PNG diagnóstico | Salida SVG |
|---|---|---|---|
| 0 | 771703 | 156608 | out/test-5.3-0.svg |
| 1 | 771705 | 170369 | out/test-5.3-1.svg |
| 17 | 771705 | 167128 | out/test-5.3-17.svg |

Salidas 512 × 512, fondo blanco. Se revisaron visualmente los tres PNG de diagnóstico: mantienen el encuadre previo, colores y trazos, sin márgenes nuevos. El espacio negativo pertenece al dibujo original. Original test.svg intacto; muestras en out/ excluidas de Git.

## Comandos de verificación

Desde services/engine con Cargo en PATH:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --quiet
cargo test -p mosaicforge-cpu --test local_svg supplied_svg_exports_match_png_pixels --locked -- --ignored --nocapture
```

Revisión local del código y diff. Revisión independiente global al terminar 5.6 según plan aprobado. No se han medido RSS, throughput ni comportamiento Fiery.
