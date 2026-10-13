import assert from "node:assert/strict";
import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  artifact,
  originalTest,
  providerRevision,
  requireArtifact,
  sha256,
  source,
} from "./signature-archive-capture.mjs";
import {
  collectPipes,
  framedMessages,
  nativeSessions,
  parseTrace,
  pipeInode,
  pipeInodes,
  processClosure,
} from "./signature-strace.mjs";

const json = (path) => JSON.parse(readFileSync(path, "utf8"));
const save = (path, value) => writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);

export function validateCapture(directory) {
  const manifest = json(join(directory, "source.json"));
  assert.equal(manifest.source, source);
  assert.equal(manifest.originalTest, originalTest);
  assert.equal(
    readFileSync(join(directory, "archive-zip.sha256"), "utf8").split(/\s+/)[0],
    artifact.digest.slice(7),
  );
  assert.match(readFileSync(join(directory, "driver-sha.txt"), "utf8").trim(), /^[0-9a-f]{40}$/);
  for (const name of [
    "driver-workflow.yml",
    "signature-archive-capture.mjs",
    "signature-archive-libdirs.mjs",
    "signature-strace.mjs",
    "signature-capture-integrity.mjs",
    "strace-version.txt",
  ]) {
    assert.ok(
      readFileSync(join(directory, name)).length > 0,
      `required retained file missing/empty: ${name}`,
    );
  }
  // Pinned nextest's human reporter writes to stderr; empty stdout is valid.
  // Require and retain both complete files without inventing test output.
  for (const name of ["test.stdout", "test.stderr"]) readFileSync(join(directory, name));
  const receipt = json(join(directory, "archive-receipt.json"));
  assert.equal(receipt.schemaVersion, 3);
  assert.equal(receipt.sha, source);
  assert.equal(receipt.tree, manifest.tree);
  assert.equal(receipt.workspaceRoot, "/home/runner/_work/vize/vize");
  assert.equal(receipt.requireTsgo, "1");
  assert.equal(receipt.disableTsgo, null);
  assert.equal(receipt.nextestVersion, "0.9.146");
  assert.equal(receipt.cargoProfile, "ci");
  assert.deepEqual(manifest.environment, {
    requireTsgo: "1",
    disableTsgo: null,
    nuxtIterations: "100",
    rustTestThreads: "4",
  });
  assert.ok(
    manifest.bodies.some((body) => body.path === manifest.binary),
    "archive binary body missing",
  );
  assert.ok(
    manifest.bodies.some((body) => body.path === manifest.provider),
    "native provider body missing",
  );
  requireArtifact(json(join(directory, "artifact.json")));
  assert.deepEqual(json(join(directory, "source-unchanged.json")), {
    source,
    tree: manifest.tree,
    unchanged: true,
  });
  for (const body of manifest.bodies) {
    const bytes = readFileSync(join(directory, body.retained));
    assert.equal(bytes.length, body.bytes);
    assert.equal(sha256(bytes), body.sha256);
  }
  assert.equal(manifest.providerRevision, providerRevision);
  const packageBody = manifest.bodies.find((body) => body.path === manifest.providerPackage);
  assert.ok(packageBody, "complete native provider package authority missing");
  const providerPackage = json(join(directory, packageBody.retained));
  assert.equal(providerPackage.version, "7.0.2");
  assert.equal(providerPackage.gitHead, providerRevision);
  const runtime = json(join(directory, "library-runtime.json"));
  assert.equal(runtime.cwd, "/home/runner/_work/vize/vize");
  assert.deepEqual(runtime.variables, {});
  assert.match(runtime.version, /cargo-nextest 0\.9\.146(?:\s|$)/);
  assert.ok(
    manifest.bodies.some(
      (body) => body.path === runtime.nextest && body.sha256 === runtime.nextestSha256,
    ),
  );
  for (const key of ["cargo", "rustup", "toolchainCargo"]) {
    assert.ok(
      manifest.bodies.some(
        (body) => body.path === runtime[key] && body.sha256 === runtime[`${key}Sha256`],
      ),
      `complete ${key} command body missing`,
    );
  }
  const mapping = json(join(directory, "library-mapping.json"));
  assert.equal(mapping.libraryEnvironmentEqual, true);
  assert.equal(mapping.inverseWholeObjectExact, true);
  assert.ok(mapping.dependencies.length > 1, "resolved ELF dependencies missing");
  assert.equal(
    mapping.archiveListSha256,
    sha256(readFileSync(join(directory, "selected-tests.json"))),
  );
  assert.equal(
    mapping.derivedListSha256,
    sha256(readFileSync(join(directory, "derived-selected-tests.json"))),
  );
  assert.equal(
    mapping.derivedSha256,
    sha256(readFileSync(join(directory, "derived-binaries-metadata.json"))),
  );
  assert.ok(
    manifest.bodies.some(
      (body) =>
        body.path === "/home/runner/_work/vize/vize/target/nextest/binaries-metadata.json" &&
        body.sha256 === mapping.originalSha256,
    ),
  );
  for (const dependency of mapping.dependencies) {
    assert.ok(
      manifest.bodies.some(
        (body) => body.path === dependency.path && body.sha256 === dependency.sha256,
      ),
    );
  }
  const names = readdirSync(directory).filter((name) => /^strace\.\d+$/.test(name));
  assert.ok(names.length > 1, "test and native child traces required");
  const records = names.flatMap((name) =>
    parseTrace(Number(name.split(".")[1]), readFileSync(join(directory, name), "utf8")),
  );
  assert.ok(
    !/strace:\s*(?:attach|ptrace|Process.*detached|.*No space left)/i.test(
      readFileSync(join(directory, "test.stderr"), "utf8"),
    ),
    "syscall observer reported lost capture",
  );
  const ownership = processClosure(
    records,
    [runtime.nextestCommand, runtime.nextest],
    manifest.binary,
    records.filter((record) => record.exec?.[0] === manifest.provider),
    originalTest,
  );
  const sessions = nativeSessions(records, manifest.provider);
  const pipes = collectPipes(records);
  assert.ok(resolve(directory) !== resolve(process.cwd()), "isolated capture directory required");
  const observed = sessions.map((session, index) => {
    const threads = new Set([session.pid]);
    // strace follows all process/thread creation; retain the original edges.
    let added;
    do {
      added = false;
      for (const record of records) {
        const child = /^(?:clone|clone3|fork|vfork)\(.*\)\s+=\s+(\d+)$/.exec(record.call);
        if (child && threads.has(record.pid) && !threads.has(Number(child[1]))) {
          assert.ok(
            record.call.includes("CLONE_THREAD"),
            "native spawned non-thread descendant; ownership is unsupported",
          );
          threads.add(Number(child[1]));
          added = true;
        }
      }
    } while (added);
    const fds = [0, 1].map((fd) => {
      const keys = new Set(
        records
          .filter((record) => threads.has(record.pid) && record.io?.fd === fd)
          .map((record) => pipeInode(record.io.target)),
      );
      assert.equal(keys.size, 1, "one observed native stdio pipe required per fd");
      const key = [...keys][0];
      assert.ok(key && pipes.has(key), "unsupported/missing native stdio target");
      const creations = records.filter(
        (record) =>
          /^pipe2?\(/.test(record.call) &&
          /\)\s+=\s+0$/.test(record.call) &&
          pipeInodes(record.call).includes(key),
      );
      assert.equal(
        creations.length,
        1,
        "native pipe needs one observed creation; inode reuse is unsupported",
      );
      const pipe = pipes.get(key);
      const bytes = pipe.writeBytes;
      const messages = bytes.length ? framedMessages(bytes) : [];
      const retained = `session-${index}-fd${fd}.bin`;
      writeFileSync(join(directory, retained), bytes);
      return {
        fd,
        pipe: key,
        retained,
        sha256: sha256(bytes),
        bytes: bytes.length,
        consumedBytes: pipe.readBytes.length,
        unconsumedBytes: bytes.length - pipe.readBytes.length,
        consumption:
          pipe.readBytes.length === bytes.length
            ? "complete"
            : "complete-writes-with-retained-unconsumed-suffix",
        messages,
      };
    });
    const signatureRequests = fds[0].messages
      .filter((frame) => frame.message.method === "textDocument/signatureHelp")
      .map((request) => {
        const response = fds[1].messages.find(
          (frame) => frame.message.id === request.message.id && !frame.message.method,
        );
        return {
          request: request.message,
          response: response?.message ?? null,
          outcome: !response
            ? "no-response-before-closed-lifecycle"
            : Object.hasOwn(response.message, "error")
              ? "error"
              : response.message.result === null
                ? "null"
                : "result",
        };
      });
    return {
      pid: session.pid,
      argv: session.exec,
      started: session.time,
      threads: [...threads],
      fds,
      signatureRequests,
    };
  });
  const timing = json(join(directory, "instrumented-outcome.json"));
  assert.ok(Number.isInteger(timing.exitCode) && timing.exitCode >= 0);
  assert.ok(Number.isFinite(timing.elapsedSeconds) && timing.elapsedSeconds >= 0);
  const junit = readFileSync(join(directory, "junit.xml"), "utf8");
  assert.ok(junit.includes(`name="${originalTest}"`), "original test result missing");
  assert.equal(
    [...junit.matchAll(/<testcase\b/g)].length,
    1,
    "only the original test may produce a result",
  );
  assert.match(junit, /tests="1"/);
  const failed = /<failure\b/.test(junit);
  assert.equal(timing.exitCode === 0, !failed, "test exit/result mismatch");
  save(join(directory, "sessions.json"), observed);
  save(
    join(directory, "process-events.json"),
    records.filter((record) => !record.io),
  );
  save(
    join(directory, "io-events.json"),
    records
      .filter((record) => record.io)
      .map((record) => ({
        ...record,
        io: { ...record.io, bytes: record.io.bytes.toString("base64") },
      })),
  );
  return {
    schemaVersion: 1,
    source,
    originalTest,
    captureComplete: true,
    instrumentedTestPassed: !failed,
    originalAcceptanceCredit: false,
    performanceCredit: false,
    rpcConsumptionComplete: observed.every((session) =>
      session.fds.every((fd) => fd.unconsumedBytes === 0),
    ),
    retainedFiles: readdirSync(directory)
      .filter((name) => name !== "bodies" && name !== "integrity.json")
      .map((name) => ({ name, sha256: sha256(readFileSync(join(directory, name))) })),
    sessions: observed.length,
    ownership,
    timing,
  };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const directory = process.argv[2];
  try {
    save(join(directory, "integrity.json"), validateCapture(directory));
  } catch (error) {
    save(join(directory, "integrity.json"), {
      captureComplete: false,
      originalAcceptanceCredit: false,
      performanceCredit: false,
      error: String(error),
    });
    throw error;
  }
}
