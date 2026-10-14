# Contratos del motor v1

Fuente neutral: generation.schema.json y result.schema.json, JSON Schema draft-07. Rust genera modelos con typify en build.rs; no mantener copias manuales de esos modelos.

## Validación de frontera

Deserializar no basta: un generador de tipos no ejecuta todas las restricciones JSON Schema. Tras deserializar GenerationSpec llamar a validate_spec; tras deserializar VariantResult llamar a validate_result. Ambas funciones son obligatorias en CLI/worker y en tests de contrato.

Seed/índice son cadenas decimales canónicas para preservar precisión en JavaScript. El patrón del schema limita forma y longitud; el validador semántico exige rango u64 (0–18446744073709551615) y comprueba desbordamiento del intervalo. No convertir estos campos a Number en TypeScript.

Objetos cerrados: campos desconocidos se rechazan. Versiones admitidas: schema 1, reglas 1, motor 0.1.0. Solo backend CPU por ahora; publicar otra versión de contrato antes de consumidores incompatibles.

Crop, artifactHash y reason pueden omitirse o ser null; se normalizan a ausencia al serializar. Una palette omitida o vacía significa no sustituir colores; límite 256. Rangos y rectángulos deben ser finitos; crop se comprueba contra el viewBox cuando haya una plantilla (tareas posteriores).

## Hash de configuración

SHA-256 de la representación JSON de GenerationSpec validado y normalizado por los tipos generados. Claves ordenadas por serde_json sin preserve_order. Se excluyen count y startIndex; se conservan plantilla, seed, reglas, salida y versiones. El esquema de hash pertenece a rulesVersion; no cambiarlo sin versionar.

Los campos numéricos equivalentes se normalizan al mismo tipo f64, pero no se promete equivalencia semántica de reglas expresadas de formas diferentes. No hay timestamps, opciones de hilos ni IDs operativos dentro del hash.

El fixture identity.json de services/engine/tests/fixtures usa un hash de plantilla cero como marcador de pruebas; no es una plantilla válida para generación real. La comprobación contra el SVG se implementará al importar.

## Estado

Estos archivos definen y validan contratos. El PRNG vive en rng (ADR 0003) y la importación segura en template (docs/svg-admitido.md); todavía no hay reglas generativas, renderizador ni CLI funcional. No hay resultados de benchmark del motor ni pruebas CUDA.
