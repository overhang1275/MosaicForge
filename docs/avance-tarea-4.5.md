# Submódulo 4.5 — Integración y determinismo

Fecha: 2026-10-14. BASE 7f9c0f5. Usuario autorizó retomar con «continua». Alcance: cerrar tarea 4; detenerse antes de tarea 5.

- [x] ~~Integrar generate_variant con los módulos de 4.1–4.4.~~
- [x] ~~Comprobar hash original normalizado e intervalo de índices.~~
- [x] ~~Producir VariantResult generado, aceptado/rechazado, con metadatos consistentes.~~
- [x] ~~Comparar índices 0–199 al cambiar rango, orden, grupos y uno/dos hilos.~~
- [x] ~~Verificación final de suite CPU y prueba local integrada.~~
- [x] ~~Revisión independiente de toda tarea 4 y correcciones verificadas.~~
- [x] ~~Documentación, commit 8031f75 y punto de reanudación antes de tarea 5.~~

## Decisiones

Variant es un modelo local con VariantResult generado, Template normalizado solo si se acepta, parámetros efectivos, cobertura y número de intentos. No duplica el modelo del contrato. La plantilla se reimporta antes de comprobar su hash; metadatos públicos alterados no se usan como fuente de verdad.

El índice debe pertenecer al intervalo solicitado; startIndex/count solo validan pertenencia, nunca alimentan el PRNG. El rechazo conserva el índice y el motivo de search_candidate. Configuración o SVG original inválidos son errores de entrada.

Ruling: artifactHash queda ausente en este paso, tanto para solicitud SVG como PNG — se devuelve un candidato generativo interno; el exportador de tarea 5 aplicará dimensiones/fondo/codificación y calculará el hash de esos bytes finales. El Template aceptado ya conserva el hash de su SVG interno. Coste si esta separación no encaja con el exportador: ajustar la unión con tarea 5, sin atribuir al PNG un hash de SVG ni publicar un artefacto sin exportarlo.

No cambiar schemas, dependencias ni añadir hilos/colas al producto: los hilos y grupos de la prueba llaman a la misma función por índice; procesamiento CLI acotado se implementará en tarea 6. No implementar todavía limpieza de geometría oculta o exportación PDF: siguen requisitos-exportacion-impresion.md.

## Evidencia inicial

Cinco tests de integración/determinismo: RED observado con stub; GREEN cinco pasan. Hubo primero un error de compilación en el test por usar Into para RulesPaletteItem; se corrigió usando su parser generado antes de observar el RED funcional.

La prueba de planificación compara SVG normalizado completo, resultado JSON, parámetros, cobertura e intentos: lote 0–199, subrango 100–199, grupos de 17 reordenados e índices en reversa, y ejecuciones con uno y dos hilos. Otras pruebas cubren vínculo de plantilla, intervalo, metadatos alterados, SVG inseguro, rechazo al octavo intento, índice posterior y u64 máximo. No se mide rendimiento con estas pruebas.

Graphify consultado con budget 500: encontró referencias a generate_variant/VariantResult/hash; mostró 13 de 80 nodos y truncó el resto. Snapshot antiguo; contratos y código actual verificados directamente.

## Revisión independiente y correcciones

review_task4_final, contexto fresco y solo lectura, revisó toda tarea 4 y sus módulos conectados. No ejecutó tests Rust. Encontró dos P2/Important; ambos reproducidos en Rust antes de corregir:

- Final: fixed cobertura medida con proporción original — coverage_uses_cropped_viewbox_not_original_viewport_proportions RED→GREEN. Normalizar width/height únicamente en la máscara según el viewBox recortado; no cambiar el SVG del candidato ni implementar exportación.
- Final: fixed rotaciones exactas rechazadas por redondeo — exact_half_and_quarter_turns_fit_without_exceeding_scale_maximum RED→GREEN. Permitir discrepancia de comparación de hasta 8 × EPSILON relativa, capando la escala final al máximo. 180°/270° aceptados con escala exactamente 1; giro real de 1° con máximo 1 sigue rechazado. Es tolerancia aritmética de comparación, no tolerancia de píxeles/GPU.

Sin hallazgos Critical o Minor reportados. Un pase de correcciones; no se solicitó otra revisión. Suite final 67/67 CPU pasa tras ambos cambios.

Comportamientos que el revisor apartó y decisiones del implementador:

- Final: Ruling: artifactHash/dimensiones/fondo/codificación diferidos — mantener separación candidato/exportador indicada arriba; coste si no encaja: ajustar unión de tarea 5 y verificar bytes antes de publicación.
- Final: Ruling: eliminación de vectores ocultos diferida — requisito ya conservado para exportación/impresión, no parte del generador; coste si no se implementa allí: archivos finales pesados y procesamiento RIP sin resolver.
- Final: Ruling: estética semántica y cero transparencia absoluta no garantizados — la máscara valida ocupación y esquinas, conserva espacio negativo; coste si se exige más: ampliar métricas/dataset, no anunciar validez universal.
- Final: Ruling: CUDA/paridad entre arquitecturas/rendimiento/RAM global no evaluados — fuera de implementación CPU actual y sin evidencia; coste para usarlos: hardware, perfil y pruebas adicionales antes de activación o promesas de capacidad.

## Verificación final

Desde services/engine, con Cargo en PATH:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --quiet
cargo test -p mosaicforge-core --test local_svg supplied_svg_has_varied_framing_without_blank_corners --locked -- --ignored --nocapture
```

Comandos ejecutados después de las correcciones: formato y Clippy exit 0; 67 tests CPU pasados, cero fallos; tres tests locales omitidos deliberadamente en suite normal. El test local integrado se ejecutó aparte y pasó: índices 0/1/17 aceptados, con SVG byte a byte igual a las muestras 4.4 guardadas. Los otros dos tests locales no se repitieron en este cierre (pasaron en 4.4). git diff --check pasa.

No se ejecutaron exportación general, PNG final, limpieza vectorial/PDF, Fiery, CLI, GPU, perfil de RAM o benchmark. Mantener rama actual para tareas siguientes; sin push/merge. Tarea 4 queda lista para checkpoint; tarea 5 necesita confirmación.
