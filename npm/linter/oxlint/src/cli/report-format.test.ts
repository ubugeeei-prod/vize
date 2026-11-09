import assert from "node:assert/strict";
import { test } from "node:test";

import {
  renderGithub,
  renderConvertedReportOrPassThrough,
  renderGitlab,
  requestedConvertedFormat,
  withJsonFormat,
} from "./report-format.ts";

const report = JSON.stringify({
  diagnostics: [
    {
      message: "Elements in iteration expect to have 'v-bind:key' directives.\nDetails: x",
      code: "vize(vue/require-v-for-key)",
      severity: "error",
      filename: "src/App.vue",
      labels: [{ span: { offset: 10, length: 4, line: 3, column: 5 } }],
    },
    {
      message: "Unexpected `any`",
      code: "typescript(no-explicit-any)",
      severity: "warning",
      filename: "src/a,b.vue",
      labels: [{ span: { offset: 1, length: 1, line: 1, column: 2 } }],
    },
  ],
  number_of_files: 2,
});

void test("detects only formats the pipeline cannot merge natively", () => {
  assert.equal(requestedConvertedFormat(["-f", "gitlab", "src"]), "gitlab");
  assert.equal(requestedConvertedFormat(["--format=github"]), "github");
  assert.equal(requestedConvertedFormat(["-f", "json"]), undefined);
  assert.equal(requestedConvertedFormat(["src", "--", "-f", "gitlab"]), undefined);
});

void test("replaces the format with json in either spelling", () => {
  assert.deepEqual(withJsonFormat(["-c", "a.json", "-f", "gitlab", "src"]), [
    "-c",
    "a.json",
    "-f",
    "json",
    "src",
  ]);
  assert.deepEqual(withJsonFormat(["--format=github", "src"]), ["--format=json", "src"]);
});

void test("renders a GitLab code quality report", () => {
  const entries = JSON.parse(renderGitlab(report)) as Array<Record<string, any>>;
  assert.equal(entries.length, 2);
  assert.equal(entries[0].severity, "critical");
  assert.equal(entries[1].severity, "major");
  assert.equal(
    entries[0].description,
    "Elements in iteration expect to have 'v-bind:key' directives.",
  );
  assert.deepEqual(entries[0].location, { path: "src/App.vue", lines: { begin: 3, end: 3 } });
  assert.match(entries[0].fingerprint, /^[0-9a-f]{16}$/u);
  assert.notEqual(entries[0].fingerprint, entries[1].fingerprint);
  assert.equal(entries[0].fingerprint, JSON.parse(renderGitlab(report))[0].fingerprint);
});

void test("renders GitHub workflow commands and escapes properties", () => {
  const lines = renderGithub(report).trimEnd().split("\n");
  assert.equal(
    lines[0],
    "::error file=src/App.vue,line=3,col=5,title=vize(vue/require-v-for-key)::src/App.vue:3:5: Elements in iteration expect to have 'v-bind:key' directives.",
  );
  assert.match(lines[1], /^::warning file=src\/a%2Cb\.vue,line=1,col=2,/u);
});

void test("an empty report renders an empty GitLab array and no GitHub lines", () => {
  const empty = JSON.stringify({ diagnostics: [] });
  assert.equal(renderGitlab(empty).trim(), "[]");
  assert.equal(renderGithub(empty), "");
});

void test("non-JSON output passes through unchanged", () => {
  assert.equal(
    renderConvertedReportOrPassThrough("No files found to lint.\n", "gitlab"),
    "No files found to lint.\n",
  );
  assert.equal(renderConvertedReportOrPassThrough("{}", "github"), "{}");
});
