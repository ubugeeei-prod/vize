import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { lstatSync, readFileSync, realpathSync } from "node:fs";
import { dirname, join, normalize } from "node:path";
import { fileImage } from "../../../support/release/computed_event_custody.ts";
import { fixtureNames } from "./fixtures.ts";
import { declarationFileName } from "./declaration-data.ts";
import { a65FileName } from "./a-data.ts";
import { genericArrowFileName } from "./generic-arrow-data.ts";

export function sourceCustody() {
  const root = execFileSync("git", ["rev-parse", "--show-toplevel"], { encoding: "utf8" }).trim();
  assert.equal(realpathSync(root), root);
  const head = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
  assert.equal(head, process.env.GITHUB_SHA);
  const ns = "tools/benchmarks/scripts/public-typechecker-q16/",
    queue = [
      ns + "run.ts",
      ...fixtureNames.map((name) => ns + name),
      ns + declarationFileName,
      ns + a65FileName,
      ns + genericArrowFileName,
      ".github/workflows/public-typechecker-consumer.yml",
    ],
    seen = new Set<string>(),
    images = [];
  while (queue.length) {
    const path = queue.shift()!;
    if (seen.has(path)) continue;
    seen.add(path);
    assert(!path.startsWith("../") && !path.startsWith("/"));
    const target = join(root, path),
      stat = lstatSync(target);
    assert(
      stat.isFile() && !stat.isSymbolicLink() && realpathSync(target) === target,
      "physical controller source required",
    );
    const approved = execFileSync("git", ["--no-replace-objects", "show", head + ":" + path], {
      cwd: root,
      env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
      maxBuffer: 2 ** 20,
    });
    assert(
      readFileSync(target).equals(approved),
      "executed controller source differs from exact checkout commit",
    );
    images.push(fileImage(target));
    if (path.endsWith(".ts"))
      for (const match of approved
        .toString("utf8")
        .matchAll(/\b(?:from\s*|import\s*\()\s*["'](\.{1,2}\/[^"']+)["']/gu)) {
        const next = normalize(join(dirname(path), match[1]!));
        if (next.endsWith(".ts")) queue.push(next);
      }
  }
  return images;
}
