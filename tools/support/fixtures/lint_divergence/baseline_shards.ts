import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

import { byteOrder, canonicalJson, get, object, unsigned, type Json } from "./value.ts";

export interface BaselineShardLimits {
  maxFileCount: number;
  maxOutputBytes: number;
  maxSourceBytes: number;
}

export interface BaselineFileShard {
  files: string[];
  sourceBytes: number;
}

export interface BaselineShardOutput {
  files: string[];
  output: Json;
}

export interface BaselineAccumulatorOptions {
  retainRawResults?: boolean;
  retainResults?: boolean;
}

export interface MergedBaselineOutput {
  version: string;
  runtimeVersions: Json;
  rawResults: Json[];
  results: Json[];
  droppedConfigMessageCount: number;
  rawPacketCount: number;
  rawPacketSha256: string;
  retainedPacketCount: number;
  retainedPacketSha256: string;
}

export const DEFAULT_BASELINE_SHARD_LIMITS: Readonly<BaselineShardLimits> = Object.freeze({
  maxFileCount: 32,
  maxOutputBytes: 256 * 1024 * 1024,
  maxSourceBytes: 8 * 1024 * 1024,
});

function positiveSafeInteger(value: number, label: string): void {
  if (!Number.isSafeInteger(value) || value <= 0)
    throw new Error(`${label} must be a positive safe integer`);
}

function assertOrderedFiles(files: readonly string[]): void {
  for (let index = 1; index < files.length; index++) {
    if (byteOrder(files[index - 1], files[index]) >= 0)
      throw new Error(
        `baseline input files must be unique and byte-ordered: ${files[index - 1]} before ${files[index]}`,
      );
  }
}

export function partitionBaselineFiles(
  cwd: string,
  files: readonly string[],
  limits: BaselineShardLimits = DEFAULT_BASELINE_SHARD_LIMITS,
): BaselineFileShard[] {
  positiveSafeInteger(limits.maxFileCount, "baseline maxFileCount");
  positiveSafeInteger(limits.maxOutputBytes, "baseline maxOutputBytes");
  positiveSafeInteger(limits.maxSourceBytes, "baseline maxSourceBytes");
  assertOrderedFiles(files);
  if (!files.length) return [{ files: [], sourceBytes: 0 }];

  const shards: BaselineFileShard[] = [];
  let current: BaselineFileShard = { files: [], sourceBytes: 0 };
  for (const file of files) {
    let stat: fs.Stats;
    try {
      stat = fs.statSync(path.resolve(cwd, file));
    } catch (error) {
      throw new Error(`cannot stat baseline input ${file}: ${String(error)}`);
    }
    if (!stat.isFile()) throw new Error(`baseline input is not a file: ${file}`);
    if (stat.size > limits.maxSourceBytes)
      throw new Error(
        `baseline input ${file} is ${stat.size} bytes, above the ${limits.maxSourceBytes}-byte shard envelope`,
      );
    if (
      current.files.length > 0 &&
      (current.files.length >= limits.maxFileCount ||
        current.sourceBytes + stat.size > limits.maxSourceBytes)
    ) {
      shards.push(current);
      current = { files: [], sourceBytes: 0 };
    }
    current.files.push(file);
    current.sourceBytes += stat.size;
  }
  if (current.files.length) shards.push(current);
  return shards;
}

function arrayField(value: Json, key: string, label: string): Json[] {
  const field = get(value, key);
  if (!Array.isArray(field)) throw new Error(`${label}.${key} must be an array`);
  return field;
}

function fileSequence(results: Json[], label: string): string[] {
  return results.map((result, index) => {
    const filePath = get(result, "filePath");
    if (typeof filePath !== "string")
      throw new Error(`${label}[${index}].filePath must be a string`);
    return path.resolve(filePath);
  });
}

function assertSequence(actual: string[], expected: string[], label: string): void {
  if (actual.length !== expected.length)
    throw new Error(`${label} reported ${actual.length} files, expected ${expected.length}`);
  for (let index = 0; index < expected.length; index++) {
    if (actual[index] !== expected[index])
      throw new Error(
        `${label} file ${index} was ${JSON.stringify(actual[index])}, expected ${JSON.stringify(expected[index])}`,
      );
  }
}

function droppedCount(value: Json, label: string): number {
  const count = unsigned(get(value, "droppedConfigMessageCount"));
  if (count === undefined || typeof count === "bigint" || !Number.isSafeInteger(count))
    throw new Error(`${label}.droppedConfigMessageCount must be a safe unsigned integer`);
  return count;
}

class PacketSequenceHasher {
  readonly #hash = createHash("sha256");
  #count = 0;

  add(packet: Json): void {
    const body = canonicalJson(packet);
    this.#hash.update(`${Buffer.byteLength(body)}:`);
    this.#hash.update(body);
    this.#count++;
  }

