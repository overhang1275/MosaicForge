package dev.mosaicforge.api;

import io.lettuce.core.ClientOptions;
import io.lettuce.core.RedisClient;
import io.lettuce.core.RedisURI;
import io.lettuce.core.SocketOptions;
import io.lettuce.core.XAddArgs;
import io.lettuce.core.XReadArgs;
import io.lettuce.core.api.StatefulRedisConnection;
import java.time.Duration;
import java.util.List;
import java.util.Map;
import java.util.UUID;

/** Bounded advisory stream. Missing entries are recovered from the database. */
final class LettuceWorkNotifications implements WorkNotifications {
  private final RedisClient client;
  private final StatefulRedisConnection<String, String> connection;
  private final String stream;

  LettuceWorkNotifications(String uri, String stream) {
    this(uri, stream, null);
  }

  LettuceWorkNotifications(String uri, String stream, String password) {
    if (!stream.matches("[a-zA-Z0-9:_-]{1,120}"))
      throw new IllegalArgumentException("Invalid stream");
    this.stream = stream;
    var address = RedisURI.create(uri);
    if (password != null && !password.isBlank()) address.setAuthentication(password.toCharArray());
    address.setTimeout(Duration.ofSeconds(2));
    client = RedisClient.create(address);
    client.setOptions(
        ClientOptions.builder()
            .requestQueueSize(100)
            .disconnectedBehavior(ClientOptions.DisconnectedBehavior.REJECT_COMMANDS)
            .socketOptions(SocketOptions.builder().connectTimeout(Duration.ofSeconds(2)).build())
            .build());
    try {
      connection = client.connect();
    } catch (RuntimeException failure) {
      client.shutdown();
      throw new IllegalStateException("Work notification service unavailable");
    }
  }

  @Override
  public void publish(UUID id) {
    connection.sync().xadd(stream, new XAddArgs().maxlen(10000), Map.of("jobId", id.toString()));
  }

  @Override
  @SuppressWarnings(
      "unchecked") // Lettuce exposes generic varargs; offsets contain only String keys.
  public List<Message> poll(int limit) {
    if (limit < 1 || limit > 100) throw new IllegalArgumentException("Invalid notification limit");
    return connection
        .sync()
        .xread(XReadArgs.Builder.count(limit), XReadArgs.StreamOffset.from(stream, "0-0"))
        .stream()
        .map(m -> new Message(m.getId(), UUID.fromString(m.getBody().get("jobId"))))
        .toList();
  }

  @Override
  public void acknowledge(String id) {
    connection.sync().xdel(stream, id);
  }

  @Override
  public void close() {
    connection.close();
    client.shutdown(Duration.ZERO, Duration.ofSeconds(2));
  }
}
