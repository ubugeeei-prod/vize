// Davinci P4-12c — the corpus half of the pug compile oracle. The committed
// baseline pins, for every inline `<template lang="pug">` SFC of the
// registered corpus at its pinned revision, the sha256 of what the pinned
// `pug@3.0.4` renders for the content `@vue/compiler-sfc` extracts. The Rust
// corpus lane (`crates/vize_s1_to_s2/tests/davinci_pug_corpus.rs`) holds
// vize's derived template to those hashes and compiles each SFC against its
// derived-HTML twin in the DOM, SSR and Vapor lanes, so together they are
// the compile oracle over the corpus. Scope proof: for every hydrated
// project the baseline rows must equal the recomputed rows exactly (a
// missing, extra or drifted pug SFC fails); rows of projects that are not
// hydrated here are carried, and the run reports which projects it proved.
// `VIZE_PUG_ORACLE_WRITE=1` rewrites the hydrated projects' rows;
// `VIZE_PUG_CORPUS_ROOT=<dir>` reads projects from `<dir>/<fixture dir name>`.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { loadGlyphCorpusProjects } from "../../tools/support/compat/fixtures/glyph-corpus.mjs";
import { parseSfc, pug, pugOptions, sha256 } from "./support/pug/oracle-runtime.ts";
import { findStep, readRealProjectMatrixWorkflow } from "./support/real-project-matrix-workflow.ts";

type Project = { id: string; fixturePath: string; fixtureDir: string; revision: string };
type Row = { project: string; revision: string; file: string; sha256: string };

const baselineUrl = new URL("../_fixtures/davinci-pug/corpus-baseline.tsv", import.meta.url);
const header = [
  "# Davinci P4-12c pug corpus baseline: sha256 of pug@3.0.4 (doctype html, pretty false)",
  '# rendering of each inline lang="pug" template, as @vue/compiler-sfc extracts it.',
  "# Regenerate: VIZE_PUG_ORACLE_WRITE=1 node --test tests/tooling/davinci-pug-corpus-oracle.test.ts",
  "# project\trevision\tfile\tsha256",
];

function readBaseline(): Row[] {
  return fs
    .readFileSync(baselineUrl, "utf8")
    .split("\n")
    .filter((line) => line !== "" && !line.startsWith("#"))
    .map((line) => {
      const [project, revision, file, digest] = line.split("\t");
      return { project, revision, file, sha256: digest };
    });
}

function vueFiles(dir: string, root = dir): string[] {
  const out: string[] = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name !== "node_modules") out.push(...vueFiles(full, root));
    } else if (entry.isFile() && entry.name.endsWith(".vue")) {
      out.push(path.relative(root, full).split(path.sep).join("/"));
    }
  }
  return out;
}

function projectDir(project: Project): string {
  const override = process.env.VIZE_PUG_CORPUS_ROOT;
  return override ? path.join(override, path.basename(project.fixturePath)) : project.fixtureDir;
}

function hydrated(project: Project): boolean {
  const dir = projectDir(project);
  return fs.existsSync(dir) && fs.readdirSync(dir).length > 0;
}

/** The pinned pug's rendering digest for every inline pug SFC of a project. */
function computeRows(project: Project): Row[] {
  const dir = projectDir(project);
  const rows: Row[] = [];
  for (const file of vueFiles(dir).sort()) {
    const source = fs.readFileSync(path.join(dir, file), "utf8");
    let template;
    try {
      template = parseSfc(source, { filename: file, sourceMap: false }).descriptor.template;
    } catch {
      continue;
    }
    if (template == null || template.lang !== "pug" || "src" in (template.attrs ?? {})) continue;
    const html = pug.render(template.content, { ...pugOptions, filename: file });
    rows.push({ project: project.id, revision: project.revision, file, sha256: sha256(html) });
  }
  return rows;
}

function serialize(rows: Row[]): string {
  const lines = rows.map((row) => [row.project, row.revision, row.file, row.sha256].join("\t"));
  return [...header, ...lines].join("\n") + "\n";
}

test("pug corpus baseline: every hydrated project's pug SFCs match the pinned pug exactly", () => {
  const projects = (loadGlyphCorpusProjects() as Project[]).sort((a, b) =>
    a.id < b.id ? -1 : a.id > b.id ? 1 : 0,
  );
  const baseline = readBaseline();
  const known = new Map(projects.map((project) => [project.id, project]));
  for (const row of baseline) {
    const project = known.get(row.project);
    if (project == null) continue; // outside this shard's selection
    assert.equal(row.revision, project.revision, `${row.project}: baseline revision is stale`);
  }
  const proved: string[] = [];
  const next: Row[] = [];
  for (const project of projects) {
    const committed = baseline.filter((row) => row.project === project.id);
    if (!hydrated(project)) {
      next.push(...committed);
      continue;
    }
    const rows = computeRows(project);
    proved.push(`${project.id}:${rows.length}`);
    next.push(...rows);
    if (process.env.VIZE_PUG_ORACLE_WRITE !== "1") {
      assert.deepEqual(committed, rows, `${project.id}: pug corpus baseline drifted`);
    }
  }
  if (process.env.VIZE_PUG_ORACLE_WRITE === "1") {
    const others = baseline.filter((row) => !known.has(row.project));
    fs.writeFileSync(baselineUrl, serialize([...others, ...next]));
  }
  // The baseline is sorted, unique, and every digest is a sha256.
  const keys = readBaseline().map((row) => `${row.project}\t${row.file}`);
  assert.deepEqual(keys, [...new Set(keys)].sort(), "baseline rows are sorted and unique");
  for (const row of readBaseline()) assert.match(row.sha256, /^[0-9a-f]{64}$/u, row.file);
  console.log(
    `pug corpus oracle proved ${proved.length} hydrated project(s): ${proved.join(", ")}`,
  );
});

test("the real-project matrix runs the pug corpus compile oracle on the hydrated corpus", () => {
  // The Rust lane over the hydrated corpus shares the SSR corpus step, before
  // the finalize step dehydrates it, and fails the job on any divergence.
  const steps = readRealProjectMatrixWorkflow().jobs?.["davinci-dom-corpus"]?.steps ?? [];
  const lane = findStep(steps, "Run S4 SSR and pug S1 differential corpora");
  assert.equal(lane["continue-on-error"], undefined);
  const corpus = "VIZE_DAVINCI_DIFFERENTIAL_CORPUS=tests/_fixtures/_git cargo test";
  assert.equal(
    lane.run,
    `${corpus} -p vize_atelier_ssr --features davinci-differential --test davinci_ssr_corpus -- --nocapture && ` +
      `${corpus} -p vize_s1_to_s2 --features davinci-differential --test davinci_pug_corpus -- --nocapture`,
  );
  assert.ok(
    steps.indexOf(lane) < steps.indexOf(findStep(steps, "Finalize S2 DOM corpus evidence")),
    "the pug corpus oracle must run before the corpus is dehydrated",
  );
});
