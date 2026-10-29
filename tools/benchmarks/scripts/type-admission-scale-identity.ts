import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const hash = (image: string) => createHash("sha256").update(fs.readFileSync(image)).digest("hex");

/** Source labels are admitted only alongside their byte-matched build receipt. */
export function scaleCliIdentity(binary?: string, source?: string) {
  if (!binary) return null;
  assert.match(source ?? "", /^[a-f0-9]{40}$/);
  const receipt = JSON.parse(fs.readFileSync(binary + ".differential-build.json", "utf8"));
  assert.equal(receipt.schema, "vize.differential.build");
  assert.equal(receipt.version, 1);
  assert.equal(receipt.sourceRevision, source);
  assert.equal(receipt.binarySha256, hash(binary));
  assert.ok(
    ["cargo build --profile ci -p vize", "cargo build --release -p vize"].includes(receipt.recipe),
  );
  return receipt;
}

/** Inspect the actual loaded image; an installed package version is not source custody. */
export function scaleNativeIdentity(provider: string, source?: string) {
  const require = createRequire(path.resolve(provider, "package.json"));
  const binding = require("@vizejs/native") as { getNativeBuildToken?: () => string };
  const images = Object.entries(require.cache).filter(
    ([file, module]) => file.endsWith(".node") && module?.exports === binding,
  );
  assert.equal(images.length, 1, "the wrapper must expose exactly its actual native image");
  const image = fs.realpathSync(images[0][0]);
  const sha256 = hash(image);
  const token = binding.getNativeBuildToken?.() ?? null;
  let receipt: unknown = null;
  if (source) {
    assert.match(source, /^[a-f0-9]{40}$/);
    const proof = JSON.parse(fs.readFileSync(image + ".source.json", "utf8"));
    assert.equal(proof.sourceRevision, source);
    assert.equal(proof.binarySha256, sha256);
    assert.match(token ?? "", /^[a-f0-9]{32}$/);
    assert.equal(proof.buildToken, token, "identity must come from the loaded source image");
    receipt = proof;
  }
  return {
    path: image,
    bytes: fs.statSync(image).size,
    sha256,
    token,
    receipt,
    source: source ?? "installed-native",
    wrapper: require.resolve("@vizejs/native"),
  };
}
