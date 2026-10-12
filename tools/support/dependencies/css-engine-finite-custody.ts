// Whole finite exceptions preserve all original authorities; no generic acceptance mask.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { gunzipSync } from "node:zlib";

const root = process.cwd();
const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
for (const [filename, checksum] of [
  [
    "crates/vize_atelier_sfc/tests/css_engine_panic_boundary.rs",
    "79f876d56738c7321c4f656a19407584e8b8a5c9a89c34833299e715fcc4fad2",
  ],
  [
    "crates/vize_glyph/tests/css_engine_boundary.rs",
    "b6f586fba17c7ccaebf79a5516a2e45c76508f23b9c428e4f7e0a8f10514fd21",
  ],
  [
    "crates/vize_patina/tests/css_engine_boundary.rs",
    "0164ed85ca33cf14cb911620ad515100f031c2d31ab2e3c0d5ac2d14b5ad96bb",
  ],
  [
    "crates/vize_atelier_sfc/tests/css_fuzz_recovery.rs",
    "7d63234d25a06a1c05042c4ca83588dcd8a7e0201e5b96ab66d92698d866cdcb",
  ],
  [
    "crates/vize_atelier_sfc/examples/css_engine_recovery.rs",
    "3fce7d350020f0c38bd1e17460c28b29ac398fbcd6e2c5a3c0a90ee7867f014a",
  ],
  [
    "crates/vize_atelier_sfc/src/css/parser/engine_boundary.rs",
    "25b8844bec264cb8b73856d9938d579dfab22198947ee37f82d2a0f34b2ee4c3",
  ],
  [
    "tests/tooling/css-engine-boundary.test.ts",
    "ba6e5a797ec7335c5e7128553ba3282bf3dd87c029ca1048e7983ad43f410ab1",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/finite-before/css_engine_panic_boundary.rs.txt",
    "7eefb93c2985526b20fe38ce76ffe09594064a17f08d85e9fcb6b54b6a197df7",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/finite-before/glyph-css_engine_boundary.rs.txt",
    "dd603ce989a5d086cf2bebbd17e3306810225c08123f241a239731ea13d19889",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/finite-before/patina-css_engine_boundary.rs.txt",
    "d926ef9fcc0c862f329771822a897fc438f8a595b9fca2b6ad325b75927cc2b6",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/finite-before/css_fuzz_recovery.rs.txt",
    "964c8a85756fc13faf9c29120fee3a6a5d9caeabdf80cd626ce7cdbd228fc9a2",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/finite-before/css_engine_recovery.rs.txt",
    "2342d90a3ea3b0902fe9728b98c0409b664fba6ee744b9a0930f52e28b5b20f5",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/finite-before/engine_boundary.rs.txt",
    "afce6fe932b9d55c8b1bd317793640185ec32a176703c52edeb03b4538f08749",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/finite-before/css-engine-boundary.test.ts.txt",
    "c2d86a73ea96298374382996e1e49704f38af00c3e823414d5f5f728974efd45",
  ],
  [
    "crates/vize_atelier_sfc/tests/support/mod.rs",
    "58d9c98a23cfc5c715f4b22528ccc6d2d5aaf56e6fc585c12bf06ba1e12d16e0",
  ],
  [
    "crates/vize_atelier_sfc/tests/support/css_engine_successors.rs",
    "3d8c156353b3f031a65600e797283b41b76e327c63a7d2909c47827265d526cf",
  ],
  [
    "tests/tooling/davinci/davinci-atelier-sfc-stage-alias.test.mjs",
    "429f0747c4f3c0d9867c929b1f5b01165cd380b21714411c56b11093f40b566c",
  ],
  [
    "tools/support/levels/css-naming-gate-continuation.ts",
    "1200a275d23ae66669657b17f18398d3b8e3bd12140d3e08f21466ed2bbf3bb2",
  ],
  [
    "tools/support/levels/profile-export-host-preflight.ts",
    "ff98c82c33930a742cc3d8a7f899e2ea59fc3950458fbb305d6d9f509f34c05b",
  ],
  [
    "tools/support/levels/select-profile-allocator-host.ts",
    "74cbfc1bc0ecec0f60f8fa9ee2cb462f4b2f3278b456d6509b5b4c2e7648445e",
  ],
  [
    "tests/tooling/profile-export-host-boundary.test.ts",
    "bd3ee4aac15b2c0afdd064a34bc63407918b199cd66d27fd110a9f56b770c43a",
  ],
  [
    "tests/tooling/profiler-allocator-ownership.test.ts",
    "61eeacdb07f4127dd13aed66dc94be8a7abd33add25e7fd8715e91f4d0bd84fc",
  ],
  [
    "tests/tooling/support/css-engine-successors.ts",
    "8edb01f09122680bb59c7e35231b389477553dcad903791aa13a1b38d882ce41",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/finite-successors.json",
    "011568d5d1e0359f8da98f846bcec5d49082c6689afb50bb339f98c510f808cc",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/finite-before-whole.json",
    "c1a1ae02b92531f8ab6f97c1648ffc5b3e1e5e648108918c70c5201b804e8701",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/bundler-after-whole.json",
    "f3e8418676a7a56005c8fab68ac649023c265786ca7e5e19344f6c9ae5f70bd2",
  ],
  [
    "crates/vize/examples/css_engine_vendor_observer.rs",
    "be0dbc6ef1fde75a20b9b1b392f4a5d7a7bd71e611b7a64f4405ba7f984e9d88",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/finite-before/rust-observer.rs.txt",
    "4e2b5889388b183aa3f63803ae555d7b9bbc5c430f300ddd5e305df10f81a513",
  ],
  [
    "tools/support/dependencies/css-engine-finite-results.ts",
    "67c65eb3d80e024ba2ac0163b8cb269e541e77f3359c871b482307beca733fcc",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/finite-values-peer-custody.json",
    "f3836e384479a7e2409d468e830e9a5fe8abf83432ffebef9d600e8279de0c13",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/finite-values-peer-semantics.json",
    "cb718c343cda22f98a7e5e3eaf362b7badd5e355051c31b6fed936d80554ddce",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/finite-cli-census-refusal.json",
    "ddab84834423b8bb28675b400392239706176e6419b1f4873c09471ba3b89c01",
  ],
  [
    "tests/_fixtures/differential/css-engine-vendor-exit/finite-before/finite-helper-before.ts.txt",
    "46ae2b852bd5df92d1be3e54f68cf46061f837b0ac700c95060a6ed1f77ef470",
  ],
  [
    "crates/vize_patina/src/lib.rs",
    "480fc720c8e3558280ea6ec594e6bbccb6fa9a8eaba164d7bd8ae3ef77e1ce15",
  ],
  [
    "tools/support/dependencies/css-engine-downstream.sh",
    "38f9b959cbdb36a57aa4930214f14cf3593da3d3ed8861b7cbe5aa26e7f66f86",
  ],
  [
    "tests/external-consumers/css-engine-identity/src/main.rs",
    "3b06ce108f147d3d3a3ca25995b3b4c53e451419039f8699a7f68e368a96d0c5",
  ],
  [
    "tests/external-consumers/css-engine-identity/typed-visitors.rs",
    "780685aaf1696808705ea2c6c33ad9eb8b097177ad2ebb90ed8f3fe43b02937a",
  ],
  [
    "tests/external-consumers/css-engine-identity/trait-implementation.rs",
    "c2127110a03fa9f46c63b88d6e2583834a50ba134347c19ecf84b9f7ee3d1ac3",
  ],
  [
    "tests/external-consumers/css-engine-identity/patina-provider-new-raw.rs",
    "fc77f35744572ffbcccffde5dd72362d487b5bc9ab4bb935714d93a638894af2",
  ],
  [
    "tests/external-consumers/css-engine-identity/patina-provider-typed-visitors.rs",
    "4bed480ad5928fdbc1badcb7420bf8f58e5f7cc6b6354dc836d1c66b6fe2fe9f",
  ],
  [
    "tests/external-consumers/css-engine-identity/patina-provider-trait-implementation.rs",
    "94c19f8996a49df84571c3958e4b45113f66fd2f87adf62d85adc1bf44d8a859",
  ],
]) {
  assert.equal(sha256(fs.readFileSync(path.join(root, filename))), checksum, filename);
}
const fixture = (name: string) =>
  JSON.parse(
    fs.readFileSync(
      path.join(root, "tests/_fixtures/differential/css-engine-vendor-exit", name),
      "utf8",
    ),
  );
