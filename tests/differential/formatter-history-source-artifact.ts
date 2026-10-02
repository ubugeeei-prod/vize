import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "./manifest.mjs";
import { stripRust } from "../../tools/support/compat/davinci/lib/rust-source.mjs";

// This is an assertion-only source witness, never a formatter output oracle.
// The original pins remain authoritative; current source must also match.
export const PRESERVED_FORMATTER_SOURCE = {
  owner: "crates/vize_glyph/tests/preserve_authored_content.rs",
  originalSha256: "9affca9435fc96cc470bf07840c71ed17844629c3361749484c893269855fa5d",
  currentSha256: "f6b9060822826c8a3942ce9a7c40f5876bf82fb7b2bfef81f10ff3c68aec8ec0",
  originalAsset:
    "tests/_fixtures/differential/formatter-history/source-witnesses/preserve_authored_content.cc87.txt",
  revisions: [
    "cc87bb5960ea9e49e82672205df919de58bb4b24",
    "a8cca54e825bf4271af1512c892378bba5e47834",
  ],
} as const;

// name, original raw function SHA256, current raw function SHA256, full .snap SHA256
const FUNCTIONS = [
  [
    "keeps_comments_between_and_after_sfc_blocks",
    "ef8af47b00b132abdc9b137e99e19ef7987c045d399b621a5fbfa7985c9a3b1f",
    "88cb019575911921684728fed283dc99f54a78d90e795efd4997de615a0bf7b6",
    "ca59cccb7ee266e4e01557ffaf5b4e546fe2f63605a42ce252f3c780a59fde1e",
  ],
  [
    "comments_follow_their_block_when_sfc_blocks_are_sorted",
    "80c65f0d790c09d7b7b28470cf6eac22d60bfb1b29de5955e35666e1e2523e72",
    "73ee81b3a0efb9c905d7dc20336bf91c07d86a4dd3710935d7bfd4c0c7e93d26",
    "dccd8e1e79fdaa03448f4225b7a77d0e669d6faf291eb6e84553141dd689f75d",
  ],
  [
    "retains_authored_css_color_spellings",
    "ae191cfc05fd31ed38e73a1b7762919923dadb7d4e1bb10098019974c02b55fc",
    "3a39a25742ce3dc0c9be4350deb98dfb3a8dd230a1d60e4e9798129b7a9f1066",
    "1bb7e9ad11d4b1ba8359209f2a94056600c90cc326fb225a65f37a08ebd30eec",
  ],
  [
    "retains_color_functions_and_hex_without_touching_strings_or_urls",
    "5f25c9ba09e658927d57fae21cae1bd7a4f2a914baabe536c08417c6e2a631b8",
    "95f60ba293ee24bcb27087f13cb08d7df0906d4a2dff1f8361a0e2b2ba8e13a9",
    "c6553a41b80069bbd5449301a21cfd21479e4ecc37d4a8b02a10fed857a1cf17",
  ],
  [
    "retains_authored_css_color_spellings_in_sfc",
    "cacbb449c8728b44229fe57ce9641d83cb21b519bc992f8f9d7054c6e7e02486",
    "260697cac771272786635a79cd2e666bc5de174303f13ef4cb304f9f54719939",
    "a36198f3ca2b10fc48cc32e4a53c70aacf20a12f968ed1256b550f42a675875b",
  ],
  [
    "css_formatting_keeps_media_features_values_and_nested_selectors",
    "3f6a9b403d26482dd668c08f659cde0410a27e2b732015926938dde1b412a828",
    "3143dfb478b430951ae7b350ca61045b1ab7d9f0099cc12685d86909c9bc258a",
    "86de29f657cdfd4056629f719ceb23f6a96abd3b1c6f0b05534a2a21a210a54a",
  ],
  [
    "scoped_css_keeps_implicit_nested_selectors",
    "4723f6a59d2b7f834abec08279909191159a4090f495d32f456cf10f3bdd0c4e",
    "b9b84fa951043112fa376877cee3ffe905e2c5493c5ceb5b9967fa10570958f6",
    "0c87c9be7227664fb40d7c3079c7c363e3386017bba88ea607cce6514e22ef98",
  ],
  [
    "css_comment_keeps_preceding_blank_line",
    "1b991a9ea3143da526eb1a82b585548ccdf641174e5afec33fe55f54c76df070",
    "5868c0cf19c7651096cd733f3f7e6b65b063bb3b465a89f270739729c4595e89",
    "f837d3f507e7744b6847cc8f76b9228aa524be7c731143ffac40233ee9ec192b",
  ],
] as const;

