import * as native from "@vizejs/native";
import type { TypeProjectContext } from "../sfc-types.ts";
import { filesystemPathIdentities, withoutVerbatimPrefix } from "./path-identity.ts";

interface ProjectWatchApi {
  typeProjectInputs(
    root: string,
    revision: string,
    knownRevision?: string,
  ): string[] | null | undefined;
  typeProjectOwners(path: string): string[];
}

// State copies used by precompilation share the same indexed cache object.
const watched = new WeakMap<object, Map<string, string>>();

export function resetTypeProjectWatches(cache: object): void {
  watched.delete(cache);
}

export const TYPE_PROJECT_CHANGED_MESSAGE =
  "[vize] TypeScript project changed while compiling; reload the component.";

/**
 * Register the watches for the type project a compile used. Returns `false`
 * without registering anything when that project revision is no longer
 * current, so the compile must not be served.
 */
export function watchTypeProject(
  cache: object,
  context: TypeProjectContext | undefined,
  addWatchFile: ((path: string) => void) | undefined,
  api: ProjectWatchApi = native,
): boolean {
  if (!context || !addWatchFile) return true;
  let projects = watched.get(cache);
  const knownRevision = projects?.get(context.root);
  const inputs = api.typeProjectInputs(context.root, context.revision, knownRevision);
  if (!inputs) return false;
  if (knownRevision === context.revision) return true;
  for (const input of new Set(inputs.map(withoutVerbatimPrefix))) addWatchFile(input);
  if (!projects) {
    projects = new Map();
    watched.set(cache, projects);
  }
  projects.set(context.root, context.revision);
  return true;
}

export function typeProjectDependencyRoots(
  cache: object,
  file: string,
  api: ProjectWatchApi = native,
): string[] {
  if (!watched.has(cache)) return [];
  const owners = new Set<string>();
  for (const identity of filesystemPathIdentities(file)) {
    for (const owner of api.typeProjectOwners(identity)) owners.add(owner);
  }
  return [...owners];
}
