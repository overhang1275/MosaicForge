# ADR0011 — persistencia de proyectos y revisiones (3.3)

Fecha:2026-10-19. Implementar después de verificar la descarga PDF binaria.

PostgreSQL ya fue elegido en ADR0001/0002. En3.3 aparece una frontera real de persistencia: un puerto Java sin Spring/JDBC guarda usuarios locales, proyectos con propietario y revisiones inmutables de SVG/configuración. El adaptador JDBC usa transacciones, claves foráneas, consultas parametrizadas y control optimista de revisión; el motor Rust sigue siendo autoridad de importación/identidad del SVG. Una migración inicial versionada se ejecuta transaccionalmente y no se vuelve a aplicar al reiniciar.

La API actual está autenticada entre servicios y solo expuesta mediante el BFF local. El propietario del estudio se configura por entorno, nunca se acepta desde el cuerpo ni cabeceras del navegador. `/v1/me` identifica ese usuario; no se inventa un login multiusuario ni se publica registro. La autenticación individual posterior deberá proporcionar el principal verificado al mismo puerto. La autorización siempre filtra por propietario, incluido acceso a revisiones; un proyecto ajeno devuelve404.

PostgreSQL se ejecutará en red privada Compose sin puerto host, con volumen duradero. La configuración/credenciales solo llegan a Java y a PostgreSQL. No se añaden Valkey, outbox, workers ni ejecución del millón en este submódulo.

API versionada: crear/listar/consultar proyectos; guardar/leer revisiones inmutables con número esperado para detectar conflictos. Listados limitados y paginados. El guardado conserva SVG, GenerationSpec, configuración PDF e índices seleccionados sin recalcular aleatoriedad. Los contratos bajo `contracts/projects/v1` y el OpenAPI correspondiente preceden a implementación y generan esquemas/tipos. Las revisiones del mismo proyecto se numeran dentro de una transacción.

Referencias: [restricciones PostgreSQL18](https://www.postgresql.org/docs/18/ddl-constraints.html), [configuración JDBC](https://jdbc.postgresql.org/documentation/use/). No se atribuyen mejoras de rendimiento a introducir una base de datos.
