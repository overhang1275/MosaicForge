# Mantenimiento del paquete ui-ux-architect

Este directorio contiene fuentes reutilizables del agente y su skill, no la aplicación. Mantener responsabilidades separadas: TOML identifica al agente; SKILL.md coordina el flujo; referencias y plantilla aportan detalles. No duplicar instrucciones completas de otras skills.

Al actualizar, validar skill y TOML, comprobar rutas/referencias y conservar las configuraciones del usuario. No sobrescribir globales automáticamente ni añadir proveedores/permisos/modelos por defecto. Guardar pruebas de navegador y limitaciones en docs/ del proyecto que se revise. Una fixture aislada no valida el frontend de MosaicForge.
