import { connect } from "node:net";

try {
  const response = await fetch("http://127.0.0.1:3000", {
    signal: AbortSignal.timeout(3000),
  });
  if (!response.ok) throw new Error("Web unavailable");
  await response.body?.cancel();
  await new Promise((resolve, reject) => {
    const socket = connect(4771, "127.0.0.1");
    socket.setTimeout(3000);
    socket.once("connect", () => {
      socket.destroy();
      resolve();
    });
    socket.once("error", reject);
    socket.once("timeout", () => {
      socket.destroy();
      reject(new Error("Engine unavailable"));
    });
  });
} catch {
  process.exitCode = 1;
}
