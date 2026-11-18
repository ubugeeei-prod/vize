/**
 * Runs the Linux GNU release artifacts inside older glibc distributions.
 *
 * Usage: node tools/commands/release/npm/smoke-glibc-floor-containers.ts
 *   --platform <linux-x64-gnu|linux-arm64-gnu> --mode <packages|binding|cli>
 *   [--cli <vize-binary>] [--node-dir <node-install-dir>] <image>...
 *
 * Each image runs in Docker with the workspace at /work.
 * - `packages` installs the packed `@vizejs/native`, its platform package,
 *   `vize`, `@vizejs/unplugin` and `@vizejs/vite-plugin` with `smoke-release-install.ts
 *   --runtime-checks` (`vize --version`, `check`, `lint`, `init`, and a
 *   `vite build`) on the Node.js at `--node-dir` (the runner's by default).
 * - `binding` only loads the platform binding and compiles an SFC, for images
 *   too old for the package managers, such as CentOS 7 at glibc 2.17 itself.
 * - `cli` runs the release CLI binary given by `--cli` (`--version`, `lint`).
 */
import { spawnSync } from "node:child_process";
import path from "node:path";

import { isEntrypoint, runEntrypoint } from "../../../support/command-runtime.ts";

const platforms = new Set(["linux-x64-gnu", "linux-arm64-gnu"]);
const modes = new Set(["packages", "binding", "cli"]);

export interface ContainerSmokeOptions {
  readonly platform: string;
  readonly mode: "packages" | "binding" | "cli";
  readonly images: readonly string[];
  readonly cli: string | undefined;
  readonly nodeDir: string;
}

export function parseContainerSmokeArguments(
  args: readonly string[],
  defaultNodeDir: string,
): ContainerSmokeOptions {
  let platform = "";
  let mode = "";
  let cli: string | undefined;
  let nodeDir = defaultNodeDir;
  const images: string[] = [];
  for (let index = 0; index < args.length; index += 1) {
    const argument = args[index]!;
    const value = () => {
      const next = args[index + 1];
      if (next === undefined || next.startsWith("--")) throw new Error(`${argument} needs a value`);
      index += 1;
      return next;
    };
    if (argument === "--platform") platform = value();
    else if (argument === "--mode") mode = value();
    else if (argument === "--cli") cli = value();
    else if (argument === "--node-dir") nodeDir = value();
    else if (argument.startsWith("--")) throw new Error(`Unknown argument: ${argument}`);
    else images.push(argument);
  }
  if (!platforms.has(platform)) {
    throw new Error(`--platform must be one of ${[...platforms].join(", ")}, got ${platform}`);
  }
  if (!modes.has(mode)) throw new Error(`--mode must be one of ${[...modes].join(", ")}`);
  if (mode === "cli" && cli === undefined) throw new Error("--mode cli requires --cli");
  if (images.length === 0) throw new Error("At least one container image is required");
  return { platform, mode: mode as ContainerSmokeOptions["mode"], images, cli, nodeDir };
}

const bindingScript = `
const binding = require(process.argv[1]);
const result = binding.compileSfc(
  '<template><div>{{ msg }}</div></template><script setup lang="ts">const msg: string = "ok";</script>',
  { filename: "Smoke.vue", isTs: true },
);
if (!result || result.errors.length > 0 || !result.code) throw new Error("compileSfc smoke failed");
console.log("loaded " + process.argv[1] + " and compiled an SFC");
`;

function shellQuote(value: string): string {
  return `'${value.replaceAll("'", "'\\''")}'`;
}

/** The bash program each container runs; every path is inside the container. */
export function containerScript(options: ContainerSmokeOptions): string {
  const { platform, mode, cli } = options;
  const lines = [
    "set -euo pipefail",
    '. /etc/os-release; echo "container: ${PRETTY_NAME}; $(ldd --version 2>&1 | head -n 1)"',
  ];
  if (mode === "cli") {
    const binary = `/work/${cli}`;
    lines.push(
      `${binary} --version`,
      "mkdir -p /tmp/vize-cli-smoke",
      "printf '%s\\n' '<template><div>{{ msg }}</div></template>' '<script setup lang=\"ts\">' 'const msg = \"ok\";' '</script>' > /tmp/vize-cli-smoke/App.vue",
      `${binary} lint /tmp/vize-cli-smoke/App.vue --no-config`,
    );
    return lines.join("\n");
  }
  lines.push(
    "export PATH=/opt/vize-node/bin:/tmp/vize-global/bin:$PATH npm_config_prefix=/tmp/vize-global",
    'echo "node $(node --version)"',
  );
  if (mode === "binding") {
    const binding = `/work/npm/native/npm/${platform}/vize-vitrine.${platform}.node`;
    lines.push(`node -e ${shellQuote(bindingScript)} ${binding}`);
  } else {
    lines.push(
      "mkdir -p /tmp/vize-global/bin",
      "corepack enable --install-directory /tmp/vize-global/bin",
      "npm install --global --allow-scripts=bun bun@1.3.14 vite-plus@0.1.24",
      `node tools/commands/release/npm/smoke-release-install.ts --prepare-manifests --runtime-checks npm/native npm/native/npm/${platform} npm/cli npm/builder/unplugin npm/builder/vite`,
    );
  }
  return lines.join("\n");
}

export function smokeGlibcFloorContainers(args: readonly string[]): number {
  const options = parseContainerSmokeArguments(args, path.dirname(path.dirname(process.execPath)));
  const script = containerScript(options);
  const failed: string[] = [];
  for (const image of options.images) {
    console.log(`::group::${image} (${options.platform}, ${options.mode})`);
    const volumes = ["--volume", `${process.cwd()}:/work`];
    if (options.mode !== "cli") volumes.push("--volume", `${options.nodeDir}:/opt/vize-node:ro`);
    const result = spawnSync(
      "docker",
      ["run", "--rm", ...volumes, "--workdir", "/work", image, "bash", "-c", script],
      { stdio: "inherit" },
    );
    console.log("::endgroup::");
    if (result.error !== undefined || result.status !== 0) {
      const reason = result.error
        ? `: ${result.error.message}`
        : ` with exit ${result.status ?? 1}`;
      console.error(`::error::${image}: ${options.platform} ${options.mode} smoke failed${reason}`);
      failed.push(image);
    } else {
      console.log(`${image}: ${options.platform} ${options.mode} smoke passed`);
    }
  }
  if (failed.length > 0)
    throw new Error(`glibc floor container smoke failed in ${failed.join(", ")}`);
  return 0;
}

if (isEntrypoint(import.meta.url)) {
  runEntrypoint(() => smokeGlibcFloorContainers(process.argv.slice(2)));
}
