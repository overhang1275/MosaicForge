# Submódulo 5.4 — Integración de exportación y hashes

Fecha: 2026-10-15. Base b6b1664. Usuario autorizó 5.4; pausar antes de 5.5.

- [x] ~~Pruebas RED de generación→exportación y metadatos.~~
- [x] ~~Integrar SVG/PNG con hash de bytes finales y ausencia de artefactos rechazados.~~
- [x] ~~Comprobar repetición, rangos/orden/hilos, errores y presupuesto PNG.~~
- [x] ~~Formato, Clippy, suite y revisión local.~~
- [x] ~~Checkpoint funcional d72883f registrado; pausa antes de 5.5.~~

## Alcance y decisión

Ruling: integrar en cpu::export::generate_and_export — CPU ya depende de core y posee el rasterizador; evita dependencia circular o un servicio/interfaz nuevos. Retorna ExportedVariant con el Variant existente y artifact opcional de bytes. No crea otro modelo del contrato. generate_variant conserva su papel de candidato sin hash final.

El presupuesto PNG disponible es explícito y lo comprueba el renderer existente; para SVG no representa una cota de XML/RSS. Un rechazo devuelve su razón/índice sin bytes ni hash. Errores de generación/exportación devuelven Err y no se convierten en rechazo estético ni éxito parcial. El hash se asigna únicamente después de obtener todos los bytes. Sin escritura de archivos ni publicación: tarea 6. Se conserva un candidato y un artefacto por llamada; el caller deberá liberar/consumir cada resultado y coordinar concurrencia.

sha2 se añade como dependencia directa CPU para SHA-256; ya está fijada en el workspace por core, sin librería externa nueva. Sin cambio de schema, lenguajes, infraestructura o PDF.

Graphify: consulta presupuesto 300, 72 nodos encontrados y ocho mostrados, truncada. Snapshot antiguo contrastado con fuentes actuales; no reconstruido.

## Evidencia y límites

RED: cuatro tests compilan con stub; tres fallan por integración no implementada y el de errores ya pasa. GREEN dirigido: cuatro tests pasan tras implementar. Luego se refuerza el test de errores comprobando código invalid_spec por presupuesto insuficiente y recuperación de la llamada siguiente; el test de planificación exige resultados aceptados con artefactos para evitar una comparación vacía de rechazos.

Los tests comparan bytes contra los exportadores SVG/PNG existentes y SHA-256 sobre todos los bytes recibidos, distinto del hash SVG interno. Comprueban dimensiones decodificadas, metadatos de plantilla/configuración/índice, conservación de parámetros/cobertura/intentos, transparencia lógica de generate_variant (sin hash final) y rechazos empty_viewport sin hash/bytes incluso con presupuesto PNG cero. Tres índices con reglas variables se repiten en subrangos de una variante, orden inverso y tres hilos; mismos bytes y metadatos en ambos formatos. Esto verifica independencia del orden, no implementa lotes N o un coordinador de memoria.

Esta llamada integra generación y exportación en memoria. accepted más artifactHash significa bytes completos disponibles, no archivo persistido ni trabajo publicado. No cambia el contrato de candidatos ni impone hash a todo accepted del schema. Las fuentes originales no se modifican. No se ejecutan muestras reales nuevas en 5.4: su validación integrada corresponde a 5.5; la evidencia visual anterior permanece en avance-tarea-5.3.md.

## Verificación

Desde services/engine con Cargo en PATH:

```powershell
cargo test -p mosaicforge-cpu --test export --offline # RED, actualiza solo arista local sha2 en lock
cargo test -p mosaicforge-cpu --test export --locked # GREEN
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --quiet
```

Sin benchmark/RSS, CUDA, PDF/Fiery ni E/S. Revisión local de diff y llamadas; revisión independiente global al finalizar 5.6.

Verificación final exit 0: formato, Clippy --all-targets --locked -D warnings y 86 tests workspace (70 core + 16 cpu), cero fallos. Cinco tests locales omitidos por defecto y no repetidos en este submódulo. Cargo emitió aviso de canonicalización de C:\Users\init sin impedir los checks. Sin dependencias externas nuevas ni cambios de versiones.
