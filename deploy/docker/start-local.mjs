import { spawn } from "node:child_process";
import { randomBytes } from "node:crypto";

// All three processes share loopback; only the web port is published.
const common = {
  ...process.env,
  MOSAICFORGE_WEB_ORIGIN:
    process.env.MOSAICFORGE_WEB_ORIGIN ?? "http://127.0.0.1:3000",
  PORT: "3000",
  HOSTNAME: "0.0.0.0",
};
// Keep each credential out of processes which do not need it.
delete common.MOSAICFORGE_API_TOKEN;
delete common.MOSAICFORGE_ENGINE_TOKEN;
delete common.MOSAICFORGE_API_URL;
delete common.MOSAICFORGE_ENGINE_URL;
const database = {};
for (const key of Object.keys(common)) {
  if (
    key.startsWith("MOSAICFORGE_DB_") ||
    key.startsWith("MOSAICFORGE_VALKEY_") ||
    key === "MOSAICFORGE_LOCAL_USER_ID" ||
    key === "MOSAICFORGE_PERSISTENCE_ENABLED" ||
    key.startsWith("MOSAICFORGE_JOBS_")
  ) {
    database[key] = common[key];
    delete common[key];
  }
}
const journalDirectory = common.MOSAICFORGE_JOURNAL_DIR;
delete common.MOSAICFORGE_JOURNAL_DIR;
const engineToken = randomBytes(32).toString("hex");
const apiToken = randomBytes(32).toString("hex");
const children = [
  spawn("/app/mosaicforge-preview", [], {
    env: {
      ...common,
      MOSAICFORGE_ENGINE_TOKEN: engineToken,
      MOSAICFORGE_ENGINE_PORT: "4771",
      ...(journalDirectory
        ? { MOSAICFORGE_JOURNAL_DIR: journalDirectory }
        : {}),
    },
    stdio: "inherit",
  }),
  spawn(
    "/opt/java/openjdk/bin/java",
    ["-Xms32m", "-Xmx384m", "-jar", "/app/mosaicforge-api.jar"],
    {
      env: {
        ...common,
        MOSAICFORGE_API_PORT: "4772",
        MOSAICFORGE_API_TOKEN: apiToken,
        MOSAICFORGE_ENGINE_TOKEN: engineToken,
        MOSAICFORGE_ENGINE_URL: "http://127.0.0.1:4771",
        ...database,
      },
      stdio: "inherit",
    },
  ),
  spawn(process.execPath, ["/app/apps/web/server.js"], {
    env: {
      ...common,
      MOSAICFORGE_API_URL: "http://127.0.0.1:4772",
      MOSAICFORGE_API_TOKEN: apiToken,
    },
    stdio: "inherit",
  }),
];
let stopping = false;
function stop(code) {
  if (stopping) return;
  stopping = true;
  process.exitCode = code;
  for (const child of children) {
    if (child.exitCode === null && child.signalCode === null)
      child.kill("SIGTERM");
  }
  const timeout = setTimeout(() => {
    for (const child of children) {
      if (child.exitCode === null && child.signalCode === null)
        child.kill("SIGKILL");
    }
  }, 5000);
  timeout.unref();
}
for (const child of children) {
  child.on("error", () => {
    console.error("A local service could not start");
    stop(1);
  });
  child.on("exit", () => stop(1));
}
process.on("SIGINT", () => stop(0));
process.on("SIGTERM", () => stop(0));
