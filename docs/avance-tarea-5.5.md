# Submódulo 5.5 — Validación integrada SVG/PNG

Fecha: 2026-10-15. Base 0056bfa. Usuario autorizó 5.5; pausar antes de 5.6.

- [x] ~~Ejecutar generación→validación→exportación→hash con el SVG del usuario.~~
- [x] ~~Comparar seis artefactos con las muestras 5.3 y repetir por subrangos.~~
- [x] ~~Comprobar equivalencia visual SVG/PNG, rechazo y fuentes intactas.~~
- [x] ~~Revisar muestras visualmente y registrar evidencia/checks.~~
- [x] ~~Checkpoint c5ef9c0 registrado; pausa antes de 5.6.~~

## Alcance

Solo pruebas y documentación del producto existente: no requiere una implementación nueva ni se fuerza un RED artificial sobre funcionalidad ya terminada. La nueva prueba local debe pasar con el flujo de 5.4. No se incorpora arte del usuario a fixtures obligatorios; se omite por defecto y exige los archivos locales indicados.

El original test.svg utiliza CSS y metadatos fuera del subconjunto seguro admitido. Continúa siendo rechazado explícitamente; se usa la copia out/test-prepared.svg del preparador específico scripts/prepare-test-svg.py, sin ampliar el importador ni alterar el original. El preparador conserva los paths, convierte únicamente el estilo conocido y no es un conversor SVG general.

La prueba usa exactamente las reglas y los índices 0, 1 y 17 de 4.4, salida blanca 512², ambos formatos. Compara bytes con 5.3, SHA-256 final y repetición con count=1/startIndex=índice. Renderiza el SVG final y compara RGBA con el PNG integrado. Guarda muestras, resultados y configuraciones en out/ sin sobrescribir contenido diferente. Estas escrituras son utilidades de pruebas, no la futura publicación segura de la CLI.

## Ejecución reproducible

Prerrequisitos locales: test.svg, out/test-prepared.svg y las seis muestras 5.3. Estas últimas son una referencia de regresión local; no se presentan como fixture portable en una copia limpia del repositorio.

```powershell
# Desde la raíz, con el Python de la sesión:
python scripts/prepare-test-svg.py
# Desde services/engine, Cargo en PATH:
cargo test -p mosaicforge-cpu --test local_pipeline --locked -- --ignored --nocapture
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --quiet
```

En esta sesión se invocó el Python incluido en C:/Users/init/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe. El preparador se volvió a ejecutar para comprobar que su salida coincide con la copia previa y que no altera el original.

Primer intento de compilación de la nueva prueba: tres errores E0277 al imprimir wrappers generados sin Display. Corregidos usando as_str(); no hubo un fallo observado del producto ni se cuenta como RED funcional. No se cambió código de producción, dependencias ni contratos.

Graphify consultado con presupuesto 300: 79 nodos, siete mostrados, truncado; snapshot antiguo contrastado por lectura directa del código actual. Revisión independiente global pendiente de 5.6.

## Resultados medidos

Seis exportaciones aceptadas, cada una en un intento, y seis repeticiones por subrango: bytes y metadatos iguales. Comparación SVG renderizado↔PNG integrado: cero diferencias RGBA en las tres muestras de 512² con fondo blanco. Se verificaron además los seis artifactHash con hashlib.sha256 de Python leyendo los archivos guardados, así como igualdad de bytes con las seis muestras 5.3.

| Índice | Bytes SVG | Bytes PNG | Archivo base |
|---|---|---|---|
| 0 | 771703 | 156608 | out/test-5.5-0 |
| 1 | 771705 | 170369 | out/test-5.5-1 |
| 17 | 771705 | 167128 | out/test-5.5-17 |

Cada base tiene .svg/.png y dos .result.json. Las configuraciones están en out/test-5.5-{svg,png}.spec.json. Son muestras de diagnóstico ignoradas por Git, sin manifiesto de lote ni publicación de CLI.

Revisión visual de los tres PNG: colores, trazos y encuadre conservados, sin márgenes nuevos. Las zonas blancas forman parte del dibujo original; esta prueba no aumenta el grosor de sus trazos ni elimina su espacio negativo.

Original test.svg: SHA-256 ceae058483698070aee16a8d1337715167142b198c47a28e626a0bb8683fbbf8. Preparador ejecutado con exit 0: 351 clases convertidas, 777922 bytes preparados, original intacto.

Checks finales exit 0: formato, Clippy --workspace --all-targets --locked -D warnings y suite normal de 86 tests (70 core + 16 cpu), cero fallos. Seis tests locales omitidos por defecto, incluido el nuevo; no se repitieron los cinco anteriores. Cargo emitió el aviso conocido de canonicalización de C:\Users\init.

Nuevo test local ejecutado aparte: exit 0, un test pasa en 86.66 s (duración de la prueba completa, no benchmark del generador). Caso adicional con opacidad cero: índice 17 rechazado como empty_viewport tras ocho intentos, sin bytes/hash incluso con presupuesto PNG cero. Original y copia preparada se compararon byte a byte al cerrar y quedaron intactos.

## Límites

Las comparaciones exactas corresponden a resvg 0.47.0, CPU y entorno fijado de esta sesión. No prueban fidelidad entre arquitecturas/renderizadores, legibilidad semántica ni aceptación Fiery. El fondo y encuadre se conservan; la geometría exterior sigue en SVG. No se midieron RSS/throughput, no se ejecutaron GPU ni PDF. No se implementaron batches, CLI, tamaños físicos, sangrado o limpieza geométrica.
