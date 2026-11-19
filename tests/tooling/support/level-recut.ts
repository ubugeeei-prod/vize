import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

/** One early re-cut phase: its files and the gate lines it must keep verbatim. */
export interface RecutPhase {
  phase: number;
  taskFiles: readonly string[];
  predecessorFiles: readonly string[];
  provisionalGateLines: readonly string[];
  strictDisjointPaths?: boolean;
}

export interface Contract {
  id: string;
  file: string;
  slug: string;
  body: string;
  lane: string;
  gate: string;
  deps: string[];
}

export const plan = new URL("../../../docs/davinci/plan/", import.meta.url);

export function readPlan(relative: string): string {
  return fs.readFileSync(new URL(relative, plan), "utf8");
}

export function slug(heading: string): string {
  return heading
    .toLowerCase()
    .replace(/[^\p{L}\p{N} _-]/gu, "")
    .replace(/ /gu, "-");
}

export function sectionBetween(source: string, start: RegExp, end: RegExp, label: string): string {
  const startMatch = start.exec(source);
  assert.ok(startMatch, `missing ${label}`);
  const tail = source.slice(startMatch.index + startMatch[0].length);
  const endMatch = end.exec(tail);
  assert.ok(endMatch, `missing end of ${label}`);
  return tail.slice(0, endMatch.index);
}

