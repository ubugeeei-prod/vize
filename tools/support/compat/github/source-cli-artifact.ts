import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  BUILD_RECIPE,
  expectedBuildIdentity,
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
  CONSUMER_USE_MEMBER,
  expectedProducerIdentity,
  sourceCliProducerOutputs,
  sourceCliCheckout,
  verifySourceCliConsumerUse,
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
  expectedProducerIdentity(identity);
  const checkout = sourceCliCheckout(root);
  assert.equal(
    identity.sourceRevision,
    checkout.sourceRevision,
    "source CLI producer identity changed: sourceRevision",
  );
  assert.equal(
    identity.sourceTree,
    checkout.sourceTree,
    "source CLI producer identity changed: sourceTree",
  );
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
  producerAttempt = identity.runAttempt,
  expectedProducerReceiptSha256?: string,
): string {
  const checkout = sourceCliCheckout(root);
  assert.equal(
    identity.sourceRevision,
    checkout.sourceRevision,
    "source CLI producer identity changed: sourceRevision",
  );
  assert.equal(
    identity.sourceTree,
    checkout.sourceTree,
    "source CLI producer identity changed: sourceTree",
  );
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
    if (expectedProducerReceiptSha256 !== undefined) {
      assert.match(expectedProducerReceiptSha256, /^[a-f0-9]{64}$/);
      assert.equal(
        regularMember(stage, TRANSPORT_MEMBER).sha256,
        expectedProducerReceiptSha256,
        "producer envelope digest changed",
      );
    }
    const envelope = JSON.parse(
      fs.readFileSync(path.join(stage, TRANSPORT_MEMBER), "utf8"),
    ) as Envelope;
    assert.deepEqual(
      envelope.identity,
      expectedProducerIdentity(identity, producerAttempt),
      "source CLI producer identity changed",
    );
    assert.equal(envelope.recipe, BUILD_RECIPE);
    assert.equal(envelope.oracleRecipe, ORACLE_RECIPE);
    const cargo = fs.readFileSync(path.join(root, "Cargo.toml"), "utf8");
    const version = cargo.match(/\[workspace\.package\][\s\S]*?^version = "([^"]+)"/m);
    assert.ok(version, "workspace package version is required");
    const expected = {
      sourceRevision: checkout.sourceRevision,
      binaryPath: CLI_MEMBERS[0],
      binarySha256: envelope.members[0].sha256,
      // Source builds report `<version>-dev` (#9714).
      cliVersion: `vize ${version[1]}-dev`,
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
    const producerReceiptSha256 = regularMember(root, "target/ci/vize.source-build.json").sha256;
    const useBytes = Buffer.from(
      `${JSON.stringify(
        {
          schema: "vize.source-cli.consumer-use",
          version: 1,
          archiveSha256: expectedSha,
          producerReceiptSha256,
          producerIdentity: envelope.identity,
          consumerIdentity: identity,
          recipe: envelope.recipe,
          oracleRecipe: envelope.oracleRecipe,
          members: envelope.members,
        },
        null,
        2,
      )}\n`,
    );
    physicalParent(root, CONSUMER_USE_MEMBER);
    const usePath = path.join(root, CONSUMER_USE_MEMBER);
    const temporary = `${usePath}.shared-${process.pid}`;
    fs.writeFileSync(temporary, useBytes, { mode: 0o644, flag: "wx" });
    fs.renameSync(temporary, usePath);
    const useSha256 = digest(useBytes);
    verifySourceCliConsumerUse(root, envelope, identity, producerReceiptSha256, useSha256);
    return useSha256;
  } finally {
    fs.rmSync(stage, { recursive: true, force: true });
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const root = process.cwd();
  const [mode, archive, expectedSha, producerAttempt, producerReceiptSha256] =
    process.argv.slice(2);
  if (mode === "producer") {
    assert.equal(archive, undefined);
    assert.ok(process.env.GITHUB_OUTPUT);
    const outputs = sourceCliProducerOutputs(
      process.env.SOURCE_CLI_PRODUCER_OUTPUTS!,
      process.env.GITHUB_RUN_ATTEMPT!,
    );
    fs.appendFileSync(
      process.env.GITHUB_OUTPUT,
      Object.entries(outputs)
        .map(([key, value]) => `${key}=${value}\n`)
        .join(""),
    );
  } else if (mode === "consumer-use") {
    assert.equal(process.argv.slice(2).length, 1);
    const checkout = sourceCliCheckout(root);
    assert.equal(
      checkout.sourceRevision,
      process.env.GITHUB_SHA,
      "checked source differs from workflow source",
    );
    assert.match(process.env.GITHUB_RUN_ID ?? "", /^[1-9][0-9]*$/);
    const producerSha = process.env.VIZE_SOURCE_CLI_RECEIPT_SHA256;
    const useSha = process.env.VIZE_SOURCE_CLI_USE_RECEIPT_SHA256;
    assert.ok(producerSha, "original producer receipt digest is required");
    assert.ok(useSha, "verified consumer use receipt digest is required");
    const producer = JSON.parse(
      fs.readFileSync(path.join(root, "target/ci/vize.source-build.json"), "utf8"),
    ) as Envelope;
    const consumer = {
      ...checkout,
      runId: process.env.GITHUB_RUN_ID!,
      runAttempt: process.env.GITHUB_RUN_ATTEMPT!,
    };
    verifySourceCliConsumerUse(root, producer, consumer, producerSha, useSha);
    process.stdout.write(
      `${JSON.stringify({
        useReceiptSha256: useSha,
        producerAttempt: producer.identity.runAttempt,
        consumerAttempt: consumer.runAttempt,
      })}\n`,
    );
  } else {
    assert.ok(archive, "archive path is required");
    const identity = runtimeIdentity(root);
    if (mode === "pack") {
      assert.equal(expectedSha, undefined);
      assert.equal(producerAttempt, undefined);
      assert.equal(producerReceiptSha256, undefined);
      const hashes = packSourceCli(root, archive, identity);
      assert.ok(process.env.GITHUB_OUTPUT);
      fs.appendFileSync(
        process.env.GITHUB_OUTPUT,
        `sha256=${hashes.sha256}\nreceipt-sha256=${hashes.receiptSha256}\nproducer-attempt=${identity.runAttempt}\n`,
      );
    } else {
      assert.equal(mode, "restore");
      assert.ok(expectedSha);
      const useSha256 = restoreSourceCli(
        root,
        archive,
        expectedSha,
        identity,
        producerAttempt,
        producerReceiptSha256,
      );
      if (producerAttempt !== undefined && producerAttempt !== identity.runAttempt) {
        assert.ok(process.env.GITHUB_ENV);
        fs.appendFileSync(
          process.env.GITHUB_ENV,
          `VIZE_SOURCE_CLI_USE_RECEIPT_SHA256=${useSha256}\n`,
        );
      }
    }
  }
}
