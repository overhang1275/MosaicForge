# Avance 6.3 — grupos y concurrencia CPU acotada

2026-10-15. Base funcional 6.2: c8094ac. Ambos submódulos autorizados juntos; detenerse antes de 6.4.

- [x] ~~Prueba roja de 201 variantes: argumento --batch-size aún no reconocido (exit 2).~~
- [x] ~~--batch-size 1..10000 (100 por defecto), --threads 1|2 (1 por defecto); límites inválidos sin crear salidas.~~
- [x] ~~Pool Rayon local con número explícito de hilos; grupos procesados en ventanas de tamaño <=threads, sin acumular imágenes del grupo.~~
- [x] ~~Presupuesto compartido PNG: reserva estimada por candidato, espera si no cabe junto al otro, liberación tras publicar/soltar bytes y también ante error.~~
- [x] ~~201 variantes por formato SVG y PNG con seis combinaciones N=1/7/100 e hilos=1/2: resultados ordenados y bytes idénticos al baseline. Incluye último grupo incompleto y subrango 108..114. Fixture exige artefactos aceptados distintos.~~
- [x] ~~Índice u64::MAX con count=1 soportado sin desbordamiento de fin exclusivo.~~
- [x] ~~Pruebas de reserva simultánea, espera/liberación por error y planificación 4096×4096 con reserva >256 MiB que cabe en 512 MiB sin dividir límite por trabajador.~~
- [x] ~~fmt --check y Clippy --workspace --all-targets --locked --offline -- -D warnings pasan.~~
- [x] ~~cargo test --workspace --locked --offline --quiet: 115 tests pasan (73 core, 22 CPU, 20 CLI), nueve locales omitidos, cero fallos.~~
- [x] ~~CLI real: tres SVG y tres PNG del SVG del usuario preparado, N=7 y threads=2, exit 0; índice 0 idéntico a 6.2. Original test.svg conserva SHA-256.~~
- [x] ~~Revisión independiente estática de 6.2/6.3: sin hallazgos ni bloqueos; revisor no ejecutó Cargo para evitar colisiones de enlace en Windows durante la suite.~~

Implementación: CPU separa export_generated_variant de generate_and_export, compartiendo exportación/hash/validación. La CLI genera candidato, planifica, reserva y llama al mismo exportador. No se modifica algoritmo ni contrato publicado. Cada trabajador publica y destruye sus bytes; devuelve solo VariantResult. El hilo principal emite resultados en orden de índice dentro de ventanas de uno/dos elementos.

## Evidencia local

`out/cli-6.3-svg/variant-{0,1,2}.svg`: 771703, 771705 y 771707 bytes.

`out/cli-6.3-png/variant-{0,1,2}.png`: 156608, 170369 y 162086 bytes.

| Índice | SHA-256 SVG | SHA-256 PNG |
|---|---|---|
| 0 | b034fb2c67793a19fc7ba0d89ec575a693a738d83b8702d1fdd5ba22c7762c52 | 79a61dd36b58ec4934fa136d2604b4de9f329a8ec43f884261438d6f0e020966 |
| 1 | 866f2340e043a6babe264f16a68471cc791fe4679329c06ad480eacf2363d935 | 54ae0989bd3a155de68fe1c876060b375723c5a284fcb44928416f68ed0373e5 |
| 2 | ee09f3f34fbb4b7db8b1b0c67e176ebe5f8625a23fec903f34c1e3599e1d2dbb | dfb3429fc82193ca71c0aa5510c6a6e1d19a728fb7a4a47567b9a82722039208 |

test.svg original: ceae058483698070aee16a8d1337715167142b198c47a28e626a0bb8683fbbf8, sin cambios.

## Límites y continuidad

512 MiB es la suma máxima de reservas estimadas de buffers PNG, **no un máximo medido de RAM/RSS del proceso**. No se rasterizó una imagen 4096×4096 en el nuevo test: se comprobó su planificación y admisión. Los tests de presupuesto usan asignaciones pequeñas/unidades abstractas para verificar coordinación sin gastar cientos de MiB. No se midió aceleración ni rendimiento; perfil de etapas y memoria sigue en 6.6.

N=1 limita la simultaneidad a una variante; N mayor permite ventanas de dos con threads=2. Ante fallo pueden existir archivos completos publicados por otro trabajador de esa ventana, incluso si sus resultados no alcanzaron stdout. No se emite resumen de éxito en ese caso. No hay transacción del trabajo ni reanudación automática. Publicación requiere filesystem con hard links, sin fallback de reemplazo.

Rayon y cinco transitivas descargadas con permiso después de fallo de red del entorno; el usuario no necesita descargar nada. Inventario en dependencias-motor.md: 94 paquetes externos, seis nuevos MIT OR Apache-2.0.

**Siguiente autorizado pendiente de nueva confirmación:** 6.4, manifiesto JSONL persistente y cancelación cooperativa. Después 6.5 PDF CLI único y 6.6 perfil/cierre de fase 1. Fiery excluido. No avanzar a web/fase 2.
