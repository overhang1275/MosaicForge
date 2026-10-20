package dev.mosaicforge.api;

import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

final class JdbcProjectStore implements ProjectStore {
  private final String url, user, password;

  JdbcProjectStore(String url, String user, String password) {
    if (!url.startsWith("jdbc:postgresql:") || user.isBlank() || password.isBlank())
      throw new IllegalArgumentException("PostgreSQL configuration required");
    this.url = url;
    this.user = user;
    this.password = password;
    migrate();
  }

  private Connection connection() throws SQLException {
    var properties = new java.util.Properties();
    properties.setProperty("user", user);
    properties.setProperty("password", password);
    properties.setProperty("connectTimeout", "5");
    properties.setProperty("socketTimeout", "15");
    properties.setProperty(
        "options",
        "-c statement_timeout=10000 -c lock_timeout=5000 -c idle_in_transaction_session_timeout=15000");
    return DriverManager.getConnection(url, properties);
  }

  @FunctionalInterface
  private interface Work<T> {
    T run(Connection connection) throws SQLException;
  }

  private <T> T transaction(Work<T> work) {
    try (var connection = connection()) {
      connection.setAutoCommit(false);
      try {
        var result = work.run(connection);
        connection.commit();
        return result;
      } catch (SQLException | RuntimeException failure) {
        connection.rollback();
        throw failure;
      }
    } catch (SQLException failure) {
      throw new IllegalStateException("Project persistence unavailable");
    }
  }

  private void migrate() {
    transaction(
        c -> {
          try (var statement = c.createStatement()) {
            statement.execute("SELECT pg_advisory_xact_lock(761304331)");
            statement.execute(
                "CREATE TABLE IF NOT EXISTS mosaicforge_schema_versions (version integer PRIMARY KEY, applied_at timestamptz NOT NULL DEFAULT now())");
            try (var rows =
                statement.executeQuery(
                    "SELECT version FROM mosaicforge_schema_versions ORDER BY version DESC")) {
              if (rows.next()) {
                if (rows.getInt(1) < 1 || rows.getInt(1) > 2)
                  throw new IllegalStateException("Unsupported database schema version");
                return null;
              }
            }
            statement.execute(
                "CREATE TABLE mosaicforge_users (id uuid PRIMARY KEY, created_at timestamptz NOT NULL DEFAULT now())");
            statement.execute(
                "CREATE TABLE mosaicforge_projects (id uuid PRIMARY KEY, owner_id uuid NOT NULL REFERENCES mosaicforge_users(id), name varchar(120) NOT NULL CHECK (length(trim(name)) > 0), current_revision integer NOT NULL DEFAULT 0 CHECK (current_revision >= 0), created_at timestamptz NOT NULL DEFAULT now(), updated_at timestamptz NOT NULL DEFAULT now())");
            statement.execute(
                "CREATE INDEX mosaicforge_projects_owner_id ON mosaicforge_projects(owner_id, id)");
            statement.execute(
                "CREATE TABLE mosaicforge_project_revisions (project_id uuid NOT NULL REFERENCES mosaicforge_projects(id), number integer NOT NULL CHECK (number > 0), snapshot jsonb NOT NULL, created_at timestamptz NOT NULL DEFAULT now(), PRIMARY KEY(project_id, number))");
            statement.execute("INSERT INTO mosaicforge_schema_versions(version) VALUES (1)");
          }
          return null;
        });
  }

  @Override
  public void ensureUser(UUID owner) {
    transaction(
        c -> {
          try (var s =
              c.prepareStatement(
                  "INSERT INTO mosaicforge_users(id) VALUES (?) ON CONFLICT DO NOTHING")) {
            s.setObject(1, owner);
            s.executeUpdate();
          }
          return null;
        });
  }

  private static Project project(ResultSet row) throws SQLException {
    return new Project(
        row.getObject("id", UUID.class),
        row.getString("name"),
        row.getInt("current_revision"),
        row.getTimestamp("created_at").toInstant(),
        row.getTimestamp("updated_at").toInstant());
  }

