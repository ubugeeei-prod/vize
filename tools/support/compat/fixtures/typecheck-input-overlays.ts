import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  writeFileSync,
  type Stats,
} from "node:fs";
import path from "node:path";
import { typecheckProvider } from "./typecheck-baseline-provider.ts";

export interface TypecheckInputOverlay {
  readonly source: string;
  readonly target: string;
  readonly sha256: string;
}

export interface PreparedTypecheckInputOverlay extends TypecheckInputOverlay {
  readonly sizeBytes: number;
}

const prefix = "tests/_fixtures/typecheck-inputs/";
const sha256 = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");

function require(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`Invalid typecheck input overlay: ${message}`);
}

function relativeFile(value: unknown): value is string {
  return (
    typeof value === "string" &&
    /^(?:[A-Za-z0-9_.-]+\/)*[A-Za-z0-9_.-]+$/.test(value) &&
    value.split("/").every((part) => ![".", "..", ".git", "node_modules"].includes(part))
  );
}

/** A sealed input is additional generated data, never a replacement for authored source. */
export function typecheckInputOverlays(value: unknown): readonly TypecheckInputOverlay[] {
  if (value === undefined) return [];
  require(Array.isArray(value) && value.length > 0, "a nonempty array is required");
  const targets = new Set<string>();
  return value.map((row: unknown) => {
    require(row !== null && typeof row === "object" && !Array.isArray(row), "object required");
    require(JSON.stringify(Object.keys(row).sort()) ===
      '["sha256","source","target"]', "fields must be source, target and sha256");
    const entry = row as Record<string, unknown>;
    const { source, target, sha256: hash } = entry;
    require(relativeFile(source) &&
      source.startsWith(
        prefix,
      ), "source must be below the repository's pinned typecheck-inputs directory");
    require(relativeFile(target), "target must be a fixture-relative file");
    require(!targets.has(target), "duplicate target");
    require([...targets].every(
      (prior) => !prior.startsWith(`${target}/`) && !target.startsWith(`${prior}/`),
    ), "targets must not overlap as files and directories");
    require(typeof hash === "string" && /^[a-f0-9]{64}$/.test(hash), "SHA-256 required");
    targets.add(target);
    return { source, target, sha256: hash };
  });
}

/** Resolve through ordinary directories only, rejecting even dangling links. */
function regularPath(root: string, relative: string, createParents: boolean): string {
  let current = realpathSync(root);
  const parts = relative.split("/");
  for (const [index, part] of parts.entries()) {
    current = path.join(current, part);
    const parent = index < parts.length - 1;
    let info: Stats;
    try {
      info = lstatSync(current);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
      if (parent && createParents) {
        mkdirSync(current);
        info = lstatSync(current);
      } else if (parent) {
        return current + path.sep + parts.slice(index + 1).join(path.sep);
      } else {
        return current;
      }
    }
    require(!info.isSymbolicLink(), `symbolic link: ${relative}`);
    require(parent ? info.isDirectory() : info.isFile(), `unexpected file kind: ${relative}`);
  }
  return current;
}

/** Keep the compared fixture's pinned revision and every tracked input intact. */
export function prepareTypecheckInputOverlays(
  repoRoot: string,
  fixtureRoot: string,
  revision: string,
  value: unknown,
): PreparedTypecheckInputOverlay[] {
  const entries = typecheckInputOverlays(value);
  if (entries.length === 0) return [];
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: fixtureRoot, encoding: "utf8" }).trim();
  require(realpathSync(git("rev-parse", "--show-toplevel")) ===
    realpathSync(fixtureRoot), "fixture Git root differs");
  require(git("rev-parse", "HEAD") === revision, "fixture revision differs");
  // Validate the whole pack before creating any directory or file.
  const planned = entries.map((entry) => {
    const bytes = readFileSync(regularPath(repoRoot, entry.source, false));
    require(sha256(bytes) === entry.sha256, `source hash differs: ${entry.source}`);
    require(git("ls-files", "--", entry.target) === "", `tracked target: ${entry.target}`);
    const target = regularPath(fixtureRoot, entry.target, false);
    try {
      require(sha256(readFileSync(target)) ===
        entry.sha256, `existing target differs: ${entry.target}`);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    }
    return { entry, bytes };
  });
  return planned.map(({ entry, bytes }) => {
    const target = regularPath(fixtureRoot, entry.target, true);
    try {
      writeFileSync(target, bytes, { flag: "wx" });
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "EEXIST") throw error;
      require(sha256(readFileSync(target)) ===
        entry.sha256, `target changed during preparation: ${entry.target}`);
    }
    return { ...entry, sizeBytes: bytes.byteLength };
  });
}

/** Revalidate the actual input before either checker can receive qualification credit. */
export function verifyTypecheckInputOverlays(
  fixtureRoot: string,
  value: unknown,
): PreparedTypecheckInputOverlay[] {
  return typecheckInputOverlays(value).map((entry) => {
    const bytes = readFileSync(regularPath(fixtureRoot, entry.target, false));
    require(sha256(bytes) === entry.sha256, `prepared target differs: ${entry.target}`);
    return { ...entry, sizeBytes: bytes.byteLength };
  });
}

/** Preserve existing stamp identities for projects without additional generated data. */
export function typecheckPreparationIdentity(
  revision: string,
  baseline:
    | {
        readonly inputOverlays?: unknown;
        readonly prepare?: unknown;
        readonly provider?: unknown;
      }
    | undefined,
): string {
  const inputs = typecheckInputOverlays(baseline?.inputOverlays);
  const provider = typecheckProvider(baseline?.provider);
  return inputs.length === 0 && provider == null
    ? revision
    : `${revision}\n${sha256(
        Buffer.from(
          JSON.stringify({
            inputs,
            prepare: baseline?.prepare,
            provider:
              provider == null
                ? undefined
                : {
                    id: provider.id,
                    descriptorSha256: provider.descriptorSha256,
                  },
          }),
        ),
      )}`;
}
