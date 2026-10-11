import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  BUILD_RECIPE,
  expectedBuildIdentity,
  sourceRevision,
  validateBuildReceipt,
} from "../../../../tests/differential/build-receipt.ts";
import {
  CLI_MEMBERS,
  TRANSPORT_MEMBER,
  ORACLE_RECIPE,
  command,
  digest,
  physicalParent,
  regularMember,
  runtimeIdentity,
} from "./source-cli-identity.ts";

type Identity = ReturnType<typeof runtimeIdentity>;
type Member = ReturnType<typeof regularMember>;
type Envelope = { identity: Identity; recipe: string; oracleRecipe: string; members: Member[] };

function validateMembers(root: string, envelope: Envelope): void {
  assert.deepEqual(
    envelope.members.map((member) => member.path),
    CLI_MEMBERS,
  );
  assert.deepEqual(
    envelope.members,
    CLI_MEMBERS.map((member) => regularMember(root, member)),
  );
  assert.deepEqual(
    envelope.members.map((member) => member.mode),
    [0o755, 0o644, 0o755],
  );
  for (const member of envelope.members) assert.ok(member.bytes > 0, "empty build member");
  const receipt = JSON.parse(fs.readFileSync(path.join(root, CLI_MEMBERS[1]), "utf8"));
  validateBuildReceipt(receipt, expectedBuildIdentity(root));
  assert.equal(receipt.sourceRevision, envelope.identity.sourceRevision);
}

export function packSourceCli(root: string, archive: string, identity: Identity) {
  const envelope: Envelope = {
    identity,
    recipe: BUILD_RECIPE,
    oracleRecipe: ORACLE_RECIPE,
    members: CLI_MEMBERS.map((member) => regularMember(root, member)),
  };
  validateMembers(root, envelope);
  const stage = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "source-cli-pack-")));
  try {
    for (const member of CLI_MEMBERS) {
      physicalParent(stage, member);
      fs.copyFileSync(path.join(root, member), path.join(stage, member));
      fs.chmodSync(
        path.join(stage, member),
        envelope.members.find((entry) => entry.path === member)!.mode,
      );
    }
    assert.deepEqual(
      envelope.members,
      CLI_MEMBERS.map((member) => regularMember(stage, member)),
    );
    const receipt = Buffer.from(`${JSON.stringify(envelope, null, 2)}\n`);
    fs.writeFileSync(path.join(stage, TRANSPORT_MEMBER), receipt, { mode: 0o644 });
    command(stage, "tar", ["-czf", path.resolve(archive), ...CLI_MEMBERS, TRANSPORT_MEMBER]);
    return { sha256: digest(fs.readFileSync(archive)), receiptSha256: digest(receipt) };
  } finally {
    fs.rmSync(stage, { recursive: true, force: true });
  }
}

export function restoreSourceCli(
  root: string,
  archive: string,
  expectedSha: string,
  identity: Identity,
): void {
  archive = path.resolve(archive);
  assert.match(expectedSha, /^[a-f0-9]{64}$/);
  assert.ok(fs.lstatSync(archive).isFile(), "regular source CLI archive required");
  assert.equal(digest(fs.readFileSync(archive)), expectedSha, "source CLI archive digest changed");
  assert.deepEqual(command(root, "tar", ["-tzf", archive]).split("\n"), [
    ...CLI_MEMBERS,
    TRANSPORT_MEMBER,
  ]);
  const types = command(root, "tar", ["-tvzf", archive])
    .split("\n")
    .map((line) => line[0]);
  assert.deepEqual(types, ["-", "-", "-", "-"], "archive requires regular members without links");
  const stage = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "source-cli-restore-")));
  try {
    command(stage, "tar", ["-xzf", archive]);
    assert.equal(regularMember(stage, TRANSPORT_MEMBER).mode, 0o644);
    const envelope = JSON.parse(
      fs.readFileSync(path.join(stage, TRANSPORT_MEMBER), "utf8"),
    ) as Envelope;
    assert.deepEqual(envelope.identity, identity, "source CLI producer identity changed");
    assert.equal(envelope.recipe, BUILD_RECIPE);
    assert.equal(envelope.oracleRecipe, ORACLE_RECIPE);
    const cargo = fs.readFileSync(path.join(root, "Cargo.toml"), "utf8");
    const version = cargo.match(/\[workspace\.package\][\s\S]*?^version = "([^"]+)"/m);
    assert.ok(version, "workspace package version is required");
    const expected = {
      sourceRevision: sourceRevision(root),
      binaryPath: CLI_MEMBERS[0],
      binarySha256: envelope.members[0].sha256,
      cliVersion: `vize ${version[1]}`,
    };
    validateBuildReceipt(
      JSON.parse(fs.readFileSync(path.join(stage, CLI_MEMBERS[1]), "utf8")),
      expected,
    );
    assert.deepEqual(
      envelope.members.map((member) => member.path),
      CLI_MEMBERS,
    );
    assert.deepEqual(
      envelope.members,
      CLI_MEMBERS.map((member) => regularMember(stage, member)),
    );
    assert.deepEqual(
      envelope.members.map((member) => member.mode),
      [0o755, 0o644, 0o755],
    );
    for (const member of envelope.members) assert.ok(member.bytes > 0, "empty build member");
    for (const member of CLI_MEMBERS) physicalParent(root, member);
    for (const member of [...CLI_MEMBERS, TRANSPORT_MEMBER]) {
      const destination = member === TRANSPORT_MEMBER ? "target/ci/vize.source-build.json" : member;
      physicalParent(root, destination);
      const temporary = `${path.join(root, destination)}.shared-${process.pid}`;
      fs.copyFileSync(path.join(stage, member), temporary, fs.constants.COPYFILE_EXCL);
      fs.chmodSync(
        temporary,
        member === CLI_MEMBERS[1] || member === TRANSPORT_MEMBER ? 0o644 : 0o755,
      );
      fs.renameSync(temporary, path.join(root, destination));
    }
    validateBuildReceipt(
      JSON.parse(fs.readFileSync(path.join(root, CLI_MEMBERS[1]), "utf8")),
      expectedBuildIdentity(root),
    );
    assert.deepEqual(
      envelope.members,
      CLI_MEMBERS.map((member) => regularMember(root, member)),
    );
  } finally {
    fs.rmSync(stage, { recursive: true, force: true });
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const root = process.cwd();
  const [mode, archive, expectedSha] = process.argv.slice(2);
  assert.ok(archive, "archive path is required");
  const identity = runtimeIdentity(root);
  if (mode === "pack") {
    assert.equal(expectedSha, undefined);
    const hashes = packSourceCli(root, archive, identity);
    assert.ok(process.env.GITHUB_OUTPUT);
    fs.appendFileSync(
      process.env.GITHUB_OUTPUT,
      `sha256=${hashes.sha256}\nreceipt-sha256=${hashes.receiptSha256}\n`,
    );
  } else {
    assert.equal(mode, "restore");
    assert.ok(expectedSha);
    restoreSourceCli(root, archive, expectedSha, identity);
  }
}
