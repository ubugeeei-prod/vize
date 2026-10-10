import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { nativePreparationIsActive } from "../../native/scripts/test-preparation.mjs";
import { formatPatinaMessage } from "./format.ts";

const root = path.resolve(fileURLToPath(new URL("../../..", import.meta.url)));
const nativeDir = path.join(root, "npm/native");
assert.ok(
  nativePreparationIsActive(nativeDir),
  "this exact source's genuine native preparation is required",
);
const native = createRequire(import.meta.url)(path.join(nativeDir, "index.js"));
assert.equal(typeof native.lintOxlintHtml, "function");
const corpusDir = path.join(
  root,
  "tests/_fixtures/differential/lint/oxlint-original-html-operation-7903",
);
const corpus = JSON.parse(fs.readFileSync(path.join(corpusDir, "cases.json"), "utf8"));
const packets = JSON.parse(fs.readFileSync(path.join(corpusDir, "expected-packets.json"), "utf8"));
const evidenceDir = path.join(root, "target/oxlint-original-html-binding-7903");
fs.mkdirSync(evidenceDir, { recursive: true });
const bytes = (text) =>
  text.startsWith("@") ? fs.readFileSync(path.join(corpusDir, text.slice(1))) : Buffer.from(text);
const hash = (value) => createHash("sha256").update(value).digest("hex");

function snapshot(directory) {
  const entries = [];
  const visit = (file) => {
    const stat = fs.lstatSync(file);
    const relative = path.relative(directory, file);
    if (stat.isDirectory()) {
      entries.push([relative, "directory"]);
      for (const child of fs.readdirSync(file).sort()) visit(path.join(file, child));
    } else if (stat.isSymbolicLink()) entries.push([relative, "symlink", fs.readlinkSync(file)]);
    else entries.push([relative, "file", fs.readFileSync(file).toString("base64")]);
  };
  visit(directory);
  return entries;
}

function camel(value) {
  if (Array.isArray(value)) return value.map(camel);
  if (value && typeof value === "object")
    return Object.fromEntries(
      Object.entries(value).map(([key, item]) => [
        key.replace(/_([a-z])/gu, (_, c) => c.toUpperCase()),
        camel(item),
      ]),
    );
  return value;
}

function jsonRows(fixture, file, packet, helpLevel) {
  const source = bytes(fixture.files[file.path]);
  return packet.diagnostics.map((finding) => {
    const prefix = source.subarray(0, finding.start).toString("utf8");
    const lines = prefix.split(/\r\n|\r|\n/u);
    const message = formatPatinaMessage(
      { message: finding.message, help: finding.help },
      { hasMappedLocation: true, blockLabel: "html", helpLevel },
    );
    return {
      message,
      code: `vize(${finding.rule_name})`,
      severity: finding.severity,
      filename: file.path,
      labels: [
        {
          span: {
            offset: finding.start,
            length: finding.end - finding.start,
            line: lines.length,
            column: Buffer.byteLength(lines.at(-1)) + 1,
          },
        },
        ...finding.labels.map((label) => {
          const lines = source
            .subarray(0, label.start)
            .toString("utf8")
            .split(/\r\n|\r|\n/u);
          return {
            label: label.message,
            span: {
              offset: label.start,
              length: label.end - label.start,
              line: lines.length,
              column: Buffer.byteLength(lines.at(-1)) + 1,
            },
          };
        }),
      ],
    };
  });
}

