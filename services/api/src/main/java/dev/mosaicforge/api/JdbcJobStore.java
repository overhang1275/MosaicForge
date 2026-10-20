package dev.mosaicforge.api;

import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.sql.Timestamp;
import java.time.Clock;
import java.time.Duration;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

final class JdbcJobStore implements JobStore {
  private final String url, user, password;
  private final Clock clock;
  private final int retryBudget;

  JdbcJobStore(String url, String user, String password, Clock clock, int retryBudget) {
    if (retryBudget < 1 || retryBudget > 100)
      throw new IllegalArgumentException("Invalid retry budget");
    new JdbcProjectStore(url, user, password);
    this.url = url;
    this.user = user;
    this.password = password;
    this.clock = clock;
    this.retryBudget = retryBudget;
    tx(
        c -> {
          try (var s = c.createStatement()) {
            s.execute("SELECT pg_advisory_xact_lock(761304331)");
            s.execute(
                "CREATE TABLE IF NOT EXISTS mosaicforge_jobs (id uuid PRIMARY KEY, owner_id uuid NOT NULL REFERENCES mosaicforge_users(id), project_id uuid NOT NULL, project_revision integer NOT NULL, idempotency_key varchar(200) NOT NULL, digest char(64) NOT NULL, request jsonb NOT NULL, snapshot jsonb NOT NULL, state varchar(20) NOT NULL DEFAULT 'queued', next_index numeric(20,0), evaluated integer NOT NULL DEFAULT 0, accepted integer NOT NULL DEFAULT 0, rejected integer NOT NULL DEFAULT 0, fence bigint NOT NULL DEFAULT 0, lease_until timestamptz, block_id uuid, block_request text, retries integer NOT NULL DEFAULT 0, UNIQUE(owner_id,project_id,idempotency_key), FOREIGN KEY(project_id,project_revision) REFERENCES mosaicforge_project_revisions(project_id,number))");
            s.execute(
                "CREATE TABLE IF NOT EXISTS mosaicforge_job_blocks (job_id uuid NOT NULL REFERENCES mosaicforge_jobs(id), block_id uuid NOT NULL, response jsonb NOT NULL, PRIMARY KEY(job_id,block_id))");
            s.execute(
                "CREATE TABLE IF NOT EXISTS mosaicforge_job_outbox (id bigserial PRIMARY KEY, job_id uuid NOT NULL REFERENCES mosaicforge_jobs(id), delivered boolean NOT NULL DEFAULT false)");
            s.execute(
                "CREATE INDEX IF NOT EXISTS mosaicforge_job_outbox_pending ON mosaicforge_job_outbox(id) WHERE NOT delivered");
            s.execute(
                "INSERT INTO mosaicforge_schema_versions(version) VALUES (2) ON CONFLICT DO NOTHING");
            s.execute(
                "ALTER TABLE mosaicforge_jobs ADD COLUMN IF NOT EXISTS revision bigint NOT NULL DEFAULT 0 CHECK (revision BETWEEN 0 AND 9007199254740991)");
            s.execute(
                "INSERT INTO mosaicforge_schema_versions(version) VALUES (3) ON CONFLICT DO NOTHING");
          }
          return null;
        });
  }

  @FunctionalInterface
  private interface Work<T> {
    T run(Connection c) throws SQLException;
  }

  private <T> T tx(Work<T> work) {
    var properties = new java.util.Properties();
    properties.setProperty("user", user);
    properties.setProperty("password", password);
    properties.setProperty("connectTimeout", "5");
    properties.setProperty("socketTimeout", "15");
    properties.setProperty(
        "options",
        "-c statement_timeout=10000 -c lock_timeout=5000 -c idle_in_transaction_session_timeout=15000");
    try (var c = DriverManager.getConnection(url, properties)) {
      c.setAutoCommit(false);
      try {
        var value = work.run(c);
        c.commit();
        return value;
      } catch (SQLException | RuntimeException e) {
        c.rollback();
        throw e;
      }
    } catch (SQLException e) {
      throw new IllegalStateException("Job persistence unavailable");
    }
  }

  private static Job row(ResultSet r) throws SQLException {
    return new Job(
        r.getObject("id", UUID.class),
        r.getObject("owner_id", UUID.class),
        r.getString("state"),
        r.getString("request"),
        r.getString("snapshot"),
        r.getString("next_index"),
        r.getInt("evaluated"),
        r.getInt("accepted"),
        r.getInt("rejected"),
        r.getLong("fence"),
        r.getObject("block_id", UUID.class),
        r.getInt("retries"),
        r.getLong("revision"));
  }

