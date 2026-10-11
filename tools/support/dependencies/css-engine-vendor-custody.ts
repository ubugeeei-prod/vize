// Verify the immutable upstream and original consumer authorities before builds.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const modified: Record<string, string | null> = {
  "lightningcss/Cargo.toml": null,
  "lightningcss/src/values/percentage.rs":
    "967f295af4d48ac2ce4a663dc7215e2ab9f99c2e25a55087c581335994075332",
  "lightningcss/src/values/color.rs":
    "97fd2d02f455bd38c4caad26d639acf00403de7bd05eb641a74b082753cb2d2a",
  "lightningcss/src/bundler.rs": "532b321252c008f1c581a1613fda4cc6e57369bf2bf712b95618f3ca60a0b5ed",
  "lightningcss/src/lib.rs": "8d6453cc1ae51e4e8c0e99f82f0114a4603c2952e7ca1d0928b49e4e3e027ffa",
  "lightningcss/src/selector.rs":
    "ffd28f41d263e9590c7b63633ed474a607449cdc967f25945be79c16ff5e1ecf",
  "lightningcss/src/main.rs": "c3b10422b3da697e0264ea8a183572384ef4e7501f43108b43ccb3aa4e802b47",
  "lightningcss/tests/cli_integration_tests.rs":
    "7c8f024acfe2a67d1fd6850e51db5aa8aade41585604a1d398332073ab5bac1e",
  "parcel_sourcemap/Cargo.toml": null,
  "parcel_sourcemap/src/lib.rs": "64dfcee3713a976f2f504b3ddc8ac1d5152698b07f5b69098c74f30677a065a6",
  "parcel_sourcemap/src/sourcemap_error.rs":
    "b94de0fedbcc198e370d018c49fad57103e33b1788316c2457e1f7f2eb2cfbb0",
};
let upstreamFiles = 0;
for (const [name, count, archive] of [
  ["lightningcss", 121, "d31b760f96e8fdfe1d0c295e4bf76c503d6f15d2d470d53bd8cd1f7aa8c7d934"],
  ["parcel_sourcemap", 9, "485b74d7218068b2b7c0e3ff12fbc61ae11d57cb5d8224f525bd304c6be05bbb"],
] as const) {
  const manifest = JSON.parse(
    fs.readFileSync(path.join(root, "vendor", name, "UPSTREAM-IMPORT.json"), "utf8"),
  );
  assert.equal(manifest.archiveSha256, archive);
  assert.equal(manifest.files.length, count);
  if (name === "lightningcss") {
    const cli = manifest.files.find((row: { path: string }) => row.path === "src/main.rs");
    assert.equal(
      sha256(
        fs.readFileSync(
          path.join(root, "vendor/lightningcss/tests/vize_fixtures/cli-original.rs.txt"),
        ),
      ),
      cli.sha256,
      "whole original CLI source remains frozen",
    );
    const tests = manifest.files.find(
      (row: { path: string }) => row.path === "tests/cli_integration_tests.rs",
    );
    assert.equal(
      sha256(
        fs.readFileSync(
          path.join(root, "vendor/lightningcss/tests/vize_fixtures/cli-original-tests.rs.txt"),
        ),
      ),
      tests.sha256,
      "whole original CLI test authority remains frozen",
    );
  }
  assert.deepEqual(
    fs.readFileSync(path.join(root, "vendor", name, "UPSTREAM-Cargo.toml")),
    fs.readFileSync(path.join(root, "vendor", name, "Cargo.toml.orig")),
    `${name} packaged upstream manifest`,
  );
  for (const row of manifest.files) {
    const key = `${name}/${row.path}`;
    const bytes = fs.readFileSync(path.join(root, "vendor", key));
    if (Object.hasOwn(modified, key)) {
      if (modified[key] !== null) assert.equal(sha256(bytes), modified[key], key);
    } else {
      assert.equal(bytes.length, row.bytes, key);
      assert.equal(sha256(bytes), row.sha256, key);
    }
    upstreamFiles += 1;
  }
}
const originals = JSON.parse(
  fs.readFileSync(
    path.join(root, "tests/_fixtures/differential/css-engine-vendor-exit/originals.json"),
    "utf8",
  ),
);
assert.equal(originals.files.length, 20);
for (const row of originals.files) {
  const bytes = fs.readFileSync(path.join(root, row.path));
  assert.equal(bytes.length, row.bytes, row.path);
  assert.equal(sha256(bytes), row.sha256, row.path);
}
for (const [filename, length, digest] of [
  [
    "lch-sign-original.css.txt",
    39,
    "f86aebc7d7028d25bab59782b664ffd3be8e7e834d4befc7a4039d8ef2a4c339",
  ],
  [
    "number-fold-original.css.txt",
    66,
    "ede20ce735d239ad7d3dc4baee83e057a827f95b4541759cf9b5483adfa8d284",
  ],
  [
    "hue-assert-original.css.txt",
    1045,
    "4a6a20d7e513adf9f55f52f58949f1524d7657c982470b899f0f8ad32083eb05",
  ],
] as const) {
  const bytes = fs.readFileSync(
    path.join(root, "vendor/lightningcss/tests/vize_fixtures", filename),
  );
  assert.equal(bytes.length, length, filename);
  assert.equal(sha256(bytes), digest, filename);
}
const metadataFile = process.argv[2];
if (metadataFile) {
  const metadata = JSON.parse(fs.readFileSync(metadataFile, "utf8"));
  const archives = metadata.packages.filter((pkg: { name: string }) => pkg.name === "rkyv");
  assert.equal(archives.length, 1, "no second or old archive provider");
  assert.equal(archives[0].version, "0.8.18");
  for (const name of ["vize_lightningcss", "vize_parcel_sourcemap"]) {
    assert.equal(
      metadata.packages.filter((pkg: { name: string }) => pkg.name === name).length,
      1,
      name,
    );
  }
  assert.equal(
    metadata.packages.filter((pkg: { name: string }) =>
      ["lightningcss", "parcel_sourcemap"].includes(pkg.name),
    ).length,
    0,
  );
}
console.log(
  JSON.stringify({
    upstreamFiles,
    originalConsumerFiles: originals.files.length,
    graphChecked: Boolean(metadataFile),
    publicationQualification: false,
  }),
);
