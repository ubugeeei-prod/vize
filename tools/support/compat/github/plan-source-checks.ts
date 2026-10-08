import { execFileSync } from "node:child_process";
import { appendFileSync } from "node:fs";

const censusScripts = new Set([
  "tools/support/compat/davinci/croquis-consumers.mjs",
  "tools/support/compat/davinci/lib/croquis-render.mjs",
  "tools/support/compat/davinci/lib/croquis-shards.mjs",
]);

// These Node-only census renderers and their owned generated reports have no
// workspace Rust input consumer. The mandatory inventory gate checks the exact
// generated shard set, including orphan files; unknown helpers/contracts stay
// conservative. A source-grounded tooling law guards the Rust consumer boundary.
export function isCensusToolingInput(path: string) {
  return (
    censusScripts.has(path) ||
    path === "docs/davinci/plan/croquis-consumption.md" ||
    /^docs\/davinci\/plan\/croquis-consumption\/vize(?:_[a-z0-9]+)*\.md$/.test(path)
  );
}

// These authored plans are compiled or read by Rust tests and compiler gates.
// Keep new contracts conservative. The dated completion ledger is verified
// prose, with no Rust input consumer; tooling still validates its docs.
export function isSharedRustInput(path: string) {
  return (
    (path.startsWith("docs/davinci/plan/") &&
      path !== "docs/davinci/plan/completion-2026-10-03.md" &&
      !isCensusToolingInput(path)) ||
    path === "npm/cli/schemas/vize.config.schema.json"
  );
}

export function planSourceChecks(
  paths: readonly string[],
  eventName: string | null = "pull_request",
) {
  if (!["pull_request", "merge_group"].includes(eventName as string)) {
    throw new Error("expected pull_request or merge_group source planning context");
  }
  if (eventName === "merge_group") {
    return { rust: true, js: true, tooling: true, playground: true };
  }
  // A new source directory must be validated until its dependencies are known.
  // Compiler changes can affect the native JS binding and its package tests.
  const result = { rust: false, js: false, tooling: false, playground: false };
  for (const path of paths) {
    if (isCensusToolingInput(path)) {
      result.tooling = true;
      continue;
    }
    if (isSharedRustInput(path)) {
      result.rust = result.js = result.tooling = result.playground = true;
      continue;
    }
    if (
      (/^(docs\/|\.changeset\/)/.test(path) && /\.(md|mdx)$/.test(path)) ||
      /(^|\/)(README|AGENTS)\.md$/.test(path)
    )
      continue;
    if (
      /^(tests\/tooling\/|tools\/support\/release\/|tools\/commands\/release\/|tools\/moon\/cmd\/release\/|tools\/support\/compat\/github\/|\.github\/workflows\/release[^/]*\.yml$)/.test(
        path,
      )
    ) {
      result.tooling = true;
      continue;
    }
    if (path.startsWith("tests/expected/")) {
      result.rust = true;
      continue;
    }
    if (/^(crates\/|davinci\/|\.cargo\/|Cargo\.(toml|lock)$|rust-toolchain\.toml$)/.test(path)) {
      result.rust = true;
      result.js = true;
      if (
        /^(davinci\/|crates\/vize_(atelier|s[12]|l[0-4]|davinci|impeto|armature)|Cargo\.(toml|lock)$|rust-toolchain\.toml$)/.test(
          path,
        )
      ) {
        result.playground = true;
      }
    } else if (
      /^(npm\/|tests\/|playground\/|editors\/|package\.json$|pnpm-|tools\/config\/)/.test(path)
    ) {
      result.js = true;
      if (
        /^(npm\/(builder\/vite\/|native\/|cli\/|compose\/)|playground\/|package\.json$|pnpm-)/.test(
          path,
        )
      ) {
        result.playground = true;
      }
    } else {
      result.rust = true;
      result.js = true;
      result.tooling = true;
      result.playground = true;
    }
  }
  return result;
}

export function changedPaths(base: string, head: string, cwd = process.cwd()) {
  return execFileSync(
    "git",
    ["diff", "--no-renames", "--name-only", "--diff-filter=ACDMRT", "-z", base, head],
    {
      cwd,
      encoding: "utf8",
    },
  )
    .split("\0")
    .filter(Boolean);
}

if (process.argv[1]?.endsWith("/plan-source-checks.ts")) {
  const [base, head, eventName = "pull_request"] = process.argv.slice(2);
  if (!/^[0-9a-f]{40}$/.test(base ?? "") || !/^[0-9a-f]{40}$/.test(head ?? "")) {
    throw new Error("expected full base and head commit SHAs");
  }
  // A new branch or an unavailable predecessor gets both gates, never a pass.
  const paths = /^0+$/.test(base) ? [".github/workflows/check.yml"] : changedPaths(base!, head!);
  if (!["pull_request", "merge_group"].includes(eventName as string))
    throw new Error("invalid source planning context");
  const plan = paths.length
    ? planSourceChecks(paths, eventName)
    : { rust: true, js: true, tooling: true, playground: true };
  const output = `rust=${plan.rust}\njs=${plan.js}\ntooling=${plan.tooling}\nplayground=${plan.playground}\n`;
  if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, output);
  process.stdout.write(
    `Scope: ${eventName}; changed paths: ${paths.length}; Rust: ${plan.rust}; JS packages: ${plan.js}; tooling: ${plan.tooling}; playground: ${plan.playground}\n`,
  );
  if (process.env.GITHUB_STEP_SUMMARY) {
    appendFileSync(
      process.env.GITHUB_STEP_SUMMARY,
      `### Source checks\n\n| Check | Run |\n| --- | --- |\n| Rust Clippy, tests, and fixtures | ${plan.rust} |\n| JS package build and tests | ${plan.js} |\n| Tooling scripts | ${plan.tooling} |\n| Playground browser tests | ${plan.playground} |\n\nChanged paths: ${paths.length}.\n`,
    );
  }
}
