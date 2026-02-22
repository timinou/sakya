#!/usr/bin/env bun
/**
 * Dev launcher that automatically finds an available port.
 * Starts Vite on the first free port (from 1420) and passes
 * the resolved port to `tauri dev` via --config override.
 */
import { createServer } from "net";
import { spawn } from "child_process";

const BASE_PORT = 1420;
const MAX_ATTEMPTS = 20;

function findFreePort(start: number): Promise<number> {
  return new Promise((resolve, reject) => {
    let attempts = 0;

    function tryPort(port: number) {
      if (attempts >= MAX_ATTEMPTS) {
        reject(new Error(`No free port found in range ${start}–${start + MAX_ATTEMPTS - 1}`));
        return;
      }
      attempts++;
      const server = createServer();
      server.unref();
      server.on("error", () => tryPort(port + 1));
      server.listen(port, () => {
        server.close(() => resolve(port));
      });
    }

    tryPort(start);
  });
}

const port = await findFreePort(BASE_PORT);

if (port !== BASE_PORT) {
  console.log(`⚡ Port ${BASE_PORT} in use, using port ${port}`);
} else {
  console.log(`⚡ Starting dev server on port ${port}`);
}

const configOverride = JSON.stringify({
  build: {
    beforeDevCommand: `bun run dev --port ${port}`,
    devUrl: `http://localhost:${port}`,
  },
});

const args = ["tauri", "dev", "--config", configOverride, ...process.argv.slice(2)];
const child = spawn("bunx", args, {
  stdio: "inherit",
  env: { ...process.env, VITE_DEV_PORT: String(port) },
});

child.on("exit", (code) => process.exit(code ?? 0));
