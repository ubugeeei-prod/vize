import assert from "node:assert/strict";
import {
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readlinkSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test, type TestContext } from "node:test";

import { createTypecheckToolVariants } from "../../tools/benchmarks/scripts/compare-tools-typecheck.mjs";
import { measureVariants } from "../../tools/benchmarks/scripts/compare-tools-measure.mjs";
import { runTypecheckCommand } from "../../tools/benchmarks/scripts/typecheck-command.mjs";
import {
  prepareMapperPackages,
  resolveMapperHosts,
} from "../../tools/benchmarks/scripts/typecheck-mapper-hosts.ts";

function scratch(t: TestContext): string {
  const dir = realpathSync(mkdtempSync(join(tmpdir(), "typecheck-mapper-lanes-")));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  return dir;
}

function write(file: string, source = ""): void {
  mkdirSync(join(file, ".."), { recursive: true });
  writeFileSync(file, source);
}

void test("each host checks the project through each mapper's tsconfig", () => {
  const calls: { bin: string; args: string[]; cwd: string }[] = [];
  const hosts = {
    tsc: "/hosts/tsc",
    tsrs: "/hosts/tsrs",
    typescriptDir: "/typescript",
    vueMapperDir: "/vue",
    vizeCliDir: "/cli",
  };
  const variants = createTypecheckToolVariants({
    fileCount: 20,
    vizeBin: "/bin/vize",
    corsaPath: "/bin/tsgo",
    resolveWorkspaceBin: (name: string) => `/bin/${name}`,
    mapperHosts: hosts,
    cpus: 32,
    runCommand: (bin: string, args: string[], options: { cwd: string }) => {
      calls.push({ bin, args, cwd: options.cwd });
      return { ms: 1, cpuMs: 1, status: 0, stdout: "", stderr: "" };
    },
  });
  const lanes = variants.filter((v: { id: string }) => v.id.includes("-mapper"));
  assert.deepEqual(
    lanes.map((v: { id: string; label: string; tool: string; setting: string }) => [
      v.id,
      v.label,
      v.tool,
      v.setting,
    ]),
    [
      ["tsc-vize-mapper", "tsc + Vize mapper", "tsc + Vize mapper", "default"],
      ...["4", "8", "16", "max"].map((n) => [
        `tsc-vize-mapper-checkers-${n}`,
        `tsc + Vize mapper (--checkers ${n === "max" ? 32 : n})`,
        "tsc + Vize mapper",
        `--checkers ${n === "max" ? 32 : n}`,
      ]),
      ["tsc-vue-mapper", "tsc + Vue mapper", "tsc + Vue mapper", "default"],
      ...["4", "8", "16", "max"].map((n) => [
        `tsc-vue-mapper-checkers-${n}`,
        `tsc + Vue mapper (--checkers ${n === "max" ? 32 : n})`,
        "tsc + Vue mapper",
        `--checkers ${n === "max" ? 32 : n}`,
      ]),
      ["tsrs-vize-mapper", "tsrs + Vize mapper", "tsrs + Vize mapper", "default"],
      ["tsrs-vize-mapper-fast", "tsrs + Vize mapper (--fast)", "tsrs + Vize mapper", "--fast"],
      ["tsrs-vue-mapper", "tsrs + Vue mapper", "tsrs + Vue mapper", "default"],
      ["tsrs-vue-mapper-fast", "tsrs + Vue mapper (--fast)", "tsrs + Vue mapper", "--fast"],
    ],
  );
  for (const lane of lanes) lane.run("/project");
  const tail = (mapper: string) => [
    "--runExternalCode",
    "--pretty",
    "false",
    "-p",
    `/project/tsconfig.${mapper}-mapper.json`,
  ];
  const checkers = ["4", "8", "16", "32"].map((n) => ["--checkers", n]);
  assert.deepEqual(
    calls.map((call) => [call.bin, call.cwd, ...call.args]),
    [
      ...["vize", "vue"].flatMap((mapper) =>
        [[], ...checkers].map((flags) => [
          "/hosts/tsc",
          "/project",
          "--noEmit",
          ...flags,
          ...tail(mapper),
        ]),
      ),
      ...["vize", "vue"].flatMap((mapper) =>
        [[], ["--fast"]].map((flags) => [
          "/hosts/tsrs",
          "/project",
          "--noEmit",
          ...flags,
          ...tail(mapper),
        ]),
      ),
    ],
  );
  // Without hosts no mapper lane is created.
  assert.equal(
    createTypecheckToolVariants({
      fileCount: 20,
      vizeBin: "/bin/vize",
      corsaPath: "/bin/tsgo",
      resolveWorkspaceBin: (name: string) => `/bin/${name}`,
    }).filter((v: { id: string }) => v.id.includes("-mapper")).length,
    0,
  );
});

