import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const plan = new URL("../../davinci-road/plan/", import.meta.url);
const taskFiles = ["phase-4-tasks.md", "phase-4-tasks-later.md", "phase-4-tasks-last.md"] as const;

function read(relative: string): string {
  return fs.readFileSync(new URL(relative, plan), "utf8");
}

const phase = read("phase-4.md");
const phase3 = read("phase-3.md");
const suites = read("test-suites.md");
const questions = read("../open-questions.md");
const contracts = Object.fromEntries(taskFiles.map((file) => [file, read(file)])) as Record<
  (typeof taskFiles)[number],
  string
>;

const provisionalGateLines = [
  "- [ ] TS-40 check parity; TS-39 lint agreement; TS-5 + TS-41 glyph gates",
  "- [ ] Consumption matrix: every computed group ≥1 consumer or gated (TS-12)",
  "- [ ] TS-36 witnesses verify; TS-37 100% recall per class; TS-38 zero untriaged candidates",
  "- [ ] canon/maestro projection duplicates + glyph byte scanner + musea hand parser: deleted",
];

interface IndexEntry {
  id: string;
  checked: boolean;
  file: string;
  anchor: string;
  lane: string;
  gate: string;
}

interface Contract {
  id: string;
  file: string;
  slug: string;
  body: string;
  lane: string;
  gate: string;
  deps: string[];
}

function slug(heading: string): string {
  return heading
    .toLowerCase()
    .replace(/[^\p{L}\p{N} _-]/gu, "")
    .replace(/ /gu, "-");
}

function sectionBetween(source: string, start: RegExp, end: RegExp, label: string): string {
  const startMatch = start.exec(source);
  assert.ok(startMatch, `missing ${label}`);
  const tail = source.slice(startMatch.index + startMatch[0].length);
  const endMatch = end.exec(tail);
  assert.ok(endMatch, `missing end of ${label}`);
  return tail.slice(0, endMatch.index);
}

