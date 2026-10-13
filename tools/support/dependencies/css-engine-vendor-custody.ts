// Verify the immutable upstream and original consumer authorities before builds.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

import "./css-engine-finite-custody.ts";
import { authenticateCssReleaseDriver } from "./css-engine-release-driver-custody.ts";

const root = process.cwd();
const preserved: Record<string, string> = {
  "crates/vize_atelier_sfc/tests/css_engine_panic_boundary.rs":
    "tests/_fixtures/differential/css-engine-vendor-exit/finite-before/css_engine_panic_boundary.rs.txt",
  "crates/vize_atelier_sfc/tests/css_fuzz_recovery.rs":
    "tests/_fixtures/differential/css-engine-vendor-exit/finite-before/css_fuzz_recovery.rs.txt",
  "crates/vize_atelier_sfc/examples/css_engine_recovery.rs":
    "tests/_fixtures/differential/css-engine-vendor-exit/finite-before/css_engine_recovery.rs.txt",
};
const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const modified: Record<string, string | null> = {
  "lightningcss/Cargo.toml": null,
  "lightningcss/src/values/percentage.rs":
    "967f295af4d48ac2ce4a663dc7215e2ab9f99c2e25a55087c581335994075332",
  "lightningcss/src/values/color.rs":
    "97fd2d02f455bd38c4caad26d639acf00403de7bd05eb641a74b082753cb2d2a",
  "lightningcss/src/bundler.rs": "469af8d6152becd4371590202d932e29ba2900ecc48190d645af6d2d3cde48ef",
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
  const bytes = fs.readFileSync(path.join(root, preserved[row.path] ?? row.path));
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
// Authenticated finite authored-order successor; all original before bytes remain.
assert.equal(
  sha256(
    fs.readFileSync(
      path.join(
        root,
        "tests/_fixtures/differential/css-engine-vendor-exit/bundler-after-shallow-refusal.json",
      ),
    ),
  ),
  "875d16fe096b741ab2c0fded7cdb2b99e6fda09814cec5490da602a5c0354f7c",
);
for (const [filename, checksum] of [
  [
    "vendor/lightningcss/src/bundler/context.rs",
    "bc708fb0d3ef548511a342b54bef71717a7e8dde96605f4af67e3a3acc69be5f",
  ],
  [
    "vendor/lightningcss/src/bundler/order_contract.rs",
    "4bd9b603c7187daf46c1ab4b5dc1ab2f58fafa1f4ada0d233bb838528f6e2ca9",
  ],
  [
    "vendor/lightningcss/tests/vize_fixtures/bundler-before-inputs.json",
    "407348d9901a5a566cc10f4e199d61c4670c018c53daed09d8633ea00ea0de1c",
  ],
  [
    "vendor/lightningcss/tests/vize_fixtures/bundler-before.rs.txt",
    "532b321252c008f1c581a1613fda4cc6e57369bf2bf712b95618f3ca60a0b5ed",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/bundler-before-inputs.json",
    "407348d9901a5a566cc10f4e199d61c4670c018c53daed09d8633ea00ea0de1c",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/bundler-before-whole.json",
    "d139e32de20854c5068a38bb3803884a12db81484fe032ba4c45c575774c1da1",
  ],
  [
    "tools/support/dependencies/css-engine-bundler-after.ts",
    "bb972630ebed008f20725334998951628ed141a43a49d83cd1ca05349388a4ea",
  ],
  [
    "tools/support/dependencies/css-engine-producer-manifests.ts",
    "31a86011e3d8cda9b3817b5cceb210f2b8d2099a8af033202755e09c73db7f1b",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/release-inheritance/root-original.toml",
    "c8fe5d4b91f49524fb4f8cd06c7453affd936b2373c29b622e94a01dc7a0793f",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/release-inheritance/lightningcss-original.toml",
    "21486cbc79ea73caf098501beef644a8d1a64f55706a9dc24b9e2fc4df4044aa",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/release-inheritance/bundler-after-original.ts.txt",
    "a126866ead8f2dd961926a217da4d18891bb3a7f92af7fc29b5179f2a70641d6",
  ],
  [
    "tools/commands/release/pr.rs",
    "db53a5ec6abf597c01dfb77b27477cfb10d69bdeba7018ab5920278d0a590d2d",
  ],
  [
    "tools/support/release/pr_pin.rs",
    "273b5f7263feb8ddf5bf132ccf329a388f467bf5e1ab586d866ba2ce8eca904d",
  ],
  [
    "tools/support/release/pr_pin_metadata.rs",
    "fa54401497bea1bfa7fc443f720c4466df54a244f11d181478c69e715114b308",
  ],
  [
    "tools/support/release/pr_pin_producer_metadata.rs",
    "9db54550ccc39b4ce4609e28dffe679bd93df4117aa3f60e3cbd4204da792f15",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/release-inheritance/metadata/Cargo.toml",
    "38babfd635c5af9df1d401b0b59923940310a405868cbb0772d9bfefbd7e6e68",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/release-inheritance/metadata/Cargo.lock",
    "194c3098330cbeb91e441a2f035fe3cfb37650195c6ff04baf88f09a08378684",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/moonbit-publish-original.test.ts.txt",
    "7d131acc8a444afc9cabddb6c5995bd8e532d1a65d389721ffa37d9af951496a",
  ],
  [
    "tools/support/dependencies/css-engine-release-driver-custody.ts",
    "894d2e91a72b93a349623657c54309196848242cec390e837a2688573c2c1431",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/release-inheritance/stable-driver/pr-original.rs.txt",
    "db53a5ec6abf597c01dfb77b27477cfb10d69bdeba7018ab5920278d0a590d2d",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/release-inheritance/stable-driver/pr-pin-original.rs.txt",
    "273b5f7263feb8ddf5bf132ccf329a388f467bf5e1ab586d866ba2ce8eca904d",
  ],
]) {
  authenticateCssReleaseDriver(filename, fs.readFileSync(path.join(root, filename)), checksum);
}
const bundledSource = fs.readFileSync(path.join(root, "vendor/lightningcss/src/bundler.rs"));
const bundledTests = fs.readFileSync(
  path.join(root, "vendor/lightningcss/tests/vize_fixtures/bundler-original-tests.rs.txt"),
);
assert.equal(
  sha256(bundledTests),
  "e1d3d9d8ba8a988177e8bd82707ab8d51e5b68f6667c4c7b1e236d89c943b098",
);
assert.deepEqual(
  bundledSource.subarray(bundledSource.indexOf("#[cfg(test)]\nmod tests {")),
  bundledTests,
);
assert.deepEqual(
  fs.readFileSync(
    path.join(root, "vendor/lightningcss/tests/vize_fixtures/bundler-before-inputs.json"),
  ),
  fs.readFileSync(
    path.join(
      root,
      "tests/_fixtures/differential/css-engine-vendor-exit/bundler-before-inputs.json",
    ),
  ),
);
const metadataFile = process.argv[2];
const atelierInputs = fs.readFileSync(
  path.join(root, "tests/_fixtures/differential/css-engine-vendor-exit/atelier-inputs.json"),
);
assert.equal(
  sha256(atelierInputs),
  "27a7d73a093c02ec80e81cfc1c8d8918e1bacbe4b2b2350e8c641352bff61786",
  "complete66 original Atelier input authority",
);
assert.equal(JSON.parse(atelierInputs.toString()).observations.length, 66);
assert.equal(JSON.parse(atelierInputs.toString()).additionalInputs.length, 6);
for (const [filename, checksum] of [
  [
    "examples/css_engine_recovery.rs",
    "2342d90a3ea3b0902fe9728b98c0409b664fba6ee744b9a0930f52e28b5b20f5",
  ],
  [
    "tests/css_fuzz_recovery.rs",
    "964c8a85756fc13faf9c29120fee3a6a5d9caeabdf80cd626ce7cdbd228fc9a2",
  ],
  [
    "tests/fixtures/css-engine/6190.css",
    "562de29c854bfc90a513528f8a653a90c21144ee31d7b1d7d8da73d5748a3431",
  ],
]) {
  assert.equal(
    sha256(
      fs.readFileSync(
        path.join(
          root,
          preserved[`crates/vize_atelier_sfc/${filename}`] ?? `crates/vize_atelier_sfc/${filename}`,
        ),
      ),
    ),
    checksum,
  );
}
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
