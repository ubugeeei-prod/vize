/**
 * `vize check` on a large synthetic pnpm workspace (#9727).
 *
 * Mirrors the reported shape: ~130 workspace packages linked pnpm-style
 * (`packages/<name>/node_modules/@ws/<dep>` symlinks), ~2k source files, ~200
 * SFCs in one app that imports many packages, and some packages that reach
 * `.vue` files. Reports the CLI's own `collect` / `imports` / `gen` / `corsa`
 * phases so Rust-side preparation can be compared run to run.
 *
 * Usage:
 *   node tools/benchmarks/scripts/check-workspace.ts --bin target/ci-opt/vize
 *   node tools/benchmarks/scripts/check-workspace.ts --bin a --bin b --runs 5
 *   node tools/benchmarks/scripts/check-workspace.ts --generate-only --out /tmp/ws
 *   (--vue / --typescript <dir> link packages when this checkout has no install)
 */

import { spawnSync } from "node:child_process";
import { mkdirSync, realpathSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

const scriptsDir = dirname(fileURLToPath(import.meta.url));
const require = createRequire(join(scriptsDir, "package.json"));

export interface WorkspaceShape {
  packages: number;
  modulesPerPackage: number;
  depsPerPackage: number;
  /** The last `vuePackages` packages export SFCs, so their routes reach
   * `.vue`; no package depends on them, so the other routes stay plain TS. */
  vuePackages: number;
  appComponents: number;
  importsPerComponent: number;
}

export const DEFAULT_SHAPE: WorkspaceShape = {
  packages: 130,
  modulesPerPackage: 13,
  depsPerPackage: 4,
  vuePackages: 10,
  appComponents: 200,
  importsPerComponent: 5,
};

const pad = (index: number) => String(index).padStart(3, "0");
const pkgName = (index: number) => `pkg-${pad(index)}`;

function write(path: string, content: string) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, content);
}

function link(target: string, path: string) {
  mkdirSync(dirname(path), { recursive: true });
  symlinkSync(target, path, "dir");
}

/** Deterministic dependency choice: lower-numbered plain-TS packages (a DAG). */
function dependencies(index: number, shape: WorkspaceShape): number[] {
  const deps = new Set<number>();
  const limit = Math.min(index, shape.packages - shape.vuePackages);
  for (let divisor = 1; divisor <= shape.depsPerPackage * 2 && limit > 0; divisor++) {
    if (deps.size < shape.depsPerPackage) deps.add(Math.floor((limit - 1) / divisor));
  }
  return [...deps].sort((a, b) => a - b);
}

function writePackage(root: string, index: number, shape: WorkspaceShape) {
  const dir = join(root, "packages", pkgName(index));
  const deps = dependencies(index, shape);
  const hasVue = index >= shape.packages - shape.vuePackages;
  write(
    join(dir, "package.json"),
    JSON.stringify(
      {
        name: `@ws/${pkgName(index)}`,
        version: "0.0.0",
        type: "module",
        exports: { ".": "./src/index.ts" },
        dependencies: Object.fromEntries(deps.map((dep) => [`@ws/${pkgName(dep)}`, "workspace:*"])),
      },
      null,
      2,
    ),
  );
  for (const dep of deps) {
    link(join("..", "..", "..", pkgName(dep)), join(dir, "node_modules", "@ws", pkgName(dep)));
  }
  const exports: string[] = [];
  for (let module = 0; module < shape.modulesPerPackage; module++) {
    const imports: string[] = [];
    const uses: string[] = [];
    if (module > 0) {
      imports.push(`import { value${module - 1} } from "./mod-${module - 1}";`);
      uses.push(`value${module - 1}`);
    }
    deps.forEach((dep, position) => {
      if ((module + position) % 3 === 0) {
        imports.push(`import { ${pkgName(dep).replace("-", "_")}_0 } from "@ws/${pkgName(dep)}";`);
        uses.push(`${pkgName(dep).replace("-", "_")}_0`);
      }
    });
    write(
      join(dir, "src", `mod-${module}.ts`),
      `${imports.join("\n")}
export interface Shape${module} { id: number; label: string; tags: readonly string[] }
export const value${module}: Shape${module} = { id: ${module}, label: "${pkgName(index)}", tags: [${uses.map((use) => `String(${use})`).join(", ")}] };
export function make${module}(input: Partial<Shape${module}> = {}): Shape${module} {
  return { ...value${module}, ...input };
}
`,
    );
    exports.push(`export * from "./mod-${module}";`);
  }
  exports.push(`export const ${pkgName(index).replace("-", "_")}_0 = ${index};`);
  if (hasVue) {
    write(
      join(dir, "src", "Widget.vue"),
      `<script setup lang="ts">
import { value0 } from "./mod-0";
defineProps<{ label?: string }>();
</script>

<template><span>{{ label ?? value0.label }}</span></template>
`,
    );
    exports.push(`export { default as Widget${index} } from "./Widget.vue";`);
  }
  write(join(dir, "src", "index.ts"), `${exports.join("\n")}\n`);
}

