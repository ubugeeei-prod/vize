// Custody and authored diagnostics only; no provider or native producer calls.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { snapshot, readEvents } from "./html-cli-oracles.mjs";
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");

const serial = (value) => JSON.parse(JSON.stringify(value));

export function assertEvents(custody) {
  const loaded = new Set();
  for (const event of readEvents(custody.calls)) {
    assert.deepEqual(event.source, custody.source);
    assert.equal(event.binary, custody.binary.path);
    assert.equal(event.binarySha256, custody.binary.sha256);
    if (event.kind === "load") loaded.add(event.pid);
    if (["call", "html-call"].includes(event.kind))
      assert.ok(loaded.has(event.pid), "prior physical addon load required");
  }
}

export function authorities(cwd) {
  const result = [];
  for (let directory = cwd; ; directory = path.dirname(directory)) {
    for (const name of [".gitignore", ".eslintignore"]) {
      const file = path.join(directory, name);
      try {
        result.push([file, snapshot(file)]);
      } catch (error) {
        if (error.code !== "ENOENT") throw error;
        result.push([file, null]);
      }
    }
    if (path.dirname(directory) === directory) return result;
  }
}

function assertSources(actual, cwd, literalTarget) {
  const rootJson = path.join(cwd, ".oxlintrc.json"),
    target = path.resolve(cwd, literalTarget);
  assert.equal(actual.cwd, cwd);
  assert.equal(actual.literalTarget, literalTarget);
  assert.equal(actual.rootJson, rootJson);
  assert.equal(actual.target, target);
  const expected = [
    { path: rootJson, role: "RootJson", bytes: Array.from(fs.readFileSync(rootJson)) },
  ];
  const directory = fs.statSync(target).isDirectory() ? target : path.dirname(target);
  for (const [file, contents] of authorities(directory)) {
    assert.ok(contents == null || contents[0][1] === "file", "regular original authority required");
    expected.push({
      path: file,
      role: path.basename(file) === ".gitignore" ? "GitIgnore" : "CustomIgnore",
      ...(contents == null ? {} : { bytes: Array.from(Buffer.from(contents[0][2], "base64")) }),
    });
  }
  expected.sort((a, b) =>
    a.path < b.path ? -1 : a.path > b.path ? 1 : a.role < b.role ? -1 : a.role > b.role ? 1 : 0,
  );
  assert.deepEqual(
    serial(actual.sources),
    expected,
    "whole source keys/bytes/absence; no phantom GitInfoExclude",
  );
}

export function expectedRow(finding, cwd) {
  const source = fs.readFileSync(path.join(cwd, finding.path));
  const lines = source
    .subarray(0, finding.start)
    .toString("utf8")
    .split(/\r\n|\r|\n/u);
  return {
    message: `${finding.message}\n    Help:\n      ${finding.help}`,
    code: `vize(${finding.rule})`,
    severity: finding.severity,
    filename: finding.path,
    labels: [
      {
        span: {
          offset: finding.start,
          length: finding.end - finding.start,
          line: lines.length,
          column: Buffer.byteLength(lines.at(-1)) + 1,
        },
      },
    ],
  };
}

export function expectedStockVueRow(finding, cwd) {
  // Stock 1.81 extracts the script and cannot map this template diagnostic.
  // plugin.ts uses [1, 1]; oxc_linter restores its trimmed LF, then the Vue
  // loader restores the script body's physical offset (24 + 1 + 1 = 26).
  const source = fs.readFileSync(path.join(cwd, finding.path));
  assert.ok(finding.path.endsWith(".vue"));
  assert.ok(source.subarray(0, 25).equals(Buffer.from('<script setup lang="ts">\n')));
  assert.ok(finding.start > source.indexOf(Buffer.from("</script>")));
  const original = expectedRow(finding, cwd);
  const span = original.labels[0].span;
  return {
    ...original,
    message: `${finding.message} (at <template>:${span.line}:${span.column})\n    Help:\n      ${finding.help}`,
    labels: [{ span: { offset: 26, length: 0, line: 2, column: 2 } }],
  };
}