function indexEntries(): IndexEntry[] {
  const index = sectionBetween(phase, /^## TODO index$/mu, /^---$/mu, "Phase 4 TODO index");
  const entries = [
    ...index.matchAll(
      /^- \[(?<checked>[ x])\] \[(?<id>P4-\d+[a-z]?)\]\(\.\/(?<file>phase-4-tasks(?:-later|-last)?\.md)#(?<anchor>[^)]+)\) .+? — lane (?<lane>[A-Z]) · (?<gate>.+)$/gmu,
    ),
  ].map(({ groups }) => ({
    id: groups!.id,
    checked: groups!.checked === "x",
    file: groups!.file,
    anchor: groups!.anchor,
    lane: groups!.lane,
    gate: groups!.gate,
  }));
  const bulletCount = [...index.matchAll(/^- \[[ x]\] /gmu)].length;
  assert.equal(entries.length, bulletCount, "every TODO bullet must match the index entry shape");
  return entries;
}

function phase3Tasks(): Map<string, boolean> {
  const index = sectionBetween(phase3, /^## TODO index$/mu, /^---$/mu, "Phase 3 TODO index");
  return new Map(
    [...index.matchAll(/^- \[(?<checked>[ x])\] (?<id>P3-\d+)\b/gmu)].map(({ groups }) => [
      groups!.id,
      groups!.checked === "x",
    ]),
  );
}

function field(body: string, name: string, id: string): string {
  const match = new RegExp(`^\\*\\*${name}:\\*\\* (?<value>.+)$`, "mu").exec(body);
  assert.ok(match, `${id} is missing **${name}:**`);
  return match.groups!.value;
}

function allContracts(): Contract[] {
  const parsed: Contract[] = [];
  for (const file of taskFiles) {
    const source = contracts[file];
    const headings = [...source.matchAll(/^## (?<heading>(?<id>P4-\d+[a-z]?) — .+)$/gmu)];
    for (const [position, match] of headings.entries()) {
      const end = headings[position + 1]?.index ?? source.length;
      const body = source.slice(match.index, end);
      const id = match.groups!.id;
      for (const name of ["Deliverable", "Acceptance", "Non-goals"]) field(body, name, id);
      assert.match(body, /^\*\*Steps:\*\*\n\n- \[[ x]\] /mu, `${id} needs a Steps checklist`);
      parsed.push({
        id,
        file,
        slug: slug(match.groups!.heading),
        body,
        lane: field(body, "Lane", id),
        gate: field(body, "Start gate", id),
        deps: [],
      });
    }
  }
  const ids = parsed.map((contract) => contract.id);
  assert.equal(new Set(ids).size, ids.length, "duplicate P4 contract id");
  for (const contract of parsed) contract.deps = dependencies(contract, ids);
  return parsed;
}

function dependencies(contract: Contract, ids: string[]): string[] {
  const raw = field(contract.body, "Deps", contract.id);
  const named = raw.match(/P[34]-\d+[a-z]?/gu) ?? [];
  if (raw.startsWith("every other phase-4 task")) {
    return [...ids.filter((id) => id !== contract.id), ...named];
  }
  if (named.length === 0) assert.match(raw, /^none\b/u, `${contract.id} deps must name tasks`);
  return named;
}

function gateClass(gate: string): string {
  const gated = /^gated on (?<task>P3-\d+)\b/u.exec(gate);
  if (gated) return `gated on ${gated.groups!.task}`;
  assert.match(gate, /^startable now\b/u, `unrecognized start gate: ${gate}`);
  return "startable now";
}

function expandTasks(cell: string): string[] {
  return cell.split(",").flatMap((part) => {
    const range = /(?<n>P4-\d+)(?<from>[a-z])…P4-\d+(?<to>[a-z])/u.exec(part);
    if (range == null) return part.match(/P4-\d+[a-z]?/gu) ?? [];
    const { n, from, to } = range.groups!;
    const codes: number[] = [];
    for (let code = from.charCodeAt(0); code <= to.charCodeAt(0); code += 1) codes.push(code);
    return codes.map((code) => `${n}${String.fromCharCode(code)}`);
  });
}

test("the TODO index links every task to exactly one full contract", () => {
  const entries = indexEntries();
  const byId = new Map(allContracts().map((contract) => [contract.id, contract]));
  assert.equal(
    new Set(entries.map((entry) => entry.id)).size,
    entries.length,
    "duplicate index id",
  );
  assert.deepEqual(new Set(entries.map((entry) => entry.id)), new Set(byId.keys()));
  for (const entry of entries) {
    const contract = byId.get(entry.id)!;
    assert.equal(entry.file, contract.file, `${entry.id} links the wrong contract file`);
    assert.equal(entry.anchor, contract.slug, `${entry.id} links a missing anchor`);
    assert.equal(entry.lane, contract.lane, `${entry.id} lane drifted between index and contract`);
    assert.equal(gateClass(entry.gate), gateClass(contract.gate), `${entry.id} start gate drifted`);
    if (entry.checked) {
      const record = `phase-4-records/${entry.id.toLowerCase()}.md`;
      assert.match(contract.body, /^\*\*Landed /mu, `${entry.id} is checked but not landed`);
      assert.ok(contract.body.includes(record), `${entry.id} must link its record`);
      assert.ok(fs.existsSync(new URL(record, plan)), `${entry.id} record file is missing`);
    } else {
      assert.doesNotMatch(contract.body, /^\*\*Landed /mu, `${entry.id} claims landed status`);
    }
  }
});

test("dependencies name real tasks and the graph is acyclic", () => {
  const all = allContracts();
  const p3 = phase3Tasks();
  const ids = new Set(all.map((contract) => contract.id));
  const graph = new Map(all.map((contract) => [contract.id, contract.deps]));
  for (const { id, deps } of all) {
    assert.ok(!deps.includes(id), `${id} depends on itself`);
    for (const dep of deps) {
      assert.ok(ids.has(dep) || p3.has(dep), `${id} depends on unknown task ${dep}`);
    }
  }
  const state = new Map<string, "visiting" | "done">();
  const visit = (id: string, trail: string[]): void => {
    if (state.get(id) === "done" || !graph.has(id)) return;
    assert.notEqual(state.get(id), "visiting", `dependency cycle: ${[...trail, id].join(" → ")}`);
    state.set(id, "visiting");
    for (const dep of graph.get(id)!) visit(dep, [...trail, id]);
    state.set(id, "done");
  };
  for (const id of graph.keys()) visit(id, []);
});

test("start gates are honest about phase 3", () => {
  const all = allContracts();
  const p3 = phase3Tasks();
  let startable = 0;
  for (const { id, gate, deps } of all) {
    const openP3 = deps.filter((dep) => p3.get(dep) === false);
    const cls = gateClass(gate);
    if (cls === "startable now") {
      startable += 1;
      assert.deepEqual(openP3, [], `${id} is marked startable now but depends on open ${openP3}`);
    } else {
      const task = cls.replace("gated on ", "");
      assert.ok(p3.has(task), `${id} is gated on unknown ${task}`);
      assert.ok(deps.includes(task), `${id} is gated on ${task} but does not depend on it`);
    }
  }
  const claim = /\*\*Startable now \(P3-independent\), (?<n>\d+) of (?<total>\d+) tasks:\*\*/u.exec(
    phase,
  );
  assert.ok(claim, "missing startable-now count");
  assert.equal(Number(claim.groups!.n), startable, "stale startable-now count");
  assert.equal(Number(claim.groups!.total), all.length, "stale task total");
  assert.ok(phase.includes(`for all ${all.length} tasks`), "stale contract total");
});

test("lanes match the contracts and own disjoint paths", () => {
  const all = allContracts();
  const table = sectionBetween(
    phase,
    /^\| Lane \| Tasks/mu,
    /^\*\*Projection consumers/mu,
    "lanes",
  );
  const rows = [...table.matchAll(/^\| (?<lane>[A-Z])\s+\| (?<tasks>[^|]+)\| (?<paths>.+) \|$/gmu)];
  assert.ok(rows.length > 0, "lane table must not be empty");
  const owned: Array<{ lane: string; path: string; cell: string }> = [];
  for (const { groups } of rows) {
    const lane = groups!.lane;
    const declared = all.filter((contract) => contract.lane === lane).map(({ id }) => id);
    assert.deepEqual(new Set(expandTasks(groups!.tasks)), new Set(declared), `lane ${lane} tasks`);
    for (const match of groups!.paths.matchAll(/`(?<path>[^`]+)`/gu)) {
      owned.push({ lane, path: match.groups!.path.replace(/\*$/u, ""), cell: groups!.paths });
    }
  }
  const lanes = new Set(rows.map(({ groups }) => groups!.lane));
  for (const contract of all) assert.ok(lanes.has(contract.lane), `${contract.id} lane unknown`);
  for (const outer of owned) {
    for (const inner of owned) {
      if (outer === inner || outer.lane === inner.lane || !inner.path.startsWith(outer.path)) {
        continue;
      }
      assert.ok(
        outer.cell.includes(`minus lane ${inner.lane}`),
        `lane ${outer.lane} path ${outer.path} overlaps lane ${inner.lane} ${inner.path}`,
      );
    }
  }
});

test("the critical path respects the dependency order", () => {
  const byId = new Map(allContracts().map((contract) => [contract.id, contract]));
  const section = sectionBetween(phase, /^## Critical path$/mu, /^## /mu, "critical path");
  const chain = [...section.matchAll(/^\d+\. (?<line>.+)$/gmu)].flatMap(
    ({ groups }) => groups!.line.match(/P4-\d+[a-z]?/gu) ?? [],
  );
  assert.ok(chain.length > 0, "critical path must name tasks");
  for (const [position, id] of chain.entries()) {
    assert.ok(byId.has(id), `critical path names unknown ${id}`);
    for (const dep of byId.get(id)!.deps) {
      if (!chain.includes(dep)) continue;
      assert.ok(chain.indexOf(dep) < position, `${id} precedes its dependency ${dep}`);
    }
  }
  for (const id of section.match(/\*\*(P4-\d+[a-z]?)\*\*/gu) ?? []) {
    assert.ok(byId.has(id.replaceAll("*", "")), `demo flag names unknown ${id}`);
  }
});

test("the exit gate keeps the provisional lines verbatim and suites are registered", () => {
  const gate = phase.slice(phase.indexOf("## Exit gate"));
  for (const line of provisionalGateLines)
    assert.ok(gate.includes(line), `gate line lost: ${line}`);
  const registered = new Set([...suites.matchAll(/^\| (TS-\d+) \|/gmu)].map((match) => match[1]));
  const sources = [phase, ...Object.values(contracts)].join("\n");
  for (const suite of new Set(sources.match(/TS-\d+/gu) ?? [])) {
    assert.ok(registered.has(suite), `${suite} is referenced by phase 4 but not registered`);
  }
  const p4Row = /^\| P4 +\| (?<suites>[^|]+)\|$/mu.exec(suites);
  assert.ok(p4Row?.groups?.suites.includes("TS-53"), "P4 mandatory suites must include TS-53");
});

test("the open questions phase 4 depends on carry a written recommendation", () => {
  for (const heading of ["Complexity metric definition", "App-level fact provider contract"]) {
    const section = sectionBetween(
      questions,
      new RegExp(`^## ${heading}$`, "mu"),
      /^## |$(?![\s\S])/mu,
      heading,
    );
    assert.match(section, /\*\*Recommendation \(phase-4 re-cut, 2026-09-21\)/u, heading);
    assert.match(section, /plan\/phase-4-tasks(?:-later|-last)?\.md#p4-/u, `${heading} owner link`);
  }
});

test("local links in the phase 4 plan resolve", () => {
  const docs = { "phase-4.md": phase, ...contracts };
  for (const [name, source] of Object.entries(docs)) {
    for (const match of source.matchAll(/\]\((?<target>[^)]+)\)/gu)) {
      const target = match.groups!.target.split("#", 1)[0];
      if (target === "" || /^[a-z]+:/u.test(target)) continue;
      const resolved = fileURLToPath(new URL(target, new URL(name, plan)));
      assert.ok(fs.existsSync(resolved), `${name} has a missing link: ${match.groups!.target}`);
    }
  }
});