  private static Optional<Job> locked(Connection c, UUID id) throws SQLException {
    try (var s = c.prepareStatement("SELECT * FROM mosaicforge_jobs WHERE id=? FOR UPDATE")) {
      s.setObject(1, id);
      try (var r = s.executeQuery()) {
        return r.next() ? Optional.of(row(r)) : Optional.empty();
      }
    }
  }

  private static void notify(Connection c, UUID id) throws SQLException {
    try (var s =
        c.prepareStatement(
            "INSERT INTO mosaicforge_job_outbox(job_id) SELECT ? WHERE NOT EXISTS (SELECT 1 FROM mosaicforge_job_outbox WHERE job_id=? AND NOT delivered)")) {
      s.setObject(1, id);
      s.setObject(2, id);
      s.executeUpdate();
    }
  }

  @Override
  public Job submit(UUID owner, String key, String request) {
    if (key == null || key.isBlank() || key.length() > 200)
      throw new IllegalArgumentException("Invalid idempotency key");
    var body = Contracts.JSON.readTree(request);
    var project = UUID.fromString(Contracts.text(body, "projectId"));
    int revision = body.path("projectRevision").asInt();
    return tx(
        c -> {
          String snapshot;
          try (var s =
              c.prepareStatement(
                  "SELECT r.snapshot FROM mosaicforge_project_revisions r JOIN mosaicforge_projects p ON p.id=r.project_id WHERE p.owner_id=? AND r.project_id=? AND r.number=?")) {
            s.setObject(1, owner);
            s.setObject(2, project);
            s.setInt(3, revision);
            try (var r = s.executeQuery()) {
              if (!r.next()) throw new ProjectStore.Missing();
              snapshot = r.getString(1);
            }
          }
          JobRequests.validate(body, Contracts.JSON.readTree(snapshot));
          String canonical = JobRequests.canonical(body), digest = JobRequests.hash(canonical);
          UUID id = UUID.randomUUID();
          try (var s =
              c.prepareStatement(
                  "INSERT INTO mosaicforge_jobs(id,owner_id,project_id,project_revision,idempotency_key,digest,request,snapshot,next_index) VALUES (?,?,?,?,?,?,?::jsonb,?::jsonb,?::numeric) ON CONFLICT(owner_id,project_id,idempotency_key) DO NOTHING RETURNING id")) {
            s.setObject(1, id);
            s.setObject(2, owner);
            s.setObject(3, project);
            s.setInt(4, revision);
            s.setString(5, key);
            s.setString(6, digest);
            s.setString(7, canonical);
            s.setString(8, snapshot);
            s.setString(9, Contracts.text(body.path("generation"), "startIndex"));
            try (var r = s.executeQuery()) {
              if (r.next()) notify(c, id);
            }
          }
          try (var s =
              c.prepareStatement(
                  "SELECT * FROM mosaicforge_jobs WHERE owner_id=? AND project_id=? AND idempotency_key=?")) {
            s.setObject(1, owner);
            s.setObject(2, project);
            s.setString(3, key);
            try (var r = s.executeQuery()) {
              r.next();
              if (!digest.equals(r.getString("digest"))) throw new ProjectStore.Conflict();
              return row(r);
            }
          }
        });
  }

  @Override
  public Optional<Job> find(UUID owner, UUID id) {
    return tx(
        c -> {
          try (var s =
              c.prepareStatement("SELECT * FROM mosaicforge_jobs WHERE id=? AND owner_id=?")) {
            s.setObject(1, id);
            s.setObject(2, owner);
            try (var r = s.executeQuery()) {
              return r.next() ? Optional.of(row(r)) : Optional.empty();
            }
          }
        });
  }

  private Timestamp deadline(Duration duration) {
    if (duration.isNegative()
        || duration.isZero()
        || duration.compareTo(Duration.ofMinutes(10)) > 0)
      throw new IllegalArgumentException("Invalid lease duration");
    return Timestamp.from(clock.instant().plus(duration));
  }

