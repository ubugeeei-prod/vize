import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { streamedDigest as digest } from "./css-native-debug-build.mjs";
import { samplingPlan } from "./css-native-cpu-profile.mjs";

export const savedAuthority = {
  artifact: 12617026395,
  run: 40239664358,
  source: "a2c227b2362ade7f2597ecbd5da7c04c5a07a616",
  bytes: 10456258,
  sha256: "1e2790869606f1385c81884264bf2af59c161fde66f28aa400e1fa4a7527561a",
};
export function validateSavedAuthority(meta) {
  assert.equal(meta.id, savedAuthority.artifact);
  assert.equal(meta.workflow_run.id, savedAuthority.run);
  assert.equal(meta.workflow_run.head_sha, savedAuthority.source);
  assert.equal(meta.size_in_bytes, savedAuthority.bytes);
  assert.equal(meta.digest, `sha256:${savedAuthority.sha256}`);
  assert.equal(meta.expired, false);
}
const script = fileURLToPath(import.meta.url),
  read = (file) => JSON.parse(fs.readFileSync(file, "utf8")),
  write = (file, data) => fs.writeFileSync(file, JSON.stringify(data, null, 2) + "\n");
async function run(configFile, destination) {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  destination = path.resolve(destination);
  assert.equal(fs.existsSync(destination), false);
  fs.mkdirSync(destination, { recursive: true });
  const record = {
    schema: "vize.css.saved-cpu-analysis-preparation",
    authority: savedAuthority,
    status: "attempt",
    probes: [],
    note: "Existing24 raw CPU captures only. Symbol-entry probes qualify decoder compatibility; they have no CPU-cost or source-input match credit. No new product build or native sampling.",
  };
  const retain = () => write(path.join(destination, "receipt.json"), record);
  retain();
  try {
    const headers = {
      Authorization: `Bearer ${process.env.GH_TOKEN}`,
      Accept: "application/vnd.github+json",
      "X-GitHub-Api-Version": "2022-11-28",
    };
    const endpoint = `https://api.github.com/repos/ubugeeei-prod/vize/actions/artifacts/${savedAuthority.artifact}`;
    const metadata = await fetch(endpoint, { headers, signal: AbortSignal.timeout(30000) });
    assert.equal(metadata.status, 200);
    const meta = await metadata.json();
    validateSavedAuthority(meta);
    write(path.join(destination, "capture-artifact-metadata.json"), meta);
    const redirect = await fetch(endpoint + "/zip", {
      headers,
      redirect: "manual",
      signal: AbortSignal.timeout(30000),
    });
    assert.equal(redirect.status, 302);
    const location = redirect.headers.get("location");
    assert.ok(location?.startsWith("https://"));
    const response = await fetch(location, { signal: AbortSignal.timeout(120000) });
    assert.equal(response.status, 200);
    const bytes = Buffer.from(await response.arrayBuffer());
    assert.equal(bytes.length, savedAuthority.bytes);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), savedAuthority.sha256);
    const temporary = fs.mkdtempSync(path.join(process.env.RUNNER_TEMP, "css-saved-samples-")),
      zip = path.join(temporary, "capture.zip"),
      origin = path.join(temporary, "original");
    fs.writeFileSync(zip, bytes);
    execFileSync("unzip", ["-q", zip, "-d", origin]);
    const savedRoot = path.join(origin, "debug-samples"),
      saved = read(path.join(savedRoot, "receipt.json"));
    assert.equal(saved.captureComplete, true);
    assert.equal(saved.harnessSource, savedAuthority.source);
    assert.deepEqual(saved.plan, samplingPlan());
    assert.equal(saved.samples.length, 24);
    record.savedRoot = savedRoot;
    record.captureReceiptSha256 = digest(path.join(savedRoot, "receipt.json"));
    record.actualSamples = saved.samples.reduce((n, row) => n + row.sampleCount, 0);
    retain();
    const config = read(configFile),
      llvm = process.env.VIZE_CSS_LLVM_ADDR2LINE ?? process.env.VIZE_CSS_ADDR2LINE;
    assert.ok(llvm && path.isAbsolute(llvm));
    record.decoder = {
      file: llvm,
      sha256: digest(llvm),
      version: execFileSync(llvm, ["--version"], { encoding: "utf8" }),
    };
    for (const side of ["base", "head"]) {
      const image = config.images[side].transportBinary;
      const symbols = execFileSync("nm", ["--defined-only", "--format=posix", image], {
        encoding: "utf8",
        maxBuffer: 128 * 1024 * 1024,
      });
      const matches = symbols
        .split("\n")
        .map((line) => line.split(/\s+/u))
        .filter(
          (fields) =>
            fields[0].endsWith("css_contains_crashing_math_function") && /^[tT]$/u.test(fields[1]),
        );
      assert.equal(matches.length, 1);
      const [symbol, , address] = matches[0];
      assert.match(address, /^[0-9a-f]+$/u);
      for (const [name, tool] of [
        ["gnu", "/usr/bin/addr2line"],
        ["llvm", llvm],
      ]) {
        const output = path.join(destination, `${side}-${name}.stdout`),
          error = path.join(destination, `${side}-${name}.stderr`),
          fds = [fs.openSync(output, "w"), fs.openSync(error, "w")];
        let result;
        const startedAt = new Date().toISOString();
        try {
          result = spawnSync(
            "timeout",
            [
              "--signal=TERM",
              "--kill-after=5s",
              "30s",
              "/usr/bin/time",
              "-v",
              tool,
              "-a",
              "-f",
              "-i",
              "-e",
              image,
              `0x${address}`,
            ],
            { stdio: ["ignore", ...fds] },
          );
        } finally {
          for (const fd of fds) fs.closeSync(fd);
        }
        record.probes.push({
          side,
          name,
          symbol,
          elfAddress: `0x${address}`,
          imageSha256: config.images[side].producer.binarySha256,
          startedAt,
          endedAt: new Date().toISOString(),
          status: result.status,
          signal: result.signal,
          stdoutSha256: digest(output),
          stderrSha256: digest(error),
        });
        retain();
        if (name === "llvm") {
          assert.equal(result.status, 0);
          assert.equal(result.signal, null);
          assert.match(fs.readFileSync(output, "utf8"), /value_guard\.rs:[1-9][0-9]*/u);
        }
      }
    }
    fs.writeFileSync(path.join(destination, "saved-root.txt"), savedRoot + "\n");
    record.status = "complete";
    retain();
  } catch (error) {
    record.status = "failed";
    record.failure = { name: error.name, message: error.message };
    retain();
    throw error;
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === script)
  await run(...process.argv.slice(2));
