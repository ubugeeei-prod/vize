import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

test("staged drivers survive replacing the product checkout and refuse links outside the script tree", (t) => {
  const root = mkdtempSync(join(tmpdir(), "vize-workflow-stage-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const workspace = join(root, "workspace");
  const temp = join(root, "runner");
  mkdirSync(temp);
  const environment = join(root, "github-env");
  const checkout = join(workspace, ".vize-workflow-tools");
  const script = join(checkout, ".github/actions/stage-workflow-scripts/stage.ts");
  const programs = {
    acceptance: join(checkout, "tools/ci/workflows/09-acceptance/scripts"),
    shared: join(checkout, "tools/ci/lib/scripts"),
  };
  function prepare() {
    mkdirSync(join(checkout, ".github/actions/stage-workflow-scripts"), { recursive: true });
    for (const [name, directory] of Object.entries(programs)) {
      mkdirSync(directory, { recursive: true });
      writeFileSync(join(directory, "receipt.ts"), name);
    }
    copyFileSync(
      new URL("../../.github/actions/stage-workflow-scripts/stage.ts", import.meta.url),
      script,
    );
  }
  const run = () =>
    spawnSync(process.execPath, [script], {
      env: {
        ...process.env,
        GITHUB_WORKSPACE: workspace,
        RUNNER_TEMP: temp,
        GITHUB_ENV: environment,
      },
      encoding: "utf8",
    });
  prepare();
  const result = run();
  assert.equal(result.status, 0, result.stderr);
  assert.equal(existsSync(checkout), false);
  const output = readFileSync(environment, "utf8");
  assert.match(output, /^VIZE_CI_FILES=[^\r\n]+\n$/u);
  const directory = output.trimEnd().slice("VIZE_CI_FILES=".length);
  assert.equal(readFileSync(join(directory, "receipt.ts"), "utf8"), "acceptance");
  // The staged copy keeps the repository layout, so shared programs keep their relative path.
  assert.equal(readFileSync(join(directory, "../../../lib/scripts/receipt.ts"), "utf8"), "shared");
  if (process.platform !== "win32") {
    prepare();
    symlinkSync(environment, join(programs.acceptance, "foreign"));
    const refused = run();
    assert.notEqual(refused.status, 0);
    assert.match(refused.stderr, /regular file/u);
    assert.equal(readFileSync(environment, "utf8"), output);
  }
});
