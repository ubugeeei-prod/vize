// Options API matrix harness: vue-tsc vs content mapper (tsgo) vs vize check.
import { execFile } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";

const S = path.dirname(new URL(import.meta.url).pathname);
const DEPS = path.join(S, "deps/node_modules");
const TSGO = path.join(DEPS, "@typescript/typescript-darwin-arm64/lib/tsc");
const PUB_VIZE_PKG = fs.realpathSync(
  path.join(S, "../coord-cls/r/node_modules/vize"),
);
const LOCAL_VIZE = process.env.LOCAL_VIZE ??
  "/Users/nishimura/Source/github.com/ubugeeei-prod/vize-wt-options-api/target/ci/vize";

const args = process.argv.slice(2);
const opt = (name: string, def: string) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 ? args[i + 1]! : def;
};
const routes = opt("routes", "vuetsc,cm,check").split(",");
const filter = new RegExp(opt("filter", "."));
const jobs = Number(opt("jobs", "6"));
const runTag = opt("tag", "loc");

const DEFAULT_TSCONFIG = {
  compilerOptions: {
    target: "ESNext",
    module: "ESNext",
    moduleResolution: "bundler",
    strict: true,
    noEmit: true,
    skipLibCheck: true,
    jsx: "preserve",
    lib: ["ESNext", "DOM"],
    types: [],
    experimentalDecorators: true,
  },
  include: ["src/**/*.ts", "src/**/*.vue", "src/**/*.js", "src/**/*.d.ts"],
  contentMappers: [{ package: "vize", extensions: [".vue"] }],
};

const SHARED = ["vue", "@vue", "vue-router", "pinia", "vuex", "vue-property-decorator",
  "vue-class-component", "vue-facing-decorator", "@vue/devtools-api", "csstype", "vue-demi"];

function routeRoot(route: string) {
  const root = path.join(S, "run", `${route}-${runTag}`);
  const nm = path.join(root, "node_modules");
  if (!fs.existsSync(nm)) {
    fs.mkdirSync(nm, { recursive: true });
    for (const name of fs.readdirSync(DEPS)) {
      if (name.startsWith(".") || name === "vize") continue;
      fs.symlinkSync(path.join(DEPS, name), path.join(nm, name));
    }
    if (route === "cm") {
      const pkg = path.join(nm, "vize");
      if (runTag === "pub") {
        fs.symlinkSync(PUB_VIZE_PKG, pkg);
      } else {
        fs.mkdirSync(pkg);
        fs.writeFileSync(path.join(pkg, "package.json"), JSON.stringify({
          name: "vize", private: true,
          typescript: { contentMapper: {
            exec: [LOCAL_VIZE, "content-mapper"],
            compilerOptions: ["checkJs", "noUnusedLocals", "rootDir", "module", "moduleResolution",
              "resolvePackageJsonExports", "customConditions", "baseUrl", "paths", "moduleSuffixes",
              "noDtsResolution", "preserveSymlinks"],
            dynamicConfig: true,
          } },
        }, null, 2));
      }
    }
  }
  return root;
}

function copyDir(src: string, dst: string) {
  fs.mkdirSync(dst, { recursive: true });
  for (const e of fs.readdirSync(src, { withFileTypes: true })) {
    if (e.name === "expect.json") continue;
    const s = path.join(src, e.name), d = path.join(dst, e.name);
    if (e.isSymbolicLink()) fs.symlinkSync(fs.readlinkSync(s), d);
    else if (e.isDirectory()) copyDir(s, d);
    else fs.copyFileSync(s, d);
  }
}

function sh(cmd: string, argv: string[], cwd: string): Promise<{ code: number; out: string; stdout: string }> {
  return new Promise((resolve) => {
    execFile(cmd, argv, { cwd, maxBuffer: 256 << 20, timeout: 180_000 }, (err, stdout, stderr) => {
      const code = err ? ((err as { code?: number }).code ?? 99) : 0;
      resolve({ code: typeof code === "number" ? code : 98, out: `${stdout}${stderr}`, stdout });
    });
  });
}

