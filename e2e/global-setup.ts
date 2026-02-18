/**
 * Global setup for Playwright: ensures a dev server is running on port 1420.
 *
 * Handles concurrent `bunx playwright test` processes by using flock to
 * serialize server startup. The server is started as a detached process
 * (not a child of Playwright) so it survives individual test process teardown.
 *
 * To stop the server: `bun run test:e2e:stop-server`
 */
import { execSync } from "child_process";
import http from "http";
import path from "path";

const PORT = 1420;
const URL = `http://localhost:${PORT}`;
const PID_FILE = "/tmp/sakya-playwright-dev-server.pid";
const LOCK_FILE = "/tmp/sakya-playwright-dev-server.lock";

function isServerRunning(): Promise<boolean> {
  return new Promise((resolve) => {
    const req = http.get(URL, () => resolve(true));
    req.on("error", () => resolve(false));
    req.setTimeout(2000, () => {
      req.destroy();
      resolve(false);
    });
  });
}

function waitForServer(timeoutMs = 120_000): Promise<void> {
  const start = Date.now();
  return new Promise((resolve, reject) => {
    const check = async () => {
      if (await isServerRunning()) return resolve();
      if (Date.now() - start > timeoutMs)
        return reject(new Error("Dev server startup timeout (120s)"));
      setTimeout(check, 500);
    };
    check();
  });
}

async function globalSetup() {
  // Fast path: server already running (user has `bun run dev` or previous test started it)
  if (await isServerRunning()) return;

  const projectRoot = path.resolve(import.meta.dirname, "..");

  // Serialize startup across concurrent processes using flock.
  // flock -n = non-blocking: first process acquires lock and starts server,
  // others skip (lock busy) and fall through to waitForServer.
  execSync(
    `flock -n "${LOCK_FILE}" -c '
    # Double-check inside lock (another process may have started it)
    if curl -sf "${URL}" > /dev/null 2>&1; then
      exit 0
    fi
    # Start detached server (not a child of this shell)
    nohup bun run dev > /tmp/sakya-playwright-dev-server.log 2>&1 &
    echo $! > "${PID_FILE}"
  ' || true`,
    { stdio: "inherit", cwd: projectRoot },
  );

  // Wait for server to be ready (whether we started it or another process did)
  await waitForServer();
}

export default globalSetup;
