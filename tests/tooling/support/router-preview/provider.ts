import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import {
  fileIdentity,
  hash,
  readAuthority,
  relativeMember,
  type FileIdentity,
} from "./authority.ts";
import { readOracles } from "./oracles.ts";

export type BuiltProvider = {
  schema: string;
  version: number;
  state: string;
  qualificationCredit: boolean;
  actor: { sourceRevision: string; runId: string; runAttempt: string };
  authoritySha256: string;
  upstream: ReturnType<typeof readAuthority>["authority"]["provider"];
  sourceInputs: ReturnType<typeof readAuthority>["authority"]["sourceInputs"];
  sourceArchive: FileIdentity;
  builder: Record<string, unknown>;
  processes: FileIdentity;
  package: {
    name: string;
    version: string;
    filename: string;
    bytes: number;
    sha256: string;
    integrity: string;
    files: FileIdentity[];
  };
};

export function verifyBuiltProvider(directory: string, expectedReceiptSha256: string) {
  assert.match(expectedReceiptSha256, /^[a-f0-9]{64}$/u);
  const bytes = fs.readFileSync(path.join(directory, "provider-build.json"));
  assert.equal(hash(bytes), expectedReceiptSha256, "same-run provider receipt authority");
  const receipt = JSON.parse(bytes.toString("utf8")) as BuiltProvider;
  const source = readAuthority();
  assert.equal(receipt.schema, "vize.router-preview-built-provider");
  assert.equal(receipt.version, 1);
  assert.equal(receipt.state, "provider-built-only");
  assert.equal(receipt.qualificationCredit, false);
  assert.equal(receipt.authoritySha256, source.sha256);
  assert.deepEqual(receipt.upstream, source.authority.provider);
  assert.deepEqual(receipt.sourceInputs, source.authority.sourceInputs);
  assert.deepEqual(receipt.actor, {
    sourceRevision: process.env.GITHUB_SHA,
    runId: process.env.GITHUB_RUN_ID,
    runAttempt: process.env.VIZE_ROUTER_PREVIEW_PRODUCER_ATTEMPT,
  });
  assert.equal(receipt.package.name, "vue-router");
  assert.equal(receipt.package.version, "5.4.0");
  assert.equal(receipt.package.filename, "vue-router-5.4.0.tgz");
  const frozen =
    readOracles().json<Pick<BuiltProvider, "upstream" | "sourceArchive" | "builder" | "package">>(
      "provider-package.json",
    );
  assert.deepEqual(
    {
      upstream: receipt.upstream,
      sourceArchive: receipt.sourceArchive,
      builder: receipt.builder,
      package: receipt.package,
    },
    frozen,
    "exact real preview build identity; version 5.4.0 alone is insufficient",
  );
  for (const expected of [receipt.sourceArchive, receipt.processes]) {
    assert.deepEqual(
      fileIdentity(directory, expected.path),
      expected,
      "complete build artifact custody",
    );
  }
  const archive = path.join(directory, relativeMember(receipt.package.filename));
  const archiveBytes = fs.readFileSync(archive);
  assert.equal(archiveBytes.length, receipt.package.bytes);
  assert.equal(hash(archiveBytes), receipt.package.sha256);
  assert.equal(
    `sha512-${createHash("sha512").update(archiveBytes).digest("base64")}`,
    receipt.package.integrity,
  );
  const members = receipt.package.files.map((entry) => relativeMember(entry.path));
  assert.equal(new Set(members).size, members.length);
  assert.deepEqual(members, [...members].sort());
  return { receipt, archive, receiptSha256: expectedReceiptSha256 };
}

export function verifyInstalledProvider(
  root: string,
  provider: ReturnType<typeof verifyBuiltProvider>,
) {
  const providerRoot = path.join(root, "node_modules/vue-router");
  const identities = provider.receipt.package.files.map((expected) => {
    const file = path.join(providerRoot, expected.path);
    assert.equal(fs.lstatSync(file).isFile(), true, expected.path);
    const bytes = fs.readFileSync(file);
    const actual = { path: expected.path, bytes: bytes.length, sha256: hash(bytes) };
    assert.deepEqual(actual, expected, `actual built package member ${expected.path}`);
    return actual;
  });
  return { root: providerRoot, identities };
}
