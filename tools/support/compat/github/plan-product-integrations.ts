import { execFileSync } from "node:child_process";
import { appendFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export type ProductIntegrationPlan = {
  nuxt: boolean;
  "musea-native": boolean;
  "musea-browser": boolean;
};

const NONE: ProductIntegrationPlan = {
  nuxt: false,
  "musea-native": false,
  "musea-browser": false,
};
const ALL: ProductIntegrationPlan = {
  nuxt: true,
  "musea-native": true,
  "musea-browser": true,
};

const shared =
  /^(package\.json$|pnpm-(lock|workspace)\.yaml$|patches\/|\.npmrc$|\.node-version$|vite\.config\.ts$|tools\/config\/|tools\/support\/compat\/github\/plan-product-integrations\.ts$|tests\/tooling\/plan-product-integrations\.test\.ts$|\.github\/actions\/(setup-ubuntu-archive|cache-vp-bootstrap)\/)/;
// These regressions replace the Vite and Nuxt JavaScript adapters in pinned
// published installations. They do not build or exercise candidate Rust.
const nuxt =
  /^(npm\/builder\/(vite|unplugin|shared)\/|npm\/framework\/nuxt\/|npm\/cli\/src\/(config\/|types\/)|crates\/vize_carton\/src\/config\/loader\/vite-runtime\.[cm]?[jt]s$|tools\/support\/compat\/nuxt\/|tests\/_fixtures\/_projects\/nuxt-(scoped-style-build|critical-css-build|dev-ssr-style)\/|tests\/package\.json$|\.github\/workflows\/nuxt-style-build\.yml$)/;
const musea =
  /^(npm\/builder\/vite-musea\/|tests\/_fixtures\/differential\/musea\/|tests\/tooling\/fixtures\/musea\/|examples\/vite-musea\/|tools\/support\/release\/public_acceptance\/gallery\.ts$|\.github\/workflows\/musea-browser\.yml$)/;
// Native inline Art consumes the source N-API compiler, CLI, and Vite adapter.
// Rust's transitive dependencies are deliberately conservative here; unrelated
// docs, tooling, and other JS integrations never start a native browser build.
const museaNative =
  /^(crates\/|davinci\/|vendor\/|\.cargo\/|Cargo\.(toml|lock)$|rust-toolchain\.toml$|npm\/(native|cli)\/|npm\/builder\/(vite|unplugin|shared)\/|tools\/commands\/build\/|tools\/support\/build\/|tools\/support\/command-runtime\.ts$|tools\/support\/github\/build-napi-package\.ts$|\.github\/actions\/setup-rust|tests\/package\.json$)/;

export function planProductIntegrations(paths: readonly string[]): ProductIntegrationPlan {
  const plan = { ...NONE };
  for (const file of paths) {
    if (
      !file ||
      file.startsWith("/") ||
      file.includes("\\") ||
      /[\0\r\n]/.test(file) ||
      file.split("/").some((part) => !part || part === "." || part === "..")
    ) {
      throw new Error(`expected a normalized repository path: ${JSON.stringify(file)}`);
    }
    if (shared.test(file)) return { ...ALL };
    if (nuxt.test(file)) plan.nuxt = true;
    if (musea.test(file)) plan["musea-native"] = plan["musea-browser"] = true;
    if (museaNative.test(file)) plan["musea-native"] = true;
    if (file === "tools/support/release/public_acceptance/browser_observation.ts") {
      plan["musea-browser"] = true;
    }
  }
  return plan;
}

export function productIntegrationPlanForEvent(
  event: string,
  base: string,
  head: string,
  cwd = process.cwd(),
): ProductIntegrationPlan {
  if (event === "workflow_dispatch" || event === "schedule") return { ...ALL };
  if (!["pull_request", "merge_group", "push"].includes(event)) {
    throw new Error(`unsupported integration planning event: ${event}`);
  }
  if (!/^[a-f0-9]{40}$/.test(base) || !/^[a-f0-9]{40}$/.test(head) || /^0+$/.test(head)) {
    throw new Error("expected full base and head commit SHAs");
  }
  if (/^0+$/.test(base)) return { ...ALL };
  const paths = execFileSync(
    "git",
    ["diff", "--no-renames", "--name-only", "--diff-filter=ACDMRT", "-z", base, head],
    { cwd, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
  )
    .split("\0")
    .filter(Boolean);
  return planProductIntegrations(paths);
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const [event = "", base = "", head = ""] = process.argv.slice(2);
  const plan = productIntegrationPlanForEvent(event, base, head);
  const outputs = Object.entries(plan).map(([name, selected]) => `${name}=${selected}`);
  if (process.env.GITHUB_OUTPUT)
    appendFileSync(process.env.GITHUB_OUTPUT, `${outputs.join("\n")}\n`);
  console.log(`Product integrations for ${event}: ${outputs.join(", ")}`);
}
