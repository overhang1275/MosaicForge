import { randomBytes, randomUUID } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { spawnSync } from "node:child_process";

export async function ensureLocalEnvironment(path) {
  await mkdir(dirname(path), { recursive: true });
  try {
    await writeFile(
      path,
      `MOSAICFORGE_DB_PASSWORD=${randomBytes(32).toString("hex")}\nMOSAICFORGE_LOCAL_USER_ID=${randomUUID()}\n`,
      { flag: "wx", mode: 0o600 },
    );
  } catch (error) {
    if (error.code !== "EEXIST") throw error;
  }
  const text = await readFile(path, "utf8");
  if (
    !/^MOSAICFORGE_DB_PASSWORD=[0-9a-f]{64}\r?\nMOSAICFORGE_LOCAL_USER_ID=[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}\r?\n$/.test(
      text,
    )
  )
    throw new Error(
      "Invalid existing local database configuration; it was not overwritten.",
    );
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(resolve(process.argv[1])).href
) {
  const root = fileURLToPath(new URL("../", import.meta.url));
  const environment = join(root, "state/local-db.env");
  await ensureLocalEnvironment(environment);
  const windowsDocker = join(
    process.env.ProgramFiles ?? "C:/Program Files",
    "Docker/Docker/resources/bin/docker.exe",
  );
  const docker =
    process.platform === "win32" && existsSync(windowsDocker)
      ? windowsDocker
      : "docker";
  const args = process.argv.slice(2);
  const result = spawnSync(
    docker,
    [
      "compose",
      "--env-file",
      environment,
      "-f",
      join(root, "deploy/compose/local.yml"),
      ...(args.length ? args : ["up", "-d", "--build", "--wait"]),
    ],
    { stdio: "inherit" },
  );
  if (result.error)
    throw new Error("Docker Compose could not start. Check Docker Desktop.");
  process.exitCode = result.status ?? 1;
}
