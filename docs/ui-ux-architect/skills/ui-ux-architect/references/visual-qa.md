# QA visual con evidencia

1. Detectar comando y puerto reales; iniciar solo los procesos necesarios. No sustituir la aplicación por una maqueta sin declararlo. No exponer servidor a toda la red por comodidad.
2. Comprobar playwright-cli --version y --help. Si no está en PATH, localizar el shim/bin instalado; no reinstalar ni tocar PATH global por defecto. Usar sesión dedicada y navegador disponible, sin perfil personal.
3. Abrir URL real, obtener snapshot y actuar mediante refs actuales o locators observados. No inventar IDs. Navegar, completar un formulario/control y comprobar resultado visible. Conservar comandos de reproducción, sin datos sensibles.
4. Revisar escritorio (por ejemplo 1366×900) y móvil (390×844). Evaluar scrollWidth frente a clientWidth y detectar elementos que fuerzan desbordamiento; redimensionar viewport no sustituye pruebas de dispositivo táctil si son relevantes.
5. Capturar pantallas en rutas del proyecto destinadas a QA e inspeccionarlas. Revisar composición, jerarquía, espacios, contenido largo, controles, contraste, estados y foco. Si solo hay una fixture estática, escribirlo explícitamente.
6. Consultar consola y solicitudes fallidas de la sesión. No copiar bodies, headers de autorización o datos personales a logs/documentación. Revisar destinos/estado de red; no declarar red limpia si no pudo consultarse.
7. Corregir defectos encontrados y repetir viewport/acción; conservar antes/después. Un fixture con fallo conocido sirve para comprobar el proceso, no demuestra que el producto ya funciona.
8. Ejecutar pruebas del proyecto y la auditoría Vercel vigente si hay UI de producto. Guardar evidencia y pendientes. Cerrar solo la sesión/procesos creados para esta revisión; nunca close-all/kill-all o perfiles ajenos.

## Checklist de aceptación

- [ ] Flujo principal completa su tarea con datos reales o fixture declarada.
- [ ] Escritorio y móvil abiertos realmente; capturas inspeccionadas.
- [ ] Sin desbordamiento global accidental, texto ilegible o controles ocultos.
- [ ] Accesible por teclado, foco visible y nombres/etiquetas adecuados.
- [ ] Estados críticos y errores comprobados; no depender solo del color.
- [ ] Consola/red revisadas o limitación concreta documentada.
- [ ] Defectos encontrados corregidos y verificados de nuevo.
- [ ] Checks y auditoría aplicables ejecutados, sin promesas de conformidad total.
- [ ] Evidencia distingue producto, fixture y pruebas no ejecutadas.
