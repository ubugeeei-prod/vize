import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { TYPECHECKER_TESTS } from "./typechecker.ts";

type SourceIdentity = { sha: string; tree: string; toolchain: string };

export function typecheckerSourceIdentity(repoRoot: string): SourceIdentity {
  const git = (...args: string[]) =>
    execFileSync("git", args, {
      cwd: repoRoot,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    }).trim();
  const sha = git("rev-parse", "HEAD");
  const committed = git("show", `${sha}:rust-toolchain.toml`);
  const channels = [...committed.matchAll(/^channel\s*=\s*"([^"]+)"\s*$/gm)];
  assert.equal(channels.length, 1, "one committed Rust toolchain stamp is required");
  const toolchain = channels[0][1];
  assert(["1.98.0", "1.99.0", "1.99.1"].includes(toolchain), "unsupported source Rust toolchain");
  return { sha, tree: git("rev-parse", `${sha}^{tree}`), toolchain };
}

// Nextest emits a fixed, local JUnit format. Only this exact binary's registered
// ASCII names are read; failures, skips and duplicate identities are rejected.
export function passedFixtureTests(xml: string) {
  assert(!/<!DOCTYPE|<!ENTITY/.test(xml), "external XML declarations are unsupported");
  const names = new Set<string>();
  for (const match of xml.matchAll(/<testcase\b([^>]*?)(?:\/>|>([\s\S]*?)<\/testcase>)/g)) {
    const attributes = new Map(
      [...match[1].matchAll(/\b([a-zA-Z]+)="([^"]*)"/g)].map((item) => [item[1], item[2]]),
    );
    if (attributes.get("classname") !== "vize_canon::fix_history_diagnostics") continue;
    const name = attributes.get("name");
    assert(
      name && Object.values(TYPECHECKER_TESTS).some((expected) => expected === name),
      "unplanned Canon test identity",
    );
    assert(!names.has(name), "duplicate Canon JUnit identity");
    assert(
      !/<(?:failure|error|skipped)\b/.test(match[2] ?? ""),
      "Canon test did not actually pass",
    );
    names.add(name);
  }
  return [...names];
}
