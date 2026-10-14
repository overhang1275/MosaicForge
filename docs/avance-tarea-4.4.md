# Submódulo 4.4 — Encuadre, distorsión y aceptación

Fecha: 2026-10-14. BASE d70a0f1. Usuario autorizó corregir las muestras y después completar 4.4. No comenzar 4.5 sin nueva autorización.

- [x] ~~Renderizar y revisar visualmente las tres muestras anteriores.~~
- [x] ~~Ajustar zoom mínimo para cubrir el crop sin exceder límites.~~
- [x] ~~Comprobar pintura real, clips, opacidad y esquinas con máscara CPU.~~
- [x] ~~Ampliar el preset local de posiciones, escalas y rotaciones; conservar las muestras anteriores.~~
- [x] ~~Distorsión acotada de paths sin deformar clips.~~
- [x] ~~Máximo ocho intentos, rechazo explícito y conservación de índices.~~
- [x] ~~Suite CPU, fmt y Clippy; pruebas locales y revisión visual.~~
- [ ] Autorización para integración 4.5.

## Implementación y decisiones

constraints.rs contiene ajuste geométrico, máscara y búsqueda de candidatos. distortion.rs deforma coordenadas locales de paths antes de geometría global. Se reutiliza el árbol XML privado de appearance.rs, ahora compartido solo dentro del crate. Sin nuevo servicio, lenguaje o contrato. La API pública generate_variant y VariantResult quedan para 4.5.

Distorsión v1: fases de campo sinusoidal con IDs de regla 9 y 10, draw=0, por seed/índice/intento. Amplitud por eje <= distortion/sqrt(2): cada extremo/control se desplaza una distancia euclídea <= distortion en coordenadas locales. Misma coordenada recibe el mismo desplazamiento; cierres siguen cerrados. Arcos se convierten a segmentos simplificados antes de deformarse. No se deforman defs/clipPaths ni figuras básicas. Esto no garantiza ausencia de auto-intersecciones ni legibilidad semántica.

Orden: distorsión local → muestreo/ajuste de escala dentro del rango → escala/espejo/rotación/traslación → apariencia/crop → cobertura. La búsqueda puede rechazar por frame_exceeds_scale_limits, invalid_geometry, empty_viewport, empty_corners o sparse_viewport. Configuración inválida o SVG original inseguro son errores de entrada, no ocho rechazos. Parámetros ajustados guardados en el candidato aceptado. No se añaden fondos ni se duplican motivos para ocultar huecos.

Ruling: adelantar resvg/tiny-skia CPU para máscara de cobertura — permite comprobar huecos reales solicitados por el usuario; coste: dependencia raster antes de tarea 5, sin API de exportación general. Detalle de reglas, límites y fuentes en adr/0004-cobertura-cpu-y-encuadre.md.

## Evidencia de tests

RED inicial: dos tests de cobertura fallan con máscara vacía. Después, tres tests fallan con stubs de encuadre/distorsión/búsqueda. Ampliación: test de parámetros fuera de rango reproduce aceptación indebida; se corrigió validando rangos y flags antes del ajuste. GREEN: ocho tests de constraints y 52 anteriores = 60 tests CPU, cero fallos. Suite normal omite tres tests locales; los tres se ejecutan expresamente sobre los archivos del usuario.

Un assert de sqrt(2) falló por comparar el último bit de dos evaluaciones trigonométricas equivalentes; se corrigió para comprobar diferencia < 1e-12. Esa tolerancia es solo para el resultado analítico del test, no una tolerancia CPU/GPU ni de píxeles.

Casos: lleno/transparente/línea/clip, opacidad cero, metadatos alterados, crop desplazado, espejos, límites de escala, esquinas inversas dentro de origen, desplazamiento de controles acotado, clips intactos, identidad de distorsión cero, repetición, aceptación después de un intento fallido, rechazo al octavo intento, entradas/candidatos inválidos e índice posterior independiente.

## Muestras del usuario

El test.svg original sigue intacto; la copia preparada y las muestras 4.3 se conservan. Se revisaron visualmente PNG CPU de las tres anteriores y las tres nuevas. Las originales tenían solo rotación ±15°, sin variar posición o escala; la cantidad de celdas ocupadas era 254, 253 y 254, con cero celdas de esquina vacías. Por ello no se afirma que estuviesen completamente vacías: el cambio añade prevención geométrica y mayor variedad.

Preset local nuevo: rotación [-180,180], posición por eje [-14,14], escala por eje [1.05,1.55], opacidad [0.8,1], misma paleta y crop [12,12,120,120]. El ajuste no supera esos máximos. Tres índices aceptados al primer intento, tres hashes SVG distintos y repetición por índice aislado idéntica.

| Índice | Celdas ocupadas / 256 | Celdas vacías en esquinas / 16 | Píxeles visibles / 65536 | Salidas locales |
|---|---|---|---|---|
| 0 | 251 | 0 | 17333 | out/test-4.4-0.svg y .png |
| 1 | 250 | 0 | 19046 | out/test-4.4-1.svg y .png |
| 17 | 253 | 0 | 18811 | out/test-4.4-17.svg y .png |

## Reproducción

Desde raíz: ejecutar scripts/prepare-test-svg.py con Python del entorno. Desde services/engine, Cargo en PATH:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --quiet
cargo test -p mosaicforge-core --test local_svg --locked -- --ignored --nocapture
```

No se midieron RAM total, throughput, contraste semántico, calidad estética general ni paridad GPU. PNG de diagnóstico 512 × 512 solo sirve para revisión local; exportación general, presupuesto de buffers y CLI siguen en tareas 5–6. Revisión independiente global sigue prevista para 4.5.
