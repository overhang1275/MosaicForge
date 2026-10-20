package dev.mosaicforge.api;

import static org.junit.jupiter.api.Assertions.*;

import java.util.UUID;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.condition.EnabledIfEnvironmentVariable;

@EnabledIfEnvironmentVariable(named = "MOSAICFORGE_TEST_VALKEY_URI", matches = ".+")
class WorkNotificationsTest {
  @Test
  void actualStreamSurvivesConnectionRestartAndAckLoss() {
    String uri = System.getenv("MOSAICFORGE_TEST_VALKEY_URI"),
        password = System.getenv("MOSAICFORGE_VALKEY_PASSWORD"),
        stream = "mosaicforge:test:" + UUID.randomUUID();
    UUID job = UUID.randomUUID();
    String message;
    try (var queue = new LettuceWorkNotifications(uri, stream, password)) {
      queue.publish(job);
      var first = queue.poll(2);
      assertEquals(1, first.size());
      assertEquals(job, first.getFirst().jobId());
      message = first.getFirst().id();
      assertEquals(first, queue.poll(2));
    }
    try (var queue = new LettuceWorkNotifications(uri, stream, password)) {
      assertEquals(message, queue.poll(2).getFirst().id());
      queue.publish(job);
      assertEquals(2, queue.poll(2).size());
      for (var item : queue.poll(2)) queue.acknowledge(item.id());
      assertTrue(queue.poll(2).isEmpty());
      assertThrows(IllegalArgumentException.class, () -> queue.poll(101));
    }
  }
}