const finite = fixture("finite-successors.json");
assert.equal(finite.rustCases.length, 11);
assert.equal(finite.cliCells.length, 26);
assert.equal(finite.atelierRoles.length, 14);
assert.equal(finite.additionalAtelier.length, 6);
assert.equal(finite.historicalFixedCases.length, 3);
const before = fixture("finite-before-whole.json");
assert.equal(before.normalCheckConclusion, "failure");
assert.equal(before.wholeRawRustFailures.length, 4);
for (const row of before.wholeRawRustFailures) {
  const raw = gunzipSync(Buffer.from(row.gzipBase64, "base64"));
  assert.equal(raw.length, row.bytes);
  assert.equal(sha256(raw), row.sha256);
}
for (const field of ["observerWholeDev", "observerWholeRelease"]) {
  const observed = before[field];
  assert.equal(observed.completed, true);
  assert.equal(observed.rustConsumers.length, 99);
  assert.equal(observed.atelier.length, 66);
  assert.equal(observed.additionalAtelier.length, 6);
}
for (const field of ["cliWholeDev", "cliWholeRelease"]) {
  assert.equal(before[field].completed, true);
  assert.equal(before[field].attempts.length, 90);
}

// The three selected migration controls are whole original bodies with only
// their AST namespace prefixes changed. No assertion/source/control is dropped.
for (const [migrated, original, count] of [
  ["patina-provider-new-raw.rs", "src/main.rs", 1],
  ["patina-provider-typed-visitors.rs", "typed-visitors.rs", 2],
  ["patina-provider-trait-implementation.rs", "trait-implementation.rs", 1],
] as const) {
  const directory = path.join(root, "tests/external-consumers/css-engine-identity");
  const body = fs.readFileSync(path.join(directory, migrated));
  const text = body.toString("utf8");
  assert.equal(text.split("vize_patina::css_ast::").length - 1, count);
  assert.deepEqual(
    Buffer.from(text.replaceAll("vize_patina::css_ast::", "lightningcss::")),
    fs.readFileSync(path.join(directory, original)),
  );
}
