// Prospective public npm consumer. Source preparation never grants installed execution credit.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import {
  campaignAuthority,
  installedHtmlPreflight,
  sourceRoot,
  wholeProcessError,
} from "./oxlint-installed-html-authority.ts";
import { installedHtmlRuntime, pinnedInstalledProvider } from "./oxlint-installed-html-runtime.ts";
import { qualifyInstalledHtml } from "./oxlint-installed-html-qualification.ts";
import {
  fixtures,
  plainHostEnvironment,
  finishCapture,
} from "../../../npm/oxlint/src/test-support/html-cli-oracles.mjs";
import type { HtmlFixture, HtmlCapture, ExpectedPacket } from "./oxlint-installed-html-types.ts";

installedHtmlPreflight();
if (process.argv[2] === "--print-campaign-authority") {
  console.log(JSON.stringify(campaignAuthority(), null, 2));
} else {
  assert.ok(
    process.argv[2] && process.argv[3],
    "new output and reviewed installed HTML plan required",
  );
  const output = path.resolve(process.argv[2]);
  const runtime = installedHtmlRuntime(output, process.argv[3]);
  const corpus = path.join(
    sourceRoot,
    "tests/_fixtures/differential/lint/oxlint-original-html-operation-7903",
  );
  const packets = {
    ...JSON.parse(fs.readFileSync(path.join(corpus, "expected-packets.json"), "utf8")),
    ...JSON.parse(
      fs.readFileSync(
        path.join(sourceRoot, "npm/oxlint/src/test-support/html-cli-original-packets.json"),
        "utf8",
      ),
    ),
  } as Record<string, ExpectedPacket>;
  const environment = plainHostEnvironment(process.env);
  const summaries = [];
  for (const providerPlan of runtime.plan.providers) {
    const provider = pinnedInstalledProvider(providerPlan, output);
    const capture: HtmlCapture = {
      schema: "vize.oxlint.installed-html-public-v1",
      complete: false,
      source: { ...runtime.identity, provider: provider.identity },
      observations: [],
      qualified: { cases: 0, formats: [], wholeNativeCalls: 0, wholeHostPhases: 0 },
    };
    const capturePath = path.join(output, `original-html-${providerPlan.version}.json`);
    const save = () => fs.writeFileSync(capturePath, JSON.stringify(capture, null, 2) + "\n");
    save();
    for (const fixture of fixtures as unknown as HtmlFixture[])
      for (const format of fixture.formats ?? ["default", "json", "unix", "stylish"]) {
        const childEnvironment = {
          ...environment,
          ...(fixture.agent ? { AI_AGENT: "qualification" } : {}),
        };
        const directory = fs.mkdtempSync(path.join(runtime.workspace, "case-"));
        const root = path.join(directory, "repo"),
          temporary = path.join(directory, "transport");
        fs.mkdirSync(root);
        fs.mkdirSync(temporary);
        const setup = spawnSync("git", ["init", "-q"], { cwd: root });
        capture.observations.push({
          kind: "setup",
          fixture: fixture.name,
          format,
          root,
          command: "git",
          args: ["init", "-q"],
          status: setup.status,
          signal: setup.signal,
          error: setup.error ? wholeProcessError(setup.error) : null,
          stdoutBytes: Array.from(setup.stdout ?? []),
          stderrBytes: Array.from(setup.stderr ?? []),
        });
        save();
        assert.equal(setup.error, undefined);
        assert.equal(setup.signal, null);
        assert.equal(setup.status, 0);
        fs.writeFileSync(path.join(root, ".git/info/exclude"), "");
        fs.mkdirSync(path.join(root, "node_modules/oxlint/bin"), { recursive: true });
        fs.symlinkSync(provider.engine, path.join(root, "node_modules/oxlint/bin/oxlint"));
        // Bare plugin resolution reaches the real consumer ancestor. No workspace package link is created.
        fs.writeFileSync(path.join(root, "package.json"), '{"private":true,"type":"module"}\n');
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
        const invoke = (publicCli: boolean) =>
          runtime.run(
            capture,
            save,
            provider,
            fixture.name,
            format,
            root,
            temporary,
            args,
            childEnvironment,
            publicCli,
            !fixture.refused,
          );
        const stock = invoke(false),
          wrapper = invoke(true);
        qualifyInstalledHtml(
          capture,
          fixture,
          format,
          root,
          providerPlan.version,
          stock,
          wrapper,
          packets,
        );
        save();
      }
    capture.qualified.formats = ["default", "json", "unix", "stylish"];
    finishCapture(capture);
    provider.recheck();
    runtime.recheck();
    save();
    summaries.push({
      version: providerPlan.version,
      capturePath,
      complete: capture.complete,
      qualified: capture.qualified,
    });
  }
  runtime.recheck();
  fs.writeFileSync(
    path.join(output, "summary.json"),
    JSON.stringify(
      {
        schema: "vize.oxlint.installed-html-summary-v1",
        identity: runtime.identity,
        success: true,
        profiles: summaries,
        qualification:
          "bounded original HTML four-format installed CLI; no full oxlint/n8n adoption or performance claim",
      },
      null,
      2,
    ) + "\n",
  );
}
