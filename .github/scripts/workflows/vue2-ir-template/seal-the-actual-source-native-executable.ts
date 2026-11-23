import { requiredEnv } from "../shared/files.ts";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFileSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";
const source = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
assert.equal(source, process.env.SOURCE_SHA);
const file = join(requiredEnv("CARGO_TARGET_DIR"), "ci/examples/vue2_ir_template");
const hash = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const files = execFileSync(
  "git",
  [
    "ls-files",
    "--",
    "davinci/vize_l4/src/expr/resolved.rs",
    "Cargo.lock",
    "Cargo.toml",
    "davinci/vize_l1_to_l2/src/lib.rs",
    "davinci/vize_l1_to_l2/src/native_vue2.rs",
    "davinci/vize_l1_to_l2/src/native_vue2",
    "davinci/vize_l1_to_l2/src/dialect/vue2.rs",
    "davinci/vize_l1_to_l2/src/dialect/vue2",
    "davinci/vize_l4/src/targets.rs",
    "davinci/vize_l4/src/targets/vue2.rs",
    "davinci/vize_l4/src/targets/vue2",
  ],
  { encoding: "utf8" },
)
  .trim()
  .split("\n");
writeFileSync(
  "target/vize-tests/vue2-ir/producer.json",
  JSON.stringify(
    {
      source,
      tree: execFileSync("git", ["rev-parse", "HEAD^{tree}"], { encoding: "utf8" }).trim(),
      executable: { file, bytes: statSync(file).size, sha256: hash(readFileSync(file)) },
      compiler: execFileSync("rustc", ["--version"], { encoding: "utf8" }).trim(),
      consumerRecipe: {
        source: process.env.RECIPE_SHA,
        files: Object.fromEntries(
          [
            "davinci/vize_l4/examples/vue2_ir_template.rs",
            "davinci/vize_l4/examples/vue2_ir_template_lexical/mod.rs",
            "tests/tooling/native-vue2-ir-template.test.mjs",
            "tests/tooling/native-vue2-ir-lexical.test.mjs",
            "tests/tooling/native-vue2-ir-vnode.mjs",
          ].map((file) => [file, hash(readFileSync(file))]),
        ),
      },
      sourceFiles: Object.fromEntries(files.map((file) => [file, hash(readFileSync(file))])),
      runId: process.env.GITHUB_RUN_ID,
    },
    null,
    2,
  ) + "\n",
);
