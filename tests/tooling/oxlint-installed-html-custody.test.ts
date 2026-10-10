// Inert refusal/input laws. No registry consumer, fake native binary, or successful runtime is credited.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  campaignAuthority,
  htmlInstalledAuthority,
  installedHtmlPreflight,
  sourceRoot,
  validateHtmlPlan,
  wholeProcessError,
} from "./support/oxlint-installed-html-authority.ts";
import { installedHtmlRuntime } from "./support/oxlint-installed-html-runtime.ts";
import { retainHtmlProcess } from "./support/oxlint-installed-html-packets.ts";
import { verifyStockArchives } from "./support/oxlint-installed-html-stock.ts";
import type { HtmlCampaignPlan } from "./support/oxlint-installed-html-types.ts";
import { runtimeOverrides } from "../../tools/support/release/public_acceptance/installed.ts";
import { fixtures, finishCapture } from "../../npm/oxlint/src/test-support/html-cli-oracles.mjs";

const plan = (): HtmlCampaignPlan => ({
  schema: "vize.oxlint.installed-html-campaign-v1",
  source: { C: "1".repeat(40), H: "2".repeat(40), tag: "v0.999.0", R: "1", sourcePr: "2" },
  installReceipt: { path: "/inert/never-executed.json", sha256: "0".repeat(64) },
  collectorSha256: "a".repeat(64),
  campaignSha256: campaignAuthority().sha256,
  includedRoutes: {
    binding: { pr: 8406, commit: "3".repeat(40) },
    cli: { pr: 8435, commit: "4".repeat(40) },
  },
  providers: ["1.78.0", "1.86.0"].map((version) => ({
    version: version as "1.78.0" | "1.86.0",
    installRoot: "/inert/never-installed/" + version,
    packageLockSha256: "b".repeat(64),
  })),
});
function withoutAmbientOverrides(run: () => void) {
  const keys = [...runtimeOverrides, "VIZE_OXLINT_PUBLIC_CUSTODY"];
  const previous = keys.map((key) => [key, process.env[key]] as const);
  try {
    for (const key of keys) delete process.env[key];
    run();
  } finally {
    for (const [key, value] of previous) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
  }
}

test("installed HTML plans retain all explicit source/receipt/collector/campaign and actual route identities", () => {
  validateHtmlPlan(plan()); // Syntax only, not a real release or execution claim.
  const mutations: Array<(value: HtmlCampaignPlan) => void> = [
    (value) => {
      value.source.H = value.source.C;
    },
    (value) => {
      value.source.tag = "latest";
    },
    (value) => {
      value.source.sourcePr = "0";
    },
    (value) => {
      value.installReceipt.sha256 = "";
    },
    (value) => {
      value.collectorSha256 = "";
    },
    (value) => {
      value.campaignSha256 = "";
    },
    (value) => {
      value.includedRoutes.cli.commit = value.includedRoutes.binding.commit;
    },
    (value) => {
      value.includedRoutes.binding.commit = "main";
    },
    (value) => {
      value.providers.reverse();
    },
    (value) => {
      value.providers[0].packageLockSha256 = "";
    },
    (value) => {
      value.providers[0].installRoot = "relative";
    },
  ];
  for (const change of mutations) {
    const value = plan();
    change(value);
    assert.throws(() => validateHtmlPlan(value));
  }
});

test("an absent merged route or changed campaign cannot access an install receipt or start a public executable", (t) => {
  withoutAmbientOverrides(() => {
    const temporary = fs.realpathSync(
      fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-installed-refusal-")),
    );
    t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
    const output = path.join(temporary, "must-not-exist");
    assert.throws(
      () => installedHtmlRuntime(output, ""),
      /explicit reviewed installed HTML campaign plan/u,
    );
    const value = plan();
    value.campaignSha256 = "0".repeat(64);
    assert.throws(() => htmlInstalledAuthority(value), /reviewed HTML campaign snapshot changed/u);
    value.campaignSha256 = campaignAuthority().sha256;
    assert.throws(
      () => htmlInstalledAuthority(value),
      /published cut must contain actual merged #8406/u,
    );
    assert.equal(fs.existsSync(output), false);
  });
});

