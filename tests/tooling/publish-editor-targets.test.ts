import assert from "node:assert/strict";
import fs from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { runReleaseCommand } from "./support/release-command.ts";
import { writeFakeCommand } from "./support/fake-command.ts";
import { galleryWith } from "./support/marketplace-gallery.ts";

const targets = [
  "darwin-x64",
  "darwin-arm64",
  "win32-x64",
  "win32-arm64",
  "linux-x64",
  "linux-arm64",
  "alpine-x64",
  "alpine-arm64",
];
const scripts = ["publish_vscode_extension", "publish_open_vsx_extension"];
function observe(t: TestContext, script: string, target: string, mode = "publish") {
  const root = fs.mkdtempSync(path.join(tmpdir(), "editor-target-")),
    bin = path.join(root, "bin");
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.mkdirSync(bin);
  const manifest = path.join(root, "package.json"),
    archive = path.join(root, "vize.vsix");
  fs.writeFileSync(
    manifest,
    JSON.stringify({ publisher: "ubugeeei", name: "vize", version: "0.57.0" }),
  );
  fs.writeFileSync(archive, "synthetic publisher unit input");
  const body = (tool: string) =>
    [
      "const fs=require('node:fs');const args=process.argv.slice(2);",
      "const file=process.env.STATE;const state=fs.existsSync(file)?JSON.parse(fs.readFileSync(file)): {calls:[],published:false};",
      `state.calls.push([${JSON.stringify(tool)},...args]);`,
      "const expected=process.env.TARGET,mode=process.env.MODE;",
      `const probe=${tool === "curl"};`,
      "if(probe){",
      " const other=expected==='darwin-x64'?'linux-x64':'darwin-x64';",
      " const selected=mode==='exact'||(state.published&&mode!=='wrong-after')?expected:other;",
      " const version=!state.published&&mode==='wrong-version'?'0.56.0':'0.57.0';",
      " const row={version,targetPlatform:selected,flags:'validated'};",
      " if(!state.published&&mode==='universal')delete row.targetPlatform;",
      // Marketplace probes POST to the gallery; Open VSX GETs one version.
      ` const gallery=${JSON.stringify(galleryWith([]))};gallery.results[0].extensions[0].versions=[row];`,
      " process.stdout.write(JSON.stringify(args.at(-1).includes('marketplace')?gallery:row));",
      "}else if(args[4]==='publish'){state.published=true;}else if(args[4]!=='create-namespace'){process.exitCode=19;}",
      "fs.writeFileSync(file,JSON.stringify(state));",
    ].join("\n");
  writeFakeCommand(bin, "vp", body("vp"));
  writeFakeCommand(bin, "curl", body("curl"));
  const state = path.join(root, "state.json");
  const result = runReleaseCommand(script, [archive, manifest], {
    env: {
      PATH: [bin, path.dirname(process.execPath), process.env.PATH].join(path.delimiter),
      VP_BIN: "vp",
      VSCE_DLX_BIN: "vp",
      VIZE_EXTENSION_TARGET: target,
      TARGET: target,
      MODE: mode,
      STATE: state,
      PUBLISH_RESOLUTION_RETRY_LIMIT: "1",
      PUBLISH_RESOLUTION_RETRY_DELAY: "0",
    },
  });
  const calls = fs.existsSync(state)
    ? (JSON.parse(fs.readFileSync(state, "utf8")).calls as string[][])
    : [];
  return { result, calls };
}
for (const script of scripts) {
  for (const target of targets)
    test(`${script} does not skip ${target} because another target is published`, (t) => {
      const { result, calls } = observe(t, script, target);
      assert.equal(result.status, 0, result.stdout + result.stderr);
      assert.equal(calls.filter((c) => c[5] === "publish").length, 1);
      if (script === "publish_open_vsx_extension")
        assert.ok(
          calls
            .filter((c) => c[0] === "curl")
            .every((c) => c.at(-1) === `https://open-vsx.org/api/ubugeeei/vize/${target}/0.57.0`),
        );
    });
  for (const mode of ["universal", "wrong-version"])
    test(`${script} refuses ${mode} metadata as an existing platform publication`, (t) => {
      const { result, calls } = observe(t, script, "linux-arm64", mode);
      assert.equal(result.status, 0, result.stdout + result.stderr);
      assert.equal(calls.filter((c) => c[5] === "publish").length, 1);
    });
  test(`${script} skips only the exact version and target`, (t) => {
    const { result, calls } = observe(t, script, "linux-arm64", "exact");
    assert.equal(result.status, 0, result.stdout + result.stderr);
    assert.equal(calls.filter((c) => c[5] === "publish").length, 0);
  });
  test(`${script} rejects a zero-exit upload while only another target becomes visible`, (t) => {
    const { result } = observe(t, script, "linux-arm64", "wrong-after");
    assert.equal(result.status, 1);
  });
  test(`${script} rejects an unknown target before any registry or publisher runs`, (t) => {
    const { result, calls } = observe(t, script, "unknown-target");
    assert.equal(result.status, 1);
    assert.deepEqual(calls, []);
  });
}