  @Override
  public Optional<Lease> claim(UUID id, Duration duration) {
    return tx(
        c -> {
          var found = locked(c, id);
          if (found.isEmpty()) return Optional.empty();
          var job = found.get();
          var request = Contracts.JSON.readTree(job.request());
          if (!List.of("queued", "running").contains(job.state())
              || job.accepted() >= request.path("requestedAcceptedCount").asInt())
            return Optional.empty();
          try (var s =
              c.prepareStatement(
                  "SELECT lease_until,block_request FROM mosaicforge_jobs WHERE id=?")) {
            s.setObject(1, id);
            try (var r = s.executeQuery()) {
              r.next();
              if (r.getTimestamp(1) != null
                  && r.getTimestamp(1).toInstant().isAfter(clock.instant()))
                return Optional.empty();
            }
          }
          if (job.retries() >= retryBudget
              || job.nextIndex() == null
              || job.evaluated() >= request.path("maxCandidates").asInt()) {
            fail(c, id);
            return Optional.empty();
          }
          String wire;
          UUID block = job.blockId();
          if (block == null) {
            block = UUID.randomUUID();
            var body = Contracts.JSON.createObjectNode();
            body.put("schemaVersion", "1");
            body.put("jobId", id.toString());
            body.put("blockId", block.toString());
            body.set("templateSvg", Contracts.JSON.readTree(job.snapshot()).path("templateSvg"));
            var pdf = Contracts.JSON.createObjectNode();
            pdf.put("schemaVersion", "1");
            pdf.set("generation", request.path("generation"));
            pdf.set("pdf", request.path("pdf"));
            body.set("job", pdf);
            body.put("startIndex", job.nextIndex());
            int count =
                Math.min(
                    request.path("execution").path("blockSize").asInt(),
                    request.path("maxCandidates").asInt() - job.evaluated());
            count =
                JobRequests.MAX_U64
                    .subtract(JobRequests.u64(job.nextIndex()))
                    .add(java.math.BigInteger.ONE)
                    .min(java.math.BigInteger.valueOf(count))
                    .intValueExact();
            body.put("candidateCount", count);
            body.put(
                "acceptedLimit", request.path("requestedAcceptedCount").asInt() - job.accepted());
            wire = JobRequests.canonical(body);
          } else {
            try (var s =
                c.prepareStatement("SELECT block_request FROM mosaicforge_jobs WHERE id=?")) {
              s.setObject(1, id);
              try (var r = s.executeQuery()) {
                r.next();
                wire = r.getString(1);
              }
            }
          }
          try (var s =
              c.prepareStatement(
                  "UPDATE mosaicforge_jobs SET revision=revision+1,state='running',fence=fence+1,retries=retries+1,lease_until=?,block_id=?,block_request=? WHERE id=?")) {
            s.setTimestamp(1, deadline(duration));
            s.setObject(2, block);
            s.setString(3, wire);
            s.setObject(4, id);
            s.executeUpdate();
          }
          return Optional.of(new Lease(locked(c, id).orElseThrow(), wire, job.fence() + 1));
        });
  }

  private static void fail(Connection c, UUID id) throws SQLException {
    try (var s =
        c.prepareStatement(
            "UPDATE mosaicforge_jobs SET revision=revision+1,state='failed',lease_until=NULL WHERE id=?")) {
      s.setObject(1, id);
      s.executeUpdate();
    }
  }

  private boolean active(Connection c, Lease lease) throws SQLException {
    try (var s =
        c.prepareStatement(
            "SELECT 1 FROM mosaicforge_jobs WHERE id=? AND fence=? AND block_id=? AND state='running' AND lease_until>?")) {
      s.setObject(1, lease.job().id());
      s.setLong(2, lease.fence());
      s.setObject(3, lease.job().blockId());
      s.setTimestamp(4, Timestamp.from(clock.instant()));
      try (var r = s.executeQuery()) {
        return r.next();
      }
    }
  }

  @Override
  public boolean heartbeat(UUID id, long fence, Duration duration) {
    return tx(
        c -> {
          try (var s =
              c.prepareStatement(
                  "UPDATE mosaicforge_jobs SET revision=revision+1,lease_until=? WHERE id=? AND fence=? AND state='running' AND lease_until>?")) {
            s.setTimestamp(1, deadline(duration));
            s.setObject(2, id);
            s.setLong(3, fence);
            s.setTimestamp(4, Timestamp.from(clock.instant()));
            return s.executeUpdate() == 1;
          }
        });
  }

