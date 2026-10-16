# UI/UX Architect — uso y mantenimiento

Agente personal reutilizable para Product Design, dirección artística, UI/UX, frontend y QA visual. Un solo flujo; no cinco agentes haciendo el mismo trabajo. Instalado y probado el 2026-10-15.

## Invocación

En cualquier proyecto abierto en Codex:

> Usa el agente ui-ux-architect para diseñar e implementar [alcance]. Conserva [funciones]. Valida escritorio y móvil y guarda evidencia en docs/.

También puede aplicarse sin otro subagente:

> Usa $ui-ux-architect para revisar y mejorar esta interfaz según su DESIGN.md.

El agente nativo se invocó con agent_type=ui-ux-architect y confirmó recibir sus instrucciones y leer su skill personal. No fija modelo ni permisos: hereda la sesión. La skill ya aparece en el catálogo de esta sesión; si otro cliente no refresca el catálogo, abrir una sesión nueva.

## Distribución de responsabilidades

| Archivo/recurso | Responsabilidad |
|---|---|
| ~/.codex/agents/ui-ux-architect.toml | Rol nativo, alcance e identidad; sin duplicar el proceso completo |
| ~/.codex/skills/ui-ux-architect/SKILL.md | Flujo descubrimiento, dirección, implementación, QA y auditoría |
| agents/openai.yaml de la skill | Nombre visible y ejemplo de invocación |
| assets/DESIGN.template.md | Plantilla adaptable para decisiones visuales por proyecto |
| references/tools.md | Cuándo usar cada capacidad y sus límites |
| references/visual-qa.md | Procedimiento y checklist con evidencia real |
| AGENTS.md del repositorio | Reglas del proyecto, autorización y funciones que preservar |

Las fuentes recuperables están en este directorio. AGENTS.md aquí contiene reglas de mantenimiento del paquete; no sustituye el AGENTS.md de tus proyectos. No se modificó el global ni el de MosaicForge para imponer un estilo en todos los trabajos.

Codex admite archivos personales TOML con name, description y developer_instructions en ~/.codex/agents; los agentes heredan capacidades/permisos. [Documentación oficial](https://learn.chatgpt.com/docs/agent-configuration/subagents). Las skills aportan el procedimiento y recursos independientes. [Skills](https://learn.chatgpt.com/docs/build-skills).

## Instalación en otro equipo

Copiar la carpeta skills/ui-ux-architect al directorio de skills personales reconocido por ese Codex y ui-ux-architect.toml a ~/.codex/agents/. En esta máquina se usan C:/Users/init/.codex/skills y C:/Users/init/.codex/agents; otros clientes también documentan ~/.agents/skills. No instalar la misma skill en ambos para evitar duplicados.

Antes de copiar, comprobar si esos destinos existen. Si existen, comparar y guardar una copia antes de actualizar; no sobreescribir configuraciones ajenas. No se necesita una entrada ficticia [agents.ui-ux-architect] en config.toml. La instalación de esta sesión creó solo archivos nuevos; el SHA-256 de config.toml coincidió antes/después.

La skill reutiliza las capacidades instaladas descritas en ../herramientas-diseno.md. Playwright CLI se verificó como 0.1.22, separado de Playwright Test. awesome-design-md es una colección local de referencias, no una skill ni dependencia del producto. No copiar toda la colección a cada repositorio.

## Actualización

- Agente/skill propios: editar estas fuentes, validar TOML/YAML y SKILL.md, comparar con la instalación y respaldar antes de reemplazar solo los archivos propios cambiados. Repetir el smoke test si se altera el flujo de QA.
- Skills de terceros: consultar su repositorio oficial; descargar la nueva versión a una carpeta temporal mediante skill-installer --dest, comparar y respaldar antes de adoptarla. Su instalador rechaza destinos existentes: no borrarlos para forzar la instalación.
- Taste: preferir la estable design-taste-frontend para páginas públicas; la variante v2-experimental instalada es histórica y no el default. La estable excluye app screens; no forzarla sobre el editor. No acumular reglas incompatibles de todas las skills.
- Playwright CLI: comprobar versión actual antes de actualizar el paquete global y repetir --version y la prueba de navegador. Su instalador admite install --skills=agents --global; aquí su skill ya estaba instalada por skill-installer, por lo que no se duplicó en otra carpeta. No sustituye a los tests de la aplicación.
- Referencias: actualizar el clon awesome-design-md solo tras comprobar su estado Git y preservar cambios locales. Extraer principios, no identidades de marcas.

## Prueba reproducible de herramientas

Desde raíz de MosaicForge, iniciar el servidor estático solo para la fixture:

```powershell
python -m http.server 5179 --bind 127.0.0.1 --directory docs/ui-ux-architect/qa
```

Desde out/uiux-browser, con playwright-cli disponible (ruta global de esta máquina en ../herramientas-diseno.md):

```powershell
playwright-cli -s=uiux-architect-qa open http://127.0.0.1:5179/smoke.html --browser=msedge
playwright-cli -s=uiux-architect-qa run-code --filename=C:/Users/init/Desktop/MosaicForge/docs/ui-ux-architect/qa/exercise.js
playwright-cli -s=uiux-architect-qa close
```

Adaptar la ruta al equipo y usar un navegador disponible. Los archivos del test son propios, sin datos personales. No se requiere Edge como dependencia del producto. No adjuntarse a un perfil personal ni usar close-all/kill-all. Cerrar solo el servidor/sesión creados para QA.

El script comprueba formulario/teclado, skip link, escritorio/móvil, desbordamiento, navegación, dos pares de contraste y consola/red; guarda capturas. Inspeccionarlas de verdad. Esta prueba demuestra las herramientas y un flujo frontend pequeño, no el editor de MosaicForge ni conformidad WCAG completa. Resultados en qa/resultado.md.

## Resolución de problemas

- CLI no está en PATH: usar el shim instalado de la versión Node activa; no editar PATH global por defecto. En esta sesión funcionó la ruta completa.
- Comillas en Windows: run-code --filename evita que el shim .cmd fragmente funciones JS en varios argumentos. El archivo debe contener una expresión de función, sin punto y coma final, como exercise.js.
- Fixture antigua en navegador: el test usa una query QA única; se verificó archivo/respuesta HTTP antes de atribuir el fallo a la UI.
- quick_validate.py necesita PyYAML. Se instaló PyYAML 6.0.3 en out/uiux-validator-deps para QA, sin alterar Python global. Ejecutar con ese PYTHONPATH o un entorno propio que incluya PyYAML. En Windows, los permisos del paquete descargado pueden exigir ejecución del validador bajo el usuario que lo instaló.
- Metadatos con acentos: conservar UTF-8/LF. El inicializador produjo CP1252/CRLF en Windows; se normalizó y se comprobó el YAML instalado.
- Navegador o permisos bloqueados: documentar el fallo exacto y lo no validado. No declarar una captura o prueba como realizada si no ocurrió.

## Estado actual

Rol y skill instalados, validados y comparados byte a byte con estas fuentes. Selección nativa del agente comprobada. Fixture abierta realmente en Edge, escritorio/móvil, corregida tras auditoría y capturas inspeccionadas. La petición visual posterior del usuario manda para MosaicForge: ../diseno/referencia-usuario.md. La landing se difiere hasta terminar el producto. HTTP 2.2 y editor 2.4 no se adelantaron por esta instalación.
