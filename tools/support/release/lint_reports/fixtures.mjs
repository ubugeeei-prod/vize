import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { apiRoot, gitBlob, sha256 } from "../typed_defaults/custody.mjs";

export const templateDirectory =
  "crates/vize_patina/tests/fixtures/unsafe-template-binding/reports-8752-8755";
export const computedDirectory =
  "tests/_fixtures/differential/lint-regressions/computed-local-reactivity-8756";
export const pins = JSON.parse(fs.readFileSync(new URL("./pins.json", import.meta.url), "utf8"));

export function scenarios(source) {
  const read = (name) => source.get(name).toString("utf8");
  const vectors = [];
  for (const role of [
    "scoped-semantic-oracles.json",
    "owner-type-supplement.json",
    "original-config-oracles.json",
  ]) {
    for (const entry of JSON.parse(read(`${templateDirectory}/${role}`)).cases) {
      const original = role === "original-config-oracles.json";
      const folder = original || entry.id === "RouterViewSlot" ? "original-reports" : "cases";
      assert.equal(read(`${templateDirectory}/${folder}/${entry.filename}`), entry.source);
      assert.equal(sha256(Buffer.from(entry.source)), entry.sourceSha256);
      const config = original ? entry.reportedConfig.source : JSON.stringify(entry.config) + "\n";
      const tsconfig = read(`${templateDirectory}/original-reports/tsconfig.json`);
      if (original) {
        assert.equal(config, read(`${templateDirectory}/original-reports/vize.config.json`));
        assert.equal(tsconfig, entry.reportedTsconfig.source);
      }
      vectors.push({ suite: "template", role, ...entry, config, tsconfig });
    }
  }
  assert.equal(vectors.length, 29);
  assert.equal(vectors.filter((v) => v.role === "original-config-oracles.json").length, 4);
  const manifest = JSON.parse(read(`${computedDirectory}/source.json`));
  for (const [name, pin] of Object.entries(manifest.files)) {
    const bytes = source.get(`${computedDirectory}/${name}`);
    assert.equal(bytes.length, pin.bytes);
    assert.equal(sha256(bytes), pin.sha256);
  }
  const original = read(`${computedDirectory}/ComputedLocal.vue.txt`);
  const issue = read(`${computedDirectory}/original-issue.md`);
  assert.equal(issue.match(/```vue\n([\s\S]*?)```/)[1], original);
  const controls = JSON.parse(read(`${computedDirectory}/controls.json`));
  assert.equal(controls.length, 35);
  for (const entry of [
    { id: "original", source: original, expectedSnapshotAndSpreadWarnings: 0 },
    ...controls,
  ]) {
    vectors.push({
      suite: "computed",
      ...entry,
      filename: "ComputedLocal.vue",
      config: read(`${computedDirectory}/vize.config.json.txt`),
      tsconfig: read(`${computedDirectory}/tsconfig.json.txt`),
    });
  }
  assert.equal(vectors.length, 65);
  return vectors;
}

export async function loadFixtures(head, treeSha, destination, json) {
  const tree = await json(`${apiRoot}/git/trees/${treeSha}?recursive=1`);
  assert.equal(tree.sha, treeSha);
  assert.equal(tree.truncated, false, "complete immutable H tree required");
  const files = new Map(tree.tree.map((entry) => [entry.path, entry]));
  const source = new Map(),
    authorities = [];
  for (const [name, pin] of Object.entries(pins)) {
    const entry = files.get(name);
    assert(
      entry?.type === "blob" && entry.mode === "100644",
      `regular authored H fixture required: ${name}`,
    );
    const blob = await json(`${apiRoot}/git/blobs/${entry.sha}`);
    assert.equal(blob.encoding, "base64");
    const bytes = Buffer.from(blob.content, "base64");
    assert(bytes.length <= 512 * 1024);
    assert.equal(blob.sha, entry.sha);
    assert.equal(blob.size, bytes.length);
    assert.equal(gitBlob(bytes), entry.sha);
    assert.equal(sha256(bytes), pin, `whole frozen fixture changed: ${name}`);
    const target = path.join(destination, name);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, bytes, { flag: "wx" });
    source.set(name, bytes);
    authorities.push({
      path: name,
      source: head,
      blob: entry.sha,
      bytes: bytes.length,
      sha256: pin,
    });
  }
  return { source, authorities, vectors: scenarios(source) };
}
