import { spawn } from "node:child_process";
import { randomBytes } from "node:crypto";
import { once } from "node:events";
import { fileURLToPath, pathToFileURL } from "node:url";
import { resolve } from "node:path";
import { setTimeout as delay } from "node:timers/promises";

const root = fileURLToPath(new URL("../", import.meta.url));
export function startSession(
  enginePort = 4771,
  webPort = 3000,
  token = randomBytes(32).toString("hex"),
) {
  if (
    ![enginePort, webPort].every(
      (p) => Number.isInteger(p) && p > 0 && p <= 65535,
    ) ||
    enginePort === webPort ||
    !/^[\x21-\x7e]{32,256}$/.test(token)
  )
    throw new Error("Invalid local configuration");
  const env = {
    ...process.env,
    MOSAICFORGE_ENGINE_TOKEN: token,
    MOSAICFORGE_ENGINE_PORT: String(enginePort),
    MOSAICFORGE_ENGINE_URL: `http://127.0.0.1:${enginePort}`,
    MOSAICFORGE_WEB_ORIGIN: `http://127.0.0.1:${webPort}`,
    PORT: String(webPort),
    HOSTNAME: "127.0.0.1",
    NEXT_TELEMETRY_DISABLED: "1",
  };
  const children = [
    spawn(
      resolve(
        root,
        `services/engine/target/debug/mosaicforge-preview${process.platform === "win32" ? ".exe" : ""}`,
      ),
      [],
      { cwd: root, env, windowsHide: true, stdio: ["ignore", "pipe", "pipe"] },
    ),
    spawn(
      process.execPath,
      [resolve(root, "apps/web/scripts/start-standalone.mjs")],
      { cwd: root, env, windowsHide: true, stdio: ["ignore", "pipe", "pipe"] },
    ),
  ];
  let stopped = false;
  async function stop() {
    if (stopped) return;
    stopped = true;
    await Promise.all(
      children.map(async (child) => {
        if (child.exitCode === null && !child.killed) {
          const exit = once(child, "exit").catch(() => {});
          child.kill();
          await Promise.race([exit, delay(3000)]);
        }
      }),
    );
  }
  for (const child of children) {
    child.on("error", () => {
      console.error(
        "Local preview could not start; build Rust and Next.js first",
      );
      void stop();
    });
    child.on("exit", () => {
      if (!stopped) void stop();
    });
  }
  return { children, stop };
}
if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  const session = startSession(
    Number(process.env.MOSAICFORGE_ENGINE_PORT ?? 4771),
    Number(process.env.PORT ?? 3000),
  );
  for (const child of session.children) {
    child.stdout.pipe(process.stdout);
    child.stderr.pipe(process.stderr);
  }
  for (const signal of ["SIGINT", "SIGTERM"])
    process.on(signal, () => {
      void session.stop();
    });
}
