package dev.mosaicforge.api;

import java.time.Duration;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

/** Durable authoritative state; queue messages only suggest which job to claim. */
interface JobStore {
  record Job(
      UUID id,
      UUID owner,
      String state,
      String request,
      String snapshot,
      String nextIndex,
      int evaluated,
      int accepted,
      int rejected,
      long fence,
      UUID blockId,
      int retries,
      long revision) {}

  record Lease(Job job, String blockRequest, long fence) {}

  record Notice(long id, UUID jobId) {}

  record FinalLease(Job job, UUID artifactId, String request, long fence) {}

  record Artifact(
      UUID id, int pages, String sha256, long bytes, String manifestSha256, long manifestBytes) {}

  Optional<FinalLease> claimFinal(UUID id, Duration duration);

  boolean completeFinal(FinalLease lease, String response);

  void retryFinal(FinalLease lease, Duration delay);

  Optional<Artifact> artifact(UUID owner, UUID id);

  List<UUID> pending(int limit);

  List<UUID> cancelled(int limit);

  void cancellationDelivered(UUID id);

  Job submit(UUID owner, String key, String request);

  Optional<Job> find(UUID owner, UUID id);

  Optional<Lease> claim(UUID id, Duration duration);

  boolean heartbeat(UUID id, long fence, Duration duration);

  boolean confirm(Lease lease, String response);

  void retry(Lease lease);

  void defer(Lease lease, Duration delay);

  void cancel(UUID owner, UUID id);

  List<Notice> outbox(int limit);

  void delivered(long id);

  int reconcile(int limit);
}
