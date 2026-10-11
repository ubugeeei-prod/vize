import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { setTimeout as delay } from "node:timers/promises";
import { referenceSource } from "./n8n-reference-source.mjs";
import { runQualificationBranches } from "./project-qualification-branches.mjs";

const schema = "vize.oxlint.n8n-preparation";
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
export const replayReadyPath = (artifacts) => path.join(artifacts, "n8n-preparation.json");

export function parsePreparationOptions(args, environment = process.env) {
  const remaining = [];
  let selected;
  for (let index = 0; index < args.length; index++) {
    const argument = args[index];
    if (argument === "--preparation-mode" || argument.startsWith("--preparation-mode=")) {
      assert.equal(selected, undefined, "duplicate preparation mode");
      selected = argument.includes("=") ? argument.slice(argument.indexOf("=") + 1) : args[++index];
      assert.ok(selected && !selected.startsWith("--"), "missing preparation mode");
    } else remaining.push(argument);
  }
  const mode = selected ?? environment.VIZE_OXLINT_PREPARATION_MODE ?? "overlap";
  assert.ok(["sequential", "overlap"].includes(mode), "unknown preparation mode");
  return { mode, args: remaining };
}

function identity({ receipt, binary }) {
  return {
    source: receipt.source,
    toolchain: receipt.toolchain,
    binary: { path: fs.realpathSync(binary), sha256: receipt.frozen.sha256 },
  };
}

function validateReplay(replay, context) {
  const { artifacts } = context;
  const expected = identity(context);
  assert.equal(replay.output, path.join(artifacts, "n8n"));
  assert.equal(replay.custody.schema, "vize.oxlint.n8n-source-native");
  assert.equal(replay.custody.version, 1);
  assert.deepEqual(replay.custody.source, expected.source);
  assert.deepEqual(replay.custody.toolchain, expected.toolchain);
  assert.deepEqual(replay.custody.binary, expected.binary);
  assert.equal(replay.custody.calls, path.join(replay.output, "native-calls.jsonl"));
  assert.equal(replay.custody.sources, path.join(replay.output, "sources"));
  assert.deepEqual(replay.reference.source, referenceSource);
  const reference = JSON.parse(fs.readFileSync(path.join(replay.output, "reference-source.json")));
  for (const key of ["source", "physicalSource", "build", "bundles", "limits"])
    assert.deepEqual(replay.reference[key], reference[key]);
  assert.deepEqual(reference.build.args, ["pack"]);
  assert.equal(replay.reference.plugin, path.join(reference.build.cwd, "dist/index.mjs"));
  assert.equal(replay.reference.wrapper, path.join(reference.build.cwd, "dist/cli.mjs"));
  for (const { file, sha256 } of reference.bundles) {
    assert.ok(file.startsWith("dist/") && !file.split("/").includes(".."));
    assert.equal(hash(fs.readFileSync(path.join(reference.build.cwd, file))), sha256);
    assert.equal(
      hash(fs.readFileSync(path.join(replay.output, "reference-bundles", file))),
      sha256,
    );
  }
  for (const { filename, sha256 } of reference.physicalSource) {
    assert.ok(filename.startsWith("npm/oxlint/") && !filename.split("/").includes(".."));
    assert.equal(hash(fs.readFileSync(path.join(reference.build.cwd, filename.slice(11)))), sha256);
  }
  assert.equal(hash(fs.readFileSync(context.binary)), expected.binary.sha256);
}

function evidenceFiles(replay) {
  return [
    path.join(replay.output, "bridge", "bridge-vectors.jsonl"),
    replay.custody.calls,
    path.join(replay.output, "reference-source.json"),
  ];
}

function writePacket(context, packet) {
  const destination = replayReadyPath(context.artifacts);
  const temporary = `${destination}.pending`;
  assert.ok(!fs.existsSync(destination), "preparation outcome already published");
  fs.writeFileSync(temporary, JSON.stringify(packet, null, 2) + "\n", { flag: "wx" });
  // Waiters can observe only the complete success or failure packet.
  fs.renameSync(temporary, destination);
}

export function publishReplayReady(context, replay) {
  validateReplay(replay, context);
  const prepared = { ...replay, hosts: [], hostCustodies: [] };
  writePacket(context, {
    schema,
    version: 1,
    status: "ready",
    identity: identity(context),
    replay: prepared,
    descriptorSha256: hash(JSON.stringify(prepared)),
    evidence: evidenceFiles(prepared).map((file) => ({
      file: path.relative(context.artifacts, file),
      sha256: hash(fs.readFileSync(file)),
    })),
  });
}

export function publishReplayFailure(context) {
  // The complete original error remains in the existing branch outcome file.
  writePacket(context, { schema, version: 1, status: "failed", identity: identity(context) });
}

export async function waitForReplayReady(context, { timeoutMs = 1_800_000 } = {}) {
  const started = performance.now();
  for (;;) {
    let packet;
    try {
      packet = JSON.parse(fs.readFileSync(replayReadyPath(context.artifacts), "utf8"));
    } catch (error) {
      if (error.code !== "ENOENT") throw error;
      assert.ok(performance.now() - started < timeoutMs, "n8n preparation wait timed out");
      await delay(100);
      continue;
    }
    assert.equal(packet.schema, schema);
    assert.equal(packet.version, 1);
    assert.deepEqual(packet.identity, identity(context));
    assert.ok(["ready", "failed"].includes(packet.status));
    assert.equal(packet.status, "ready", "n8n baseline/reference preparation failed");
    assert.equal(
      packet.descriptorSha256,
      hash(JSON.stringify(packet.replay)),
      "replay descriptor changed",
    );
    validateReplay(packet.replay, context);
    const files = evidenceFiles(packet.replay);
    assert.deepEqual(
      packet.evidence,
      files.map((file) => ({
        file: path.relative(context.artifacts, file),
        sha256: hash(fs.readFileSync(file)),
      })),
      "joined baseline or reference changed before host replay",
    );
    return packet.replay;
  }
}

export async function runPreparationBranches(context, { mode, prepare, runHosts }) {
  assert.ok(["sequential", "overlap"].includes(mode));
  return runQualificationBranches(
    [
      {
        id: "baseline",
        run: async () => {
          try {
            const replay = await prepare();
            publishReplayReady(context, replay);
            return replay;
          } catch (error) {
            try {
              publishReplayFailure(context);
            } catch (publicationError) {
              throw new AggregateError(
                [error, publicationError],
                "baseline failed; readiness publication also refused",
              );
            }
            throw error;
          }
        },
      },
      {
        id: "hosts",
        run: async () => {
          if (mode === "sequential") await waitForReplayReady(context);
          return runHosts();
        },
      },
    ],
    path.join(context.artifacts, "n8n-preparation-outcomes.json"),
  );
}
