import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { BOUND_PATHS } from "../lsp-initial-ready-pair/readiness-parity.ts";
export const BASE_SOURCE = "6395f93131f6d4cdb4b429dece08d9b15bbbef4c";
export const CANDIDATE_SOURCE = "03ad1ca3e78fc97e8995882bd61942ce6fcbda23";
export const INTERMEDIATE_SOURCE = "3f276046ca36ee4764fd6b9f6cd5c123f5048052";
export const SECOND_SOURCE = "dbbe171c70503ae3017cf293e58c2301399cc5bc";
export const FIRST_SOURCE = "1248e5a903c6de56f2e4630650a048744d135edf";
export const PREVIOUS_WORKFLOW = CANDIDATE_SOURCE;
export const SOURCE_ROWS = [
  {
    path: ".github/workflows/lsp-scheduler-cost-carrier.yml",
    B: {
      mode: "100644",
      type: "blob",
      gitBlob: "6fd3042ddbf073ace3642ab459a802f8be070dd6",
      bytes: 4992,
      sha256: "ff89a0dd80dbedea2cdade5d4de7f28117fa903003484468be5f6fb28b2489b8",
    },
    C: {
      mode: "100644",
      type: "blob",
      gitBlob: "71ab70ea9360d209353cff0b6b56b86c3de52ff1",
      bytes: 5019,
      sha256: "71c25d47236297eb6e3c5c56de4ba71358fd5e8cb52592d0c17a437cc55b8200",
    },
  },
  {
    path: "crates/vize_maestro/src/ide/context.rs",
    B: {
      mode: "100644",
      type: "blob",
      gitBlob: "f27356d53d8c8ff37cea98c1d4c0f1bf5a8b3375",
      bytes: 10542,
      sha256: "be6eb2af0f4364738915f16e965f5366fd6fb021a6f2557a0ff93091d34f33f9",
    },
    C: {
      mode: "100644",
      type: "blob",
      gitBlob: "e2643ca8a05a211618782bbf56543f7649601a55",
      bytes: 11452,
      sha256: "39c428552abb0fe2e1c883f8b54a17acc24e48538b739e0c574524581bfda10e",
    },
  },
  {
    path: "crates/vize_maestro/src/ide/corsa_support/canonical/open.rs",
    B: {
      mode: "100644",
      type: "blob",
      gitBlob: "cdb3c1820f204dc5ec24db6a1971062ec73f34f9",
      bytes: 9496,
      sha256: "36ef2dbdb1e013a28fcbd19565837d01e64d444be4c7c0592d299e457ed21218",
    },
    C: {
      mode: "100644",
      type: "blob",
      gitBlob: "b832973c37744a8cbcd5d10af674bcf8e050b48e",
      bytes: 9537,
      sha256: "a8d1b667d6d53505c41b65d94883dfd6f8b90db4237dbe35856c26ab6539daa5",
    },
  },
  {
    path: "crates/vize_maestro/src/server/handlers/typed.rs",
    B: {
      mode: "100644",
      type: "blob",
      gitBlob: "d378281fbcda6dda2f550e21fa3ccde3905d8a7d",
      bytes: 9129,
      sha256: "a287bbc3060de2a5c01963fe82962501b5f688091c912ec10e97adc402e6859a",
    },
    C: {
      mode: "100644",
      type: "blob",
      gitBlob: "a4b1e81960029f488921f917e46589f1d7655da9",
      bytes: 10272,
      sha256: "752179a706971407290c7144720bbcbc63261031bf6b5378bdb8cc9427b601ac",
    },
  },
  {
    path: "crates/vize_maestro/src/server/initial_diagnostics.rs",
    B: {
      mode: "100644",
      type: "blob",
      gitBlob: "ba62587e75de5287b4d5f80f099b6c7f9d17d024",
      bytes: 11294,
      sha256: "75f8703e8b7e0b46761e4b1cf414e7816e61ab59e15b58694dda0a1e6a93c1c4",
    },
    C: {
      mode: "100644",
      type: "blob",
      gitBlob: "f0e5ec1d9bd815b527ebb4203d28ba81b15ebd71",
      bytes: 11365,
      sha256: "6d9c7de30a7194623d5888387d4be20fe9075fee4050aae5c71257206ed69491",
    },
  },
  {
    path: "crates/vize_maestro/src/server/initial_diagnostics/foreground_ready.rs",
    B: null,
    C: {
      mode: "100644",
      type: "blob",
      gitBlob: "9158478f98434ca0575e49f11aabbe51bdac75d0",
      bytes: 2018,
      sha256: "d933915024dd12ef9eee3a84d79ffbc23f04828f97c6e21e273fd1e967269279",
    },
  },
  {
    path: "crates/vize_maestro/src/server/initial_diagnostics/foreground_ready/tests.rs",
    B: null,
    C: {
      mode: "100644",
      type: "blob",
      gitBlob: "7eedaace7c1c47d333d1772aef2669a8c164759b",
      bytes: 15878,
      sha256: "6325fa82a4abea67ed56be2271e1191cb20c1a2522d2ebe3c625a4097e27a2bd",
    },
  },
  {
    path: "crates/vize_maestro/src/server/initial_diagnostics/pending.rs",
    B: {
      mode: "100644",
      type: "blob",
      gitBlob: "2c12ae181a0b132a93ee4fec34d4b0ca23cf5121",
      bytes: 4676,
      sha256: "62e5082c00ac99b996901c1ef3b92d79c756af54619585296b97a5f59fb1488f",
    },
    C: {
      mode: "100644",
      type: "blob",
      gitBlob: "4b924d3ddda758513c974f3770086ad20e9df42d",
      bytes: 5017,
      sha256: "0b2892c94a102a8005a8874431a1edf82903ea0bd78dee2e348bc13c7e03185f",
    },
  },
  {
    path: "crates/vize_maestro/src/server/native_requests.rs",
    B: {
      mode: "100644",
      type: "blob",
      gitBlob: "e84428ea57066cc8b52a17fb27c2ab7ae8011c16",
      bytes: 8625,
      sha256: "97c7d38b5110ee47d104f1d305af75b6aff59fd3cd75d3a3099f555ffd405489",
    },
    C: {
      mode: "100644",
      type: "blob",
      gitBlob: "10cf352069a11193e97120aa9ddd9689e305de66",
      bytes: 10544,
      sha256: "1729d0b02b9981eaaef8f3691ba45cc8b1f6399cf86ddec25b74ab86b5dc6600",
    },
  },
  {
    path: "crates/vize_maestro/src/server/project_contexts.rs",
    B: {
      mode: "100644",
      type: "blob",
      gitBlob: "dc4e9abf9a511f34b4fa2bb8b160c3d6114ac50e",
      bytes: 6927,
      sha256: "bab88463abc9bb505dfa5daf4c6759bab738de74a57e8571b1d44511f2ec3ae2",
    },
    C: {
      mode: "100644",
      type: "blob",
      gitBlob: "ab9eb8278cfd8a7c3f9942c98ea472b3e09d346c",
      bytes: 8199,
      sha256: "35192a331f50978bb3d3521f392bc32cbec2ddffbb3c635c593be0cf1fc18315",
    },
  },
  {
    path: "editors/vscode/test/css-hover-foreground-source.ts",
    B: null,
    C: {
      mode: "100644",
      type: "blob",
      gitBlob: "40e9df2df978d87153511208a1f789d2e54c33e1",
      bytes: 6672,
      sha256: "ccb878667e4be0b05118d172a9e43499ed9623356679ea734f2c77d9885ec0a1",
    },
  },
  {
    path: "editors/vscode/test/css-hover-source-custody.mjs",
    B: {
      mode: "100644",
      type: "blob",
      gitBlob: "3ba1c8a9b4353e25ca0e222313371a53a3014788",
      bytes: 21087,
      sha256: "0708f2ba69b87e326ac2011eb6b99ab910a1d44aaf93a04a0163dce3966e5b70",
    },
    C: {
      mode: "100644",
      type: "blob",
      gitBlob: "73b22b130b6b02f0533e4af0021924f55fac6a95",
      bytes: 20839,
      sha256: "18806ed7a1b533ac9c3d548cfa8c32d392fcf552c6bd761f2a0c356aa63600ea",
    },
  },
  {
    path: "tests/tooling/vscode-css-hover-foreground-source.test.ts",
    B: null,
    C: {
      mode: "100644",
      type: "blob",
      gitBlob: "4dbb38efd505abf222ede8993e24b836edd42a15",
      bytes: 5624,
      sha256: "d03891046d7576ec8e663e54df0309953387c77b9fed724737a093a7b96f7c51",
    },
  },
] as const;
export const SOURCE_PATHS = SOURCE_ROWS.map((row) => row.path);
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
export function assertForegroundSources(repo: string): void {
  for (const [role, revision] of [
    ["B", BASE_SOURCE],
    ["C", CANDIDATE_SOURCE],
  ] as const) {
    for (const row of SOURCE_ROWS) {
      const metadata = git(repo, "ls-tree", revision, "--", row.path).toString().trimEnd();
      if (!metadata) {
        checkSourceRow(null, row[role]);
        continue;
      }
      const match = /^(100644) (blob) ([0-9a-f]{40})\t(.+)$/.exec(metadata);
      assert.ok(match);
      assert.equal(match[4], row.path);
      checkSourceRow(
        {
          mode: match[1]!,
          type: match[2]!,
          gitBlob: match[3]!,
          body: git(repo, "show", `${revision}:${row.path}`),
        },
        row[role],
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
  assert.deepEqual(changed, ["crates/vize_maestro/src/ide/context.rs"]);
  assert.ok(SOURCE_ROWS.some((row) => row.path === changed[0]));
  for (const file of BOUND_PATHS)
    if (file !== changed[0])
      assert.equal(
        git(repo, "rev-parse", `${BASE_SOURCE}:${file}`).toString(),
        git(repo, "rev-parse", `${CANDIDATE_SOURCE}:${file}`).toString(),
      );
}