for (const fixture of corpus.cases)
  for (const hostProfile of ["1.78.0", "1.86.0"])
    for (const format of ["default", "json", "unix", "stylish"]) {
      const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-html-binding-"));
      const cwd = path.join(temporary, "repo");
      const recordPath = path.join(evidenceDir, `${fixture.name}-${hostProfile}-${format}.json`);
      const record = {
        scope: "exact-source additive binding; no public CLI or host setup claim",
        fixture,
        hostProfile,
        format,
        nativeDir,
        setup: null,
        before: null,
        outcome: null,
        after: null,
        passed: false,
      };
      const save = () => fs.writeFileSync(recordPath, JSON.stringify(record, null, 2) + "\n");
      save();
      try {
        fs.mkdirSync(cwd);
        const setup = spawnSync("git", ["init", "-q"], { cwd, encoding: "utf8" });
        record.setup = {
          program: "git",
          argv: ["init", "-q"],
          cwd,
          status: setup.status,
          signal: setup.signal,
          error: setup.error?.message,
          stdout: setup.stdout,
          stderr: setup.stderr,
        };
        save();
        assert.equal(setup.status, 0);
        fs.writeFileSync(path.join(cwd, ".git/info/exclude"), "");
        for (const [filename, text] of Object.entries(fixture.files)) {
          fs.mkdirSync(path.dirname(path.join(cwd, filename)), { recursive: true });
          fs.writeFileSync(path.join(cwd, filename), bytes(text));
        }
        const rootJson = path.join(cwd, ".oxlintrc.json");
        fs.writeFileSync(rootJson, fixture.root);
        const options = {
          cwd,
          literalTarget: fixture.target,
          rootJson,
          rootBytes: Array.from(Buffer.from(fixture.expected_root ?? fixture.root)),
          hostProfile,
          noIgnore: fixture.no_ignore ?? false,
          cliIgnorePatterns: fixture.cli ?? [],
          customIgnoreFilename: ".eslintignore",
          format,
          presentation: {
            stylishRelative: false,
            cwd,
            graphicalTheme: "plain",
            links: false,
            width: 400,
            stylishNoColor: true,
          },
        };
        record.options = options;
        record.before = snapshot(temporary);
        save();
        const outcome = native.lintOxlintHtml(options);
        record.outcome = outcome;
        save();
        record.after = snapshot(temporary);
        save();
        assert.deepEqual(record.after, record.before, "whole original/authority tree is unchanged");
        if (fixture.refusal) {
          assert.equal(outcome.completed, undefined);
          assert.equal(outcome.refused.kind, fixture.refusal.kind);
          assert.equal(outcome.refused.path, path.join(cwd, fixture.refusal.path));
          const expected =
            fixture.refusal.bytes_ref === "@root"
              ? Buffer.from(fixture.root)
              : bytes(fixture.refusal.bytes_ref);
          assert.deepEqual(outcome.refused.originalBytes, Array.from(expected));
          assert.ok(outcome.refused.details.length > 0);
        } else {
          assert.equal(outcome.refused, undefined);
          const actual = outcome.completed;
          const selected =
            hostProfile === "1.86.0"
              ? (fixture.selected_186 ?? fixture.selected)
              : fixture.selected;
          assert.equal(actual.hostProfile, hostProfile);
          assert.equal(actual.cwd, cwd);
          assert.equal(actual.repository, cwd);
          assert.equal(actual.target, path.resolve(cwd, fixture.target));
          assert.equal(actual.literalTarget, fixture.target);
          assert.equal(actual.rootJson, rootJson);
          assert.equal(actual.noIgnore, options.noIgnore);
          assert.deepEqual(actual.cliIgnorePatterns, options.cliIgnorePatterns);
          assert.equal(actual.customIgnoreFilename, ".eslintignore");
          assert.equal(
            actual.rootDecision,
            (hostProfile === "1.86.0"
              ? (fixture.root_decision_186 ?? fixture.root_decision)
              : fixture.root_decision) ?? "Eligible",
          );
          const projection = camel(fixture.projection);
          projection.rules = projection.rules.map(({ severity, ...rule }) => ({
            ...rule,
            ...(severity == null ? {} : { severity }),
          }));
          assert.deepEqual(actual.projection, projection);
          assert.equal(actual.executedFileCount, selected.length);
          assert.equal(actual.files.length, selected.length);
          assert.equal(actual.originals.length, selected.length);
          assert.ok(Number.isFinite(actual.elapsedSeconds) && actual.elapsedSeconds >= 0);
          const rootSource = actual.sources.find((source) => source.role === "RootJson");
          assert.equal(rootSource.path, rootJson);
          assert.deepEqual(rootSource.bytes, Array.from(Buffer.from(fixture.root)));
          const expectedRows = [];
          for (const [index, file] of selected.entries()) {
            const packet = packets[file.packet];
            assert.deepEqual(actual.originals[index], {
              path: path.join(cwd, file.path),
              cwdRelative: file.path,
              origin: fixture.target === "." ? "DirectoryDiscovery" : "ExplicitFile",
              bytes: Array.from(bytes(fixture.files[file.path])),
            });
            const expectedPacket = camel(packet);
            expectedPacket.diagnostics = expectedPacket.diagnostics.map(
              ({ fix, help, ...finding }, index) => ({
                ...finding,
                ...(help == null ? {} : { help }),
                ...(fix == null ? {} : { fix: packet.diagnostics[index].fix }),
              }),
            );
            assert.deepEqual(actual.files[index], {
              path: path.join(cwd, file.path),
              filename: path.join(cwd, file.path),
              ...expectedPacket,
            });
            expectedRows.push(
              ...jsonRows(fixture, file, packet, fixture.projection.settings.help_level),
            );
          }
          assert.deepEqual(
            JSON.parse(actual.jsonDiagnostics),
            expectedRows,
            "complete independently authored native JSON presentation rows",
          );
          assert.equal(
            actual.errors,
            selected.reduce((sum, file) => sum + packets[file.packet].error_count, 0),
          );
          assert.equal(
            actual.warnings,
            selected.reduce((sum, file) => sum + packets[file.packet].warning_count, 0),
          );
          assert.equal(actual.format, format);
          assert.deepEqual(actual.presentation, options.presentation);
          assert.equal(actual.engineConfigValidation, "not-performed");
          if (
            selected.length === 0 ||
            format === "json" ||
            (["unix", "stylish"].includes(format) && expectedRows.length === 0)
          )
            assert.equal(actual.output, "");
          record.jsonRowsSha256 = hash(actual.jsonDiagnostics);
        }
        record.passed = true;
        save();
      } finally {
        fs.rmSync(temporary, { recursive: true, force: true });
      }
    }
console.log(
  `Qualified ${corpus.cases.length * 8} genuine HTML binding observations with original tree custody.`,
);
