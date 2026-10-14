# Tarea 4 — Ejecución por submódulos

Fecha: 2026-10-14. Usuario autorizó empezar tarea 4 y pidió confirmar antes de cada submódulo siguiente para controlar sus tokens. Esta instrucción sustituye la ejecución continua de Superpowers para esta tarea.

- [x] ~~**4.1 Muestreo geométrico:** obtener traslación, rotación, escala y flags de espejo reproducibles desde GenerationSpec, seed, índice e intento. Congelar IDs de reglas y comprobar identidad/rangos/reordenamiento.~~ No modifica SVG todavía.
- [ ] **4.2 Aplicación geométrica:** componer transforms en el orden acordado alrededor del centro del viewBox, aplicar a SVG seguro y verificar identidad/orden/compatibilidad con importador.
- [ ] **4.3 Apariencia y composición:** color/paleta, opacidad, orden de capas y crop; determinismo y límites por capa.
- [ ] **4.4 Distorsión y restricciones:** deformación acotada, geometría válida, aceptación/rechazo por índice, máximo ocho intentos y motivos explícitos.
- [ ] **4.5 Integración:** generate_variant y pruebas completas de índices/rangos/hilos/grupos, commit de cierre de tarea 4.

Cada submódulo termina con tests/fmt/Clippy, commit y registro de avance. Preguntar al usuario antes de comenzar el siguiente. Si la sesión termina, retomar el primer submódulo pendiente sin repetir los completados.

## Decisiones 4.1

- Reutilizar validate_spec, sample_u32 y map_u32_to_range; sin dependencias nuevas.
- Parámetros calculados son un modelo local de dominio, no una copia manual de contratos HTTP/JSON.
- IDs de reglas v1: translateX=1, translateY=2, rotationDegrees=3, scaleX=4, scaleY=5. Cada campo usa draw=0; attempt 0–7 y variante global no dependen del lote.
- mirrorX/Y son flags explícitos del contrato y se conservan; no convertirlos en probabilidades no solicitadas.
- El sampler no comprueba pertenencia del índice a un trabajo: solo deriva parámetros por índice global. La orquestación/validación de generate_variant se concretará en integración; startIndex/count jamás alimentan el PRNG.
- La compatibilidad de los parámetros con SVG, recortes y validez visual se verificará en submódulos posteriores. 4.1 no genera archivos ni promete variantes válidas.
- Se consultó Graphify con presupuesto de 800 tokens: encontró generate_variant, orden de transformaciones y dominio seed/índice; resultado truncado. Se comprobó el plan/código actual directamente porque el snapshot precede tareas 2–3. No reconstruir todo el grafo para este paso pequeño.

## Punto de reanudación

4.1 terminado. Esperar respuesta afirmativa antes de empezar 4.2.

## Evidencia 4.1

BASE f4961f1. RED: cinco pruebas de transforms, cuatro fallan por sampler vacío; la prueba de rechazo pasa con el stub que rechazaba todo. GREEN: cinco nuevas + 32 anteriores = 37 tests, cero fallos/omitidos. Formato y Clippy --locked -D warnings pasan. Revisión local del diff: IDs distintos, draw fijo, entrada validada, sin estado compartido y flags explícitos. Revisión global independiente de tarea 4 se realizará al cerrar integración; no despachar revisores para cada submódulo pequeño consume menos contexto, conforme a la petición del usuario.

Archivos: core/src/rules.rs, core/tests/transforms.rs y export de módulo en lib.rs. Sin dependencias nuevas. Se probaron seed/índice u64 máximos, intentos 0–7 y fuera de rango, rangos inválidos construidos en Rust, aislamiento entre reglas y reordenamiento/subrango del lote.

Limitación deliberada: parámetros muestreados no son todavía matrices ni variantes SVG y no incluyen reglas de apariencia, crop o distorsión. No declarar tarea 4 completa.