  @Override
  public Project create(UUID owner, String name) {
    return transaction(
        c -> {
          try (var s =
              c.prepareStatement(
                  "INSERT INTO mosaicforge_projects(id,owner_id,name) VALUES (?,?,?) RETURNING *")) {
            s.setObject(1, UUID.randomUUID());
            s.setObject(2, owner);
            s.setString(3, name);
            try (var rows = s.executeQuery()) {
              rows.next();
              return project(rows);
            }
          }
        });
  }

  @Override
  public List<Project> list(UUID owner, UUID after, int limit) {
    if (limit < 1 || limit > 101) throw new IllegalArgumentException("Invalid page limit");
    return transaction(
        c -> {
          try (var s =
              c.prepareStatement(
                  "SELECT * FROM mosaicforge_projects WHERE owner_id=? AND (?::uuid IS NULL OR id > ?::uuid) ORDER BY id LIMIT ?")) {
            s.setObject(1, owner);
            s.setObject(2, after);
            s.setObject(3, after);
            s.setInt(4, limit);
            var result = new ArrayList<Project>();
            try (var rows = s.executeQuery()) {
              while (rows.next()) result.add(project(rows));
            }
            return List.copyOf(result);
          }
        });
  }

  @Override
  public Optional<Project> find(UUID owner, UUID id) {
    return transaction(
        c -> {
          try (var s =
              c.prepareStatement("SELECT * FROM mosaicforge_projects WHERE owner_id=? AND id=?")) {
            s.setObject(1, owner);
            s.setObject(2, id);
            try (var rows = s.executeQuery()) {
              return rows.next() ? Optional.of(project(rows)) : Optional.empty();
            }
          }
        });
  }

  @Override
  public Revision append(UUID owner, UUID id, int expected, String snapshot) {
    return transaction(
        c -> {
          try (var s =
              c.prepareStatement(
                  "SELECT current_revision FROM mosaicforge_projects WHERE owner_id=? AND id=? FOR UPDATE")) {
            s.setObject(1, owner);
            s.setObject(2, id);
            try (var rows = s.executeQuery()) {
              if (!rows.next()) throw new Missing();
              if (rows.getInt(1) != expected || expected == Integer.MAX_VALUE) throw new Conflict();
            }
          }
          Revision result;
          try (var s =
              c.prepareStatement(
                  "INSERT INTO mosaicforge_project_revisions(project_id,number,snapshot) VALUES (?,?,?::jsonb) RETURNING created_at")) {
            s.setObject(1, id);
            s.setInt(2, expected + 1);
            s.setString(3, snapshot);
            try (var rows = s.executeQuery()) {
              rows.next();
              result = new Revision(id, expected + 1, rows.getTimestamp(1).toInstant(), snapshot);
            }
          }
          try (var s =
              c.prepareStatement(
                  "UPDATE mosaicforge_projects SET current_revision=?,updated_at=now() WHERE id=? AND owner_id=?")) {
            s.setInt(1, expected + 1);
            s.setObject(2, id);
            s.setObject(3, owner);
            s.executeUpdate();
          }
          return result;
        });
  }

  @Override
  public Optional<Revision> revision(UUID owner, UUID id, int number) {
    return transaction(
        c -> {
          try (var s =
              c.prepareStatement(
                  "SELECT r.* FROM mosaicforge_project_revisions r JOIN mosaicforge_projects p ON p.id=r.project_id WHERE p.owner_id=? AND p.id=? AND r.number=?")) {
            s.setObject(1, owner);
            s.setObject(2, id);
            s.setInt(3, number);
            try (var rows = s.executeQuery()) {
              return rows.next()
                  ? Optional.of(
                      new Revision(
                          id,
                          number,
                          rows.getTimestamp("created_at").toInstant(),
                          rows.getString("snapshot")))
                  : Optional.empty();
            }
          }
        });
  }
}
