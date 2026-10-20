package dev.mosaicforge.api;

import static org.junit.jupiter.api.Assertions.*;

import java.sql.DriverManager;
import java.util.UUID;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.Executors;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.condition.EnabledIfEnvironmentVariable;

@EnabledIfEnvironmentVariable(named = "MOSAICFORGE_TEST_DB_URL", matches = ".+")
class ProjectStoreTest {
  @Test
  void migrationOwnershipPaginationAtomicRevisionsAndRestart() throws Exception {
    String url = System.getenv("MOSAICFORGE_TEST_DB_URL"),
        user = System.getenv("MOSAICFORGE_DB_USER"),
        password = System.getenv("MOSAICFORGE_DB_PASSWORD");
    var owner = UUID.randomUUID();
    var other = UUID.randomUUID();
    try {
      var store = new JdbcProjectStore(url, user, password);
      new JdbcProjectStore(url, user, password);
      store.ensureUser(owner);
      store.ensureUser(other);
      var project = store.create(owner, "Deterministic artwork");
      store.create(owner, "Second");
      assertEquals(2, store.list(owner, null, 20).size());
      var first = store.list(owner, null, 1).getFirst();
      assertEquals(1, store.list(owner, first.id(), 20).size());
      assertEquals(project, store.find(owner, project.id()).orElseThrow());
      assertTrue(store.find(other, project.id()).isEmpty());
      assertThrows(ProjectStore.Missing.class, () -> store.append(other, project.id(), 0, "{}"));
      assertThrows(
          IllegalStateException.class, () -> store.append(owner, project.id(), 0, "bad json"));
      assertEquals(0, store.find(owner, project.id()).orElseThrow().currentRevision());
      String snapshot = "{\"seed\":\"18446744073709551615\",\"svg\":\"<svg/>\"}";
      var ready = new CountDownLatch(2);
      var go = new CountDownLatch(1);
      try (var threads = Executors.newFixedThreadPool(2)) {
        java.util.concurrent.Callable<Boolean> append =
            () -> {
              ready.countDown();
              go.await();
              try {
                store.append(owner, project.id(), 0, snapshot);
                return true;
              } catch (ProjectStore.Conflict conflict) {
                return false;
              }
            };
        var one = threads.submit(append);
        var two = threads.submit(append);
        ready.await();
        go.countDown();
        assertNotEquals(one.get(), two.get());
      }
      var restarted = new JdbcProjectStore(url, user, password);
      assertEquals(1, restarted.find(owner, project.id()).orElseThrow().currentRevision());
      assertEquals(
          Contracts.JSON.readTree(snapshot),
          Contracts.JSON.readTree(
              restarted.revision(owner, project.id(), 1).orElseThrow().snapshot()));
      assertTrue(restarted.revision(other, project.id(), 1).isEmpty());
      assertThrows(
          ProjectStore.Conflict.class, () -> restarted.append(owner, project.id(), 0, snapshot));
    } finally {
      try (var c = DriverManager.getConnection(url, user, password)) {
        c.setAutoCommit(false);
        for (String sql :
            new String[] {
              "DELETE FROM mosaicforge_project_revisions WHERE project_id IN (SELECT id FROM mosaicforge_projects WHERE owner_id IN (?,?))",
              "DELETE FROM mosaicforge_projects WHERE owner_id IN (?,?)",
              "DELETE FROM mosaicforge_users WHERE id IN (?,?)"
            }) {
          try (var s = c.prepareStatement(sql)) {
            s.setObject(1, owner);
            s.setObject(2, other);
            s.executeUpdate();
          }
        }
        c.commit();
      }
    }
  }
}
