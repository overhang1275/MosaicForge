package dev.mosaicforge.api;

import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.Test;

class JobRequestsTest {
  @Test
  void canonicalObjectsIgnoreKeyOrderButArraysDoNot() {
    assertEquals(
        JobRequests.canonical(Contracts.JSON.readTree("{\"b\":1,\"a\":2}")),
        JobRequests.canonical(Contracts.JSON.readTree("{\"a\":2,\"b\":1}")));
    assertNotEquals(
        JobRequests.canonical(Contracts.JSON.readTree("[1,2]")),
        JobRequests.canonical(Contracts.JSON.readTree("[2,1]")));
  }

  @Test
  void unsignedOverflowRejected() {
    assertThrows(IllegalArgumentException.class, () -> JobRequests.u64("18446744073709551616"));
    assertEquals("18446744073709551615", JobRequests.u64("18446744073709551615").toString());
  }
}
