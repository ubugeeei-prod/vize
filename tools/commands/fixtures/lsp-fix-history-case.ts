// Register LSP fix-history response sessions (#6883) and ledger dispositions.
//
//   node tools/commands/fixtures/lsp-fix-history-case.ts print SPEC.json...
//   node tools/commands/fixtures/lsp-fix-history-case.ts register SPEC.json...
//
// A SPEC.json names one historical fix (a row of the LSP fix-history ledger)
// and exactly one of:
//   { commit, cases: [{ id, requirement, files, steps, initializationOptions?, ... }] }
//   { commit, existing: { cases: ["lsp/fix-history/...", ...], note } }
//   { commit, noResponse: { reason, coveredBy: ["path::test", ...] } }
// `cases` are replayed twice against the source-built `target/ci/vize lsp`;
// both captures must agree before case.json and expected.json are written and
// the manifest is re-pinned. `print` shows the captured records only.
// Build first: cargo build --locked -p vize --profile ci.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { isDeepStrictEqual } from "node:util";
import { fileURLToPath } from "node:url";
import { sha256 } from "../../../tests/differential/sha256.ts";
import {
  HISTORY_LEDGER,
  HISTORY_MANIFEST,
  HISTORY_ROOT,
  caseDirectory,
  runHistoryCase,
  validateHistoryCase,
  type HistoryCase,
  type HistoryManifest,
} from "../../../tests/differential/lsp-history.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const binary = path.join(root, "target/ci", process.platform === "win32" ? "vize.exe" : "vize");
const legacyManifest = "tests/_fixtures/differential/lsp/manifest.json";

// Spec file contents may be inline strings, line arrays, repository files or
// blobs of a revision; registration stores the resolved bytes in case.json.
type FileSpec = string | { lines: string[] } | { path: string } | { git: string };
type CaseSpec = Omit<HistoryCase, "version" | "commit" | "files"> & {
  files: Record<string, FileSpec>;
};
type Spec = {
  commit: string;
  cases?: CaseSpec[];
  existing?: { cases: string[]; note: string };
  noResponse?: { reason: string; coveredBy: string[] };
};

const json = (value: unknown) => `${JSON.stringify(value, null, 2)}\n`;
const read = (file: string) => fs.readFileSync(path.join(root, file), "utf8");
const clean = (text: string) => text.replace(/[\t\n]+/g, " ").trim();

function resolveFile(value: FileSpec): string {
  if (typeof value === "string") return value;
  if ("lines" in value) return value.lines.join("\n");
  if ("path" in value) return read(value.path);
  return execFileSync("git", ["show", value.git], { cwd: root, encoding: "utf8" });
}

function loadSpec(file: string): Spec {
  const spec = JSON.parse(fs.readFileSync(file, "utf8")) as Spec;
  for (const item of spec.cases ?? [])
    item.files = Object.fromEntries(
      Object.entries(item.files).map(([name, value]) => [name, resolveFile(value)]),
    );
  if (/^[0-9a-f]{7,39}$/.test(spec.commit)) {
    const matches = ledger().rows.filter((columns) => columns[0].startsWith(spec.commit));
    assert.equal(matches.length, 1, `${file}: ambiguous or unknown commit prefix`);
    spec.commit = matches[0][0];
  }
  assert.match(spec.commit, /^[0-9a-f]{40}$/, `${file}: commit`);
  execFileSync("git", ["cat-file", "-e", `${spec.commit}^{commit}`], { cwd: root });
  const kinds = [spec.cases, spec.existing, spec.noResponse].filter(Boolean).length;
  assert.equal(kinds, 1, `${file}: exactly one of cases, existing, noResponse`);
  for (const item of spec.cases ?? [])
    validateHistoryCase({ version: 1, commit: spec.commit, ...(item as HistoryCase) });
  if (spec.noResponse) {
    assert(spec.noResponse.reason.length > 0 && spec.noResponse.coveredBy.length > 0);
  }
  return spec;
}

function ledger() {
  const [header, ...lines] = read(HISTORY_LEDGER).split("\n").filter(Boolean);
  return { header, rows: lines.map((line) => line.split("\t")) };
}

