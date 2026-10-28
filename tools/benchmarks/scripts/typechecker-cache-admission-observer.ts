/** Exact OS-exec and fresh-capture protocol controls for the untimed admission carrier. */
import assert from "node:assert/strict";
import fs from "node:fs";
import { createColdBackendObserver } from "./typechecker-cache-backend.ts";
export function createAdmissionBackend(backend: string, out: string) {
  const observer = createColdBackendObserver(backend, out);
  // Preserve all emitted TypeScript bytes before the backend starts, excluding package symlinks.
  const injection = `
function collectInputs(directory, relative='') {
  const rows=[];
  for(const name of fs.readdirSync(directory).sort()) {
  const full=path.join(directory,name), rel=path.join(relative,name), stat=fs.lstatSync(full);
  if(stat.isSymbolicLink()) { rows.push({path:rel,kind:'symlink',target:fs.readlinkSync(full)}); continue; }
  if(stat.isDirectory()) { if(name!=='node_modules'&&name!=='.vize-buildinfo')rows.push(...collectInputs(full,rel)); continue; }
  if(stat.isFile()&&(/\\.tsx?$/.test(name))) {
    const bytes=fs.readFileSync(full);
    rows.push({path:rel,kind:'typescript',bytes:bytes.length,sha256:crypto.createHash('sha256').update(bytes).digest('hex'),base64:bytes.toString('base64')});
  }
  }
  return rows;
}
if(project) fs.writeFileSync(path.join(directory,'virtual-inputs.json'),JSON.stringify(collectInputs(process.cwd()),null,2)+'\\n');
`;
  let wrapper = fs.readFileSync(observer.wrapper, "utf8");
  assert.equal(wrapper.split("const before=snapshot(buildInfo)").length, 2);
  wrapper = wrapper.replace(
    "const before=snapshot(buildInfo)",
    injection + "\nconst before=snapshot(buildInfo)",
  );
  fs.writeFileSync(observer.wrapper, wrapper);

  return observer;
}
export function assertFreshCapture(entries: string[]) {
  assert.deepEqual(entries, [], "stale capture entries must never qualify");
}
export function hasOwnedExec(trace: string, pid: number, program: string): boolean {
  const rows = trace
    .split("\n")
    .flatMap((line) => {
      const prefix = line.match(/^\s*([0-9]+)\s+/u);
      if (!prefix) return [];
      const body = line.slice(prefix[0].length),
        open = "execve(";
      if (!body.startsWith(open)) return [];
      let image: unknown;
      try {
        image = JSON.parse(body.slice(open.length, body.indexOf('",', open.length) + 1));
      } catch {
        return [];
      }
      return typeof image === "string"
        ? [{ pid: Number(prefix[1]), image, succeeded: body.endsWith("= 0") }]
        : [];
    })
    .filter((row) => row.pid === pid);
  return rows.length === 1 && rows[0]!.image === program && rows[0]!.succeeded;
}
export function traceNegativeControls() {
  const good = '  12   execve("/native/go", ["--project"], 0x1 /* 4 vars */) = 0';
  assert.equal(hasOwnedExec(good, 12, "/native/go"), true);
  assert.equal(hasOwnedExec(good, 13, "/native/go"), false);
  assert.equal(hasOwnedExec(good, 12, "/wrapper"), false);
  assert.equal(hasOwnedExec(good.replace('"/native/go"', '"/wrapper"'), 12, "/native/go"), false);
  assert.equal(hasOwnedExec(good.replace("= 0", "= -1 ENOENT"), 12, "/native/go"), false);
  assert.equal(
    hasOwnedExec(good + "\n" + good, 12, "/native/go"),
    false,
    "reused/duplicate PID exec must be ambiguous",
  );
  assert.throws(() => assertFreshCapture(["process-12"]));
  assertFreshCapture([]);
}
