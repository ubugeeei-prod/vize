import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  commentStyle,
  withGeneratedHeader,
  withoutGeneratedHeader,
} from "../../tools/support/ci/generated-header.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const regenerate = "node tools/commands/generate/repository-config.ts";

test("generated YAML opens with the do-not-edit notice for its Pkl source", () => {
  const text = withGeneratedHeader("name: Check\n", "hash", "tools/config/pkl/workflows/x.pkl");

  assert.equal(
    text,
    "# DO NOT EDIT. Generated from tools/config/pkl/workflows/x.pkl.\n" +
      `# Edit the Pkl source and run \`${regenerate}\`.\n` +
      "name: Check\n",
  );
});

test("the notice follows an interpreter line and stays silent in batch files", () => {
  const shell = withGeneratedHeader("#!/usr/bin/env bash\necho hi\n", "hash", "a.sh.pkl");
  const batch = withGeneratedHeader("@echo off\r\nexit /b 0\r\n", "batch", "a.cmd.pkl");

  assert.equal(
    shell,
    "#!/usr/bin/env bash\n# DO NOT EDIT. Generated from a.sh.pkl.\n" +
      `# Edit the Pkl source and run \`${regenerate}\`.\necho hi\n`,
  );
  assert.equal(
    batch,
    "@REM DO NOT EDIT. Generated from a.cmd.pkl.\n" +
      `@REM Edit the Pkl source and run \`${regenerate}\`.\n@echo off\r\nexit /b 0\r\n`,
  );
});

test("removing the notice restores the body even after the source moves", () => {
  const body = "#!/usr/bin/env bash\necho hi\n";
  const text = withGeneratedHeader(body, "hash", "old/place.sh.pkl");

  assert.equal(withoutGeneratedHeader(text, "hash"), body);
  assert.equal(withoutGeneratedHeader(body, "hash"), body);
});

test("comment style follows the output's language", () => {
  assert.deepEqual(
    ["a.yml", "a.sh", "a.ps1", ".envrc", "a.cmd", "scripts/package-tasks.cjs", "README.md"].map(
      commentStyle,
    ),
    ["hash", "hash", "hash", "hash", "batch", "slash", "html"],
  );
});

test("checked-in outputs carry the notice", () => {
  const workflow = readFileSync(path.join(root, ".github/workflows/check.yml"), "utf8");
  const catalog = readFileSync(path.join(root, ".github/workflows/README.md"), "utf8");

  assert.equal(
    workflow.split("\n")[0],
    "# DO NOT EDIT. Generated from tools/config/pkl/workflows/checks/check.pkl.",
  );
  assert.equal(
    catalog.split("\n")[0],
    "<!-- DO NOT EDIT. Generated from tools/config/pkl/workflows. -->",
  );
});
