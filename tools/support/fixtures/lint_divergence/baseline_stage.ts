import fs from "node:fs";
import path from "node:path";

import { relocatePaths } from "./isolation.ts";
import { canonicalJson, get, object, type Json, type JsonObject } from "./value.ts";

const stagePrefix = ".lint-divergence-stage-";

function writeAll(fd: number, source: string | Uint8Array): void {
  if (typeof source === "string") {
    writeAll(fd, Buffer.from(source));
    return;
  }
  let offset = 0;
  while (offset < source.byteLength)
    offset += fs.writeSync(fd, source, offset, source.byteLength - offset);
}

export function removeStagedBaselineArtifacts(outputDir: string): void {
  let entries: fs.Dirent[];
  try {
    entries = fs.readdirSync(outputDir, { withFileTypes: true });
  } catch {
    return;
  }
  for (const entry of entries) {
    if (entry.isDirectory() && entry.name.startsWith(stagePrefix))
      fs.rmSync(path.join(outputDir, entry.name), { recursive: true, force: true });
  }
}

export class BaselineArtifactStage {
  readonly #directory: string;
  readonly #rawFilename: string;
  #rawCount = 0;
  #rawFd: number | null;
  readonly #relocation: { from: string; to: string } | null;

  /** `relocation` renames a mirror's paths in staged results to the fixture's. */
  constructor(outputDir: string, relocation: { from: string; to: string } | null = null) {
    this.#relocation = relocation;
    fs.mkdirSync(outputDir, { recursive: true });
    this.#directory = fs.mkdtempSync(path.join(outputDir, stagePrefix));
    this.#rawFilename = path.join(this.#directory, "eslint-results.json-fragments");
    this.#rawFd = fs.openSync(this.#rawFilename, "wx", 0o600);
  }

  append(results: readonly Json[]): void {
    if (this.#rawFd === null) throw new Error("baseline artifact stage is already closed");
    for (const result of results) {
      const relocated =
        this.#relocation === null
          ? result
          : relocatePaths(result, this.#relocation.from, this.#relocation.to);
      const body = canonicalJson(relocated, true).replaceAll("\n", "\n      ");
      writeAll(this.#rawFd, `${this.#rawCount ? ",\n" : "\n"}      ${body}`);
      this.#rawCount++;
    }
  }

  writeArtifact(filename: string, artifact: JsonObject): void {
    if (this.#rawFd === null) throw new Error("baseline artifact stage is already closed");
    const raw = object(get(artifact, "raw"));
    if (!raw || !Array.isArray(get(raw, "eslintResults")))
      throw new Error("lint artifact raw.eslintResults must be an array placeholder");
    fs.fsyncSync(this.#rawFd);
    fs.closeSync(this.#rawFd);
    this.#rawFd = null;

    const placeholder = `${stagePrefix}${path.basename(this.#directory)}`,
      staged: Json = { ...artifact, raw: { ...raw, eslintResults: placeholder } },
      body = canonicalJson(staged, true),
      marker = JSON.stringify(placeholder),
      markerIndex = body.indexOf(marker);
    if (markerIndex < 0 || markerIndex !== body.lastIndexOf(marker))
      throw new Error("lint artifact stage placeholder must occur exactly once");

    const stagedFilename = path.join(this.#directory, "artifact.json"),
      output = fs.openSync(stagedFilename, "wx", 0o600);
    try {
      writeAll(output, body.slice(0, markerIndex));
      if (this.#rawCount === 0) writeAll(output, "[]");
      else {
        writeAll(output, "[");
        const input = fs.openSync(this.#rawFilename, "r"),
          buffer = Buffer.allocUnsafe(1024 * 1024);
        try {
          for (;;) {
            const count = fs.readSync(input, buffer, 0, buffer.byteLength, null);
            if (count === 0) break;
            writeAll(output, buffer.subarray(0, count));
          }
        } finally {
          fs.closeSync(input);
        }
        writeAll(output, "\n    ]");
      }
      writeAll(output, `${body.slice(markerIndex + marker.length)}\n`);
      fs.fsyncSync(output);
    } finally {
      fs.closeSync(output);
    }
    fs.renameSync(stagedFilename, filename);
  }

  cleanup(): void {
    if (this.#rawFd !== null) {
      fs.closeSync(this.#rawFd);
      this.#rawFd = null;
    }
    fs.rmSync(this.#directory, { recursive: true, force: true });
  }
}
