# Línea base CPU de fase 1 — 2026-10-15

Medición local exploratoria, una corrida por caso. No es una promesa de rendimiento ni capacidad máxima.

## Entorno e insumos

- Windows x64, kernel reportado Microsoft Windows 10.0.26200; Intel Core i5-1235U. RAM inventariada en esta máquina: 7.69 GiB. Otras aplicaciones de escritorio seguían disponibles: no se aisló el equipo ni la caché.
- rustc 1.99.0 b940084d7, Cargo 1.99.0 5f94df478, LLVM 23.1.1, target x86_64-pc-windows-msvc; build release. Sin CUDA ni Python en el proceso del motor.
- test.svg original intacto, SHA-256 ceae058483698070aee16a8d1337715167142b198c47a28e626a0bb8683fbbf8. Se usa out/test-prepared.svg (777922 bytes), preparado con el conversor específico ya documentado; hash normalizado e8cfd87f3c8684427494222a03db48bbc8f2bfa1e5c6170ede7a556046474cac.
- Salida SVG/PNG 512×512, fondo blanco, seed u64 máximo y reglas de 6.3. 12 índices 0..11 por caso; PDF usa tres índices 0..2, 100×100 mm sin sangrado. No comparar throughput PDF contra SVG/PNG como si fueran cargas iguales.
- Configuraciones exactas, métricas y hashes versionados en mediciones/2026-10-15-configuraciones.json, 2026-10-15-cli-release.json, 2026-10-15-etapas-release.json y 2026-10-15-pdf-qa.json. No se versionan los artefactos de usuario ni builds.

## Memoria y duración de la CLI

scripts/profile-cli-phase1.py ejecuta serialmente cada CLI mediante measure-test-memory.py. El monitor consulta GetProcessMemoryInfo cada 50 ms, leyendo picos acumulados del sistema operativo: PeakWorkingSetSize y PeakPagefileUsage (commit privado). No son lo mismo ni equivalen al presupuesto de buffers. Incluye inicialización, importación, reglas, validación, exportación, hash, publicación y manifiesto. Excluye compilador, proceso Python monitor y QA posterior. El elapsed incluye lanzamiento/monitorización, no es tiempo CPU exclusivo.

| Formato | Variantes | N | Hilos | Segundos | Pico working set MiB | Pico commit privado MiB | Bytes artefactos |
|---|---:|---:|---:|---:|---:|---:|---:|
| SVG | 12 | 1 | 1 | 6.128 | 24.33 | 22.02 | 9260440 |
| SVG | 12 | 7 | 1 | 5.269 | 20.03 | 19.33 | 9260440 |
| SVG | 12 | 100 | 1 | 4.949 | 20.65 | 19.63 | 9260440 |
| PNG | 12 | 1 | 1 | 6.103 | 27.05 | 33.40 | 1937757 |
| PNG | 12 | 7 | 1 | 6.152 | 27.17 | 33.77 | 1937757 |
| PNG | 12 | 100 | 1 | 6.105 | 27.51 | 34.14 | 1937757 |
| PNG | 12 | 7 | 2 | 4.295 | 41.04 | 57.31 | 1937757 |
| PDF | 3 | 7 | 1 | 4.777 | 19.13 | 17.56 | 2730275 |

Todos los casos terminaron exit 0, sin rechazos ni temporales/artefactos adicionales. Se compararon hashes/bytes entre N/hilos y primeros tres índices con los baselines de 6.3. PDF completo idéntico a 5.6/6.5: SHA-256 dd1208f9fea7fa7150e4811bf56e33879ba2c99237aa847167b95508789cfa01. Original SVG sin cambios.

Dos hilos usaron más memoria y esa corrida fue más corta; no se infiere aceleración estable ni causalidad por N. N=100 es mayor que count=12 en estas mediciones. La prueba de independencia de grupos completos/incompletos N=1/7/100 sobre 201 índices es automatizada, con fixture pequeño, y se documenta aparte; no confundirla con este perfil real de 12.

## Etapas separadas

