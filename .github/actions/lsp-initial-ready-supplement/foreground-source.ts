import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { BOUND_PATHS } from "../lsp-initial-ready-pair/readiness-parity.ts";
export const BASE_SOURCE = "6e0b9ab7ba4ca66087db7617c97ed8bd7c7f2f36";
export const CANDIDATE_SOURCE = "ee587836acf76b07cab63abe5f9c30866a773dae";
export const PREVIOUS_WORKFLOW = CANDIDATE_SOURCE;
/** Each immutable data tuple is path, B blob:bytes:SHA256 or absent, C blob:bytes:SHA256. */
export const SOURCE_ROWS = [
  [
    "crates/vize/tests/lsp_corsa_responsiveness_cli.rs",
    "c98bb98f1cd60f63ac60e6802cdf91c70c3752bf:12165:0930222ec4fbb48f1ddb8307e79c1962e07810b0d9f7c64c6a48f7bc8ca92ada",
    "79d720e38b911ca12d72fe2262c8456e96c4e918:12623:ccaba05109caf53590d4b91d4de9bfcb7d951711aba28d626afe13343224ea57",
  ],
  [
    "crates/vize_canon/src/corsa_bridge.rs",
    "4fa7e7afe6c5cdf912e25512aa3954335447647c:8196:d663d91d1dd55d0f4973c0d7b58a4de1553f94acefc9b00de00bf3a1d9bd0f99",
    "0dffc769a97c9cfa8dad229cb24e5bfb5f994fee:8968:126a5e8aaf1e0c4a218495f1c53303fe3b413dedbc1c753fcada1a281b1f56bb",
  ],
  [
    "crates/vize_canon/src/corsa_bridge/bridge.rs",
    "205c03413b2eb406a51a5d52fb62c0d95097aef7:13024:91e321e78c98f33db894ad3ec073ff28551b26f739cc3d45a74fe7f292febcfb",
    "57d0c1bc5a296ca02f637420f3f276f93c6fb3c4:11632:4b8ce1b623a8312fb4fa3b570be1e86ac844d53749fa8b10a8c447e815592c93",
  ],
  [
    "crates/vize_canon/src/corsa_bridge/bridge/startup.rs",
    null,
    "28a13a4dd047f1930913288b4ea314f06d26db7b:11049:88bcc649664c72e702c85bcd3cc9b1d77056cc5452969c00df7a77f9e9396a72",
  ],
  [
    "crates/vize_canon/src/corsa_bridge/bridge/startup/tests.rs",
    null,
    "61e73204ebbc8a0e4051671b6ba7f882e17d3ba9:12108:96d8be78cdb35fb2a5ddb6f0fd5faab960142538761b158c4c52f4d8f871aeda",
  ],
  [
    "crates/vize_canon/src/corsa_bridge/editor_session.rs",
    "4a98dbe855841febfff33659384ac6225c394e4f:9751:0137e778b51e59cdbcdf711188cce8564eb25e9f8e0bde14fec3aab4b407da9c",
    "0cf9d89cadd9bff3e539266cb8813ed981bc96c4:13235:8c6f9938833a95bec4ea432536df91ce7773f998677d91037a1f933ad56983ab",
  ],
  [
    "crates/vize_canon/src/corsa_bridge/editor_session/retained_tests.rs",
    null,
    "9fc3c629fd60bd6b9345a813b50da2da34c4f9cb:3201:27ab638f1cd7c000cbb658ddf86d1ed356f0da74e1b654d3b7daf19f503287bb",
  ],
  [
    "crates/vize_canon/src/corsa_bridge/session.rs",
    "4b5ee7dbab642f574409a49594e86640f7ce937a:2048:d3c9f62e2bc3dca29f32a250615ce502de053694c4b2c94aac655651bf6a23ed",
    "6b268902172e1eab46cb8fe62e02d333e8b599b1:2929:57f0d130d2353a315636392d35471ef30d105f212617ae1cb3f0125e9e51c523",
  ],
  [
    "crates/vize_canon/src/corsa_bridge/vue_document.rs",
    "b1e21f8a743e2d49466f937ed55398d5693bfc65:9789:6b53a07a035fc933828c8e68a5e13f432d86001394513aa344f06d2a9e419a4c",
    "c1e9ae00851209d82b789349dd52e38580ab9c56:9886:5398eab8dd20be99759055781cac3d144ee2ca178d28759511d3a039d049cfc6",
  ],
  [
    "crates/vize_canon/src/corsa_bridge/vue_document/initial_project.rs",
    null,
    "b7e4c6c2c9546331695bc739fcf97c206d488f8e:12726:18990a40129b98a2ac1a1367333b880ce62b7c867dae6e4f3beb9738ca247ce5",
  ],
  [
    "crates/vize_canon/src/corsa_bridge/vue_document/initial_project/native_tests.rs",
    null,
    "02655d6e9f1f5fa59d6fde39430e70bed8f49c04:6290:40e5758bff4470172be582d36d03ccb7e1e8ac0115731213e7acaeed59af4a4a",
  ],
  [
    "crates/vize_canon/src/corsa_bridge/vue_document/initial_project/tests.rs",
    null,
    "dbb802926cb4b0f96b747245c0ca605f24cb8050:13616:1f9e80a7c8da4c8734e0b46747daeaef846367a4fef42bf9d787621bd3f2765b",
  ],
  [
    "crates/vize_canon/src/corsa_bridge/worker.rs",
    "d946b8f6005a380f1f36965c10faddf01a099cec:12243:e445da9d069e627af7d1463e9520c2469a7b4cb23743d4e56c03dde5afdc273b",
    "6a19e87f3995f2339f73dfbe232d4431e4ce948c:15337:8d155a1792e4e56cad7ca8939dc79705732a494f78e9dfca8c53344ac0b0c06b",
  ],
  [
    "crates/vize_canon/src/lib.rs",
    "7e8861ea5d42fe5a865476624b62998f37c55ff2:9776:138a142d165848d6a093c9780553543a01023473592c634264bf161b06e7c24c",
    "28509ae92cb303c410eaff52fa010d0c2b1bd84e:9829:aac29d38afa0cf15af6463fa6dea0ffe5c3b6771a86bad44c59f7920465895b0",
  ],
  [
    "crates/vize_canon/src/lsp_client/editor_lsp/project_identity.rs",
    "72de2d185499b0c82e4bd3ed181cfbd60f945e95:5340:b9785464b98fbfa035fa4c0240e1f7098849fdfbcff67dbd461ee5bbf90457e2",
    "55287fc8a04101a3a02d0ec41479e189de01361f:6234:66a5374c19bcb59af23ee59b63fdbc3ae67012efa45858c9eed48a232e33dfdd",
  ],
  [
    "crates/vize_canon/src/lsp_client/lifecycle/workspace.rs",
    "cac8c76034900d821f6303a9ba45b771cf7783f0:2421:6aaa6530b98e115460f4141210f8818d3b1d4f5dae82a061e24ec4216c83bd74",
    "b995c8b919d7dc6b5c53676c0e9fe55229c99bd5:3950:753276aec69a0ef195f62745c82a9c93b6f5ac6a061eb8e57a83fc6c23e87d9a",
  ],
  [
    "crates/vize_canon/src/lsp_client/lifecycle/workspace/initial_project_tests.rs",
    null,
    "ea9126f0655618707889fdb5521b14858513bd12:10192:9618b6603521f52c3afbe30c57df8bd6fe43a2b898c78511a8a63f75a3be409e",
  ],
  [
    "crates/vize_canon/src/lsp_client/workspace_project.rs",
    "cf9cf5b4a64c722bd59a0f51618506adfe40d1b4:16980:156838d1841967d40786d238c78e18b93edbef7c7abaa219369de961d90e8f48",
    "6842d6786d370bbdcfcf2357670f2f6bde66af11:16994:08774437fae78288baa58ab855f3aee329188e69839f45d090ea6461790c19ea",
  ],
  [
    "crates/vize_canon/src/lsp_client/workspace_project/initial.rs",
    null,
    "448f1e15d4214535de3126144cd8e155bb431386:8888:3cda28ebb701720d218af6c5b80a724670575d57faae78f2a83fd650ffa58b20",
  ],
  [
    "crates/vize_maestro/src/ide/context.rs",
    "e2643ca8a05a211618782bbf56543f7649601a55:11452:39c428552abb0fe2e1c883f8b54a17acc24e48538b739e0c574524581bfda10e",
    "c0fbf6562ae9ba757b57bd9a7b313107717057ee:11771:e38a2c655100546581aa241b27637cb9bef6b87b4632171efb5b0f65f5a0845e",
  ],
  [
    "crates/vize_maestro/src/ide/context/initial_project.rs",
    null,
    "5e64797638e70742b60e017bf5772c6bc9b535b9:2463:de1e12e939c7fe21caccdf4e2dd47f7bcac13746109ae7759f0b31533fbf7d6c",
  ],
  [
    "crates/vize_maestro/src/ide/context/initial_project/tests.rs",
    null,
    "579ffc04b0b43ad7333a3bc15370aafa828e9287:3181:02340710ab1a9eac67e967d076978e75dcef27a971341956a2d3a8d8b4f518ae",
  ],
  [
    "crates/vize_maestro/src/ide/corsa_support/canonical/open.rs",
    "b832973c37744a8cbcd5d10af674bcf8e050b48e:9537:a8d1b667d6d53505c41b65d94883dfd6f8b90db4237dbe35856c26ab6539daa5",
    "5817a3b407aa2b07f1a0123f0c7cba3edb701da5:9770:3fea161c2862a3c364997fc072d81cec8cb76d1729d80f1c4ac4f1083e136a59",
  ],
  [
    "crates/vize_maestro/src/server.rs",
    "acba0b259c60c96caff490c7a365bc086a875b71:12748:3961feac1d3c4a8ec571e083fdbef58ed59390a9237b6dbbb85bc6925fd21265",
    "eed1ea939adfd17fc1e9a818f1b0913a2cf94b7a:12816:035f90a568afacca9a5eaadec57c2940dbd6627ccb6001bc59d9c9f1c95d65e9",
  ],
  [
    "crates/vize_maestro/src/server/diagnostic_publishing/retry_tests.rs",
    "f917ade1f0758ad1e105439ae443288c6e1348cc:8673:2898721ccf337876f6448af6741310fa88b2128db14c606ad39dae33ae2bbb54",
    "19113e80d6267729e05b0b940a305ac77cde4ec0:9999:a830d784385037afabc4e952265ec2fe019ad6d0b4ee3de8c616d90a7b7a0aef",
  ],
  [
    "crates/vize_maestro/src/server/handlers/typed.rs",
    "a4b1e81960029f488921f917e46589f1d7655da9:10272:752179a706971407290c7144720bbcbc63261031bf6b5378bdb8cc9427b601ac",
    "80ba6f52a001082d23f117172e5045acb99dcaa8:10784:8b7f11857234b5e2b1ad41bd8777993e60303589a5d64eb1b393ef307a27449e",
  ],
  [
    "crates/vize_maestro/src/server/initial_diagnostics/preparation.rs",
    "ad7bcb7a24dfdb3ea8a1e5cd354cb6da2114ca3a:3908:3bc266e17890fa01ebc2cd97ac40ce822ea5b0be695504f5cda577fdd138ec66",
    "6f5c07aeb18232827fc3417cf2904c45d49f197b:4115:69111593f4289cc3ef8ae828d44da51c8219d1a4522e37ddd44ee87f018a189a",
  ],
  [
    "crates/vize_maestro/src/server/state/corsa.rs",
    "f7ed11bb37d53fc17f4edae7cb1408695472620b:12846:2a78202e5d8dbfbb1f502af16cdd45f6fe8170ff0aa7d6d82356ac255df43efa",
    "42ea965793a0ed883c31a0b60b7a3875f3c5091c:13635:ec4e4738cd47e5ed3ad7498042d1cc2fdcc5651eaae814960a2f7f36313806fd",
  ],
  [
    "crates/vize_maestro/src/server/state/corsa/initial_project.rs",
    null,
    "6652bc579b5457125756294be2b6cb562c2a9439:5978:fd85fab341bab57ff59d8efeb7feda3acaa9dee504f7ae17c1618df7b854a59d",
  ],
  [
    "crates/vize_maestro/src/server/state/corsa/initial_project/tests.rs",
    null,
    "3e6f0be33daa70cee702e6ad7737f21307dd21b9:5941:5cc90da77a818a911958026fcd7fc520b15f2fe32159fdddbbc8b796661ee8e3",
  ],
  [
    "crates/vize_maestro/src/server/state/project_contexts/shutdown.rs",
    "bdf552f52a34779c4e06365d13576b560e3b1560:2065:bd747c703b70bc23a77eb43c92c15005612856f848ecbc6875a52e8683811d84",
    "6bcd1f701360e8af6dae43704c582f2dc8f67b0f:2549:0d1b4e4b8d5f036eb74645a6443829d749703e01a8b919cb1c7a5f6daababfb6",
  ],
  [
    "crates/vize_maestro/src/server/state/project_contexts/shutdown/tests.rs",
    null,
    "679e324fe2b63d37d14881fe8613d872261b7a01:1312:ea0170ebc8b21b1ba4fde96eed9b25ef9964b54d4a64c52b9aaf84eb2390486c",
  ],
  [
    "editors/vscode/test/css-hover-foreground-source.ts",
    "40e9df2df978d87153511208a1f789d2e54c33e1:6672:ccb878667e4be0b05118d172a9e43499ed9623356679ea734f2c77d9885ec0a1",
    "0f4fb4b6d634b41b7a2b06f43350911ebf42a147:8248:77de355fd5c2120a35a48c7c9358990c66d222d873a1cd14138dea136dd075d0",
  ],
  [
    "editors/vscode/test/css-hover-initial-project-source.ts",
    null,
    "cdbcd28ad29ee6f96b6d06fd9c0b68e2317e0473:2761:b493bb88de7d307294f6826a3809eee8c16ad04f69ed431db2db60a0559f760e",
  ],
  [
    "tests/tooling/fixtures/css-hover-initial-project/foreground-before.rs.txt",
    null,
    "a4b1e81960029f488921f917e46589f1d7655da9:10272:752179a706971407290c7144720bbcbc63261031bf6b5378bdb8cc9427b601ac",
  ],
  [
    "tests/tooling/fixtures/css-hover-initial-project/initial-project.rs.txt",
    null,
    "80ba6f52a001082d23f117172e5045acb99dcaa8:10784:8b7f11857234b5e2b1ad41bd8777993e60303589a5d64eb1b393ef307a27449e",
  ],
  [
    "tests/tooling/vscode-css-hover-foreground-source.test.ts",
    "4dbb38efd505abf222ede8993e24b836edd42a15:5624:d03891046d7576ec8e663e54df0309953387c77b9fed724737a093a7b96f7c51",
    "3e10f30a50a71b8a63e2aa59edf9057e11c6a356:5666:cd09384ef54ce621a8d0f630b91c50cc7003c35ee5105ca5528327eab7cf0bb5",
  ],
  [
    "tests/tooling/vscode-css-hover-initial-project-source.test.ts",
    null,
    "481f5602faef04e7ce0ec6379cb4b166f8b273c5:8337:9385ed4045d78d9a079c01e4fea34db2d05154dc7919400ccd48756ad3e1d8a4",
  ],
] as const;
export const SOURCE_PATHS = SOURCE_ROWS.map(([file]) => file);
export const REUSED_HELPER_BLOBS = {
  "shipping-adapter.ts": "34df6a9be8a54c0e2236764b0f3d62b3789cfedb",
  "readiness-parity.ts": "8e7e6e3f542d7292caf7bb7fa999e409ef9f8088",
} as const;
type Pin = { mode: string; type: string; gitBlob: string; bytes: number; sha256: string };
export function checkSourceRow(
  actual: { mode: string; type: string; gitBlob: string; body: Buffer } | null,
  expected: Pin | null,
): void {
  if (expected === null) {
    assert.equal(actual, null, "absent baseline path must remain absent");
    return;
  }
  assert.ok(actual);
  assert.equal(actual.mode, expected.mode);
  assert.equal(actual.type, expected.type);
  assert.equal(actual.gitBlob, expected.gitBlob);
  assert.equal(actual.body.length, expected.bytes);
  assert.equal(createHash("sha256").update(actual.body).digest("hex"), expected.sha256);
  assert.equal(
    createHash("sha1")
      .update(Buffer.from(`blob ${actual.body.length}\0`))
      .update(actual.body)
      .digest("hex"),
    expected.gitBlob,
  );
}
function git(repo: string, ...args: string[]): Buffer {
  const result = spawnSync("git", args, { cwd: repo, maxBuffer: 8 * 1024 * 1024 });
  assert.equal(result.status, 0, result.stderr.toString());
  return result.stdout;
}
export function decodePin(value: string | null): Pin | null {
  if (value === null) return null;
  const parts = /^([a-f0-9]{40}):([1-9][0-9]*):([a-f0-9]{64})$/.exec(value);
  assert.ok(parts);
  const bytes = Number(parts[2]);
  assert.ok(Number.isSafeInteger(bytes));
  return { mode: "100644", type: "blob", gitBlob: parts[1]!, bytes, sha256: parts[3]! };
}
export function assertForegroundSources(repo: string): void {
  for (const [role, revision] of [
    ["B", BASE_SOURCE],
    ["C", CANDIDATE_SOURCE],
  ] as const) {
    for (const [file, before, after] of SOURCE_ROWS) {
      const expected = decodePin(role === "B" ? before : after);
      const metadata = git(repo, "ls-tree", revision, "--", file).toString().trimEnd();
      if (!metadata) {
        checkSourceRow(null, expected);
        continue;
      }
      const match = /^(100644) (blob) ([0-9a-f]{40})\t(.+)$/.exec(metadata);
      assert.ok(match);
      assert.equal(match[4], file);
      checkSourceRow(
        {
          mode: match[1]!,
          type: match[2]!,
          gitBlob: match[3]!,
          body: git(repo, "show", `${revision}:${file}`),
        },
        expected,
      );
    }
  }
  assert.equal(BOUND_PATHS.length, 50);
  const changed = BOUND_PATHS.filter(
    (file) =>
      !git(repo, "show", `${BASE_SOURCE}:${file}`).equals(
        git(repo, "show", `${CANDIDATE_SOURCE}:${file}`),
      ),
  );
  assert.deepEqual(changed, [
    "crates/vize_maestro/src/ide/context.rs",
    "crates/vize_maestro/src/server/state/corsa.rs",
  ]);
  assert.ok(changed.every((file) => (SOURCE_PATHS as readonly string[]).includes(file)));
}
