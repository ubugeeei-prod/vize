/**
 * One window in which the filesystem facts the plugin has observed still hold.
 *
 * Every compiled SFC indexes the identities of its dependencies, and the
 * components of one project overwhelmingly depend on the same files: 1,000
 * components typed through a 200-module barrel asked for the same 200
 * `realpath`s 1,000 times over (#13557). An epoch asks about each distinct
 * path once, absent paths included, and every compiled-module cache in the
 * process shares it.
 *
 * It is not a memo of the filesystem. An epoch ends, dropping every fact so it
 * is asked again, whenever something says the filesystem may have changed:
 *
 * - a config resolution, which begins every build and every dev server, or a
 *   build that starts without one or for an environment that already built;
 * - a host notification: `watchChange`, a hot update, or any watcher event.
 *   Directory events count, since that is how a remounted workspace link under
 *   a watched directory is reported;
 * - a compile that ran against a new revision of a TypeScript project. Nuxt
 *   rewrites `.nuxt/tsconfig.*.json` inside a directory its watcher ignores
 *   (#12871), so no host reports it. The native project capture re-validates
 *   on every compile and is the authority for that case.
 *
 * Ending an epoch early only costs the questions again, so each plugin
 * instance ends the shared epoch on its own signals. A reader therefore sees
 * only facts observed after its own most recent signal. That is what lets the
 * environments of one `vite build --app` share an epoch: they are separate
 * plugin instances, but all of them are configured before the first one builds.
 *
 * The pre-compile's view of each SFC lives here as well (#13561): its `stat`
 * and its text are taken once in an epoch, so the environments of one build
 * compile one snapshot of a source instead of reading it once each.
 */

import fs from "node:fs";
import path from "node:path";

import type { TypeProjectContext } from "../sfc-types.ts";
import { normalizeCachePath } from "./cache-path.ts";
import {
  filesystemPathIdentities,
  probePhysicalPath,
  type PhysicalPathProbe,
} from "./path-identity.ts";
import type { PrecompileFileMetadata } from "./precompile.ts";

/** One `stat` of a source file, asked of the filesystem now. */
export function statSourceFile(file: string): PrecompileFileMetadata {
  const stat = fs.statSync(file);
  return { mtimeMs: stat.mtimeMs, size: stat.size };
}

export class FilesystemEpoch {
  readonly #identities = new Map<string, readonly string[]>();
  readonly #physicalPaths = new Map<string, string | null | false>();
  readonly #typeProjects = new Map<string, string>();
  readonly #sourceMetadata = new Map<string, PrecompileFileMetadata>();
  readonly #sources = new Map<string, string>();
  #generation = 0;

  /** A missing parent is shared by every absent path below it, so probes are kept as well. */
  readonly #probe: PhysicalPathProbe = (file) => {
    let physical = this.#physicalPaths.get(file);
    if (physical === undefined) {
      physical = probePhysicalPath(file);
      this.#physicalPaths.set(file, physical);
    }
    return physical;
  };

  /** Counts the epochs: what was learned in one holds only while this stays the same. */
  get generation(): number {
    return this.#generation;
  }

  /** The filesystem may have changed: ask every question again. */
  advance(): void {
    this.#generation++;
    this.#identities.clear();
    this.#physicalPaths.clear();
    this.#typeProjects.clear();
    this.#sourceMetadata.clear();
    this.#sources.clear();
  }

  /** `filesystemPathIdentities`, probing each path once in this epoch. */
  pathIdentities(file: string): readonly string[] {
    let identities = this.#identities.get(file);
    if (identities === undefined) {
      identities = filesystemPathIdentities(file, this.#probe);
      // A relative path also names the working directory, which is not a filesystem fact.
      if (path.isAbsolute(file)) this.#identities.set(file, identities);
    }
    return identities;
  }

  /**
   * The TypeScript project a compile just used. A revision other than the one
   * this epoch saw for the same root means the project's configuration changed
   * with no host event, so the facts gathered under the old revision go.
   */
  observeTypeProject(project: TypeProjectContext): void {
    const known = this.#typeProjects.get(project.root);
    if (known === project.revision) return;
    if (known !== undefined) this.advance();
    this.#typeProjects.set(project.root, project.revision);
  }

  /** `statSourceFile`, taken once in this epoch. A failure is thrown and not kept. */
  sourceMetadata(file: string): PrecompileFileMetadata {
    const key = normalizeCachePath(file);
    let metadata = this.#sourceMetadata.get(key);
    if (metadata === undefined) {
      metadata = statSourceFile(file);
      this.#sourceMetadata.set(key, metadata);
    }
    return metadata;
  }

  /** The text of a source file, read once in this epoch. A failure is thrown and not kept. */
  readSource(file: string): string {
    const key = normalizeCachePath(file);
    let source = this.#sources.get(key);
    if (source === undefined) {
      source = fs.readFileSync(file, "utf-8");
      this.#sources.set(key, source);
    }
    return source;
  }

  /** The text `readSource` holds for `file`, without reading it. */
  knownSource(file: string): string | undefined {
    return this.#sources.get(normalizeCachePath(file));
  }

  /** Keep nothing about a source that did not compile: its next reader starts from the file. */
  forgetSource(file: string): void {
    const key = normalizeCachePath(file);
    this.#sourceMetadata.delete(key);
    this.#sources.delete(key);
  }
}

/** The epoch of this process, shared by every plugin instance in it. */
export const processFilesystemEpoch = new FilesystemEpoch();