test("initial overrides are refused before any reviewed receipt, consumer payload or process access", () => {
  withoutAmbientOverrides(() => {
    for (const key of [...runtimeOverrides, "VIZE_OXLINT_PUBLIC_CUSTODY"]) {
      process.env[key] = "inert-forbidden-value";
      try {
        assert.throws(
          () => installedHtmlPreflight(),
          /runtime override must be absent|ambient public HTML observer/u,
        );
      } finally {
        delete process.env[key];
      }
    }
  });
});

test("the Rust entry refuses overrides before Node can execute an actual preload", (t) => {
  const temporary = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "html-entry-guard-")));
  t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
  const entry = path.join(sourceRoot, "tests/tooling/support/oxlint-installed-html-acceptance.rs");
  const executable = path.join(temporary, process.platform === "win32" ? "guard.exe" : "guard");
  const compiled = spawnSync("rustc", ["--edition=2024", entry, "-o", executable], {
    encoding: "utf8",
    timeout: 30_000,
  });
  assert.equal(compiled.error, undefined);
  assert.equal(compiled.signal, null);
  assert.equal(compiled.status, 0, compiled.stdout + compiled.stderr);
  const marker = path.join(temporary, "node-preload-started");
  const preload = path.join(temporary, "preload.cjs");
  fs.writeFileSync(
    preload,
    `require('node:fs').writeFileSync(${JSON.stringify(marker)}, 'started')`,
  );
  withoutAmbientOverrides(() => {
    for (const key of [...runtimeOverrides, "VIZE_OXLINT_PUBLIC_CUSTODY"]) {
      const observed = spawnSync(executable, [process.execPath, "--print-campaign-authority"], {
        env: {
          ...process.env,
          RUST_SCRIPT_PATH: entry,
          [key]: key === "NODE_OPTIONS" ? `--require ${preload}` : "forbidden",
        },
        encoding: "utf8",
        timeout: 10_000,
      });
      assert.equal(observed.error, undefined);
      assert.equal(observed.signal, null);
      assert.equal(observed.status, 1);
      assert.equal(observed.stdout, "");
      assert.equal(observed.stderr.trim(), `initial source/runtime override must be empty: ${key}`);
      assert.equal(fs.existsSync(marker), false);
    }
  });
});

test("all four formats and original Standalone/mixed inputs survive the installed campaign unchanged", () => {
  assert.equal(
    fixtures.reduce((sum, fixture) => sum + (fixture.formats?.length ?? 4), 0),
    41,
  );
  assert.deepEqual(fixtures.find(({ name }) => name === "literal-mixed-originals")?.files, {
    "src/AppPanel.vue": "OriginalAppPanel.vue.txt",
    "src/Standalone.html": "OriginalStandalone.html.txt",
    "src/Live.html": "Warning.html",
    "dist/Built.html": "Warning.html",
    "vendor/Vendor.html": "Warning.html",
    "src/Skipped.html": "Warning.html",
  });
  const corpus = path.join(sourceRoot, "tests/_fixtures/differential/lint");
  for (const [copy, original] of [
    ["OriginalAppPanel.vue.txt", "oxlint-original-ignore-7903/AppPanel.vue.txt"],
    ["OriginalStandalone.html.txt", "oxlint-script-safe-carrier-7903/Standalone.html.txt"],
  ])
    assert.deepEqual(
      fs.readFileSync(path.join(corpus, "oxlint-original-html-operation-7903", copy)),
      fs.readFileSync(path.join(corpus, original)),
    );
  const capture = {
    schema: "inert",
    complete: false,
    source: null,
    observations: [],
    qualified: {
      cases: 41,
      formats: ["default", "json", "unix", "stylish"],
      wholeNativeCalls: 36,
      wholeHostPhases: 57,
    },
  };
  finishCapture(capture); // Counter law only; no successful actual campaign is written.
  assert.equal(capture.complete, true);
  capture.qualified.wholeHostPhases--;
  assert.throws(() => finishCapture(capture));
});

