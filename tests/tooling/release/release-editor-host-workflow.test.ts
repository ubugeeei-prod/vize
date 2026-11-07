import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import {
  prepareReleaseZedProfile,
  writeReleaseZedSettings,
} from "../../editor-conformance/release-zed-settings.ts";
import { readRepoFile } from "../support/github-workflows.ts";

type Step = {
  id?: string;
  name?: string;
  if?: string;
  run?: string;
  uses?: string;
  with?: Record<string, unknown>;
};
type Job = {
  needs?: string[];
  if?: string;
  "timeout-minutes"?: number;
  steps?: Step[];
};

const workflow = parse(readRepoFile(".github", "workflows", "release.yml")) as {
  jobs: Record<string, Job>;
};
const editorHostAction = readRepoFile(".github", "actions", "vscode-host-smoke", "action.yml");
const releaseZed = readRepoFile("tests", "editor-conformance", "release-zed.ts");
const releaseZedHost = readRepoFile(".github", "workflows", "release-zed-host.yml");

test("release editor hosts consume the packed candidate before the tag exists", () => {
  const job = workflow.jobs["smoke-editor-artifacts"]!;
  assert.deepEqual(job.needs, [
    "version",
    "build-cli",
    "build-packages",
    "build-editor-extensions",
  ]);
  assert.equal(job.if, "needs.version.outputs.ref != ''");
  assert.ok((job["timeout-minutes"] ?? 0) >= 45);
  const node = job.steps?.find((step) => step.uses?.startsWith("actions/setup-node@"));
  assert.equal(node?.with?.["node-version-file"], "package.json");
  assert.equal(node?.with?.["package-manager-cache"], false);
  const serialized = JSON.stringify(job.steps);
  assert.match(serialized, /"name":"cli-x86_64-unknown-linux-gnu"/);
  assert.match(serialized, /"name":"editor-extensions"/);
  assert.match(serialized, /"name":"npm-packages"/);
  assert.match(serialized, /vize-x86_64-unknown-linux-gnu\.tar\.gz/);
  assert.match(serialized, /vize \$RELEASE_VERSION/);
  assert.match(serialized, /for editor in zed nvim vim helix emacs/);
  assert.match(serialized, /target\/source-editors\/\$editor/);
  assert.match(serialized, /\$\{editor\}-vize-extension\.tar\.gz/);
  const install = job.steps?.find((step) => step.id === "artifacts")?.run ?? "";
  assert.match(install, /node "\$packaged_cli" init [\s\S]*--typecheck --editor --no-install/);
  assert.match(install, /"typescript-native": "npm:typescript@7\.1\.0-beta"/);
  assert.match(install, /typescript-native@npm:typescript@7\.1\.0-beta/);
  assert.match(install, /typescript-native\/lib\/getExePath\.js/);
  assert.match(install, /test -x "\$native_checker"/);
  assert.match(install, /fresh_real=\$\(readlink -f "\$fresh"\)/);
  assert.match(install, /native checker escapes the fresh project/);
  assert.match(serialized, /VIZE_TS45_EXISTING_WORKSPACE/);
  for (const client of ["neovim", "helix", "vscode"]) {
    assert.match(serialized, new RegExp(`tests/editor-conformance/run\\.ts ${client}`));
  }
  assert.match(serialized, /project-links\.txt/);
  assert.match(serialized, /project-links-outside-node-modules\.txt/);
  assert.match(serialized, /project-links-after-hosts-outside-node-modules\.txt/);
  assert.equal(serialized.match(/-path \\"\$[A-Za-z0-9_]+\/node_modules\\" -prune/g)?.length, 2);
  assert.match(serialized, /dependency-links\.txt/);
  assert.match(serialized, /dependency-links-after-hosts\.txt/);
  assert.equal(serialized.match(/target=\$\(readlink -f \\"\$link\\"\)/g)?.length, 2);
  assert.equal(serialized.match(/node_modules\\"\/\*/g)?.length, 3);
  assert.match(serialized, /workspace-links\.txt/);
  assert.match(serialized, /vue\/require-v-for-key/);
  const host = job.steps?.find((step) => step.uses === "./.github/actions/vscode-host-smoke");
  assert.equal(host?.with?.["prebuilt-server"], "${{ steps.artifacts.outputs.server }}");
  assert.equal(host?.with?.["prebuilt-vsix"], "${{ steps.artifacts.outputs.vsix }}");
  assert.equal(host?.with?.["expected-version"], "${{ needs.version.outputs.version }}");
  assert.equal(host?.with?.["expected-source-sha"], "${{ needs.version.outputs.sha }}");
  assert.equal(host?.with?.["release-workspace"], "${{ steps.artifacts.outputs.fresh }}");
  assert.equal(host?.with?.["release-evidence"], "${{ steps.artifacts.outputs.evidence }}");
});

test("release editor hosts install the supplied CLI and VSIX without rebuilding them", () => {
  const action = parse(editorHostAction) as {
    inputs: Record<string, unknown>;
    runs: { steps: Step[] };
  };
  assert.ok(action.inputs["prebuilt-server"]);
  assert.ok(action.inputs["prebuilt-vsix"]);
  assert.ok(action.inputs["expected-source-sha"]);
  const build = action.runs.steps.find((step) =>
    step.run?.includes("cargo build --profile ci -p vize"),
  );
  assert.equal(build?.if, "inputs.prebuilt-server == ''");
  assert.match(editorHostAction, /install -m 0755 "\$PREBUILT_SERVER" target\/ci\/vize/);
  assert.match(editorHostAction, /target\/ci\/vize --version[\s\S]*vize \$EXPECTED_VERSION/);
  assert.match(
    editorHostAction,
    /install -m 0644 "\$PREBUILT_VSIX" editors\/vscode\/dist\/vize\.vsix/,
  );
  assert.match(editorHostAction, /vp node editors\/vscode\/test\/run-extension-host-real\.mjs/);
  assert.match(editorHostAction, /zed-linux-x86_64\.tar\.gz/);
  assert.match(editorHostAction, /node tests\/editor-conformance\/release-zed\.ts/);
  const zed = action.runs.steps.find(
    (step) => step.name === "Run actual Zed with the packaged release extension",
  );
  assert.equal(zed?.if, "inputs.release-workspace != ''");
});

test("release Zed settings trust the isolated worktree and opt Vize into Vue", (t) => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-zed-settings-"));
  t.after(() => fs.rmSync(temporary, { force: true, recursive: true }));
  const config = path.join(temporary, "config");
  const evidence = path.join(temporary, "evidence");
  writeReleaseZedSettings(config, evidence);
  const settings = JSON.parse(fs.readFileSync(path.join(config, "settings.json"), "utf8"));

  assert.equal(settings.session?.trust_all_worktrees, true);
  assert.deepEqual(settings.languages?.["Vue.js"]?.language_servers, ["vize", "..."]);
  assert.equal(settings.auto_install_extensions?.vue, false);
  assert.equal(settings.auto_update_extensions?.vue, false);
  assert.deepEqual(
    JSON.parse(fs.readFileSync(path.join(evidence, "zed-settings.json"), "utf8")),
    settings,
  );
});

test("release Zed profile changes run in the actual pinned editor before merge", () => {
  const host = parse(releaseZedHost) as {
    on: { pull_request: { paths: string[] } };
    jobs: Record<string, Job>;
  };
  assert.ok(host.on.pull_request.paths.includes("tests/editor-conformance/release-zed*"));
  const serialized = JSON.stringify(host.jobs.zed.steps);
  assert.match(serialized, /cargo build --profile ci -p vize/);
  assert.match(serialized, /zed-linux-x86_64\.tar\.gz/);
  assert.match(serialized, /node tests\/editor-conformance\/release-zed\.ts/);
});

test("release Zed smoke prepares its packaged profile and retains failure evidence", (t) => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-zed-profile-"));
  t.after(() => fs.rmSync(temporary, { force: true, recursive: true }));
  const packaged = path.join(temporary, "packaged");
  const evidence = path.join(temporary, "evidence");
  fs.mkdirSync(path.join(packaged, "grammars"), { recursive: true });
  fs.mkdirSync(path.join(packaged, "languages", "art-vue"), { recursive: true });
  fs.writeFileSync(
    path.join(packaged, "extension.toml"),
    [
      'id = "vize"',
      "",
      "[grammars.vue]",
      'repository = "https://github.com/tree-sitter-grammars/tree-sitter-vue"',
      'rev = "fixture"',
      "",
    ].join("\n"),
  );
  fs.writeFileSync(path.join(packaged, "grammars", "vue.wasm"), "fixture grammar");
  fs.writeFileSync(
    path.join(packaged, "languages", "art-vue", "config.toml"),
    'name = "Art Vue"\npath_suffixes = ["art.vue"]\n',
  );

  const profile = prepareReleaseZedProfile(path.join(temporary, "session"), packaged, evidence);
  const generated = JSON.parse(
    fs.readFileSync(path.join(profile.userData, "config", "settings.json"), "utf8"),
  );
  assert.equal(generated.session?.trust_all_worktrees, true);
  assert.equal(
    fs.readFileSync(path.join(profile.installed, "extension.toml"), "utf8"),
    [
      'id = "vize"',
      "",
      "[grammars.vue]",
      'repository = "https://github.com/tree-sitter-grammars/tree-sitter-vue"',
      'rev = "fixture"',
      "",
    ].join("\n"),
  );
  assert.ok(fs.statSync(profile.shim).isDirectory());
  assert.ok(fs.existsSync(path.join(profile.userData, "extensions", "installed", ".rescan")));
  assert.equal(
    fs.readFileSync(path.join(profile.vueLanguage, "languages", "vue", "config.toml"), "utf8"),
    'name = "Vue.js"\npath_suffixes = ["vue"]\n',
  );
  assert.match(
    fs.readFileSync(path.join(profile.vueLanguage, "extension.toml"), "utf8"),
    /^id = "vue"[\s\S]*^languages = \["languages\/vue"\]$/mu,
  );
  assert.equal(
    fs.readFileSync(path.join(profile.vueLanguage, "grammars", "vue.wasm"), "utf8"),
    "fixture grammar",
  );
  assert.deepEqual(
    JSON.parse(fs.readFileSync(path.join(evidence, "zed-settings.json"), "utf8")),
    generated,
  );
  assert.match(releaseZed, /if \(!passed\) retainFailureSession\(session, evidence\)/);
  assert.match(releaseZed, /path\.join\(evidence, "failure-session"\)/);
});