export function judgeCli(fixture, format, record, cwd, engine) {
  const stdout = Buffer.from(record.stdoutBytes).toString("utf8");
  assert.equal(record.status, fixture.expectedExit, stdout + Buffer.from(record.stderrBytes));
  assert.deepEqual(record.stderrBytes, []);
  const html = record.events.filter((event) => event.kind === "html-call");
  const explicitVue = fixture.argv[0] === "app/App.vue";
  assert.equal(html.length, explicitVue ? 0 : 1, "one original HTML operation; no extra query");
  if (!explicitVue) {
    assert.equal(html[0].outcome, "return");
    const actual = html[0].result.completed;
    assert.ok(actual);
    assert.equal(html[0].result.refused, undefined);
    assert.equal(actual.hostProfile, "1.81.0");
    assert.equal(actual.cwd, cwd);
    assert.equal(actual.repository, path.parse(cwd).root);
    assert.deepEqual(
      actual.originals.map((file) => file.cwdRelative),
      fixture.selectedHtml,
    );
    assert.equal(actual.executedFileCount, fixture.expectedHtmlExecutions);
    for (const file of actual.originals)
      assert.deepEqual(Buffer.from(file.bytes), fs.readFileSync(file.path));
    assertSources(actual, cwd, fixture.argv[0]);
    const expected = fixture.expectedDiagnostics.filter((finding) =>
      finding.path.endsWith(".html"),
    );
    assert.deepEqual(
      actual.files.flatMap((file) =>
        file.diagnostics.map((finding) => ({
          path: path.relative(cwd, file.path),
          ...finding,
        })),
      ),
      expected.map((finding) => ({
        path: finding.path,
        ruleName: finding.rule,
        severity: finding.severity,
        message: finding.message,
        start: finding.start,
        end: finding.end,
        help: finding.help,
        labels: [],
      })),
    );
    if (format !== "json" && actual.executedFileCount) assert.ok(stdout.endsWith(actual.output));
  }
  const host = record.events.filter((event) => event.kind === "host-child");
  assert.ok(host.length > 0, "actual Oxlint child must run");
  for (const event of host) {
    assert.equal(event.signal, null);
    assert.equal(event.error, null);
    assert.ok([0, 1].includes(event.status));
    assert.equal(event.cwd, cwd);
    assert.equal(fs.realpathSync(event.args[0]), engine);
    assert.deepEqual(event.stderrBytes, []);
    assert.equal(event.providerSha256, hash(fs.readFileSync(engine)));
  }
  const handshake = record.events.filter((event) => event.kind === "host-handshake");
  assert.equal(handshake.length, 1);
  assert.equal(Buffer.from(handshake[0].stdoutBytes).toString("utf8").trim(), "Version: 1.81.0");
  assert.equal(handshake[0].status, 0);
  assert.equal(handshake[0].signal, null);
  assert.equal(handshake[0].error, null);
  assert.equal(fs.realpathSync(handshake[0].args[0]), engine);
  assert.equal(handshake[0].providerSha256, hash(fs.readFileSync(engine)));
  if (format === "json") {
    const report = JSON.parse(stdout);
    assert.deepEqual(
      report.diagnostics,
      fixture.expectedDiagnostics.map((finding) => expectedRow(finding, cwd)),
    );
  } else
    for (const finding of fixture.expectedDiagnostics) {
      assert.ok(stdout.includes(finding.path) && stdout.includes(finding.message));
      assert.ok(stdout.includes(finding.help));
    }
}

export function judgeNative(fixture, record, cwd, workspace, custody) {
  const { before, options } = record;
  assert.equal(record.error, null);
  assert.deepEqual(record.after, before);
  assert.deepEqual(record.sourceAuthoritiesAfter, record.sourceAuthorities);
  const calls = record.events.filter((event) => event.kind === "html-call");
  assert.equal(calls.length, 1);
  assert.equal(calls[0].outcome, "return");
  assertEvents(custody);
  assert.deepEqual(calls[0].args, [options]);
  assert.deepEqual(calls[0].result, serial(record.result));
  if (fixture.refusal) {
    const expected = {
      NestedConfig: {
        path: path.join(cwd, "app/.oxlintrc.json"),
        details: "only explicit root JSON authority is qualified",
      },
      ExternalCustomIgnore: {
        path: path.join(workspace, ".eslintignore"),
        details: "custom ignore above owned boundary",
      },
      NestedVcs: {
        path: path.join(cwd, "app/nested"),
        details: "nested VCS boundary is unqualified",
      },
    };
    if (fixture.refusal === "NestedConfig")
      expected.NestedConfig.originalBytes = Array.from(fs.readFileSync(expected.NestedConfig.path));
    assert.deepEqual(serial(record.result), {
      refused: { kind: fixture.refusal, ...expected[fixture.refusal] },
    });
  } else {
    assert.equal(record.result.refused, undefined);
    const actual = record.result.completed;
    assert.equal(actual.hostProfile, "1.81.0");
    assert.equal(actual.repository, path.parse(cwd).root);
    assert.equal(actual.rootDecision, fixture.rootDecision);
    assert.deepEqual(
      actual.originals.map((file) => file.cwdRelative),
      fixture.selectedHtml,
    );
    assert.equal(actual.executedFileCount, fixture.selectedHtml.length);
    for (const file of actual.originals)
      assert.deepEqual(Buffer.from(file.bytes), fs.readFileSync(file.path));
    assertSources(actual, cwd, fixture.target);
    assert.deepEqual(
      actual.files,
      fixture.selectedHtml.map((file) => ({
        path: path.join(cwd, file),
        filename: path.join(cwd, file),
        errorCount: 0,
        warningCount: 0,
        diagnostics: [],
      })),
    );
    assert.equal(actual.errors, 0);
    assert.equal(actual.warnings, 0);
    assert.deepEqual(JSON.parse(actual.jsonDiagnostics), []);
    assert.equal(actual.output, "");
    assert.equal(actual.format, "json");
    assert.deepEqual(actual.presentation, options.presentation);
    assert.equal(actual.engineConfigValidation, "not-performed");
    assert.deepEqual(actual.projection, {
      rules: [],
      settings: { locale: "en", helpLevel: "full", preset: "general-recommended" },
      denyWarnings: false,
    });
  }
}
