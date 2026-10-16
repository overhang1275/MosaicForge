# Herramientas de contratos

Paquete privado de desarrollo; no instala Next.js ni un servicio. Requiere Node >=24.19.0 y pnpm 11.25.0. Ejecutar desde raíz:

```powershell
pnpm --dir scripts/contracts install --frozen-lockfile
pnpm --dir scripts/contracts run generate
pnpm --dir scripts/contracts run check
pnpm --dir scripts/contracts run test
pnpm --dir scripts/contracts run typecheck
pnpm --dir scripts/contracts run lint
pnpm --dir scripts/contracts run format
```

generate crea contracts/generated/typescript/preview.ts desde los esquemas reales; check solo compara, devuelve exit 1 si falta/difiere y no escribe. schema.mjs resuelve referencias locales dentro de contracts/, incluidas las del motor; no descarga esquemas. Comparación normalizada a LF para checkouts Windows con CRLF.

check.mjs es un verificador de fixtures, no motor generativo: Ajv draft-07 sin coerción/defaults ni eliminación de campos, más semántica de sobres. Rust reutiliza validate_spec/validate_result del core. Los fixtures de contracts/preview/v1/fixtures/index.json tienen el mismo veredicto en ambos entornos. Números u64 son cadenas. artifact:null representa ausencia; accepted exige bytes reales/base64 canónico y hash coincidente.

ESLint comprueba sintaxis y reglas de errores/variables en .mjs; Prettier formatea el paquete; TypeScript estricto comprueba las declaraciones generadas y ejemplos de consumo. No se tipa estáticamente el JS de herramientas como si fuera una app TS.

Inventario de licencias declaradas de los paquetes instalados: docs/dependencias-preview.json. Son metadatos de los paquetes locales, no una auditoría legal. Lockfile incluye resolución/integridad y variantes opcionales por plataforma; solo se ejecutaron checks en Windows x64.
