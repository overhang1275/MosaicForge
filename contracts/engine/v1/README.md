# Contratos del motor v1

Fuente neutral: generation.schema.json y result.schema.json, JSON Schema draft-07. Rust genera modelos con typify en build.rs; no mantener copias manuales de esos modelos.

## Validación de frontera

Deserializar no basta: un generador de tipos no ejecuta todas las restricciones JSON Schema. Tras deserializar GenerationSpec llamar a validate_spec; tras deserializar VariantResult llamar a validate_result. Ambas funciones son obligatorias en CLI/worker y en tests de contrato.

Seed/índice son cadenas decimales canónicas para preservar precisión en JavaScript. El patrón del schema limita forma y longitud; el validador semántico exige rango u64 (0–18446744073709551615) y comprueba desbordamiento del intervalo. No convertir estos campos a Number en TypeScript.

Objetos cerrados: campos desconocidos se rechazan. Versiones admitidas: schema 1, reglas 1, motor 0.1.0. Solo backend CPU por ahora; publicar otra versión de contrato antes de consumidores incompatibles.

Crop, artifactHash y reason pueden omitirse o ser null; se normalizan a ausencia al serializar. Una palette omitida o vacía significa no sustituir colores; límite 256. Rangos y rectángulos deben ser finitos; crop se comprueba contra el viewBox durante generación.

## Hash de configuración

SHA-256 de la representación JSON de GenerationSpec validado y normalizado por los tipos generados. Claves ordenadas por serde_json sin preserve_order. Se excluyen count y startIndex; se conservan plantilla, seed, reglas, salida y versiones. El esquema de hash pertenece a rulesVersion; no cambiarlo sin versionar.

Los campos numéricos equivalentes se normalizan al mismo tipo f64, pero no se promete equivalencia semántica de reglas expresadas de formas diferentes. No hay timestamps, opciones de hilos ni IDs operativos dentro del hash.

El fixture identity.json de services/engine/tests/fixtures usa un hash de plantilla cero como marcador de pruebas; sustituirlo por el hash del SVG importado para generación real. generate_variant reimporta el SVG normalizado y exige que coincida con templateHash; también valida pertenencia del índice al intervalo solicitado.

## Candidato y artefacto exportado

generate_variant devuelve un Variant local con VariantResult generado y, si se acepta, Template con SVG interno, parámetros efectivos, cobertura y número de intentos. Rechazo conserva el índice y motivo, sin SVG aceptado. Los metadatos solo declaran backend/dispositivo CPU actualmente.

artifactHash queda ausente hasta exportar: el SVG interno todavía no tiene aplicadas las dimensiones/fondo finales de output, y una solicitud PNG todavía no está codificada. Template.hash permite comparar el SVG interno; el exportador de tarea 5 calculará artifactHash de los bytes finales. No interpretar un candidato accepted como un archivo ya publicado.

## Estado

Contratos, PRNG, importador, generación y render PNG CPU están implementados. La máscara pequeña comprueba cobertura; SVG final, integración de bytes/hashes de exportación y CLI siguen pendientes. render_png aplica dimensiones/fondo y valida presupuesto de superficies; generate_variant sigue devolviendo candidatos sin artifactHash. No hay benchmark del motor ni pruebas CUDA. No se modificaron schemas.
