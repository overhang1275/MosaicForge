package dev.mosaicforge.api;

import java.time.Instant;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

/** Persistence boundary. Snapshot is the exact validated versioned contract JSON. */
interface ProjectStore {
  record Project(UUID id, String name, int currentRevision, Instant createdAt, Instant updatedAt) {}

  record Revision(UUID projectId, int number, Instant createdAt, String snapshot) {}

  void ensureUser(UUID owner);

  Project create(UUID owner, String name);

  List<Project> list(UUID owner, UUID after, int limit);

  Optional<Project> find(UUID owner, UUID project);

  Revision append(UUID owner, UUID project, int expectedRevision, String snapshot);

  Optional<Revision> revision(UUID owner, UUID project, int number);

  final class Missing extends RuntimeException {
    private static final long serialVersionUID = 1L;
  }

  final class Conflict extends RuntimeException {
    private static final long serialVersionUID = 1L;
  }
}
