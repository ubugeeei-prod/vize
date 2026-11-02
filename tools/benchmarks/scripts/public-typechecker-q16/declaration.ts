import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { assertCustody } from "../../../support/release/computed_event_custody.ts";
import { authenticate } from "./authority.ts";
import {
  limitations,
  requireEnvironment,
  sha256,
  type DeclarationProfile,
  type SourceFile,
} from "./contract.ts";
import { declarationData, declarationInputs, type DeclarationState } from "./declaration-data.ts";
import {
  capture,
  finalCustodiesBeforeAssessment,
  healthy,
  inputImages,
  recorder,
  safeFailure,
  type Packet,
  type Recorder,
} from "./execute.ts";
import { install } from "./install.ts";
import { sourceCustody } from "./source.ts";

export interface EmittedDeclaration {
  name: string;
  bytes: Buffer;
}
function physicalFile(path: string) {
  const stat = lstatSync(path);
  assert(
    stat.isFile() && !stat.isSymbolicLink() && realpathSync(path) === path,
    "declaration input/output must be a regular canonical physical file",
  );
  assert(stat.size > 0 && stat.size <= 4 * 1024 ** 2, "bounded nonempty declaration data required");
  const raw = readFileSync(path);
  new TextDecoder("utf8", { fatal: true }).decode(raw);
  return raw;
}
function physicalDirectory(path: string) {
  const stat = lstatSync(path);
  assert(
    stat.isDirectory() && !stat.isSymbolicLink() && realpathSync(path) === path,
    "canonical physical declaration directory required",
  );
}
export function retainAndCopyDeclarations(
  output: string,
  consumer: string,
  record: Recorder,
): EmittedDeclaration[] {
  physicalDirectory(output);
  physicalDirectory(consumer);
  physicalDirectory(join(consumer, "src"));
  const names = readdirSync(output).sort();
  record.json(
    "emitted-roster.json",
    names.map((name) => {
      const stat = lstatSync(join(output, name));
      return { name, bytes: stat.size, file: stat.isFile(), symlink: stat.isSymbolicLink() };
    }),
  );
  assert.deepEqual(
    names,
    [...declarationData.outputFiles].sort(),
    "complete emitted declaration roster must be exactly the original two files",
  );
  const files = declarationData.outputFiles.map((name) => ({
    name,
    bytes: physicalFile(join(output, name)),
  }));
  for (const file of files) record.raw("emitted-" + file.name + ".raw", file.bytes);
  const vendor = join(consumer, "src/vendor");
  assert.equal(
    lstatSync(vendor, { throwIfNoEntry: false }),
    undefined,
    "generated destination must be absent before whole copy",
  );
  mkdirSync(vendor);
  for (const file of files) writeFileSync(join(vendor, file.name), file.bytes, { flag: "wx" });
  const copies = files.map((file) => {
    const copied = physicalFile(join(vendor, file.name));
    assert(copied.equals(file.bytes), "whole generated declaration copy differs");
    record.raw("copied-" + file.name + ".raw", copied);
    return { name: file.name, bytes: copied.length, sha256: sha256(copied), wholeByteEqual: true };
  });
  record.json("whole-declaration-transfer.json", {
    copies,
    sourceDeclarationsInjected: false,
    sourceHashesUsedAsPublicGolden: false,
    writerPid: "UNKNOWN",
    publicAcceptance: false,
  });
  return files;
}
export function verifyDeclarationCopies(
  output: string,
  consumer: string,
  files: EmittedDeclaration[],
) {
  physicalDirectory(output);
  physicalDirectory(join(consumer, "src/vendor"));
  assert.deepEqual(readdirSync(output).sort(), [...declarationData.outputFiles].sort());
  assert.deepEqual(
    readdirSync(join(consumer, "src/vendor")).sort(),
    [...declarationData.outputFiles].sort(),
  );
  assert.deepEqual(
    files.map((file) => file.name),
    declarationData.outputFiles,
  );
  for (const file of files) {
    assert(
      physicalFile(join(output, file.name)).equals(file.bytes),
      "producer output changed after whole retention",
    );
    assert(
      physicalFile(join(consumer, "src/vendor", file.name)).equals(file.bytes),
      "consumer declaration changed after whole copy",
    );
  }
}
export function consumerInputImages(root: string, files: SourceFile[]) {
  physicalDirectory(root);
  physicalDirectory(join(root, "src"));
  assert.deepEqual(
    readdirSync(join(root, "src")).sort(),
    ["Probe.vue", "vendor"],
    "consumer has only its original source and generated vendor closure",
  );
  return files.map((file) => {
    const path = join(root, file.path),
      stat = lstatSync(path);
    assert(
      stat.isFile() && !stat.isSymbolicLink() && realpathSync(path) === path,
      "consumer authored input must stay regular and canonical",
    );
    const raw = readFileSync(path);
    assert(
      raw.equals(Buffer.from(file.source)),
      "consumer authored input bytes differ: " + file.path,
    );
    return { path: file.path, bytes: raw.length, sha256: sha256(raw), source: file.source };
  });
}
export function assessDeclarationConsumer(state: DeclarationState, packet: Packet) {
  healthy(packet, state.expected.status);
  assert(packet.stdout === state.expected.stdout, "complete intended consumer stdout differs");
  assert(packet.stderr === state.expected.stderr, "complete intended consumer stderr differs");
  return {
    id: state.id,
    status: packet.status,
    wholeConsumerVectorMatches: true,
    publicAcceptance: false,
  };
}
export async function runDeclaration(
  options: { out: string; npm: string; npx: string },
  profile: DeclarationProfile,
) {
  assert.equal(profile.causalFix, null);
  assert.equal(profile.knownGoodSource, declarationData.knownGoodSource);
  assert.equal(realpathSync(dirname(options.out)), dirname(options.out));
  mkdirSync(options.out);
  const record = recorder(options.out),
    own = sourceCustody(),
    checkSource = () => assertCustody(own),
    base = join(dirname(options.out), "q16-a64"),
    cache = new Map<string, Buffer>();
  const producerRoot = join(base, "producer"),
    consumerRoot = join(base, "consumer"),
    output = join(base, "out");
  try {
    requireEnvironment(process.env, (rows) =>
      record.json("initial-environment-presence.json", rows),
    );
    const head = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
    assert.equal(head, process.env.GITHUB_SHA);
    record.json("controller.json", {
      head,
      run: process.env.GITHUB_RUN_ID,
      attempt: process.env.GITHUB_RUN_ATTEMPT,
      profile,
      subset: "A64",
      own,
      sourceProtocolSha256: declarationData.protocolSha256,
      knownGoodSource: profile.knownGoodSource,
      causalFix: null,
      productRepairClaimed: false,
      environment: requireEnvironment(process.env),
      limitations,
    });
    mkdirSync(base);
    const authorityOut = join(options.out, "authority");
    mkdirSync(authorityOut);
    const authority = await authenticate("after", recorder(authorityOut));
    const producerEvidence = join(options.out, "producer"),
      consumerEvidence = join(options.out, "consumer");
    mkdirSync(producerEvidence);
    mkdirSync(consumerEvidence);
    const producerRecord = recorder(producerEvidence),
      consumerRecord = recorder(consumerEvidence),
      producerFiles = declarationInputs(profile, true),
      consumerFiles = declarationInputs(profile, false);
    const producer = await install(
      producerRoot,
      producerFiles,
      options.npm,
      options.npx,
      authority,
      producerRecord,
      cache,
      ["vize"],
    );
    const consumer = await install(
      consumerRoot,
      consumerFiles,
      options.npm,
      options.npx,
      authority,
      consumerRecord,
      cache,
      ["vue-tsc"],
    );
    assert.equal(
      lstatSync(output, { throwIfNoEntry: false }),
      undefined,
      "declaration output must be absent before the original producer command",
    );
    const verifyProducer = () => {
      checkSource();
      producer.verify();
      inputImages(producerRoot, producerFiles);
    };
    producerRecord.json("inputs-before-producer.json", inputImages(producerRoot, producerFiles));
    const produced = capture(
      producerRecord,
      "declaration-producer",
      [process.execPath, options.npx, ...declarationData.producerArgs.slice(1)],
      producerRoot,
      verifyProducer,
    );
    // Successful real production is a mandatory input prerequisite, never a guessed declaration oracle.
    healthy(produced, 0);
    const emitted = retainAndCopyDeclarations(output, consumerRoot, record);
    let currentFiles = consumerFiles;
    const verifyConsumer = () => {
      checkSource();
      consumer.verify();
      consumerInputImages(consumerRoot, currentFiles);
      verifyDeclarationCopies(output, consumerRoot, emitted);
    };
    const captured: { id: string; packet: Packet }[] = [];
    for (const [index, state] of declarationData.states.entries()) {
      verifyConsumer();
      writeFileSync(join(consumerRoot, "src/Probe.vue"), state.source);
      currentFiles = consumerFiles.map((file) =>
        file.path === "src/Probe.vue" ? { ...file, source: state.source } : file,
      );
      consumerRecord.json("state-" + index + ".input.json", {
        id: state.id,
        files: consumerInputImages(consumerRoot, currentFiles),
      });
      const packet = capture(
        consumerRecord,
        "state-" + index + ".vue-tsc",
        [process.execPath, options.npx, ...declarationData.consumerArgs.slice(1)],
        consumerRoot,
        verifyConsumer,
      );
      captured.push({ id: state.id, packet });
      consumerRecord.json("state-" + index + ".complete.json", {
        id: state.id,
        packet,
        publicAcceptance: false,
      });
    }
    record.json("all-complete-observations.json", {
      producer: produced,
      consumer: captured,
      producerStreamsOwned: false,
      emitted: emitted.map((file) => ({
        name: file.name,
        bytes: file.bytes.length,
        sha256: sha256(file.bytes),
      })),
      causalFix: null,
      publicAcceptance: false,
    });
    const final = finalCustodiesBeforeAssessment(
      [
        {
          id: "producer",
          verify() {
            verifyProducer();
            verifyDeclarationCopies(output, consumerRoot, emitted);
          },
        },
        { id: "consumer", verify: verifyConsumer },
      ],
      record,
    );
    assert(
      final.every((row) => row.retainedRouteAndInputs),
      "final declaration route/input/copy custody refused; all raw remains retained",
    );
    const outcomes = captured.map((row, index) => {
      try {
        return {
          result: assessDeclarationConsumer(declarationData.states[index]!, row.packet),
          failure: null,
        };
      } catch (error) {
        return { result: null, failure: safeFailure(error) };
      }
    });
    record.json("result.json", {
      status: "COMPLETE_DECLARATION_CAPTURE_SOURCE_INTENT_ASSESSED",
      outcomes,
      causalFix: null,
      knownGoodSource: profile.knownGoodSource,
      productRepairClaimed: false,
      producerStreamsOwned: false,
      actualWholeTwoFileTransfer: true,
      sourceDeclarationsInjected: false,
      accepted: false,
      publicAcceptance: false,
      issueQualified: false,
      selectedCompilerPid: null,
      wholeNodeModulesFilesetQualified: false,
      limitations,
    });
    assert(
      outcomes.every((row) => row.failure === null),
      "whole intended declaration consumer vectors differ; retain failed raw without fitting",
    );
  } catch (error) {
    record.json("failure.json", {
      failure: safeFailure(error),
      accepted: false,
      publicAcceptance: false,
      causalFix: null,
    });
    throw error;
  }
}
