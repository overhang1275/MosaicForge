# Producción de hasta un millón de variantes

Requisito confirmado por el usuario el 2026-10-15 durante 2.4. Objetivo de producto obligatorio; no equivale a capacidad ya implementada o medida.

- Editor local: muestra interactiva de 1–12 variantes PNG; nunca cargar un millón de imágenes o Blob URLs en navegador.
- Estado actual: GenerationSpec engine/v1 permite count hasta 10 000 por ejecución; preview/v1 exige count=1 por petición. No cambiar esos contratos publicados de forma silenciosa.
- Fase 3: nuevo contrato de trabajo de producción permite total hasta 1 000 000 y divide en bloques con rangos absolutos de índices. Conserva plantilla/configuración/seed; no reiniciar índices en cada bloque. Elegir tamaño efectivo mediante medición de memoria/tiempo y presupuesto del dispositivo.
- Workers con concurrencia acotada, escritura incremental, progreso durable, cancelación y reintento idempotente. Separar total del trabajo y tamaño del bloque. Sin listas de artefactos enteras en RAM ni respuestas HTTP gigantes.
- Navegador con progreso y galería paginada; descargas sin materializar el trabajo completo en JS. PDF único con todas las variantes sigue siendo requisito; evaluar escritor/spool/disco y costes a esta escala antes de anunciarlo.
- Validación escalonada real: 1 000, 10 000, 100 000 y 1 000 000; tiempos por etapa, RAM pico, disco, cancelación, reinicio y reproducibilidad de índices a ambos lados de cada límite de bloque.

No se ejecutó una corrida de un millón en 2.4 ni se verificó su consumo/tiempo. Los tests de determinismo por subrangos existentes son una base, no evidencia de capacidad masiva.
