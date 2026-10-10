// Mandatory hosted acceptance path. Missing current capture is an error.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  rawProcess7502,
  validate7502Build,
  validateProcess7502,
} from "./native-attribute-values-7502-build.ts";
import { validateEnvelopes7502 } from "./native-attribute-values-7502-envelopes.ts";
import {
  hash7502,
  outputUrl7502,
  requireReviewed7502,
} from "./native-attribute-values-7502-inputs.ts";
import {
  classAnchorDeletion7502,
  codec7502,
  maps7502,
} from "./native-attribute-values-7502-maps.ts";
import { validateCapture7502 } from "./native-attribute-values-7502-oracle.ts";
import { validateRuntime7502 } from "./native-attribute-values-7502-runtime-protocol.ts";
import { root7502, source7502 } from "./native-attribute-values-7502-source.ts";
import { validateHistory7502 } from "./native-attribute-values-7502-history.ts";

export function mandatory7502Paths(env: NodeJS.ProcessEnv) {
  const get = (name: string) => {
    const value = env[name];
    assert(value && path.isAbsolute(value), `${name}: actual hosted evidence path required`);
    return value;
  };
  return {
    directory: get("VIZE_NATIVE_ATTRIBUTE_VALUES_7502_EVIDENCE_DIR"),
    capture: get("VIZE_NATIVE_ATTRIBUTE_VALUES_7502_CAPTURE"),
    build: get("VIZE_NATIVE_ATTRIBUTE_VALUES_7502_BUILD_RECEIPT"),
    history: get("VIZE_NATIVE_ATTRIBUTE_VALUES_7502_HISTORY_RECEIPT"),
  };
}
export function judge7502(env = process.env) {
  const paths = mandatory7502Paths(env);
  mkdirSync(paths.directory, { recursive: true });
  assert.equal(paths.capture, path.join(paths.directory, "first.capture.json"));
  assert.equal(paths.build, path.join(paths.directory, "build-receipt.json"));
  assert.equal(paths.history, path.join(paths.directory, "history-receipt.json"));
  const frame: any = {
    schema: "vize.native-attribute-values-7502.qualification",
    version: 2,
    acceptance: "unreviewed",
    nativeAccepted: 0,
    source: null,
    captureSha256: null,
    envelopesSha256: null,
    attempts: [],
    reviewedOutputMatched: false,
    historicalQualified: false,
    classAnchorDeletionRejected: false,
    historyReceiptSha256: null,
    failure: null,
  };
  const save = () =>
    writeFileSync(
      path.join(paths.directory, "qualification.json"),
      `${JSON.stringify(frame, null, 2)}\n`,
    );
  save();
  try {
    frame.source = source7502();
    save();
    const build = JSON.parse(readFileSync(paths.build, "utf8"));
    validate7502Build(build, paths.capture);
    const bytes = readFileSync(paths.capture),
      envelopes = readFileSync(`${paths.capture}.envelopes.json`);
    frame.captureSha256 = hash7502(bytes);
    frame.envelopesSha256 = hash7502(envelopes);
    save();
    const packet = validateCapture7502(JSON.parse(bytes.toString("utf8")));
    validateEnvelopes7502(JSON.parse(envelopes.toString("utf8")));
    const codec = codec7502();
    maps7502(packet, codec);
    classAnchorDeletion7502(packet, codec);
    frame.classAnchorDeletionRejected = true;
    save();
    const runtime = fileURLToPath(
      new URL("native-attribute-values-7502-runtime.ts", import.meta.url),
    );
    for (const mode of ["development", "production"])
      for (const suffix of ["first", "repeat"]) {
        const argv = [runtime, paths.capture, mode];
        const actual = spawnSync(process.execPath, argv, {
          cwd: root7502,
          timeout: 120_000,
          maxBuffer: 64 * 1024 * 1024,
          env: { ...process.env, ...env, NODE_ENV: mode },
        });
        const raw = rawProcess7502(actual),
          stem = `runtime-${mode}-${suffix}`;
        writeFileSync(
          path.join(paths.directory, `${stem}.stdout`),
          actual.stdout ?? Buffer.alloc(0),
        );
        writeFileSync(
          path.join(paths.directory, `${stem}.stderr`),
          actual.stderr ?? Buffer.alloc(0),
        );
        writeFileSync(
          path.join(paths.directory, `${stem}.process.json`),
          `${JSON.stringify(raw, null, 2)}\n`,
        );
        frame.attempts.push({ mode, suffix, executable: process.execPath, argv, ...raw });
        save();
      }
    // All raw processes, including throws, exist before any status/output judgment.
    for (const mode of ["development", "production"]) {
      const attempts = frame.attempts.filter((entry: any) => entry.mode === mode);
      assert.equal(attempts.length, 2);
      const streams = attempts.map((entry: any) => {
        const raw = Object.fromEntries(
          [
            "exitStatus",
            "signal",
            "processError",
            "stdoutBase64",
            "stderrBase64",
            "stdoutSha256",
            "stderrSha256",
          ].map((key) => [key, entry[key]]),
        );
        validateProcess7502(raw);
        assert.equal(raw.processError, null);
        assert.equal(raw.signal, null);
        assert.equal(raw.exitStatus, 0);
        const stream = Buffer.from(raw.stdoutBase64, "base64");
        const result = validateRuntime7502(JSON.parse(stream.toString("utf8")), packet, mode);
        writeFileSync(
          path.join(paths.directory, `runtime-${mode}-${entry.suffix}.json`),
          `${JSON.stringify(result, null, 2)}\n`,
        );
        return stream;
      });
      assert(streams[0].equals(streams[1]), "fresh identical whole runtime streams");
    }
    assert.deepEqual(source7502(), frame.source, "source changed during runtime qualification");
    validate7502Build(build, paths.capture);
    const historyBytes = readFileSync(paths.history);
    validateHistory7502(JSON.parse(historyBytes.toString("utf8")), frame.source, paths.directory);
    assert(
      envelopes.equals(
        readFileSync(path.join(paths.directory, "history", "first.capture.json.envelopes.json")),
      ),
      "all 336 original whole envelope outcomes remain exact",
    );
    frame.historicalQualified = true;
    frame.historyReceiptSha256 = hash7502(historyBytes);
    save();
    requireReviewed7502(packet, JSON.parse(readFileSync(outputUrl7502, "utf8")));
    assert(
      build.attempts.every((attempt: any) => attempt.exitStatus === 0),
      "both actual Rust frozen-output tests must pass",
    );
    frame.reviewedOutputMatched = true;
    save();
  } catch (error) {
    frame.failure = {
      name: error instanceof Error ? error.name : typeof error,
      message: error instanceof Error ? error.message : String(error),
      stack: error instanceof Error ? error.stack : null,
    };
    save();
    throw error;
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  judge7502();
