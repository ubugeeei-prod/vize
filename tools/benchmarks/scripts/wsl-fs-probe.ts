/**
 * Filesystem latency probe for the content mapper's serial shape: per-call
 * cost by path depth, pinned directory fds, realpath variants, listings,
 * reads, and sequential vs concurrent independent stats.
 *
 * Usage: node wsl-fs-probe.ts <root> <label>
 */
import * as fs from "node:fs";
import { join } from "node:path";

const [root, label] = process.argv.slice(2);
const project = join(root, "projects", "vue-1000");
const dir = join(project, "src", "features", "f0001");
const shared = join(project, "src", "shared");
fs.mkdirSync(dir, { recursive: true });
fs.mkdirSync(shared, { recursive: true });
for (let i = 0; i < 20; i++) fs.writeFileSync(join(dir, `Widget${String(i).padStart(2, "0")}.vue`), "<template />\n");
fs.writeFileSync(join(dir, "model.ts"), "export interface Row { id: number }\n".repeat(20));
fs.writeFileSync(join(shared, "types.ts"), "export interface Entity { id: number }\n".repeat(20));
process.chdir(project);

const N = Number(process.env.PROBE_N ?? "200");
const rows: string[] = [];
function median(xs: number[]): number {
  const s = [...xs].sort((a, b) => a - b);
  return s[Math.floor(s.length / 2)];
}
function time(name: string, op: () => void, n = N) {
  for (let i = 0; i < 5; i++) op();
  const xs: number[] = [];
  for (let i = 0; i < n; i++) {
    const t = process.hrtime.bigint();
    op();
    xs.push(Number(process.hrtime.bigint() - t) / 1000);
  }
  rows.push(`| ${name} | ${median(xs).toFixed(1)} |`);
}
async function timeAsync(name: string, op: () => Promise<unknown>, n = Math.min(N, 100)) {
  for (let i = 0; i < 3; i++) await op();
  const xs: number[] = [];
  for (let i = 0; i < n; i++) {
    const t = process.hrtime.bigint();
    await op();
    xs.push(Number(process.hrtime.bigint() - t) / 1000);
  }
  rows.push(`| ${name} | ${median(xs).toFixed(1)} |`);
}
const tryOp = (op: () => unknown) => () => {
  try { op(); } catch {}
};
const ancestors: string[] = [];
for (let d = dir; ; ) {
  ancestors.push(join(d, "node_modules", "vue"));
  const parent = join(d, "..");
  if (parent === d) break;
  d = parent;
}

function suite(tag: string) {
  time(`${tag} stat project (cwd)`, () => fs.statSync(project));
  time(`${tag} stat src (depth 1)`, () => fs.statSync(join(project, "src")));
  time(`${tag} stat f0001 (depth 3)`, () => fs.statSync(dir));
  time(`${tag} stat f0001/model.ts (depth 4)`, () => fs.statSync(join(dir, "model.ts")));
  time(`${tag} stat f0001/missing (depth 4, ENOENT)`, tryOp(() => fs.statSync(join(dir, "missing"))));
  time(`${tag} stat project/node_modules/vue (ENOENT)`, tryOp(() => fs.statSync(join(project, "node_modules", "vue"))));
  time(`${tag} lstat src/shared (depth 2)`, () => fs.lstatSync(shared));
  time(`${tag} readlink f0001 (EINVAL)`, tryOp(() => fs.readlinkSync(dir)));
  time(`${tag} realpath.native f0001 (glibc)`, () => fs.realpathSync.native(dir));
  time(`${tag} open O_PATH-ish + readlink /proc/self/fd + close`, () => {
    const fd = fs.openSync(dir, "r");
    if (fs.existsSync("/proc/self/fd")) fs.readlinkSync(`/proc/self/fd/${fd}`);
    fs.closeSync(fd);
  });
  time(`${tag} readdir f0001 (20 entries, withFileTypes)`, () => fs.readdirSync(dir, { withFileTypes: true }));
  time(`${tag} readFile f0001/model.ts`, () => fs.readFileSync(join(dir, "model.ts"), "utf8"));
  time(`${tag} ${ancestors.length} node_modules/vue probes, sequential`, () => {
    for (const p of ancestors) { try { fs.statSync(p); } catch {} }
  });
}

suite("plain");
// Pin the importer directory's dentries for the duration, as an open fd does.
const pin = fs.openSync(dir, "r");
suite("pinned f0001");
fs.closeSync(pin);
await timeAsync(`plain ${ancestors.length} node_modules/vue probes, concurrent (libuv pool ${process.env.UV_THREADPOOL_SIZE ?? 4})`, () =>
  Promise.all(ancestors.map((p) => fs.promises.stat(p).catch(() => undefined))),
);
await timeAsync(`plain ${ancestors.length} node_modules/vue probes, sequential async`, async () => {
  for (const p of ancestors) await fs.promises.stat(p).catch(() => undefined);
});

const out = [`### fs probe: ${label} (${root}), median µs over ${N}`, "", "| op | µs |", "|---|---:|", ...rows, ""].join("\n");
console.log(out);
if (process.env.PROBE_OUT) fs.appendFileSync(process.env.PROBE_OUT, out + "\n");
