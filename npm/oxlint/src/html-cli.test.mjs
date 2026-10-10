import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import {
  fixtures,
  snapshot,
  expectedRows,
  completeCase,
  readEvents,
  plainHostEnvironment,
  finishCapture,
  expectedOriginalRows,
  assertStandaloneOutput,
} from "./test-support/html-cli-oracles.mjs";
import { presentationInputs } from "./cli/presentation-context.ts";
import { mergeHtmlOutput, unavailableHtml } from "./cli/html-project-output.ts";

const packageDir = fileURLToPath(new URL("..", import.meta.url));
const repository = path.resolve(packageDir, "../..");
const corpus = path.join(
  repository,
  "tests/_fixtures/differential/lint/oxlint-original-html-operation-7903",
);
const packets = JSON.parse(fs.readFileSync(path.join(corpus, "expected-packets.json")));
Object.assign(
  packets,
  JSON.parse(
    fs.readFileSync(new URL("./test-support/html-cli-original-packets.json", import.meta.url)),
  ),
);
const engine = process.env.VIZE_OXLINT_TEST_ENTRYPOINT;
assert.ok(
  engine && process.env.VIZE_OXLINT_NATIVE_CUSTODY,
  "actual pinned provider and authenticated source addon are required",
);
const custody = JSON.parse(fs.readFileSync(process.env.VIZE_OXLINT_NATIVE_CUSTODY));
const capturePath = process.env.VIZE_OXLINT_HTML_CAPTURE;
const capture = {
  schema: "vize.oxlint.original-html-public.v1",
  complete: false,
  source: custody.source,
  observations: [],
  qualified: { cases: 0, formats: [], wholeNativeCalls: 0, wholeHostPhases: 0 },
};
const save = () => fs.writeFileSync(capturePath, JSON.stringify(capture, null, 2) + "\n");
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const events = () => readEvents(custody.calls);
const environment = plainHostEnvironment(process.env);
for (const fixture of fixtures)
  for (const format of fixture.formats ?? ["default", "json", "unix", "stylish"]) {
    const childEnvironment = {
      ...environment,
      ...(fixture.agent ? { AI_AGENT: "qualification" } : {}),
    };
    const workspace = fs.realpathSync(
      fs.mkdtempSync(path.join(os.tmpdir(), "vize-original-html-cli-")),
    );
    const root = path.join(workspace, "repo"),
      temporary = path.join(workspace, "transport");
    fs.mkdirSync(root);
    fs.mkdirSync(temporary);
    const setup = spawnSync("git", ["init", "-q"], { cwd: root });
    capture.observations.push({
      kind: "setup",
      fixture: fixture.name,
      root,
      args: ["init", "-q"],
      status: setup.status,
      signal: setup.signal,
      error: setup.error?.message ?? null,
      stdoutBytes: Array.from(setup.stdout),
      stderrBytes: Array.from(setup.stderr),
    });
    save();
    assert.equal(setup.status, 0);
    fs.writeFileSync(path.join(root, ".git/info/exclude"), "");
    fs.mkdirSync(path.join(root, "node_modules/oxlint/bin"), { recursive: true });
    fs.symlinkSync(engine, path.join(root, "node_modules/oxlint/bin/oxlint"));
    fs.symlinkSync(packageDir, path.join(root, "node_modules/oxlint-plugin-vize"));
    const config = {
      jsPlugins: fixture.badPlugin
        ? ["missing-original-jsplugin", "oxlint-plugin-vize"]
        : ["oxlint-plugin-vize"],
      rules: fixture.rules ?? { "vize/vue/no-v-html": "warn" },
      ...(fixture.settings ? { settings: fixture.settings } : {}),
      ...(fixture.denyWarnings ? { options: { denyWarnings: true } } : {}),
      ...(fixture.ignored ? { ignorePatterns: ["vendor/**"] } : {}),
    };
    fs.writeFileSync(path.join(root, ".oxlintrc.json"), JSON.stringify(config) + "\n");
    fs.writeFileSync(
      path.join(root, ".gitignore"),
      "node_modules/\n" + (fixture.ignored ? "dist/\n" : ""),
    );
    for (const [name, source] of Object.entries(fixture.files)) {
      fs.mkdirSync(path.dirname(path.join(root, name)), { recursive: true });
      fs.copyFileSync(path.join(corpus, source), path.join(root, name));
    }
    const args = [
      "--threads",
      "1",
      ...(fixture.implicitDefault && format === "default" ? [] : ["-f", format]),
      ...(fixture.ignored ? ["--ignore-pattern", "src/Skipped.html"] : []),
      ...(fixture.deny ? ["--deny-warnings"] : []),
      fixture.target ?? ".",
    ];
    const before = snapshot(workspace);
    const run = (entrypoint) => {
      const index = events().length;
      const result = spawnSync(process.execPath, [entrypoint, ...args], {
        cwd: root,
        env: { ...childEnvironment, TMPDIR: temporary },
        timeout: 60_000,
        maxBuffer: 64 * 1024 * 1024,
      });
      const observed = events().slice(index);
      capture.observations.push({
        kind: "process",
        fixture: fixture.name,
        format,
        cwd: root,
        entrypoint,
        args,
        environment: Object.fromEntries(
          presentationInputs.map((key) => [key, childEnvironment[key] ?? null]),
        ),
        provider: {
          path: engine,
          sha256: sha256(fs.readFileSync(engine)),
          packageBytes: Array.from(
            fs.readFileSync(path.join(path.dirname(engine), "../package.json")),
          ),
        },
        before,
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
        stdoutBytes: Array.from(result.stdout ?? []),
        stderrBytes: Array.from(result.stderr ?? []),
        events: observed,
      });
      save();
      const after = snapshot(workspace);
      capture.observations.push({ kind: "custody", fixture: fixture.name, format, after });
      save();
      assert.equal(result.signal, null);
      assert.equal(result.error, undefined);
      assert.deepEqual(after, before);
      return {
        ...result,
        stdout: result.stdout.toString("utf8"),
        stderr: result.stderr.toString("utf8"),
        events: observed,
      };
    };
    try {
      const stock = run(engine),
        wrapper = run(path.join(packageDir, "dist/cli.mjs"));
      assert.equal(wrapper.status, fixture.status ?? 0, wrapper.stdout + wrapper.stderr);
      const nativeCalls = wrapper.events.filter((event) => event.kind === "html-call");
      const hostPhases = wrapper.events.filter((event) => event.kind === "host-child");
      const originalOutput = Buffer.from(hostPhases[0]?.stdoutBytes ?? []).toString("utf8");
      const handshakes = wrapper.events.filter((event) => event.kind === "host-handshake");
      assert.equal(
        handshakes.length,
        1,
        "the original version query is retained once without added provider queries",
      );
      assert.equal(handshakes[0].status, 0);
      assert.equal(handshakes[0].signal, null);
      assert.equal(handshakes[0].error, null);
      assert.equal(
        hostPhases.length,
        fixture.original ? 5 : 1,
        "original stock/Vue phases are conserved; HTML adds no provider phase",
      );

      if (fixture.refused) {
        assert.equal(nativeCalls.length, 0);
        assert.equal(wrapper.stdout, originalOutput);
        assert.match(wrapper.stderr, /Original HTML unavailable/u);
        completeCase(capture, nativeCalls, hostPhases);
        save();
        continue;
      }
      assert.equal(wrapper.stderr, "");
      assert.equal(nativeCalls.length, 1);
      assert.equal(nativeCalls[0].outcome, "return");
      const actual = nativeCalls[0].result.completed;
      assert.ok(actual);
      assert.equal(nativeCalls[0].result.refused, undefined);
      assert.equal(actual.executedFileCount, Object.keys(fixture.selected).length);
      assert.deepEqual(
        actual.originals.map((original) => original.cwdRelative),
        Object.keys(fixture.selected).sort(),
      );
      for (const original of actual.originals)
        assert.deepEqual(
          Buffer.from(original.bytes),
          fs.readFileSync(path.join(root, original.cwdRelative)),
        );
      const htmlRows = [];
      for (const file of actual.files) {
        const name = path.relative(root, file.path),
          packet = packets[fixture.selected[name]];
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
          ...expectedRows(
            name,
            packet,
            fs.readFileSync(file.path),
            fixture.settings?.vize?.helpLevel,
          ),
        );
      }
      assert.deepEqual(JSON.parse(actual.jsonDiagnostics), htmlRows);
      assertStandaloneOutput(fixture, format, wrapper.stdout, repository);
      if (actual.executedFileCount > 0 && !fixture.original) {
        const original = {
          stdout: stock.stdout,
          stderr: stock.stderr,
          status: stock.status,
          observation: { signal: null, error: null },
        };
        for (const rejected of [
          { ...original, status: null },
          { ...original, observation: { signal: "SIGTERM", error: null } },
          { ...original, observation: { signal: null, error: "partial stream failure" } },
          { ...original, stdout: stock.stdout + "unexpected extra output\n" },
        ]) {
          let caught;
          try {
            mergeHtmlOutput(rejected, actual, args);
          } catch (error) {
            caught = error;
          }
          assert.ok(caught, "abnormal or unqualified whole host packets must refuse composition");
          const preserved = unavailableHtml(rejected, caught);
          assert.equal(preserved.stdout, rejected.stdout);
          assert.ok(preserved.stderr.startsWith(rejected.stderr));
          assert.ok(preserved.status >= 1);
        }
      }

      assert.ok(hostPhases.length > 0);
      for (const event of hostPhases) {
        assert.equal(event.signal, null);
        assert.equal(event.error, null);
        assert.deepEqual(event.source, custody.source);
        assert.equal(event.binarySha256, custody.binary.sha256);
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
        const originalRows = expectedOriginalRows(
          fixture,
          packets.warning,
          root,
          stockReport.diagnostics,
        );
        assert.deepEqual(report.diagnostics, [...originalRows, ...htmlRows]);
        assert.equal(
          report.number_of_files,
          stockReport.number_of_files + actual.executedFileCount,
        );
        assert.equal(report.number_of_rules, stockReport.number_of_rules);
        assert.equal(report.threads_count, stockReport.threads_count);
        assert.ok(report.start_time >= actual.elapsedSeconds);
        assert.deepEqual(Object.keys(report), Object.keys(stockReport));
      } else {
        assert.ok(
          wrapper.stdout.endsWith(actual.output),
          "complete native report bytes must survive the public join",
        );
        const preceding = wrapper.stdout.slice(0, wrapper.stdout.length - actual.output.length);
        if (!fixture.original) {
          if (format === "default")
            assert.match(
              preceding,
              /^Finished in [0-9.]+(?:ms|s) on 0 files with [0-9]+ rules using 1 threads\.\n$/u,
            );
          else assert.equal(preceding, "");
        } else
          assert.ok(preceding.includes("AppPanel.vue") && !preceding.includes("Standalone.html"));
        if (format === "unix" || format === "stylish") {
          let expected = "";
          let errors = 0,
            warnings = 0;
          for (const file of actual.files) {
            const name = path.relative(root, file.path),
              rows = htmlRows.filter((row) => row.filename === name);
            if (format === "stylish" && rows.length) expected += `\n${name}\n`;
            for (const row of rows) {
              const { line, column } = row.labels[0].span;
              errors += row.severity === "error";
              warnings += row.severity === "warning";
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
      save();
    } finally {
      fs.rmSync(workspace, { recursive: true, force: true });
    }
  }
capture.qualified.formats = ["default", "json", "unix", "stylish"];
finishCapture(capture);
save();
