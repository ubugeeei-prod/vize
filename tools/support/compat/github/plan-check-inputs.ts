import { execFileSync } from "node:child_process";
import { appendFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { comparisonBase } from "./comparison-base.ts";

export function planCheckInputs(paths: readonly string[], full = false) {
  // A change to the selector or task definitions must exercise its consumers.
  const shared = paths.some((path) =>
    /^(?:\.github\/workflows\/check\.yml|tools\/support\/compat\/github\/plan-check-inputs\.ts|tools\/config\/vite-plus\/|vite\.config\.ts)/.test(
      path,
    ),
  );
  const matches = (pattern: RegExp) => full || shared || paths.some((path) => pattern.test(path));
  const dependencies = matches(
    /(^|\/)(?:Cargo\.(?:toml|lock)|package\.json|pnpm-lock\.yaml|pnpm-workspace\.yaml)$|^patches\/|^tools\/support\/security\/|^tools\/commands\/ci\/npm-audit|^\.cargo\/audit\.toml$/,
  );
  return {
    full,
    rust: matches(/\.rs$|(^|\/)Cargo\.toml$|^rust-toolchain(?:\.toml)?$|^\.cargo\//),
    dependencies,
    engines:
      dependencies ||
      matches(
        /^tools\/(?:config|commands|support)\/.*\.(?:[cm]?js|[cm]?ts)$|^tests\/tooling\/(?:node-engine-matrix|package-manifests|dependency-security-compat)\.test\./,
      ),
    apps:
      dependencies ||
      // npm SFCs, including composable examples, are authored inputs to the
      // current-source opinionated lint step in check-vize-apps.
      matches(
        /^(?:crates|davinci|vendor|playground|examples\/vite-musea)\/|^npm\/.*\.vue$|^npm\/(?:cli|native|builder\/(?:vite|vite-musea|unplugin))\/|^\.cargo\/|^rust-toolchain(?:\.toml)?$|^docs\/davinci\/plan\/(?!.*\.md$)|^docs\/davinci\/plan\/.*fact-alpha-schemas\.md$|^\.github\/actions\/(?:setup-rust-sticky-cache|check-vize-apps)\/|^tools\/commands\/ci\/opinionated-source-lint\./,
      ),
  };
}

function main() {
  const event = process.env.GITHUB_EVENT_NAME;
  const full = event === "schedule" || event === "workflow_dispatch";
  if (!full && !["pull_request", "merge_group", "push"].includes(event ?? "")) {
    throw new Error(`Unsupported Check event: ${event}`);
  }
  const git = (...args: string[]) =>
    execFileSync("git", args, { encoding: "utf8", maxBuffer: 16 * 1024 * 1024 });
  let base = process.env.EVENT_BASE_SHA ?? "";
  let paths: string[] = [];
  if (!full) {
    if (event === "pull_request" || event === "merge_group") {
      base = comparisonBase({
        event,
        eventBase: base,
        head: process.env.GITHUB_SHA,
        checkedOutHead: git("rev-parse", "HEAD").trim(),
        prHead: process.env.PR_HEAD_SHA,
        commit: git("cat-file", "-p", "HEAD"),
      }).base;
    }
    if (!/^[0-9a-f]{40}$/.test(base) || /^0+$/.test(base)) {
      throw new Error("Check requires an exact nonzero comparison base");
    }
    // A failed fetch/diff must fail the plan, never silently select no checks.
    git("fetch", "--no-tags", "--depth=1", "origin", base);
    paths = git("diff", "--name-only", "--no-renames", "-z", base, "HEAD")
      .split("\0")
      .filter(Boolean);
  }
  const plan = planCheckInputs(paths, full);
  const output = Object.entries(plan)
    .map(([key, value]) => `${key}=${value}`)
    .join("\n");
  if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, `${output}\n`);
  if (process.env.GITHUB_STEP_SUMMARY) {
    appendFileSync(
      process.env.GITHUB_STEP_SUMMARY,
      `Check inputs: ${full ? "full qualification" : `${paths.length} changed paths against ${base}`}\n\n\`\`\`\n${output}\n\`\`\`\n`,
    );
  }
  console.log(output);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