test("spawn failures retain every own provider error field and complete nested causes", () => {
  const cause = Object.assign(new Error("retained inner failure"), {
    errno: -2,
    syscall: "spawn",
    path: "/actual/provider",
  });
  const error = Object.assign(new Error("retained outer failure", { cause }), {
    code: "ENOENT",
    spawnargs: ["literal", "argv"],
  });
  const captured = wholeProcessError(error) as Record<string, unknown>;
  assert.deepEqual(
    Object.keys(captured).sort(),
    ["name", ...Object.getOwnPropertyNames(error)].sort(),
  );
  assert.equal(captured.message, error.message);
  assert.equal(captured.stack, error.stack);
  assert.deepEqual(captured.cause, {
    name: cause.name,
    ...Object.fromEntries(
      Object.getOwnPropertyNames(cause).map((key) => [
        key,
        (cause as unknown as Record<string, unknown>)[key],
      ]),
    ),
  });
  assert.deepEqual(captured.spawnargs, ["literal", "argv"]);
});

test("actual child status and byte streams survive invalid output and observer journals", (t) => {
  const directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "html-packet-laws-")));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  withoutAmbientOverrides(() => {
    const child = spawnSync(process.execPath, [
      "-e",
      "process.stdout.write(Buffer.from([0,255,10]));process.stderr.write('retained stderr');process.exit(23)",
    ]);
    assert.equal(child.error, undefined);
    assert.equal(child.signal, null);
    assert.equal(child.status, 23);
    for (const fault of ["parse", "utf8", "read", "stdout"]) {
      const events = path.join(directory, fault + ".events");
      const journal = path.join(directory, fault + ".native");
      if (fault === "read") fs.mkdirSync(events);
      else fs.writeFileSync(events, fault === "parse" ? "{unterminated\n" : '{"kind":"inert"}\n');
      const nativeBytes =
        fault === "utf8" ? Buffer.from([255, 10]) : Buffer.from("retained whole native\n");
      fs.writeFileSync(journal, nativeBytes);
      const capture = {
        schema: "inert",
        complete: false,
        source: null,
        observations: [],
        qualified: { cases: 0, formats: [], wholeNativeCalls: 0, wholeHostPhases: 0 },
      };
      const saved: unknown[][] = [];
      assert.throws(() =>
        retainHtmlProcess(
          capture,
          () => saved.push(structuredClone(capture.observations)),
          { fixture: fault },
          child,
          events,
          journal,
        ),
      );
      const first = saved[0][0] as Record<string, unknown>;
      assert.equal(first.pid, child.pid);
      assert.equal(first.status, 23);
      assert.deepEqual(first.stdoutBytes, [0, 255, 10]);
      assert.deepEqual(first.stderrBytes, Array.from(Buffer.from("retained stderr")));
      assert.equal(first.events, undefined);
      const last = saved.at(-1)?.[0] as Record<string, unknown>;
      if (fault === "read") assert.ok(last.journalReadFailure);
      else
        assert.deepEqual(
          (last.nativeJournal as { bytes: number[] }).bytes,
          Array.from(nativeBytes),
        );
      assert.equal(capture.complete, false);
    }
  });
});

test("the stock archive adapter rejects an empty identity before public archive access and retains its real failure", (t) => {
  const directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "html-stock-refusal-")));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  withoutAmbientOverrides(() => {
    const filename = path.join(directory, "refused.json");
    assert.throws(() => verifyStockArchives([], filename), /finite pinned stock wrapper/u);
    const actual = JSON.parse(fs.readFileSync(filename, "utf8"));
    assert.equal(actual.status, 1);
    assert.equal(actual.signal, null);
    assert.equal(actual.error, null);
    assert.deepEqual(actual.stdoutBytes, []);
    assert.match(Buffer.from(actual.stderrBytes).toString("utf8"), /finite pinned stock wrapper/u);
    assert.equal(actual.input, "[]");
    assert.match(actual.scope, /no installed Vize execution or semantic credit/u);
  });
});