function writeApp(root: string, shape: WorkspaceShape) {
  const dir = join(root, "apps", "web");
  const deps = Array.from({ length: shape.packages }, (_, index) => index);
  write(
    join(dir, "package.json"),
    JSON.stringify(
      {
        name: "@ws/web",
        private: true,
        type: "module",
        dependencies: Object.fromEntries(deps.map((dep) => [`@ws/${pkgName(dep)}`, "workspace:*"])),
      },
      null,
      2,
    ),
  );
  for (const dep of deps) {
    link(
      join("..", "..", "..", "..", "packages", pkgName(dep)),
      join(dir, "node_modules", "@ws", pkgName(dep)),
    );
  }
  write(
    join(dir, "tsconfig.json"),
    JSON.stringify(
      {
        compilerOptions: {
          target: "ES2022",
          module: "ESNext",
          moduleResolution: "bundler",
          strict: true,
          jsx: "preserve",
          noEmit: true,
          skipLibCheck: true,
          types: [],
        },
        include: ["src/**/*.ts", "src/**/*.vue"],
      },
      null,
      2,
    ),
  );
  write(
    join(dir, "src", "shims.d.ts"),
    `declare module "*.vue" {\n  import type { DefineComponent } from "vue";\n  const component: DefineComponent;\n  export default component;\n}\n`,
  );
  for (let component = 0; component < shape.appComponents; component++) {
    const imports: string[] = [];
    const uses: string[] = [];
    for (let slot = 0; slot < shape.importsPerComponent; slot++) {
      const dep = (component * 31 + slot * 17) % shape.packages;
      const binding = `make${slot % shape.modulesPerPackage}`;
      imports.push(`import { ${binding} as ${binding}_${slot} } from "@ws/${pkgName(dep)}";`);
      uses.push(`${binding}_${slot}().label`);
    }
    const next = component + 1 < shape.appComponents ? component + 1 : null;
    if (next !== null && component % 4 === 0) {
      imports.push(`import Child from "./Component${next}.vue";`);
    }
    write(
      join(dir, "src", "components", `Component${component}.vue`),
      `<script setup lang="ts">
import { computed, ref } from "vue";
${imports.join("\n")}
const props = defineProps<{ title: string; count?: number }>();
const open = ref(false);
const labels = computed(() => [${uses.join(", ")}]);
</script>

<template>
  <section :title="props.title" @click="open = !open">
    <h2>{{ props.title }} {{ props.count ?? 0 }}</h2>
    <ul v-if="open"><li v-for="label in labels" :key="label">{{ label }}</li></ul>${next !== null && component % 4 === 0 ? '\n    <Child title="child" />' : ""}
  </section>
</template>
`,
    );
  }
  write(
    join(dir, "src", "main.ts"),
    `import { createApp } from "vue";\nimport Root from "./components/Component0.vue";\ncreateApp(Root).mount("#app");\n`,
  );
}

