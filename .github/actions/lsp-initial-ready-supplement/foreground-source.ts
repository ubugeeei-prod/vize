import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { BOUND_PATHS } from "../lsp-initial-ready-pair/readiness-parity.ts";
export const BASE_SOURCE = "22f85eb5dbaf663021ab03cbe6654a5081e5543a";
export const CANDIDATE_SOURCE = "dada313bba79b00443c6e07fce6d13efbfd4ddcd";
export const CANDIDATE_PARENT = "97bdba69e1ead424f14761af2d1b3c2f796eed80";
export const PREVIOUS_WORKFLOW = CANDIDATE_SOURCE;
/** Each immutable data tuple is path, B blob:bytes:SHA256 or absent, C blob:bytes:SHA256. */
export const SOURCE_ROWS = [
  [
    "crates/vize_canon/src/corsa_bridge/bridge/documents.rs",
    "88a8fd687839a8954cec303eb9a6eac4c405c773:11859:d34e785609a2f24591fb0a459fcdac7ed1baff8ab2301c9e118f2e216052a110",
    "975fbf48300f7e32ef25192494795a6a57a45bac:12775:25022572eeeb5d7f001e00b2d4ca5a1b8dc94a5f31c3e57ed20feccdb293d86f"
  ],
  [
    "crates/vize_canon/src/corsa_bridge/source_catalog.rs",
    "087c4f70c8c9bed7254aa0bb2c4c0a4ff371332c:4966:6578e435efcd153477ff57fac169bfe5d59ec1b85ec976caf7163d735297f6fc",
    "945963bc5944d575816388346d1f889682511b70:5928:2a22671c778248e21485d2c282d4a64fcd826c863a83a341dc1d14ab03d9a806"
  ],
  [
    "crates/vize_canon/src/corsa_bridge/vue_document.rs",
    "c1e9ae00851209d82b789349dd52e38580ab9c56:9886:5398eab8dd20be99759055781cac3d144ee2ca178d28759511d3a039d049cfc6",
    "3ff97624980f121d0757df97bbd06341527c1cd3:9967:9f489d42f7c5c8dbaaac6bbab7c60566fae6d8add6857cec9c488eb7723bcd0e"
  ],
  [
    "crates/vize_canon/src/corsa_bridge/vue_document/build.rs",
    "e8a7620ef941e59dc3f03e84fd2a94e592cde217:6159:9167d5a64f428d9b03f21f61f315f9ec7ef2d8aa8bbd17bd2e9fb69e708f3374",
    "a082c8e8b0946248605946e6e5b447dcac6a067d:6359:c5fd987816970f962bf9f6c60220ab67173e5462d3bf02d74e9dcd423fde8102"
  ],
  [
    "crates/vize_canon/src/corsa_bridge/vue_document/build/tests.rs",
    "1170fd24ea4d15d3c7fa0ebbfcb77c750c1a03e9:10853:183b4f2731d23f7e0d90b106ab4eabf02f86adcf24a2c3fac4124852ac51152f",
    "58d09ff814983de0586f6a3d42d53a90711541b8:10874:358f0ff1dfc4d11a722e7e1f3424635af6212b11c76b77c83636e8b6f438e0e1"
  ],
  [
    "crates/vize_canon/src/corsa_bridge/vue_document/build/tests/native_union.rs",
    "d62f9bb221cdb6872e2390a98ee7ddeacfb14882:10843:082458dc2a9d3d9322928eb7884e94201f1163f5543c61f070f8ffbeecd6e83e",
    "f8891a3eefbf3c62d9f9d4532aa62fca8fdf3798:10850:01fbd06fa7c242fd6e74e93d78f043a56df46e09a4337fd64265bb6b85a83fe2"
  ],
  [
    "crates/vize_canon/src/corsa_bridge/vue_document/build/tests/shared_payload.rs",
    null,
    "619b299b45179ab560908369a070f446d37c4a46:6986:5ba8d72f02051ab27c29b2fd71a52ba1d3ae45854644069a846454ec8de8cf5d"
  ],
  [
    "crates/vize_canon/src/corsa_bridge/vue_document/build/tests/windows_overlays.rs",
    "3cde8612c3de27bdaad6cf2e5f25754db0d4c294:6760:80f14fe0a5c97e96419bb841c8c8d499a9b5ae46b0268ba8e6e9f332b7046c94",
    "4861e4dbe28928fea7969bf6fb513d9e9072c114:6825:bc6cf13c919b4a766094e6c4b8f2184a7f0f30c1e2d19916958dbcce5197298d"
  ],
  [
    "crates/vize_canon/src/corsa_bridge/vue_document/initial_project.rs",
    "b7e4c6c2c9546331695bc739fcf97c206d488f8e:12726:18990a40129b98a2ac1a1367333b880ce62b7c867dae6e4f3beb9738ca247ce5",
    "701c449f60e2a92520093b6c4dd357576bafda6c:12736:275b9054c70e643ff3a593d4bf55e756fd12e3389d56f68bb0ac4013bd49218f"
  ],
  [
    "crates/vize_canon/src/corsa_bridge/vue_document/initial_project/tests.rs",
    "dbb802926cb4b0f96b747245c0ca605f24cb8050:13616:1f9e80a7c8da4c8734e0b46747daeaef846367a4fef42bf9d787621bd3f2765b",
    "b54dae9938fde0203b3913b280dd7af97f6306c9:13841:dd6f57fe592e559eb2dcabf4febb08311fa1eff33117db1cba539f322ef68cfc"
  ],
  [
    "crates/vize_canon/src/corsa_bridge/vue_document/model.rs",
    "064cf0178eed58f3a28be9653dc4c1d929983e40:3803:09a48a0d0c714991ac58b9d65ce07ba3c67e86e9f726975a822e8b2eafaadffe",
    "5084e1e070ecbf998f170ef4a294d432b40462a3:6262:23b8716298cd89369b6273dd465566803497495acc01a5c589299fd338d6dcdc"
  ],
  [
    "crates/vize_canon/src/corsa_bridge/vue_document_package_tests/symlink_workspace.rs",
    "af93028150996ac25f299186c202fc89420e0b81:16005:9c202327ad3402ea3426047762605ce5c5558158e2a85fa638c0c35d81de372c",
    "9b5964d60080b39b11320e948ec89339f504a5fa:16006:db247084d26a778f934a1b5836c3e5db94e79e9b55cfca1264e078ef13de6bfd"
  ],
  [
    "crates/vize_canon/src/corsa_server/diagnostics.rs",
    "eeebc69d5befae2d9d4a5c4b71343ac7f961a94a:14047:da55baab4315611a1a8d6b208c77ac07d6abcfca5c195e28c4ee3d444b81c815",
    "c10e218945841dfc5c16413ade98973f8355ba53:14054:3245787e6f898318876430a533f5e5d1f969b7e467a0514d15875787390b382a"
  ],
  [
    "crates/vize_canon/src/lsp_client/editor_lsp/configured_project/shared_transport/tests/native/replay.rs",
    "93763a414519d3428372d838646058a74a33ae8f:21021:dce5811ec19de4b3bb9037b052a75c195128e2ea5833156147cf75acd7ab5308",
    "1435f1f2e292546e2f2c54e38e7e8f2debfa1b0d:21040:edf5b338aa8a5e002431eb08ac484bba97130569e1f74be213b081d1d0c7d26f"
  ]
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
  assert.deepEqual(changed, []);
  assert.ok(changed.every((file) => (SOURCE_PATHS as readonly string[]).includes(file)));
}
