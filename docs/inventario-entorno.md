# Inventario del entorno

Fecha de comprobación: 2026-10-14. Inventario de solo lectura; no se instalaron herramientas ni se cambiaron PATH o configuración global.

## Hardware observado

| Recurso | Resultado |
|---|---|
| CPU | Intel Core i5-1235U, 10 núcleos, 12 procesadores lógicos |
| RAM física reportada | 8,259,252,224 bytes, aproximadamente 7.69 GiB |
| GPU reportada por Windows | Intel UHD Graphics |
| Driver de gráficos | 31.0.101.5590 |
| NVIDIA | No se detectó en Win32_VideoController; nvidia-smi no se encontró en PATH ni en rutas habituales consultadas |
| CUDA Toolkit | nvcc no encontrado en la ruta habitual consultada; no se comprobó funcionamiento CUDA |

La RAM disponible para el proceso será menor que la física. Empezar con paralelismo y colas acotados; medir antes de aumentarlos. La GPU Intel no permite validar el backend CUDA NVIDIA requerido.

## Herramientas comprobadas

| Herramienta | Versión ejecutada | Ruta / situación |
|---|---|---|
| Java/Javac | Temurin 21.0.12.1 | `C:/Users/init/scoop/apps/temurin21-jdk/current/bin/`; no resuelto inicialmente por PATH |
| Cargo | 1.99.0 | `C:/Users/init/.cargo/bin/cargo.exe`; no resuelto inicialmente por PATH |
| Rustc | 1.99.0 | `C:/Users/init/.cargo/bin/rustc.exe` |
| Clippy | Ejecutables presentes | Versión y ejecución de checks pendientes |
| Node | 24.19.0 | Runtime bundled de Codex, invocado por ruta absoluta |
| pnpm | 11.25.0 | Runtime bundled de Codex, invocado por ruta absoluta |
| Python | 3.12.14 | Runtime bundled de Codex, invocado por ruta absoluta |
| Git | 2.56.0.windows.2 | `C:/Program Files/Git/cmd/git.exe` |
| Maven / Gradle | No encontrados en PATH | Se propone Maven Wrapper, sin requerir instalación global |
| Docker / Podman | No encontrados en PATH | Docker no encontrado en la ruta habitual consultada; contenedores no probados |

Los comandos node/pnpm sin ruta absoluta, en el contexto elevado, devolvieron un aviso de nvm sin versión activa. Los binarios bundled sí devolvieron versiones. No concluir que una herramienta está ausente de todo el disco por no encontrarla en PATH.

## Evidencia y límites

- CPU, RAM y GPU: consultas CIM de solo lectura. El sandbox denegó las primeras consultas; funcionaron con permisos elevados.
- Versiones: comandos `java -version`, `javac -version`, `cargo --version`, `rustc --version`, `node --version`, `pnpm --version`, `python --version`, `git --version` por las rutas indicadas.
- No se ejecutaron compilaciones, instalaciones, servicios de base de datos/cola ni pruebas de contenedores.
- No se midió rendimiento CPU ni GPU.

## Acciones posteriores

- [ ] Configurar las rutas en scripts de desarrollo, sin hardcodear rutas personales en el código del producto.
- [ ] Comprobar rustfmt/clippy y build con un proyecto real al implementar.
- [ ] Confirmar PostgreSQL/Valkey y el método para ejecutarlos localmente.
- [ ] Validar herramientas de contenedores antes de prometer ejecución compose.
- [ ] Conseguir un entorno NVIDIA para la fase CUDA; conservar checks CPU independientes.
