import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  compiledTarget,
  listUshPrograms,
  pinnedUshVersion,
  runtimeDependentBuilds,
  syncUshPrograms,
} from "../../tools/support/ci/ush-programs.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function repository(files: Record<string, string>): string {
  const directory = mkdtempSync(path.join(tmpdir(), "vize-ush-programs-"));
  execFileSync("git", ["init", "--quiet"], { cwd: directory });
  for (const [file, text] of Object.entries(files)) {
    mkdirSync(path.dirname(path.join(directory, file)), { recursive: true });
    writeFileSync(path.join(directory, file), text);
  }
  return directory;
}

test("a program's build sits in compiled/ beside it", () => {
  assert.equal(
    compiledTarget("tools/ci/workflows/11-docs-build/scripts/build-docs.ush"),
    "tools/ci/workflows/11-docs-build/scripts/compiled/build-docs.sh",
  );
});

test("modules under lib/ are not programs", () => {
  const directory = repository({
    "tools/ci/lib/github.ush": "",
    "tools/ci/workflows/01-check/scripts/plan.ush": "",
  });
  assert.deepEqual(listUshPrograms(directory), ["tools/ci/workflows/01-check/scripts/plan.ush"]);
});

test("a build whose program is gone is reported before ush runs", () => {
  const directory = repository({ "scripts/compiled/renamed.sh": "#!/bin/sh\n" });
  assert.throws(() => syncUshPrograms(directory, true), /scripts\/compiled\/renamed\.sh/u);
});

test("a build that calls the ush runtime is found", () => {
  const directory = repository({
    "scripts/compiled/portable.sh": "#!/bin/sh\nprintf '%s\\n' ok\n",
    "scripts/compiled/runtime.sh":
      '#!/bin/sh\n__ush_std_host() {\n  "${USH_RUNTIME:-ush}" "$@"\n}\n',
  });
  assert.deepEqual(
    runtimeDependentBuilds(directory, ["scripts/portable.ush", "scripts/runtime.ush"]),
    ["scripts/compiled/runtime.sh"],
  );
});

test("no committed build calls the ush runtime", () => {
  assert.deepEqual(runtimeDependentBuilds(root, listUshPrograms(root)), []);
});

test("the flake input pins the ush release", () => {
  const directory = repository({
    "flake.nix": 'ush = {\n  url = "github:ubugeeei-prod/ush/v1.2.3";\n};\n',
  });
  assert.equal(pinnedUshVersion(directory), "1.2.3");
  const unpinned = repository({ "flake.nix": 'url = "github:ubugeeei-prod/ush";\n' });
  assert.throws(() => pinnedUshVersion(unpinned), /release tag/u);
});

test("every committed program has its build", () => {
  for (const source of listUshPrograms(root)) {
    assert.ok(existsSync(path.join(root, compiledTarget(source))), `missing build of ${source}`);
  }
});
