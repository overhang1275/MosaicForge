package dev.mosaicforge.api;

import java.util.List;
import java.util.UUID;

interface WorkNotifications extends AutoCloseable {
  record Message(String id, UUID jobId) {}

  void publish(UUID jobId);

  List<Message> poll(int limit);

  void acknowledge(String id);

  @Override
  void close();
}
