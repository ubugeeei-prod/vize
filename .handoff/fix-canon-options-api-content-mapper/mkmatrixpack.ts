// Build a pack spec + captures for the passing Options API matrix cases.
import * as fs from "node:fs";
import * as path from "node:path";
const S = path.dirname(new URL(import.meta.url).pathname);
const F = "/Users/nishimura/Source/github.com/ubugeeei-prod/vize-wt-options-api/tests/_fixtures/differential/typechecker";
const pack = "options-api-this-bridge-1366";
const ids = process.argv.slice(2);
const results = JSON.parse(fs.readFileSync(path.join(S, "results-fix4.json"), "utf8"));
const existing = JSON.parse(fs.readFileSync(path.join(F, pack, "cases.json"), "utf8"));
const reg = JSON.parse(fs.readFileSync(path.join(F, "fix-history-packs.json"), "utf8"));
const [, module, test] = reg[pack].test.split("::");
const sources: Record<string, string> = {};
const cases = existing.cases.map((c: any) => ({ id: c.id, inputs: c.inputs }));
const capDir = path.join(S, "pins/cap", pack);
fs.mkdirSync(capDir, { recursive: true });
const DEFAULT_TSCONFIG = fs.readFileSync(path.join(S, "run/check-loc/dc-rich-clean/tsconfig.json"), "utf8");
for (const id of ids) {
  const dir = path.join(S, "matrix", id);
  const inputs: { file: string; source: string }[] = [];
  const walk = (d: string, rel: string) => {
    for (const e of fs.readdirSync(d, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
      const f = path.join(d, e.name), r = rel ? `${rel}/${e.name}` : e.name;
      if (e.isDirectory()) walk(f, r);
      else { const source = `matrix/${id}/${r}.txt`; sources[source] = fs.readFileSync(f, "utf8"); inputs.push({ file: r, source }); }
    }
  };
  walk(dir, "");
  if (!inputs.some((i) => i.file === "tsconfig.json")) {
    const source = `matrix/${id}/tsconfig.json.txt`; sources[source] = DEFAULT_TSCONFIG;
    inputs.unshift({ file: "tsconfig.json", source });
  }
  cases.push({ id: `matrix-${id}`, inputs });
  // Captured from the content-mapper route (same keys as vize check here).
  const out = fs.readFileSync(path.join(S, "cmraw", `${id}.txt`), "utf8").split("\n");
  const diags: any[] = [];
  for (const line of out) {
    const m = /^(.+?)\((\d+),(\d+)\): error TS(\d+): (.*)$/.exec(line);
    if (m) diags.push({ file: m[1], line: +m[2]!, column: +m[3]!, severity: 1, code: +m[4]!, message: m[5] });
    else if (/^\s+\S/.test(line) && diags.length) diags[diags.length - 1].message += `\n${line.trim()}`;
  }
  diags.sort((a, b) => a.file.localeCompare(b.file) || a.line - b.line || a.column - b.column);
  fs.writeFileSync(path.join(capDir, `matrix-${id}.json`), JSON.stringify(diags));
}
fs.writeFileSync(path.join(S, "pins/spec-1366.json"), JSON.stringify({
  pack, module, test, regressionCommit: existing.regressionCommit, historicalIssue: existing.historicalIssue,
  checkerOptions: existing.checkerOptions, projectOptions: existing.projectOptions, sources, cases,
}, null, 2));
console.log(cases.length, "cases");
