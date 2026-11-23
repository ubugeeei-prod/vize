import { configuredNodeLauncher } from "../../support/process-launcher.ts";
/** Untimed direct backend observer; no shell or user-source evaluation. */
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

export function createColdBackendObserver(backend: string, out: string) {
  const calls = join(out, "backend");
  mkdirSync(calls, { recursive: true });
  const wrapper = join(out, "backend-observer.cjs");
  const source = configuredNodeLauncher(
    new URL("./programs/observe-typechecker-backend.cjs", import.meta.url),
    { backend, root: calls },
  );
  writeFileSync(wrapper, source, { flag: "wx", mode: 0o755 });
  return { wrapper, calls };
}
