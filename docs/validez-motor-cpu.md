# Validez de las variantes CPU

Contrato comprobable de fase 1; no equivale a una evaluación universal de diseño o tipografía.

| Restricción implementada | Criterio | Evidencia |
|---|---|---|
| Entrada SVG | Lista blanca, sin contenido activo/recursos externos, <=5 MiB y límites de segmentos/IDs/referencias | tests template/svg y svg-admitido.md |
| Configuración | Rangos finitos, escala positiva, dimensiones/superficies y límites de trabajo; hashes/versiones coherentes | tests contratos/validation/rules |
| Aleatoriedad | Philox por seed, índice global, regla, intento y muestra; máximo ocho intentos por índice | tests rng, generate, constraints |
| Encuadre | Crop dentro de viewBox; ajuste geométrico dentro de rangos; no ampliar máximos para aceptar | tests constraints y ADR 0004 |
| Cobertura visible | Máscara alfa 256×256; alpha>=8, >=512 píxeles pintados, >=250 de 256 tiles ocupados (>=2 píxeles por tile), sin tiles vacíos en cuatro esquinas 2×2 | tests constraints con fixtures analíticos |
| Viewport de salida | Cobertura después del mismo cover xMidYMid slice que exporta SVG/PNG, antes de añadir fondo | regresión final_cover_must_not_accept_an_empty_visible_output |
| Encuadre PDF | Misma composición de viewport SVG, cover a trim y extensión a media/sangrado que geometría final; no contar fondo configurado | regresión physical_pdf_cover_cannot_publish_blank_artwork_hidden_by_background |
| Publicación | Artefacto completo y exclusivo; PDF se cierra antes de publicar; hashes y mapas de páginas validados | tests CLI/export/pdf_document/publish |

Los umbrales de cobertura son heurísticas explícitas sobre una grilla pequeña, no una prueba geométrica de cada punto ni una garantía de resolución final. Los contornos finos pueden ocupar muchos tiles con poca tinta; eso es válido para patrones de líneas como la muestra del usuario. El fondo añadido no sirve para ocultar un arte vacío.

El control PDF físico aborta con RenderFailed si una candidata aceptada en GenerationSpec no supera la cobertura de su página física. No inventa un rechazo de generación ni cambia su configuración/hash; no se publica documento parcial. Ajustar crop, proporciones físicas o reglas y generar otro trabajo. El sangrado no se inventa ampliando artificialmente artwork que no existe.

No se implementaron OCR/legibilidad de paths, contraste entre vecinos, área mínima por motivo, solapamiento semántico ni clasificación estética/ML. Se conservan como criterios de diseño futuros que requieren fixtures y umbrales propios antes de prometerlos. La fase 1 entrega restricciones geométricas/seguridad/cobertura verificadas; el producto debe mostrar rechazos y permitir revisión visual. Python/ML solo se considerará si las reglas resultan insuficientes en casos documentados.

Fix de cierre 6.6: antes se medía únicamente el crop, lo que podía aceptar un arte vacío tras cambiar proporción. Ambas regresiones se observaron rojas antes de corregir. La revisión independiente comprobó por lectura la alineación con el exportador final. Cambian decisiones de casos afectados en este prototipo no publicado; los fixtures reales cuadrados conservan sus bytes, sujetos a verificación del perfil y QA final. No prometer reproducibilidad entre snapshots de desarrollo distintos sin conservar el commit además de seed/config/backend/versión.