  @Override
  public boolean confirm(Lease lease, String response) {
    BlockResults.validate(lease.blockRequest(), response);
    return tx(
        c -> {
          var job = locked(c, lease.job().id()).orElseThrow();
          if (!active(c, lease)) return false;
          var result = Contracts.JSON.readTree(response);
          try (var s =
              c.prepareStatement(
                  "INSERT INTO mosaicforge_job_blocks(job_id,block_id,response) VALUES (?,?,?::jsonb) ON CONFLICT DO NOTHING")) {
            s.setObject(1, job.id());
            s.setObject(2, job.blockId());
            s.setString(3, response);
            if (s.executeUpdate() == 0) return false;
          }
          var request = Contracts.JSON.readTree(job.request());
          int evaluated = job.evaluated() + result.path("evaluatedCount").asInt(),
              accepted = job.accepted() + result.path("acceptedCount").asInt();
          boolean exhausted =
              accepted < request.path("requestedAcceptedCount").asInt()
                  && (evaluated >= request.path("maxCandidates").asInt()
                      || result.path("nextCandidateIndex").isNull());
          try (var s =
              c.prepareStatement(
                  "UPDATE mosaicforge_jobs SET revision=revision+1,next_index=?::numeric,evaluated=?,accepted=?,rejected=rejected+?,lease_until=NULL,block_id=NULL,block_request=NULL,retries=0,state=? WHERE id=?")) {
            s.setString(
                1,
                result.path("nextCandidateIndex").isNull()
                    ? null
                    : Contracts.text(result, "nextCandidateIndex"));
            s.setInt(2, evaluated);
            s.setInt(3, accepted);
            s.setInt(4, result.path("rejectedCount").asInt());
            s.setString(5, exhausted ? "failed" : "running");
            s.setObject(6, job.id());
            s.executeUpdate();
          }
          if (!exhausted && accepted < request.path("requestedAcceptedCount").asInt())
            notify(c, job.id());
          return true;
        });
  }

  @Override
  public void retry(Lease lease) {
    tx(
        c -> {
          locked(c, lease.job().id());
          if (active(c, lease)) {
            try (var s =
                c.prepareStatement(
                    "UPDATE mosaicforge_jobs SET revision=revision+1,lease_until=NULL,state=CASE WHEN retries>=? THEN 'failed' ELSE state END WHERE id=?")) {
              s.setInt(1, retryBudget);
              s.setObject(2, lease.job().id());
              s.executeUpdate();
            }
            notify(c, lease.job().id());
          }
          return null;
        });
  }

  @Override
  public void defer(Lease lease, Duration delay) {
    tx(
        c -> {
          locked(c, lease.job().id());
          if (active(c, lease)) {
            try (var s =
                c.prepareStatement(
                    "UPDATE mosaicforge_jobs SET revision=revision+1,state='queued',fence=fence+1,retries=GREATEST(retries-1,0),lease_until=? WHERE id=?")) {
              s.setTimestamp(1, deadline(delay));
              s.setObject(2, lease.job().id());
              s.executeUpdate();
            }
            notify(c, lease.job().id());
          }
          return null;
        });
  }

  @Override
  public void cancel(UUID owner, UUID id) {
    tx(
        c -> {
          try (var s =
              c.prepareStatement(
                  "UPDATE mosaicforge_jobs SET revision=revision+1,state='cancelled',lease_until=NULL WHERE id=? AND owner_id=? AND state IN ('queued','running','cancelling')")) {
            s.setObject(1, id);
            s.setObject(2, owner);
            s.executeUpdate();
          }
          return null;
        });
  }

  @Override
  public List<Notice> outbox(int limit) {
    bounded(limit);
    return tx(
        c -> {
          var result = new ArrayList<Notice>();
          try (var s =
              c.prepareStatement(
                  "SELECT id,job_id FROM mosaicforge_job_outbox WHERE NOT delivered ORDER BY id LIMIT ?")) {
            s.setInt(1, limit);
            try (var r = s.executeQuery()) {
              while (r.next()) result.add(new Notice(r.getLong(1), r.getObject(2, UUID.class)));
            }
          }
          return List.copyOf(result);
        });
  }

  @Override
  public void delivered(long id) {
    tx(
        c -> {
          try (var s =
              c.prepareStatement("UPDATE mosaicforge_job_outbox SET delivered=true WHERE id=?")) {
            s.setLong(1, id);
            s.executeUpdate();
          }
          return null;
        });
  }

  private static void bounded(int limit) {
    if (limit < 1 || limit > 100) throw new IllegalArgumentException("Invalid batch limit");
  }

  @Override
  public int reconcile(int limit) {
    bounded(limit);
    return tx(
        c -> {
          var ids = new ArrayList<UUID>();
          try (var s =
              c.prepareStatement(
                  "SELECT id FROM mosaicforge_jobs WHERE state IN ('queued','running') AND (lease_until IS NULL OR lease_until<=?) AND accepted<(request->>'requestedAcceptedCount')::integer ORDER BY id LIMIT ? FOR UPDATE SKIP LOCKED")) {
            s.setTimestamp(1, Timestamp.from(clock.instant()));
            s.setInt(2, limit);
            try (var r = s.executeQuery()) {
              while (r.next()) ids.add(r.getObject(1, UUID.class));
            }
          }
          for (var id : ids) notify(c, id);
          return ids.size();
        });
  }
}
