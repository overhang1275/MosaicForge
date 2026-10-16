# Herramientas de diseño del asistente

2026-10-15. Instalación solicitada por el usuario. Son herramientas locales del asistente, no dependencias runtime de MosaicForge. No se añaden repositorios de terceros al producto.

## Skills instaladas

- frontend-app-builder: openai/plugins, plugins/build-web-apps/skills/frontend-app-builder. Instalada previamente con skill-installer.
- Leonxlnx/taste-skill: instaladas sus 14 carpetas de skills mediante skill-installer (equivalente a la selección completa del repositorio solicitado): brandkit, brutalist-skill, gpt-tasteskill, image-to-code-skill, imagegen-frontend-mobile, imagegen-frontend-web, minimalist-skill, output-skill, redesign-skill, soft-skill, stitch-skill, taste-skill-v1, taste-skill-v2-experimental y taste-skill.
- web-design-guidelines: vercel-labs/agent-skills, skills/web-design-guidelines.
- playwright-cli: microsoft/playwright-cli, skills/playwright-cli.

Destino: C:/Users/init/.codex/skills/. Cada instalación reportó éxito y conserva su SKILL.md. Las skills nuevas se ofrecen al asistente a partir del siguiente turno; instalar no equivale a ejecutar sus flujos completos ni obliga a aplicar recetas incompatibles simultáneamente.

Fuentes: [Taste](https://github.com/Leonxlnx/taste-skill), [Vercel](https://github.com/vercel-labs/agent-skills/tree/main/skills/web-design-guidelines), [Playwright](https://github.com/microsoft/playwright-cli/tree/main/skills/playwright-cli), [frontend-app-builder](https://github.com/openai/plugins/tree/main/plugins/build-web-apps/skills/frontend-app-builder).

## Referencias de diseño

awesome-design-md no es una skill: se clonó como colección de documentos en C:/Users/init/.codex/design-references/awesome-design-md. Snapshot local 3b605329240ce735b510037cca322f1831fdd7f8, 84 carpetas de referencias, licencia MIT del repositorio. No se ejecutó su contenido ni se copió un DESIGN.md de marca al proyecto.

Su uso debe respetar la originalidad exigida por AGENTS.md. La instalación aporta material de consulta, no autorización para copiar o aproximar UI, identidad o assets propietarios. [Repositorio fuente](https://github.com/voltagent/awesome-design-md).

## Playwright CLI

Instalado globalmente con npm install --global @playwright/cli@latest --ignore-scripts; versión comprobada 0.1.22, licencia declarada Apache-2.0. Las dependencias locales de ese paquete son Playwright/Core 1.64.0-alpha-1790635538000; todavía no se seleccionaron versiones para E2E del producto.

El shim global no está en el PATH de esta sesión. Comando verificado:

```powershell
& 'C:/Users/init/AppData/Local/Author Software/nvm/installs/v24.22.0/playwright-cli.cmd' --version
```

La ejecución devolvió 0.1.22. Después se ejecutó QA real en Edge sobre una fixture del agente, con teclado, capturas de escritorio/móvil y revisión visual. Evidencia y límites: [resultado](ui-ux-architect/qa/resultado.md). El frontend de MosaicForge todavía no está implementado ni probado.

## Agente de diseño

Agente reutilizable ui-ux-architect y skill personal instalados, validados y seleccionados mediante invocación nativa. Fuentes, uso y mantenimiento: [guía](ui-ux-architect/README.md). La propuesta inicial del agente quedó sustituida por la referencia posterior del usuario: [sistema visual vigente](diseno/DESIGN.md), estudio oscuro con acentos violeta/cian. La landing se difiere al cierre del producto.
