import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const generated = path.join(root, "crates/vize_patina/src/lent_walk/generated");

test("the lending traversal follows the locked OXC revision and generated output hashes", () => {
  const provenance = JSON.parse(
    fs.readFileSync(path.join(generated, "provenance.json"), "utf8"),
  ) as {
    revision: string;
    outputs: Record<string, string>;
  };
  const manifest = fs.readFileSync(path.join(root, "Cargo.toml"), "utf8");
  const selected = manifest.match(/^oxc_ast_visit = \{[^\n]*rev = "([a-f0-9]{40})"[^\n]*\}$/mu);
  assert.equal(provenance.revision, selected?.[1], "regenerate the walk when OXC changes");
  assert.deepEqual(
    fs
      .readdirSync(generated)
      .filter((file) => file !== "provenance.json")
      .sort(),
    Object.keys(provenance.outputs).sort(),
  );
  for (const [file, expected] of Object.entries(provenance.outputs)) {
    const bytes = fs.readFileSync(path.join(generated, file));
    assert.equal(createHash("sha256").update(bytes).digest("hex"), expected, file);
  }
});
