import { execFileSync } from "node:child_process";
import { appendFileSync } from "node:fs";

export function planSourceChecks(paths, eventName = "pull_request") {
  if (!["pull_request", "merge_group"].includes(eventName)) {
    throw new Error("expected pull_request or merge_group source planning context");
  }
  if (eventName === "merge_group") {
    return { rust: true, js: true, tooling: true, playground: true };
  }
  // A new source directory must be validated until its dependencies are known.
  // Compiler changes can affect the native JS binding and its package tests.
  const result = { rust: false, js: false, tooling: false, playground: false };
  for (const path of paths) {
    if (/^(docs\/|\.changeset\/)/.test(path) || /(^|\/)README\.md$/.test(path)) continue;
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
    if (/^(crates\/|\.cargo\/|Cargo\.(toml|lock)$|rust-toolchain\.toml$)/.test(path)) {
      result.rust = true;
      result.js = true;
      if (
        /^(crates\/vize_(atelier|s[12]|davinci|impeto|armature)|Cargo\.(toml|lock)$|rust-toolchain\.toml$)/.test(
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

export function changedPaths(base, head, cwd = process.cwd()) {
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

if (process.argv[1]?.endsWith("/plan-source-checks.mjs")) {
  const [base, head, eventName = "pull_request"] = process.argv.slice(2);
  if (!/^[0-9a-f]{40}$/.test(base ?? "") || !/^[0-9a-f]{40}$/.test(head ?? "")) {
    throw new Error("expected full base and head commit SHAs");
  }
  // A new branch or an unavailable predecessor gets both gates, never a pass.
  const paths = /^0+$/.test(base) ? [".github/workflows/check.yml"] : changedPaths(base, head);
  if (!["pull_request", "merge_group"].includes(eventName))
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