function todoIndex(source: string, label: string): string {
  return sectionBetween(source, /^## TODO index$/mu, /^---$/mu, `${label} TODO index`);
}

/** Every task of the earlier phases, linked (re-cut) or plain (provisional) index shape. */
export function predecessorTasks(config: RecutPhase): Map<string, boolean> {
  const tasks = new Map<string, boolean>();
  for (const file of config.predecessorFiles) {
    const index = todoIndex(readPlan(file), file);
    for (const { groups } of index.matchAll(/^- \[(?<c>[ x])\] \[?(?<id>P\d+-\d+[a-z]?)\b/gmu)) {
      tasks.set(groups!.id, groups!.c === "x");
    }
  }
  return tasks;
}

function field(body: string, name: string, id: string): string {
  const match = new RegExp(`^\\*\\*${name}:\\*\\* (?<value>.+)$`, "mu").exec(body);
  assert.ok(match, `${id} is missing **${name}:**`);
  return match.groups!.value;
}

function phaseOf(id: string): number {
  return Number(/^P(?<n>\d+)-/u.exec(id)!.groups!.n);
}

export function contracts(config: RecutPhase): Contract[] {
  const parsed: Contract[] = [];
  const heading = new RegExp(`^## (?<heading>(?<id>P${config.phase}-\\d+[a-z]?) — .+)$`, "gmu");
  for (const file of config.taskFiles) {
    const source = readPlan(file);
    const headings = [...source.matchAll(heading)];
    for (const [position, match] of headings.entries()) {
      const body = source.slice(match.index, headings[position + 1]?.index ?? source.length);
      const id = match.groups!.id;
      for (const name of ["Deliverable", "Acceptance", "Non-goals"]) field(body, name, id);
      assert.match(body, /^\*\*Steps:\*\*\n\n- \[[ x]\] /mu, `${id} needs a Steps checklist`);
      const lane = field(body, "Lane", id);
      const gate = field(body, "Start gate", id);
      parsed.push({ id, file, slug: slug(match.groups!.heading), body, lane, gate, deps: [] });
    }
  }
  const ids = parsed.map((contract) => contract.id);
  assert.equal(new Set(ids).size, ids.length, `duplicate P${config.phase} contract id`);
  for (const contract of parsed) {
    const raw = field(contract.body, "Deps", contract.id);
    const named = (raw.match(/P\d+-\d+[a-z]?/gu) ?? []).filter((id) => phaseOf(id) <= config.phase);
    if (raw.startsWith(`every other phase-${config.phase} task`)) {
      contract.deps = [...ids.filter((id) => id !== contract.id), ...named];
    } else {
      if (named.length === 0) assert.match(raw, /^none\b/u, `${contract.id} deps must name tasks`);
      contract.deps = named;
    }
  }
  return parsed;
}

function gateClass(gate: string, phase: number): string {
  const gated = /^gated on (?<task>P(?<n>\d+)-\d+[a-z]?)\b/u.exec(gate);
  if (gated) {
    assert.ok(Number(gated.groups!.n) < phase, `a gate must name an earlier phase: ${gate}`);
    return `gated on ${gated.groups!.task}`;
  }
  assert.match(gate, /^startable now\b/u, `unrecognized start gate: ${gate}`);
  return "startable now";
}

function expandTasks(cell: string): string[] {
  return cell.split(",").flatMap((part) => {
    const range = /(?<n>P\d+-\d+)(?<from>[a-z])…P\d+-\d+(?<to>[a-z])/u.exec(part);
    if (range == null) return part.match(/P\d+-\d+[a-z]?/gu) ?? [];
    const { n, from, to } = range.groups!;
    const codes: number[] = [];
    for (let code = from.charCodeAt(0); code <= to.charCodeAt(0); code += 1) codes.push(code);
    return codes.map((code) => `${n}${String.fromCharCode(code)}`);
  });
}

/** Registers the structural checks every early re-cut phase must pass. */
export function registerRecutChecks(config: RecutPhase): void {
  const n = config.phase;
  const phaseFile = `phase-${n}.md`;
  const phase = readPlan(phaseFile);
  const label = `Phase ${n}`;

  test(`${label}: the TODO index links every task to exactly one full contract`, () => {
    const entry = new RegExp(
      `^- \\[(?<checked>[ x])\\] \\[(?<id>P${n}-\\d+[a-z]?)\\]\\(\\./(?<file>phase-${n}-tasks(?:-later|-last)?\\.md)#(?<anchor>[^)]+)\\) .+? — lane (?<lane>[A-Z]) · (?<gate>.+)$`,
      "gmu",
    );
    const index = todoIndex(phase, label);
    const entries = [...index.matchAll(entry)].map(({ groups }) => groups!);
    assert.equal(
      entries.length,
      [...index.matchAll(/^- \[[ x]\] /gmu)].length,
      "index entry shape",
    );
    const byId = new Map(contracts(config).map((contract) => [contract.id, contract]));
    const ids = entries.map((groups) => groups.id);
    assert.equal(new Set(ids).size, ids.length, "duplicate index id");
    assert.deepEqual(new Set(ids), new Set(byId.keys()), "index and contracts must be a bijection");
    for (const groups of entries) {
      const contract = byId.get(groups.id)!;
      assert.equal(groups.file, contract.file, `${groups.id} links the wrong contract file`);
      assert.equal(groups.anchor, contract.slug, `${groups.id} links a missing anchor`);
      assert.equal(groups.lane, contract.lane, `${groups.id} lane drifted`);
      assert.equal(
        gateClass(groups.gate, n),
        gateClass(contract.gate, n),
        `${groups.id} gate drifted`,
      );
      const record = `phase-${n}-records/${groups.id.toLowerCase()}.md`;
      if (groups.checked === "x") {
        assert.match(contract.body, /^\*\*Landed /mu, `${groups.id} is checked but not landed`);
        assert.ok(contract.body.includes(record), `${groups.id} must link its record`);
        assert.ok(fs.existsSync(new URL(record, plan)), `${groups.id} record file is missing`);
      } else {
        assert.doesNotMatch(contract.body, /^\*\*Landed /mu, `${groups.id} claims landed status`);
      }
    }
  });

  test(`${label}: dependencies name real tasks and the graph is acyclic`, () => {
    const all = contracts(config);
    const earlier = predecessorTasks(config);
    const ids = new Set(all.map((contract) => contract.id));
    const graph = new Map(all.map((contract) => [contract.id, contract.deps]));
    for (const { id, deps } of all) {
      assert.ok(!deps.includes(id), `${id} depends on itself`);
      for (const dep of deps) assert.ok(ids.has(dep) || earlier.has(dep), `${id} → unknown ${dep}`);
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

  test(`${label}: start gates are honest about earlier phases`, () => {
    const all = contracts(config);
    const earlier = predecessorTasks(config);
    let startable = 0;
    for (const { id, gate, deps } of all) {
      const open = deps.filter((dep) => earlier.get(dep) === false);
      const cls = gateClass(gate, n);
      if (cls === "startable now") {
        startable += 1;
        assert.deepEqual(open, [], `${id} is startable now but depends on open ${open.join(", ")}`);
      } else {
        const task = cls.replace("gated on ", "");
        assert.ok(earlier.has(task), `${id} is gated on unknown ${task}`);
        assert.ok(deps.includes(task), `${id} is gated on ${task} but does not depend on it`);
      }
    }
    const claim = /\*\*Startable now[^*]*?, (?<n>\d+) of (?<total>\d+) tasks:\*\*/u.exec(phase);
    assert.ok(claim, "missing startable-now count");
    assert.equal(Number(claim.groups!.n), startable, "stale startable-now count");
    assert.equal(Number(claim.groups!.total), all.length, "stale task total");
    assert.ok(phase.includes(`for all ${all.length} tasks`), "stale contract total");
  });

  test(`${label}: lanes match the contracts and own disjoint paths`, () => {
    const all = contracts(config);
    const start = phase.indexOf("| Lane | Tasks");
    assert.notEqual(start, -1, "missing lane table");
    const table = phase.slice(start).split("\n\n", 1)[0];
    const rows = [
      ...table.matchAll(/^\| (?<lane>[A-Z])\s+\| (?<tasks>[^|]+)\| (?<paths>.+) \|$/gmu),
    ];
    assert.ok(rows.length > 0, "lane table must not be empty");
    const owned: Array<{ lane: string; path: string; cell: string }> = [];
    for (const { groups } of rows) {
      const lane = groups!.lane;
      const declared = all.filter((contract) => contract.lane === lane).map(({ id }) => id);
      assert.deepEqual(
        new Set(expandTasks(groups!.tasks)),
        new Set(declared),
        `lane ${lane} tasks`,
      );
      for (const match of groups!.paths.matchAll(/`(?<path>[^`]+)`/gu)) {
        owned.push({ lane, path: match.groups!.path.replace(/\*$/u, ""), cell: groups!.paths });
      }
    }
    const lanes = new Set(rows.map(({ groups }) => groups!.lane));
    for (const contract of all) assert.ok(lanes.has(contract.lane), `${contract.id} lane unknown`);
    for (const outer of owned) {
      for (const inner of owned) {
        if (outer.lane === inner.lane || !inner.path.startsWith(outer.path)) continue;
        assert.ok(
          !config.strictDisjointPaths && outer.cell.includes(`minus lane ${inner.lane}`),
          `lane ${outer.lane} path ${outer.path} overlaps lane ${inner.lane} ${inner.path}`,
        );
      }
    }
  });

  test(`${label}: the critical path respects the dependency order`, () => {
    const byId = new Map(contracts(config).map((contract) => [contract.id, contract]));
    const section = sectionBetween(phase, /^## Critical path$/mu, /^## /mu, "critical path");
    const chain = [...section.matchAll(/^\d+\. (?<line>.+)$/gmu)]
      .flatMap(({ groups }) => groups!.line.match(/P\d+-\d+[a-z]?/gu) ?? [])
      .filter((id) => phaseOf(id) === n);
    assert.ok(chain.length > 0, "critical path must name tasks");
    for (const [position, id] of chain.entries()) {
      assert.ok(byId.has(id), `critical path names unknown ${id}`);
      for (const dep of byId.get(id)!.deps) {
        if (chain.includes(dep)) assert.ok(chain.indexOf(dep) < position, `${id} precedes ${dep}`);
      }
    }
    for (const id of section.match(new RegExp(`\\*\\*P${n}-\\d+[a-z]?\\*\\*`, "gu")) ?? []) {
      assert.ok(byId.has(id.replaceAll("*", "")), `demo flag names unknown ${id}`);
    }
  });

  test(`${label}: the exit gate keeps the provisional lines verbatim and suites are registered`, () => {
    const gate = phase.slice(phase.indexOf("## Exit gate"));
    for (const line of config.provisionalGateLines) {
      assert.ok(gate.includes(line), `gate line lost: ${line}`);
    }
    const suites = readPlan("test-suites.md");
    const registered = new Set([...suites.matchAll(/^\| (TS-\d+) \|/gmu)].map((match) => match[1]));
    const sources = [phase, ...config.taskFiles.map(readPlan)].join("\n");
    for (const suite of new Set(sources.match(/TS-\d+/gu) ?? [])) {
      assert.ok(registered.has(suite), `${suite} is referenced but not registered`);
    }
  });

  test(`${label}: local links resolve`, () => {
    for (const name of [phaseFile, ...config.taskFiles]) {
      for (const match of readPlan(name).matchAll(/\]\((?<target>[^)]+)\)/gu)) {
        const target = match.groups!.target.split("#", 1)[0];
        if (target === "" || /^[a-z]+:/u.test(target)) continue;
        const resolved = fileURLToPath(new URL(target, new URL(name, plan)));
        assert.ok(fs.existsSync(resolved), `${name} has a missing link: ${match.groups!.target}`);
      }
    }
  });
}
