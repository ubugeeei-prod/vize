import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { fileImage, inside } from "../computed_event_custody.ts";
import type { Image } from "./model.ts";
/** Capture the actual generic import closure as well as the separate preload
 * and probe entry points. This assigns tool custody, not product source credit. */
export function controllerFiles(): Image[] {
  const here = dirname(fileURLToPath(import.meta.url)),
    root = resolve(here, "../../../..");
  const pending = ["run.ts", "bootstrap.ts", "native-observer.ts", "identity-probe.ts"].map((p) =>
    resolve(here, p),
  );
  const seen = new Set<string>();
  pending.push(
    resolve(here, "original-inputs.json"),
    resolve(here, "after-input-plan.json.txt"),
    resolve(here, "pnpm-bootstrap.json"),
    resolve(here, "pnpm-shim-templates.json.txt"),
    resolve(here, "dependency-bin-policy.json.txt"),
    resolve(here, "before-capture-policy.json.txt"),
    resolve(root, ".github/workflows/published-human-reports-11252-11253.yml"),
  );
  for (let index = 0; index < pending.length; index++) {
    const file = pending[index]!;
    if (seen.has(file)) continue;
    assert(inside(root, file));
    assert(
      /\.(?:ts|json|yml)$/u.test(file) ||
        [
          "after-input-plan.json.txt",
          "pnpm-shim-templates.json.txt",
          "dependency-bin-policy.json.txt",
          "before-capture-policy.json.txt",
        ].some((name) => file === resolve(here, name)),
    );
    seen.add(file);
    const source = readFileSync(file, "utf8");
    for (const match of source.matchAll(
      /(?:from\s*|import\s*|import\()\s*["'](\.{1,2}\/[^"']+)["']/gu,
    ))
      pending.push(resolve(dirname(file), match[1]!));
  }
  return [...seen].sort().map(fileImage);
}
