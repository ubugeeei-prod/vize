import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { repoRoot } from "../_helpers/realworld-patch.ts";
import { resolveTsgoBinary } from "../_helpers/realworld-typecheck.ts";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";
import { previewPage, readAuthority } from "./support/router-preview/authority.ts";
import { prepareProject } from "./support/router-preview/project.ts";
import { verifyBuiltProvider } from "./support/router-preview/provider.ts";
import { readOracles, verifyControl } from "./support/router-preview/oracles.ts";
import type { Observation } from "./support/router-preview/processes.ts";

assert.equal(process.env.GITHUB_ACTIONS, "true", "the real preview provider runs in Actions");
const receiptSha = process.env.VIZE_ROUTER_PREVIEW_RECEIPT_SHA256;
assert.ok(receiptSha, "same-run provider receipt is required");
const providerDirectory = process.env.VIZE_ROUTER_PREVIEW_PROVIDER;
assert.ok(providerDirectory, "actual source-built provider artifact is required");
const output = path.join(repoRoot, "target/differential/router-preview-593d7ad");
fs.mkdirSync(output, { recursive: true });
const oracles = readOracles();
const provider = verifyBuiltProvider(providerDirectory, receiptSha);
const sourceBinary = process.env.VIZE_TEST_BIN ?? process.env.VIZE_LSP_BIN;
assert.ok(sourceBinary, "receipted source CLI required");
const [source] = resolveVizeLaunchCommand(undefined, sourceBinary, {
  required: true,
  repoRoot,
  onVerifiedLaunch(launch) {
    fs.writeFileSync(
      path.join(output, "source-cli-launch.json"),
      `${JSON.stringify(launch, null, 2)}\n`,
    );
  },
});
const fixture = prepareProject(output, provider);
const page = path.join(fixture.root, previewPage);
const basePage = fixture.pinned.files.get(previewPage)!.toString("utf8");
const baseConfig = JSON.parse(fixture.pinned.files.get("tsconfig.json")!.toString("utf8"));
const controls: Array<{ id: string; source: string; invalid: boolean }> = [];
const summaries: unknown[] = [];
const captures: Array<{
  id: string;
  source: string;
  reference: Observation;
  publicControl: Observation;
  native: Observation;
}> = [];
try {
  for (const [newlineId, newline] of [
    ["lf", "\n"],
    ["crlf", "\r\n"],
  ] as const) {
    const valid = basePage.replaceAll("\n", newline);
    const phases = [
      { name: "valid", source: valid, invalid: false },
      {
        name: "wrong-id",
        source: valid.replace("const id: number", "const id: string"),
        invalid: true,
      },
      {
        name: "wrong-date",
        source: valid.replace("const at: Date | undefined", "const at: string | undefined"),
        invalid: true,
      },
      {
        name: "missing-component-prop",
        source: valid.replace(
          "defineProps<_RoutePropsForFilePath<PageFile>>()",
          "defineProps<_RoutePropsForFilePath<PageFile> & { requiredExtra: string }>()",
        ),
        invalid: true,
      },
      { name: "repaired", source: valid, invalid: false },
    ];
    for (const enabled of [true, false]) {
      const config = structuredClone(baseConfig);
      config.vueCompilerOptions.plugins = enabled ? ["vue-router/volar/sfc-typed-router"] : [];
      fs.writeFileSync(
        path.join(fixture.root, "tsconfig.json"),
        `${JSON.stringify(config, null, 2)}\n`,
      );
      for (const phase of phases) {
        const id = `${newlineId}/${enabled ? "enabled" : "disabled"}/${phase.name}`;
        fs.writeFileSync(page, phase.source);
        controls.push({ id, source: phase.source, invalid: phase.invalid });
        fs.writeFileSync(
          path.join(output, "controls.json"),
          `${JSON.stringify(controls, null, 2)}\n`,
        );
        const reference = fixture.processes.run(
          `${id}/reference`,
          process.execPath,
          [
            path.join(fixture.root, "node_modules/vue-tsc/bin/vue-tsc.js"),
            "--noEmit",
            "--pretty",
            "false",
            "-p",
            "tsconfig.json",
          ],
          fixture.root,
        );
        const publicControl = fixture.processes.run(
          `${id}/public-control`,
          process.execPath,
          [
            path.join(fixture.root, "node_modules/vize/bin/vize"),
            "check",
            "src",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
            "--quiet",
            "--corsa-path",
            resolveTsgoBinary(),
          ],
          fixture.root,
        );
        const native = fixture.processes.run(
          `${id}/source`,
          source,
          [
            "check",
            "src",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
            "--quiet",
            "--corsa-path",
            resolveTsgoBinary(),
          ],
          fixture.root,
        );
        captures.push({ id, source: phase.source, reference, publicControl, native });
        assert.deepEqual(
          fs.readFileSync(path.join(fixture.root, "typed-router.d.ts")),
          fixture.generated,
          "mutations cannot regenerate away the authored error",
        );
        summaries.push({
          id,
          expectedInvalid: phase.invalid,
          reference: reference.status,
          publicControl: publicControl.status,
          source: native.status,
        });
        fs.writeFileSync(
          path.join(output, "observed-statuses.json"),
          `${JSON.stringify(summaries, null, 2)}\n`,
        );
      }
    }
  }
  const authority = readAuthority();
  const result = {
    schema: "vize.router-preview-probe",
    version: 1,
    providerReceiptSha256: provider.receiptSha256,
    sourceRevision: process.env.GITHUB_SHA,
    state: "discovery-only",
    qualificationCredit: false,
    automaticProjection: "not-qualified",
    provider: authority.authority.provider,
    oracleAuthoritySha256: oracles.sha256,
    qualifiedHosts: [] as string[],
    summaries,
  };
  fs.writeFileSync(path.join(output, "result.json"), `${JSON.stringify(result, null, 2)}\n`);
  assert.deepEqual(controls, oracles.controls, "entire control denominator and authored bytes");
  assert.equal(captures.length * 3, oracles.authority.scope.processes);
  for (const capture of captures) {
    verifyControl(oracles, capture.id, capture.source, capture, fixture.root);
  }
  result.state = "qualified-manually-explicit-helpers";
  result.qualificationCredit = true;
  result.qualifiedHosts = ["check"];
  fs.writeFileSync(path.join(output, "result.json"), `${JSON.stringify(result, null, 2)}\n`);
} finally {
  fixture.cleanup();
}