  finish(): { count: number; sha256: string } {
    return { count: this.#count, sha256: this.#hash.digest("hex") };
  }
}

export function baselineAuthoritySha256(
  cwd: string,
  files: readonly string[],
  rules: Json,
): string {
  assertOrderedFiles(files);
  const hash = createHash("sha256");
  hash.update(canonicalJson({ files: files.length, rules }));
  for (const file of files) {
    const filename = path.resolve(cwd, file);
    let source: Buffer;
    try {
      source = fs.readFileSync(filename);
    } catch (error) {
      throw new Error(`cannot read baseline authority ${file}: ${String(error)}`);
    }
    hash.update(`${Buffer.byteLength(file)}:`);
    hash.update(file);
    hash.update(`${source.byteLength}:`);
    hash.update(source);
  }
  return hash.digest("hex");
}

export class BaselineShardAccumulator {
  readonly #cwd: string;
  readonly #expected: string[];
  readonly #rawResults: Json[] = [];
  readonly #rawPackets = new PacketSequenceHasher();
  readonly #results: Json[] = [];
  readonly #retainedPackets = new PacketSequenceHasher();
  readonly #retainRawResults: boolean;
  readonly #retainResults: boolean;
  #coveredCount = 0;
  #droppedConfigMessageCount = 0;
  #runtimeIdentity = "";
  #runtimeVersions: Json = null;
  #shardCount = 0;
  #finished = false;
  #version = "";

  constructor(cwd: string, files: readonly string[], options: BaselineAccumulatorOptions = {}) {
    assertOrderedFiles(files);
    this.#cwd = cwd;
    this.#expected = files.map((file) => path.resolve(cwd, file));
    this.#retainRawResults = options.retainRawResults ?? true;
    this.#retainResults = options.retainResults ?? true;
  }

  add(shard: BaselineShardOutput): void {
    if (this.#finished) throw new Error("baseline aggregate is already complete");
    const index = this.#shardCount,
      label = `baseline shard ${index}`,
      output = object(shard.output);
    if (!output) throw new Error(`${label} output must be an object`);
    const shardVersion = get(output, "version"),
      shardRuntimeVersions = get(output, "runtimeVersions"),
      shardRaw = arrayField(output, "rawResults", label),
      shardResults = arrayField(output, "results", label),
      expected = shard.files.map((file) => path.resolve(this.#cwd, file)),
      expectedOffset = this.#expected.slice(
        this.#coveredCount,
        this.#coveredCount + expected.length,
      );
    if (typeof shardVersion !== "string" || !shardVersion)
      throw new Error(`${label}.version must be a non-empty string`);
    if (!object(shardRuntimeVersions))
      throw new Error(`${label}.runtimeVersions must be an object`);
    assertSequence(expected, expectedOffset, `${label} input coverage`);
    assertSequence(fileSequence(shardRaw, `${label}.rawResults`), expected, `${label}.rawResults`);
    assertSequence(fileSequence(shardResults, `${label}.results`), expected, `${label}.results`);

    const identity = canonicalJson({
      version: shardVersion,
      runtimeVersions: shardRuntimeVersions,
    });
    if (index === 0) {
      this.#version = shardVersion;
      this.#runtimeVersions = shardRuntimeVersions;
      this.#runtimeIdentity = identity;
    } else if (identity !== this.#runtimeIdentity)
      throw new Error(`${label} runtime identity differs from baseline shard 0`);

    for (const result of shardRaw) {
      this.#rawPackets.add(result);
      if (this.#retainRawResults) this.#rawResults.push(result);
    }
    for (const result of shardResults) {
      this.#retainedPackets.add(result);
      if (this.#retainResults) this.#results.push(result);
    }
    this.#coveredCount += expected.length;
    this.#droppedConfigMessageCount += droppedCount(output, label);
    if (!Number.isSafeInteger(this.#droppedConfigMessageCount))
      throw new Error("baseline droppedConfigMessageCount overflowed a safe integer");
    this.#shardCount++;
  }

  finish(): MergedBaselineOutput {
    if (this.#finished) throw new Error("baseline aggregate is already complete");
    if (!this.#shardCount) throw new Error("baseline produced no shards");
    if (this.#coveredCount !== this.#expected.length)
      throw new Error(
        `baseline shard coverage reported ${this.#coveredCount} files, expected ${this.#expected.length}`,
      );
    if (this.#retainRawResults)
      assertSequence(
        fileSequence(this.#rawResults, "baseline rawResults"),
        this.#expected,
        "baseline rawResults",
      );
    if (this.#retainResults)
      assertSequence(
        fileSequence(this.#results, "baseline results"),
        this.#expected,
        "baseline results",
      );
    const rawIdentity = this.#rawPackets.finish(),
      retainedIdentity = this.#retainedPackets.finish();
    this.#finished = true;
    return {
      version: this.#version,
      runtimeVersions: this.#runtimeVersions,
      rawResults: this.#rawResults,
      results: this.#results,
      droppedConfigMessageCount: this.#droppedConfigMessageCount,
      rawPacketCount: rawIdentity.count,
      rawPacketSha256: rawIdentity.sha256,
      retainedPacketCount: retainedIdentity.count,
      retainedPacketSha256: retainedIdentity.sha256,
    };
  }
}

export function mergeBaselineShardOutputs(
  cwd: string,
  files: readonly string[],
  shards: readonly BaselineShardOutput[],
): MergedBaselineOutput {
  const accumulator = new BaselineShardAccumulator(cwd, files);
  for (const shard of shards) accumulator.add(shard);
  return accumulator.finish();
}