void test("the Vize mapper is installed as its published files, never into a linked scope", (t) => {
  const root = scratch(t);
  const cli = join(root, "cli");
  write(
    join(cli, "package.json"),
    JSON.stringify({
      name: "vize",
      files: ["bin", "dist"],
      dependencies: { "dep-a": "1", "dep-b": "1" },
    }),
  );
  write(join(cli, "bin", "vize"), "#!/usr/bin/env node\n");
  write(join(cli, "dist", "cli.mjs"), "export {};\n");
  write(join(cli, "tests", "unpublished.ts"));
  write(join(cli, "node_modules", "dep-a", "package.json"), "{}");
  write(join(cli, "node_modules", "dep-b", "package.json"), "{}");
  const typescript = join(root, "typescript");
  write(join(typescript, "package.json"), "{}");
  const vueMapper = join(root, "vue-mapper");
  write(join(vueMapper, "package.json"), "{}");
  const store = join(root, "store-scope");
  write(join(store, "shared", "package.json"), "{}");
  const project = join(root, "project");
  write(join(project, "node_modules", "dep-b", "package.json"), "{}");
  symlinkSync(store, join(project, "node_modules", "@vue"));
  const hosts = {
    tsc: "/tsc",
    tsrs: "/tsrs",
    typescriptDir: typescript,
    vueMapperDir: vueMapper,
    vizeCliDir: cli,
  };

  prepareMapperPackages(project, hosts);
  write(join(project, "node_modules", "vize", "stale.txt"));
  prepareMapperPackages(project, hosts);

  const vize = join(project, "node_modules", "vize");
  assert.ok(!lstatSync(vize).isSymbolicLink());
  assert.ok(existsSync(join(vize, "bin", "vize")) && existsSync(join(vize, "dist", "cli.mjs")));
  assert.ok(!existsSync(join(vize, "tests")) && !existsSync(join(vize, "stale.txt")));
  // The project's own install wins; Vue's mapper loads the project's typescript.
  assert.ok(!lstatSync(join(project, "node_modules", "dep-b")).isSymbolicLink());
  assert.equal(readlinkSync(join(project, "node_modules", "typescript")), typescript);
  assert.equal(
    readlinkSync(join(project, "node_modules", "dep-a")),
    join(cli, "node_modules", "dep-a"),
  );
  const scope = join(project, "node_modules", "@vue");
  assert.ok(!lstatSync(scope).isSymbolicLink());
  assert.equal(readlinkSync(join(scope, "shared")), join(store, "shared"));
  assert.equal(readlinkSync(join(scope, "content-mapper")), vueMapper);
  assert.ok(!existsSync(join(store, "content-mapper")));
  for (const [id, name] of [
    ["vize", "vize"],
    ["vue", "@vue/content-mapper"],
  ]) {
    assert.deepEqual(
      JSON.parse(readFileSync(join(project, `tsconfig.${id}-mapper.json`), "utf8")),
      { extends: "./tsconfig.json", contentMappers: [{ package: name, extensions: [".vue"] }] },
    );
  }

  rmSync(join(cli, "dist"), { recursive: true });
  assert.throws(() => prepareMapperPackages(join(root, "unbuilt"), hosts), /vize is not built/u);
  assert.throws(() => resolveMapperHosts(join(root, "no-hosts"), cli), /not found/u);
});

void test("process lanes publish CPU and a cold first run; wall-only lanes are unchanged", async () => {
  const process = [
    { ms: 50, cpuMs: 90, rssKb: 900 },
    { ms: 10, cpuMs: 30, rssKb: 700 },
    { ms: 12, cpuMs: 20, rssKb: 500 },
    { ms: 11, cpuMs: 40, rssKb: 600 },
  ];
  const measured = await measureVariants(
    [
      { id: "process", label: "process", files: 1, measure: () => process.shift() },
      { id: "wall", label: "wall", files: 1, measure: () => 5 },
      {
        id: "partial",
        label: "partial",
        files: 1,
        measure: (() => {
          const samples = [
            { ms: 9, cpuMs: 1 },
            { ms: 9, cpuMs: null },
            { ms: 9, cpuMs: 1 },
          ];
          return () => samples.shift() ?? { ms: 9, cpuMs: 1 };
        })(),
      },
    ],
    { warmups: 1, runs: 3 },
  );
  assert.deepEqual(measured[0], {
    id: "process",
    label: "process",
    medianMs: 11,
    runs: [10, 12, 11],
    throughput: "91 files/s",
    coldMs: 50,
    coldCpuMs: 90,
    cpuRuns: [30, 20, 40],
    cpuMedianMs: 30,
    rssRuns: [700, 500, 600],
    rssMedianKb: 600,
  });
  assert.deepEqual(Object.keys(measured[1]), ["id", "label", "medianMs", "runs", "throughput"]);
  // A lane whose CPU is missing for any run publishes no CPU figure at all.
  assert.ok(!("cpuRuns" in measured[2]) && !("cpuMedianMs" in measured[2]));
});

void test("an invocation is charged the CPU of the processes it waited for", (t) => {
  const cwd = scratch(t);
  const burn = "const end = Date.now() + 300; while (Date.now() < end) {}";
  const result = runTypecheckCommand(
    process.execPath,
    [
      "-e",
      `require("node:child_process").spawnSync(process.execPath, ["-e", ${JSON.stringify(burn)}])`,
    ],
    { cwd },
  );
  assert.equal(result.status, 0);
  if (process.platform === "linux") {
    assert.ok((result.cpuMs ?? 0) >= 250, `grandchild CPU missing: ${result.cpuMs}ms`);
    assert.ok((result.rssKb ?? 0) > 0, "peak RSS missing");
  } else {
    assert.equal(result.cpuMs, null);
    assert.equal(result.rssKb, null);
  }
});