Test local ignorado render::profile::supplied_artwork_profiles_stages_in_release. Importación inicial: 17.391 ms; lectura: 0.607 ms. Una medición por cada candidata 0,1,2; intentos reales 1,1,4.

| Etapa | Rango observado ms |
|---|---:|
| Generación completa, incluidos reintentos/validación/reimportaciones | 208.671–277.678 |
| Muestreo de reglas de la candidata aceptada | 0.0009–0.0015 |
| Ajuste de encuadre | 16.512–17.854 |
| Distorsión, incluida revalidación (distorsión configurada 0) | 16.577–18.610 |
| Transformación geométrica | 36.809–44.790 |
| Apariencia/crop/paleta | 44.971–48.338 |
| Validación de cobertura de salida | 57.296–60.581 |
| Serialización/validación SVG | 76.022–78.927 |
| Preparación/validación/árbol PNG | 63.357–67.205 |
| Asignación/llenado canvas | 0.236–0.460 |
| Raster resvg | 31.573–37.907 |
| Codificación PNG | 12.339–14.214 |
| SHA-256 SVG+PNG | 0.468–0.487 |
| Escritura/sync SVG+PNG | 7.447–8.235 |

El test primero genera normalmente y después **repite** las etapas de la candidata aceptada. Comprueba parámetros, SVG interno, cobertura y bytes SVG/PNG contra la implementación/baselines. La generación inclusiva no se suma con el replay ni con los tiempos de CLI; las funciones de transformación/serialización incluyen sus propias reimportaciones. Los descartes de intentos previos solo están en el tiempo inclusivo. La escritura medida aquí excluye hard_link y manifiesto, que sí están en la medición end-to-end de CLI.

Las reservas estimadas PNG de esos tres candidatos fueron 47750490, 42792888 y 40955518 bytes; no son los picos reales de memoria. La repetición por etapas no es un benchmark de loops puros ni separa asignador de todas las etapas. No hay evidencia para decidir una migración a GPU; en estas muestras la generación/validación y preparación ocupan más que codificar PNG, y falta un entorno CUDA para comparar.

## Reproducción y límites

```powershell
cargo build --manifest-path services/engine/Cargo.toml --release -p mosaicforge-cli --locked --offline
cargo test --manifest-path services/engine/Cargo.toml --release -p mosaicforge-cpu --lib --no-run --locked --offline
python scripts/profile-cli-phase1.py
services/engine/target/release/deps/mosaicforge_cpu-dd34b1e0e33230bf.exe render::profile::supplied_artwork_profiles_stages_in_release --ignored --exact --nocapture
```

El nombre exacto del ejecutable de tests lo imprime Cargo y puede variar con toolchain/features. La prueba se ejecuta directamente tras compilar, sin compilador concurrente ni otro proceso del motor durante la medición. Los scripts crean out/phase1-6.6 y logs/artefactos de forma exclusiva: para otra corrida usar otra copia/ruta de QA; nunca borrar o sobrescribir el resultado anterior automáticamente. Necesitan los insumos locales/baselines de 6.3/5.6 indicados en el script/test; las configuraciones están preservadas en mediciones/ para regenerarlos, no se promete reproducción de herramientas QA sin preparar esos insumos.

Solo Windows/este equipo/este fixture y resolución. No se midieron límites de RAM, OOM real, tiempo máximo de cancelación, concurrencia de varios procesos, discos remotos, caché fría, Linux/macOS, GPU o millones de variantes. Una corrida por caso no da intervalos estadísticos ni prueba ausencia de crecimiento para todos los inputs. La CLI tiene límites de trabajo y presupuesto estimado separados de estas observaciones.

QA posterior: pypdf estricto y Poppler a 300 DPI pasan en el PDF nuevo: tres páginas, 14 objetos alcanzables, tres Forms, cero imágenes raster/recursos sin uso. Los PNG renderizados nuevos son idénticos byte por byte a los de la revisión visual de 5.6; primera página revisada visualmente otra vez. MAE/RMSE frente a resvg permanecen en los valores de 6.5, sin tinta extra/faltante lejos del borde interior. Diferencias de borde de 6/7/8 píxeles persisten; no afirmar paridad exacta entre rasterizadores.
