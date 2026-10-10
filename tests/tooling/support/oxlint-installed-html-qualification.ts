import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import {
  expectedRows,
  expectedOriginalRows,
  assertStandaloneOutput,
  completeCase,
} from "../../../npm/oxlint/src/test-support/html-cli-oracles.mjs";
import { sourceRoot } from "./oxlint-installed-html-authority.ts";
import type {
  HtmlFixture,
  ExpectedPacket,
  HtmlRow,
  HtmlProcess,
  HtmlCapture,
} from "./oxlint-installed-html-types.ts";

const rowsFromPacket = expectedRows as unknown as (
  file: string,
  packet: ExpectedPacket,
  source: Buffer,
  helpLevel?: string,
) => HtmlRow[];
const originalRowsFromPacket = expectedOriginalRows as unknown as (
  fixture: HtmlFixture,
  packet: ExpectedPacket,
  root: string,
  stockRows: HtmlRow[],
) => HtmlRow[];

/** Same whole original input/report laws as the source-owned public HTML suite. */
export function qualifyInstalledHtml(
  capture: HtmlCapture,
  fixture: HtmlFixture,
  format: string,
  root: string,
  version: string,
  stock: HtmlProcess,
  wrapper: HtmlProcess,
  packets: Record<string, ExpectedPacket>,
) {
  assert.equal(wrapper.status, fixture.status ?? 0, wrapper.stdout + wrapper.stderr);
  const nativeCalls = wrapper.events.filter(({ kind }) => kind === "html-call");
  const hostPhases = wrapper.events.filter(({ kind }) => kind === "host-child");
  const handshakes = wrapper.events.filter(({ kind }) => kind === "host-handshake");
  assert.equal(handshakes.length, 1, "exactly the existing provider version query");
  assert.equal(handshakes[0].status, 0);
  assert.equal(handshakes[0].signal, null);
  assert.equal(handshakes[0].error, null);
  assert.equal(
    Buffer.from(handshakes[0].stdoutBytes ?? []).toString("utf8"),
    `Version: ${version}\n\n`,
  );
  assert.deepEqual(handshakes[0].stderrBytes, []);
  assert.equal(
    hostPhases.length,
    fixture.original ? 5 : 1,
    "conserve the genuine original stock/Vue phases",
  );
  const originalOutput = Buffer.from(hostPhases[0].stdoutBytes ?? []).toString("utf8");
  if (fixture.refused) {
    assert.equal(nativeCalls.length, 0);
    assert.equal(wrapper.stdout, originalOutput);
    assert.match(wrapper.stderr, /Original HTML unavailable/u);
    completeCase(capture, nativeCalls, hostPhases);
    return;
  }
  assert.equal(wrapper.stderr, "");
  assert.equal(nativeCalls.length, 1);
  assert.equal(nativeCalls[0].outcome, "return");
  const actual = nativeCalls[0].result?.completed;
  assert.ok(actual);
  assert.equal(nativeCalls[0].result?.refused, undefined);
  const rootJson = path.join(root, ".oxlintrc.json");
  const presentation = {
    cwd: root,
    graphicalTheme: "plain",
    links: false,
    width: 400,
    stylishNoColor: true,
    stylishRelative: true,
  };
  const cliIgnorePatterns = fixture.ignored ? ["src/Skipped.html"] : [];
  assert.deepEqual(nativeCalls[0].args, [
    {
      cwd: root,
      literalTarget: fixture.target ?? ".",
      rootJson,
      rootBytes: Array.from(fs.readFileSync(rootJson)),
      hostProfile: version,
      noIgnore: false,
      cliIgnorePatterns,
      customIgnoreFilename: ".eslintignore",
      format,
      presentation,
    },
  ]);
  for (const [key, expected] of Object.entries({
    cwd: root,
    hostProfile: version,
    literalTarget: fixture.target ?? ".",
    target: path.resolve(root, fixture.target ?? "."),
    repository: root,
    rootJson,
    noIgnore: false,
    cliIgnorePatterns,
    customIgnoreFilename: ".eslintignore",
    format,
    presentation,
    rootDecision: "Eligible",
    engineConfigValidation: "not-performed",
  }))
    assert.deepEqual(
      actual[key as keyof typeof actual],
      expected,
      "complete authored HTML metadata: " + key,
    );
  assert.deepEqual(actual.projection, {
    rules: Object.entries(fixture.rules ?? { "vize/vue/no-v-html": "warn" })
      .filter(([name]) => name.startsWith("vize/"))
      .map(([name, severity]) => ({
        name: name.slice(5),
        active: severity !== "off",
        authoredOptions: [],
        ...(severity === "off" ? {} : { severity: severity === "warn" ? "warning" : "error" }),
      })),
    settings: {
      locale: "en",
      helpLevel: fixture.settings?.vize?.helpLevel ?? "full",
      preset: fixture.settings?.vize?.preset ?? "incremental",
    },
    denyWarnings: fixture.denyWarnings ?? false,
  });
  assert.deepEqual(
    actual.sources.filter(({ role }) => role === "RootJson"),
    [{ path: rootJson, role: "RootJson", bytes: Array.from(fs.readFileSync(rootJson)) }],
  );
  assert.equal(
    new Set(actual.sources.map(({ path: filename, role }) => role + "\0" + filename)).size,
    actual.sources.length,
  );
  for (const source of actual.sources) {
    assert.ok(["RootJson", "GitInfoExclude", "GitIgnore", "CustomIgnore"].includes(source.role));
    if (fs.existsSync(source.path))
      assert.deepEqual(source.bytes, Array.from(fs.readFileSync(source.path)));
    else assert.equal(source.bytes, undefined);
  }
  assert.equal(actual.executedFileCount, Object.keys(fixture.selected).length);
  assert.ok(Number.isFinite(actual.elapsedSeconds) && actual.elapsedSeconds >= 0);
  assert.equal(
    actual.errors,
    actual.files.reduce((sum, file) => sum + file.errorCount, 0),
  );
  assert.equal(
    actual.warnings,
    actual.files.reduce((sum, file) => sum + file.warningCount, 0),
  );
  assert.deepEqual(
    actual.originals.map(({ cwdRelative }) => cwdRelative),
    Object.keys(fixture.selected).sort(),
  );
  for (const original of actual.originals)
    assert.deepEqual(
      Buffer.from(original.bytes),
      fs.readFileSync(path.join(root, original.cwdRelative)),
    );
  const htmlRows: HtmlRow[] = [];
  for (const file of actual.files) {
    const name = path.relative(root, file.path),
      packet = packets[fixture.selected[name]];
    assert.ok(packet, name + ": every actual finding has an independent original packet");
    const expected = packet.diagnostics.map((finding) => ({
      ruleName: finding.rule_name,
      severity: finding.severity,
      message: finding.message,
      start: finding.start,
      end: finding.end,
      ...(finding.help == null ? {} : { help: finding.help }),
      labels: finding.labels,
      ...(finding.fix == null ? {} : { fix: finding.fix }),
    }));
    assert.deepEqual(file.diagnostics, expected);
    assert.equal(file.errorCount, packet.error_count);
    assert.equal(file.warningCount, packet.warning_count);
    htmlRows.push(
      ...rowsFromPacket(
        name,
        packet,
        fs.readFileSync(file.path),
        fixture.settings?.vize?.helpLevel,
      ),
    );
  }
  assert.deepEqual(JSON.parse(actual.jsonDiagnostics), htmlRows);
  assertStandaloneOutput(fixture, format, wrapper.stdout, sourceRoot);
  for (const event of hostPhases) {
    assert.equal(event.signal, null);
    assert.equal(event.error, null);
  }
  if (actual.executedFileCount === 0) {
    assert.equal(wrapper.stdout, originalOutput);
    assert.equal(wrapper.status, stock.status);
  } else if (format === "json") {
    const report = JSON.parse(wrapper.stdout);
    const stockReport = JSON.parse(
      stock.stdout.replace(
        /^No files found to lint\. Please check your paths and ignore patterns\.\n/u,
        "",
      ),
    );
    const originalRows = originalRowsFromPacket(
      fixture,
      packets.warning,
      root,
      stockReport.diagnostics,
    );
    assert.deepEqual(report.diagnostics, [...originalRows, ...htmlRows]);
    assert.equal(report.number_of_files, stockReport.number_of_files + actual.executedFileCount);
    assert.equal(report.number_of_rules, stockReport.number_of_rules);
    assert.equal(report.threads_count, stockReport.threads_count);
    const hostJson = (index: number) =>
      JSON.parse(
        Buffer.from(hostPhases[index].stdoutBytes ?? [])
          .toString("utf8")
          .replace(
            /^No files found to lint\. Please check your paths and ignore patterns\.\n/u,
            "",
          ),
      );
    const source = hostJson(fixture.original ? 2 : 0);
    const carried = fixture.original ? hostJson(4) : undefined;
    assert.equal(
      report.start_time,
      source.start_time + (carried?.start_time ?? 0) + actual.elapsedSeconds,
      "conserve the elapsed time of the actual original report operands",
    );
    assert.equal(report.number_of_files, source.number_of_files + actual.executedFileCount);
    assert.equal(report.number_of_rules, source.number_of_rules + (carried?.number_of_rules ?? 0));
    assert.equal(report.threads_count, Math.max(source.threads_count, carried?.threads_count ?? 0));
    assert.deepEqual(Object.keys(report), Object.keys(stockReport));
  } else {
    assert.ok(
      wrapper.stdout.endsWith(actual.output),
      "preserve the complete actual native report operand",
    );
    const preceding = wrapper.stdout.slice(0, wrapper.stdout.length - actual.output.length);
    if (!fixture.original) {
      if (format === "default")
        assert.match(
          preceding,
          /^Finished in [0-9.]+(?:ms|s) on 0 files with [0-9]+ rules using 1 threads\.\n$/u,
        );
      else assert.equal(preceding, "");
    } else {
      const carried = hostPhases[4].args?.at(-1);
      assert.equal(typeof carried, "string");
      assert.ok((carried as string).endsWith("/inside/src/AppPanel.vue"));
      const originalReport = Buffer.from(hostPhases[2].stdoutBytes ?? []).toString("utf8");
      const carriedReport = Buffer.from(hostPhases[4].stdoutBytes ?? []).toString("utf8");
      assert.equal(
        preceding,
        originalReport + carriedReport.replaceAll(carried as string, "src/AppPanel.vue"),
        "preserve both complete actual original Vue report operands and only their owning path",
      );
    }
    if (format === "unix" || format === "stylish") {
      let expected = "",
        errors = 0,
        warnings = 0;
      for (const file of actual.files) {
        const name = path.relative(root, file.path),
          rows = htmlRows.filter(({ filename }) => filename === name);
        if (format === "stylish" && rows.length) expected += `\n${name}\n`;
        for (const row of rows) {
          const { line, column } = row.labels[0].span;
          errors += Number(row.severity === "error");
          warnings += Number(row.severity === "warning");
          expected +=
            format === "unix"
              ? `${name}:${line}:${column}: ${row.message} [${row.severity === "error" ? "Error" : "Warning"}/${row.code}]\n`
              : `  ${line}:${column}  ${row.severity}  ${row.message}  ${row.code}\n`;
        }
      }
      const total = errors + warnings;
      if (total)
        expected +=
          format === "unix"
            ? `\n${total} problem${total === 1 ? "" : "s"}\n`
            : `\n✖ ${total} problem${total === 1 ? "" : "s"} (${errors} error${errors === 1 ? "" : "s"}, ${warnings} warning${warnings === 1 ? "" : "s"})\n`;
      assert.equal(actual.output, expected);
    }
  }
  completeCase(capture, nativeCalls, hostPhases);
}