export function retainedFormatterFunction(bytes: Buffer, name: string) {
  const source = bytes.toString("utf8");
  assert(Buffer.from(source).equals(bytes), "Rust law source must be lossless UTF-8");
  const clean = stripRust(source);
  const pattern = new RegExp(`\\bfn ${name}\\(\\)\\s*\\{`, "g");
  const match = pattern.exec(clean);
  assert(match, "retained Rust law is missing");
  assert.equal(pattern.exec(clean), null, "ambiguous Rust law");
  const open = clean.indexOf("{", match.index);
  let end = open + 1;
  let depth = 1;
  while (depth && end < clean.length) {
    if (clean[end] === "{") depth++;
    else if (clean[end] === "}") depth--;
    end++;
  }
  assert.equal(depth, 0, "unterminated Rust law");
  // Masking finds syntax boundaries only. Compare every original authored byte.
  return Buffer.from(source.slice(match.index, end));
}

function sourceBytes(root: string, relative: string) {
  const resolved = fs.realpathSync(path.resolve(root, relative));
  const contained = path.relative(fs.realpathSync(root), resolved);
  assert(contained && !contained.startsWith("..") && !path.isAbsolute(contained));
  return fs.readFileSync(resolved);
}

function preservedSource(root: string, originalSha256: string) {
  const authority = PRESERVED_FORMATTER_SOURCE;
  assert.equal(originalSha256, authority.originalSha256, "original preserved owner pin changed");
  const current = sourceBytes(root, authority.owner);
  const currentHash = sha256(current);
  if (currentHash === authority.originalSha256) return current;
  assert.equal(currentHash, authority.currentSha256, "current preserved owner changed");
  const original = sourceBytes(root, authority.originalAsset);
  assert.equal(original.length, 3907, "original preserved owner bytes changed");
  assert.equal(sha256(original), authority.originalSha256, "original preserved owner changed");
  const macro = "    insta::assert_snapshot!(formatted);";
  for (const [name, originalHash, currentFunctionHash, snapshotHash] of FUNCTIONS) {
    const before = retainedFormatterFunction(original, name);
    const after = retainedFormatterFunction(current, name);
    assert.equal(sha256(before), originalHash, "original preserved function changed");
    assert.equal(sha256(after), currentFunctionHash, "current preserved function changed");
    const split = before.toString().split(macro);
    assert.equal(split.length, 2, "original snapshot assertion changed");
    assert(after.toString().startsWith(split[0]!), "original input/options prefix changed");
    assert(after.toString().endsWith(split[1]!), "original fixed-point suffix changed");
    const snapshot = sourceBytes(
      root,
      `crates/vize_glyph/tests/snapshots/preserve_authored_content__${name}.snap`,
    );
    assert.equal(sha256(snapshot), snapshotHash, "preserved snapshot bytes changed");
  }
  return original;
}

export function resolvePreservedFormatterSource(
  root: string,
  artifact: { path: string; sha256: string },
) {
  if (artifact.path !== PRESERVED_FORMATTER_SOURCE.owner) return null;
  return preservedSource(root, artifact.sha256);
}

export function validatePreservedFormatterWitness(
  root: string,
  entry: { path: string; sha256: string; revisions: string[] },
  name: string,
) {
  if (entry.path !== PRESERVED_FORMATTER_SOURCE.owner) return false;
  assert.deepEqual(
    entry.revisions,
    [...PRESERVED_FORMATTER_SOURCE.revisions],
    "original preserved revisions changed",
  );
  assert(
    FUNCTIONS.some(([registered]) => registered === name),
    "unregistered preserved function",
  );
  preservedSource(root, entry.sha256);
  return true;
}
