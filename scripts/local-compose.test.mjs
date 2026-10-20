import assert from "node:assert/strict";
import { test } from "node:test";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { ensureLocalEnvironment } from "./local-compose.mjs";
import * as launcher from "./local-compose.mjs";

test("Valkey uses its own persistent credential without replacing existing files", async () => {
  assert.equal(typeof launcher.ensureValkeyEnvironment, "function");
  const directory = await mkdtemp(join(tmpdir(), "mosaicforge-env-test-"));
  try {
    const path = join(directory, "valkey.env");
    await launcher.ensureValkeyEnvironment(path);
    const original = await readFile(path, "utf8");
    assert.match(original, /^MOSAICFORGE_VALKEY_PASSWORD=[0-9a-f]{64}\n$/);
    await launcher.ensureValkeyEnvironment(path);
    assert.equal(await readFile(path, "utf8"), original);
    await writeFile(path, "invalid");
    await assert.rejects(launcher.ensureValkeyEnvironment(path), /existing/i);
    assert.equal(await readFile(path, "utf8"), "invalid");
  } finally {
    assert.ok(directory.startsWith(join(tmpdir(), "mosaicforge-env-test-")));
    await rm(directory, { recursive: true, force: true });
  }
});

test("local database credentials are created once and never silently replaced", async () => {
  const directory = await mkdtemp(join(tmpdir(), "mosaicforge-env-test-"));
  try {
    const path = join(directory, "local.env");
    await ensureLocalEnvironment(path);
    const original = await readFile(path, "utf8");
    assert.match(original, /MOSAICFORGE_DB_PASSWORD=[0-9a-f]{64}\n/);
    assert.match(original, /MOSAICFORGE_LOCAL_USER_ID=[0-9a-f-]{36}\n/);
    await ensureLocalEnvironment(path);
    assert.equal(await readFile(path, "utf8"), original);
    await writeFile(path, "invalid-existing-configuration");
    await assert.rejects(ensureLocalEnvironment(path), /existing/i);
    assert.equal(
      await readFile(path, "utf8"),
      "invalid-existing-configuration",
    );
  } finally {
    assert.ok(directory.startsWith(join(tmpdir(), "mosaicforge-env-test-")));
    await rm(directory, { recursive: true, force: true });
  }
});