type Diag = { key: string; text: string };
function parseTsc(out: string): Diag[] {
  const diags: Diag[] = [];
  for (const raw of out.split("\n")) {
    const m = /^(.+?)\((\d+),(\d+)\): (error|warning) TS(\d+): (.*)$/.exec(raw.trim());
    if (m) diags.push({ key: `${m[1]}:${m[2]}:${m[3]} TS${m[5]}`, text: m[6]! });
    else if (/error TS\d+/.test(raw)) diags.push({ key: `GLOBAL ${raw.trim().slice(0, 80)}`, text: raw });
  }
  return diags;
}
function parseCheck(out: string): Diag[] {
  const start = out.indexOf("{");
  if (start < 0) return [{ key: "CRASH", text: out.slice(0, 400) }];
  let json: { files?: Array<{ file: string; diagnostics: string[] }> };
  try { json = JSON.parse(out.slice(start)); } catch { return [{ key: "CRASH", text: out.slice(0, 400) }]; }
  const diags: Diag[] = [];
  for (const f of json.files ?? []) {
    for (const d of f.diagnostics) {
      const m = /^(error|warning):(\d+):(\d+) \[TS(\d+)\] ?(.*)$/s.exec(d);
      if (m) {
        if (m[1] === "warning") continue;
        diags.push({ key: `${f.file.replaceAll("\\", "/")}:${m[2]}:${m[3]} TS${m[4]}`, text: m[5]!.split("\n")[0]! });
      } else diags.push({ key: `${f.file}:? ${d.slice(0, 60)}`, text: d });
    }
  }
  return diags;
}

async function runRoute(route: string, id: string, caseDir: string): Promise<Diag[]> {
  const dir = path.join(routeRoot(route), id);
  fs.rmSync(dir, { recursive: true, force: true });
  copyDir(caseDir, dir);
  if (!fs.existsSync(path.join(dir, "tsconfig.json"))) {
    fs.writeFileSync(path.join(dir, "tsconfig.json"), JSON.stringify(DEFAULT_TSCONFIG, null, 2));
  }
  if (route === "vuetsc") {
    const r = await sh("node", [path.join(DEPS, "vue-tsc/bin/vue-tsc.js"), "--noEmit", "-p", "tsconfig.json", "--pretty", "false"], dir);
    return parseTsc(r.out);
  }
  if (route === "cm") {
    const r = await sh(TSGO, ["--runExternalCode", "--noEmit", "-p", "tsconfig.json", "--pretty", "false"], dir);
    const d = parseTsc(r.out);
    if (r.code !== 0 && d.length === 0) return [{ key: "CRASH", text: r.out.slice(0, 600) }];
    return d;
  }
  const bin = runTag === "pub" ? path.join(S, "../coord-cls/r/node_modules/.bin/vize") : LOCAL_VIZE;
  const r = await sh(bin, ["check", "--format", "json", "--corsa-path", TSGO], dir);
  return parseCheck(r.stdout || r.out);
}

const cases = fs.readdirSync(path.join(S, "matrix")).filter((id) => filter.test(id)).sort();
const results: Record<string, Record<string, Diag[]>> = {};
let next = 0;
async function worker() {
  while (next < cases.length) {
    const id = cases[next++]!;
    const caseDir = path.join(S, "matrix", id);
    results[id] = {};
    for (const route of routes) results[id]![route] = await runRoute(route, id, caseDir);
  }
}
await Promise.all(Array.from({ length: jobs }, worker));

const report: string[] = [];
let diverging = 0;
for (const id of cases) {
  const r = results[id]!;
  const base = r.vuetsc ? new Set(r.vuetsc.map((d) => d.key)) : null;
  const lines: string[] = [];
  for (const route of routes) {
    if (route === "vuetsc" || !base) continue;
    const got = new Set(r[route]!.map((d) => d.key));
    const missing = [...base].filter((k) => !got.has(k));
    const extra = r[route]!.filter((d) => !base.has(d.key));
    if (missing.length || extra.length) {
      for (const k of missing) lines.push(`  [${route}] MISSING ${k} :: ${r.vuetsc!.find((d) => d.key === k)!.text.slice(0, 160)}`);
      for (const d of extra) lines.push(`  [${route}] EXTRA   ${d.key} :: ${d.text.slice(0, 160)}`);
    }
  }
  if (lines.length) diverging++;
  report.push(`${lines.length ? "DIFF" : "ok  "} ${id} (vue-tsc ${r.vuetsc?.length ?? "-"})`);
  report.push(...lines);
}
report.push(`\n${cases.length} cases, ${diverging} diverging`);
fs.writeFileSync(path.join(S, `report-${runTag}.txt`), report.join("\n"));
fs.writeFileSync(path.join(S, `results-${runTag}.json`), JSON.stringify(results, null, 1));
console.log(report.join("\n"));
