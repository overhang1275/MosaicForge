# Submódulo 4.3 — Apariencia y prueba del SVG suministrado

Fecha: 2026-10-14. Implementación: core/src/appearance.rs. Export: lib.rs. Pruebas: tests/appearance.rs y tests/local_svg.rs. Preparación local: scripts/prepare-test-svg.py.

- [x] ~~Paleta y opacidad dentro de capas, preservando none y alfa.~~
- [x] ~~Orden de capas reproducible con aislamiento respecto a apariencia.~~
- [x] ~~Recorte comprobado dentro del viewBox.~~
- [x] ~~Pruebas CPU, fmt y Clippy.~~
- [x] ~~Probar test.svg sin modificar el original y guardar muestras SVG.~~
- [ ] Autorización del usuario para 4.4.

## Archivo del usuario

test.svg: 757135 bytes, viewBox 0 0 144 144; 623 paths, 273 grupos, 272 clipPaths y una sola capa de primer nivel. Incluye xmlns:xlink sin uso, style, class, title y data-name, fuera del subconjunto admitido. Importación directa ejecutada: rechazo explícito `unsupported XML namespace`. No ampliar el importador ni eliminar silenciosamente elementos del archivo original.

SHA-256 de los bytes originales: ceae058483698070aee16a8d1337715167142b198c47a28e626a0bb8683fbbf8. Se comprobó que los bytes no cambiaron después de preparación y generación.

El script local prepara únicamente este formato de exportación: exige exactamente su única regla CSS cls-1, convierte 351 atributos class en atributos de presentación equivalentes, retira style/title/data-name y la declaración de namespace sin uso. Comprueba que todos los datos d de paths se conservan; falla ante otra regla/clase. No es un conversor CSS general ni parte del importador del producto. La copia tiene 777922 bytes.

Importación preparada: 22239 segmentos; hash normalizado e8cfd87f3c8684427494222a03db48bbc8f2bfa1e5c6170ede7a556046474cac. Aplicación de rotación en [-15,15], tres colores, opacidad en [0.35,1] y crop [12,12,120,120]. Repetición por índice aislado coincide byte a byte. El barajado de múltiples capas se verifica en fixtures sintéticos porque este SVG tiene una sola capa de primer nivel.

| Índice | Muestra local | SHA-256 normalizado |
|---|---|---|
| 0 | out/test-4.3-0.svg | 9cbb2df42176b70402d4ac95e55ed8af5935e8ecbd3a369fb2fc3cd0cc4af2f8 |
| 1 | out/test-4.3-1.svg | 54f9a6a1665de3441ad12be59b42dfd045eb4361e1383eadf569d43a37a404df |
| 17 | out/test-4.3-17.svg | 6ebb1fad4c2282047acc515f6887a7851c9f69ca2ffbc4cd2fe57923fdd03c6c |

Las muestras y copia preparada están en out/ (ignorado por Git). El original permanece como entrada local del usuario. La prueba local lee archivos en ejecución: la suite estándar no depende de ellos.

## Reproducción y resultados

Desde raíz, con el Python disponible en el entorno:

```powershell
python scripts/prepare-test-svg.py
```

Desde services/engine, con Cargo en PATH:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --quiet
cargo test -p mosaicforge-core --test local_svg --locked -- --ignored --nocapture
```

Se usó Rust/Cargo 1.99 y Python 3.12 del runtime local. Formato y Clippy: exit 0. Suite CPU: 52 pasados, cero fallos, una prueba local omitida intencionalmente. Prueba local ejecutada aparte: una pasada, cero fallos. El script ejecutó un self-check de conversión CSS y comprobaciones de conservación de paths/original. No se midieron RAM ni throughput.

No se ejecutó rasterización PNG ni comparación de píxeles; corresponden a tarea 5. La admisión estructural no garantiza validez visual, cobertura o legibilidad; restricciones y reintentos siguen pendientes de 4.4. Generador completo pendiente de 4.5.