function knownFixtureIds(manifest: HistoryManifest) {
  const legacy = JSON.parse(read(legacyManifest)) as { cases: { id: string }[] };
  return new Set([...legacy.cases.map((c) => c.id), ...manifest.cases.map((c) => c.id)]);
}

async function capture(data: HistoryCase) {
  const first = await runHistoryCase(data, binary, { capture: true });
  const second = await runHistoryCase(data, binary, { capture: true });
  assert(
    isDeepStrictEqual(first.records, second.records),
    `${data.id}: two captures disagree\n${json(first.records)}\n${json(second.records)}`,
  );
  return first.records;
}

async function main() {
  const [command, ...files] = process.argv.slice(2);
  assert(command === "print" || command === "register", "usage: print|register SPEC.json...");
  assert(fs.existsSync(binary), `build ${binary} first`);
  const specs = files.map(loadSpec);
  if (command === "print") {
    for (const spec of specs)
      for (const item of spec.cases ?? []) {
        const data = { version: 1 as const, commit: spec.commit, ...(item as HistoryCase) };
        const { records, stderr } = await runHistoryCase(data, binary, { capture: true });
        console.log(`== ${item.id}\n${json(records)}`);
        if (process.env.LSP_HISTORY_STDERR) console.log(stderr);
      }
    return;
  }
  const manifestPath = path.join(root, HISTORY_MANIFEST);
  const manifest: HistoryManifest = fs.existsSync(manifestPath)
    ? JSON.parse(fs.readFileSync(manifestPath, "utf8"))
    : { schema: "vize.lsp.fix-history.sessions", version: 1, cases: [] };
  const table = ledger();
  for (const spec of specs) {
    const row = table.rows.find((columns) => columns[0] === spec.commit);
    assert(row, `${spec.commit} is not an LSP ledger row`);
    if (spec.noResponse) {
      row[2] = "no-response-behavior";
      row[3] = "-";
      row[4] = clean(
        `${spec.noResponse.reason} Covered by: ${spec.noResponse.coveredBy.join(", ")}.`,
      );
      continue;
    }
    const ids: string[] = [];
    for (const item of spec.cases ?? []) {
      const data: HistoryCase = { version: 1, commit: spec.commit, ...(item as HistoryCase) };
      const previous = manifest.cases.find((entry) => entry.id === data.id);
      assert(!previous || previous.commit === spec.commit, `${data.id} belongs to another fix`);
      const records = await capture(data);
      const directory = path.join(root, HISTORY_ROOT, caseDirectory(data.id));
      fs.mkdirSync(directory, { recursive: true });
      const caseBytes = json(data);
      const expectedBytes = json(records);
      fs.writeFileSync(path.join(directory, "case.json"), caseBytes);
      fs.writeFileSync(path.join(directory, "expected.json"), expectedBytes);
      const entry = {
        id: data.id,
        commit: spec.commit,
        case: `${caseDirectory(data.id)}/case.json`,
        sha256: sha256(caseBytes),
        expectedSha256: sha256(expectedBytes),
      };
      if (previous) manifest.cases[manifest.cases.indexOf(previous)] = entry;
      else manifest.cases.push(entry);
      ids.push(data.id);
      console.log(`captured ${data.id}: ${records.length} records`);
    }
    manifest.cases.sort((a, b) => (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));
    if (spec.existing) {
      const known = knownFixtureIds(manifest);
      for (const id of spec.existing.cases) assert(known.has(id), `unknown fixture ${id}`);
      ids.push(...spec.existing.cases);
    }
    const previousIds =
      row[3] === "-" || row[2] !== "response-fixture-covered" ? [] : row[3].split(",");
    row[2] = "response-fixture-covered";
    row[3] = [...new Set([...previousIds, ...ids])].join(",");
    row[4] = spec.existing ? clean(spec.existing.note) : "-";
  }
  fs.mkdirSync(path.dirname(manifestPath), { recursive: true });
  fs.writeFileSync(manifestPath, json(manifest));
  fs.writeFileSync(
    path.join(root, HISTORY_LEDGER),
    `${[table.header, ...table.rows.map((columns) => columns.join("\t"))].join("\n")}\n`,
  );
  const counts: Record<string, number> = {};
  for (const columns of table.rows) counts[columns[2]] = (counts[columns[2]] ?? 0) + 1;
  console.log(json({ sessions: manifest.cases.length, ledger: counts }));
}

await main();
