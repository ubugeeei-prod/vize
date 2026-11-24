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
 * - a config resolution, which begins every build and every dev server;
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
 */

import path from "node:path";

import type { TypeProjectContext } from "../sfc-types.ts";
import {
  filesystemPathIdentities,
  probePhysicalPath,
  type PhysicalPathProbe,
} from "./path-identity.ts";

export class FilesystemEpoch {
  readonly #identities = new Map<string, readonly string[]>();
  readonly #physicalPaths = new Map<string, string | null | false>();
  readonly #typeProjects = new Map<string, string>();

  /** A missing parent is shared by every absent path below it, so probes are kept as well. */
  readonly #probe: PhysicalPathProbe = (file) => {
    let physical = this.#physicalPaths.get(file);
    if (physical === undefined) {
      physical = probePhysicalPath(file);
      this.#physicalPaths.set(file, physical);
    }
    return physical;
  };

  /** The filesystem may have changed: ask every question again. */
  advance(): void {
    this.#identities.clear();
    this.#physicalPaths.clear();
    this.#typeProjects.clear();
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
}

/** The epoch of this process, shared by every plugin instance in it. */
export const processFilesystemEpoch = new FilesystemEpoch();
