import { spawn } from "node:child_process";
import { randomBytes } from "node:crypto";

// Both processes share loopback; the private engine has no published port.
const env = {
  ...process.env,
  MOSAICFORGE_ENGINE_TOKEN: randomBytes(32).toString("hex"),
  MOSAICFORGE_ENGINE_PORT: "4771",
  MOSAICFORGE_ENGINE_URL: "http://127.0.0.1:4771",
  MOSAICFORGE_WEB_ORIGIN:
    process.env.MOSAICFORGE_WEB_ORIGIN ?? "http://127.0.0.1:3000",
  PORT: "3000",
  HOSTNAME: "0.0.0.0",
};
const children = [
  spawn("/app/mosaicforge-preview", [], { env, stdio: "inherit" }),
  spawn(process.execPath, ["/app/apps/web/server.js"], {
    env,
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
