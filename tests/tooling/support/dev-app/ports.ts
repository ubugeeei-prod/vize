import { spawnSync } from "node:child_process";
import { createServer } from "node:net";
import { BASE_ENV, REPO_ROOT } from "./env.ts";
import { commandAvailable } from "./commands.ts";

export function replacePortInArgs(args: string[], nextPort: number): string[] {
  const nextArgs = [...args];
  const portFlagIndex = nextArgs.findIndex((arg) => arg === "--port" || arg === "-p");
  if (portFlagIndex >= 0 && portFlagIndex + 1 < nextArgs.length) {
    nextArgs[portFlagIndex + 1] = String(nextPort);
  }
  return nextArgs;
}

export function replacePortInUrl(url: string, currentPort: number, nextPort: number): string {
  return url.replace(`:${currentPort}`, `:${nextPort}`);
}

/**
 * Uses `lsof` as a fast preflight for ports already owned by long-running dev
 * servers.
 *
 * The Node bind probe below catches the same class of failures eventually, but
 * `lsof` avoids transient server allocation when the port is obviously busy.
 * The helper is best-effort so environments without `lsof` still work.
 */
function hasListeningProcessOnPort(port: number): boolean {
  if (!commandAvailable("lsof", ["-v"])) {
    return false;
  }

  const result = spawnSync("lsof", ["-nP", `-iTCP:${port}`, "-sTCP:LISTEN"], {
    cwd: REPO_ROOT,
    env: BASE_ENV,
    stdio: "ignore",
  });
  return result.status === 0;
}

async function isPortAvailable(port: number): Promise<boolean> {
  if (hasListeningProcessOnPort(port)) {
    return false;
  }

  return await new Promise((resolve) => {
    const server = createServer();

    server.unref();
    server.once("error", () => {
      resolve(false);
    });
    // Probe the default bind target so wildcard IPv6 listeners are treated as busy too.
    server.listen(port, () => {
      server.close(() => {
        resolve(true);
      });
    });
  });
}

/**
 * Finds a usable port near the preferred fixture port.
 *
 * Real-world dev fixtures often assume well-known ports. We keep that default
 * stable when possible, then scan a small deterministic range so parallel local
 * sessions and CI leftovers do not make the whole launcher fail immediately.
 */
export async function resolveAvailablePort(preferredPort: number): Promise<number> {
  for (let offset = 0; offset < 20; offset += 1) {
    const port = preferredPort + offset;
    if (await isPortAvailable(port)) {
      return port;
    }
  }

  throw new Error(`Unable to find an available port starting at ${preferredPort}.`);
}
