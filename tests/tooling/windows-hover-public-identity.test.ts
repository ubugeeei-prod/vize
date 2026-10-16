import assert from "node:assert/strict";
import { test } from "node:test";
import { verifyPackageIdentity } from "../../tools/support/release/windows_hover_acceptance.ts";

const version = "0.457.0",
  head = "a".repeat(40);
const digest = Buffer.alloc(64, 1),
  integrity = `sha512-${digest.toString("base64")}`;
function fixture() {
  const name = "@vizejs/native-win32-x64-msvc";
  const tarball =
    "https://registry.npmjs.org/@vizejs/native-win32-x64-msvc/-/native-win32-x64-msvc-0.457.0.tgz";
  return {
    name,
    metadata: { name, version, dist: { integrity, tarball } },
    installed: { version, integrity, resolved: tarball },
    statement: {
      predicateType: "https://slsa.dev/provenance/v1",
      subject: [
        {
          name: `pkg:npm/%40vizejs/native-win32-x64-msvc@${version}`,
          digest: { sha512: digest.toString("hex") },
        },
      ],
      predicate: {
        buildDefinition: {
          externalParameters: {
            workflow: {
              repository: "https://github.com/ubugeeei-prod/vize",
              path: ".github/workflows/release.yml",
              ref: `refs/heads/release/v${version}`,
            },
          },
          resolvedDependencies: [
            {
              uri: `git+https://github.com/ubugeeei-prod/vize@refs/heads/release/v${version}`,
              digest: { gitCommit: head },
            },
          ],
        },
        runDetails: {
          metadata: {
            invocationId: "https://github.com/ubugeeei-prod/vize/actions/runs/1/attempts/1",
          },
        },
      },
    },
  };
}

test("public package identity retains exact registry artifact and official source", () => {
  const f = fixture();
  const result = verifyPackageIdentity(f.name, version, head, f.metadata, f.statement, f.installed);
  assert.equal(result.source, head);
  assert.equal(result.integrity, integrity);
  assert.equal(result.tarball, f.metadata.dist.tarball);
});

test("Windows acceptance rejects wrong source, provider, archive and checkout substitutions", () => {
  const mutations = [
    (f: ReturnType<typeof fixture>) => {
      f.statement.predicate.buildDefinition.resolvedDependencies[0].digest.gitCommit = "b".repeat(
        40,
      );
    },
    (f: ReturnType<typeof fixture>) => {
      f.statement.predicate.buildDefinition.externalParameters.workflow.repository =
        "https://github.com/foreign/vize";
    },
    (f: ReturnType<typeof fixture>) => {
      f.statement.predicate.buildDefinition.externalParameters.workflow.ref = "refs/heads/main";
    },
    (f: ReturnType<typeof fixture>) => {
      f.statement.subject[0].digest.sha512 = "00".repeat(64);
    },
    (f: ReturnType<typeof fixture>) => {
      f.metadata.name = "@vizejs/native-linux-x64-gnu";
    },
    (f: ReturnType<typeof fixture>) => {
      f.installed.version = "0.456.0";
    },
    (f: ReturnType<typeof fixture>) => {
      f.installed.resolved = "file:../../npm/native";
    },
    (f: ReturnType<typeof fixture>) => {
      f.installed.integrity = `sha512-${Buffer.alloc(64, 2).toString("base64")}`;
    },
  ];
  for (const mutate of mutations) {
    const f = fixture();
    mutate(f);
    assert.throws(() =>
      verifyPackageIdentity(f.name, version, head, f.metadata, f.statement, f.installed),
    );
  }
});