export function generateWorkspace(
  root: string,
  shape: WorkspaceShape = DEFAULT_SHAPE,
  links: { vue?: string; typescript?: string } = {},
) {
  rmSync(root, { recursive: true, force: true });
  write(join(root, "package.json"), JSON.stringify({ name: "ws", private: true }, null, 2));
  write(join(root, "pnpm-workspace.yaml"), "packages:\n  - packages/*\n  - apps/*\n");
  // Corsa needs typescript@7; Vue supplies the SFC runtime types.
  for (const name of ["vue", "typescript"] as const) {
    const dir = links[name] ?? dirname(require.resolve(`${name}/package.json`));
    link(realpathSync(dir), join(root, "node_modules", name));
  }
  for (let index = 0; index < shape.packages; index++) {
    writePackage(root, index, shape);
  }
  writeApp(root, shape);
  return join(root, "apps", "web");
}

const PHASES = ["collect", "imports", "gen", "corsa"] as const;
type Phase = (typeof PHASES)[number];

function seconds(text: string): number {
  const match = /^([\d.]+)(ns|µs|us|ms|s)$/.exec(text.trim());
  if (!match) return Number.NaN;
  const scale = { ns: 1e-9, µs: 1e-6, us: 1e-6, ms: 1e-3, s: 1 }[match[2] as "s"];
  return Number(match[1]) * scale;
}

function runCheck(bin: string, cwd: string): Record<Phase | "total", number> {
  const result = spawnSync(bin, ["check", "-q", "src"], { cwd, encoding: "utf8" });
  const output = `${result.stdout}\n${result.stderr}`;
  const line = /Type checked \d+ files in ([\d.]+\w+) \(([^)]*)\)/.exec(output);
  if (!line) throw new Error(`no timing line from ${bin}:\n${output.slice(-2000)}`);
  const row = { total: seconds(line[1]) } as Record<Phase | "total", number>;
  for (const part of line[2].split(",")) {
    const [name, value] = part.split(":").map((piece) => piece.trim());
    if ((PHASES as readonly string[]).includes(name)) row[name as Phase] = seconds(value);
  }
  return row;
}

const median = (values: number[]) => {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.floor(sorted.length / 2)];
};

function main() {
  const { values } = parseArgs({
    options: {
      bin: { type: "string", multiple: true },
      runs: { type: "string", default: "5" },
      out: { type: "string" },
      vue: { type: "string" },
      typescript: { type: "string" },
      "generate-only": { type: "boolean", default: false },
    },
  });
  const root = resolve(values.out ?? join(os.tmpdir(), "vize-check-workspace"));
  const app = generateWorkspace(root, DEFAULT_SHAPE, {
    vue: values.vue,
    typescript: values.typescript,
  });
  console.log(`workspace: ${root}`);
  if (values["generate-only"]) return;
  const bins = (values.bin ?? []).map((bin) => resolve(bin));
  if (bins.length === 0) throw new Error("pass --bin <vize> at least once");
  const runs = Number(values.runs);
  const rows = new Map(bins.map((bin) => [bin, [] as Record<Phase | "total", number>[]]));
  runCheck(bins[0], app);
  // Interleave binaries so machine-load drift hits each one equally.
  for (let run = 0; run < runs; run++) {
    for (const bin of bins) rows.get(bin)?.push(runCheck(bin, app));
  }
  console.log(`| binary | total | ${PHASES.join(" | ")} |`);
  console.log(`| --- | ---: | ${PHASES.map(() => "---:").join(" | ")} |`);
  for (const [bin, samples] of rows) {
    const cell = (key: Phase | "total") => `${median(samples.map((row) => row[key])).toFixed(2)}s`;
    console.log(`| ${bin} | ${cell("total")} | ${PHASES.map(cell).join(" | ")} |`);
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) main();
