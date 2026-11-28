import { isAbsolute } from "node:path";

interface SharedDependencies {
  inputs: readonly string[];
  identities: Set<string>;
  owners: Set<string>;
  generation: number;
  fingerprint: number;
  shared: boolean;
}

// Unique dependency lists retain the direct index. These bounds also limit the
// extra snapshots kept when a project has few opportunities to share a list.
export const MIN_SHARED_DEPENDENCIES = 16;
const MAX_SHARED_GROUPS = 256;
const MAX_SHARED_INPUTS = 65_536;
const MAX_FINGERPRINT_CANDIDATES = 8;

/** Dependency ownership, sharing identical large lists within one filesystem epoch. */
export class CompiledDependencyIndex {
  readonly #directOwners: Map<string, Set<string>>;
  readonly #sharedOwners = new Map<string, Set<SharedDependencies>>();
  readonly #memberships = new Map<string, Set<string> | SharedDependencies>();
  readonly #groups = new Map<number, Set<SharedDependencies>>();
  readonly #order = new Map<string, number>();
  readonly #identities: (file: string) => readonly string[];
  readonly #generation: (() => number) | undefined;
  #sequence = 0;
  #groupCount = 0;
  #inputCount = 0;

  constructor(
    identities: (file: string) => readonly string[],
    generation?: () => number,
    direct?: {
      owners: Map<string, Set<string>>;
      memberships: ReadonlyMap<string, Set<string>>;
    },
  ) {
    this.#identities = identities;
    this.#generation = generation;
    this.#directOwners = direct?.owners ?? new Map();
    // These sets already capture their filesystem epoch. Transfer their
    // ownership without resolving paths again when sharing first becomes useful.
    if (direct)
      for (const [owner, identities] of direct.memberships)
        this.#memberships.set(owner, identities);
  }

  add(file: string, inputs: readonly string[]): void {
    if (inputs.length === 0) return;
    // Before sharing starts, the direct sets already retain every relevant
    // order. Such owners precede all later groups, so they can keep order 0.
    if (this.#generation && (this.#groupCount > 0 || inputs.length >= MIN_SHARED_DEPENDENCIES)) {
      this.#order.set(file, ++this.#sequence);
    }
    const group = this.#sharedGroup(inputs);
    if (group) {
      if (!group.shared) this.#indexDirect(file, group.identities);
      group.owners.add(file);
      this.#memberships.set(file, group);
      return;
    }
    const dependencies = new Set<string>();
    for (const input of inputs)
      for (const identity of this.#identities(input)) dependencies.add(identity);
    this.#indexDirect(file, dependencies);
    this.#memberships.set(file, dependencies);
  }

  delete(file: string): void {
    const entry = this.#memberships.get(file);
    this.#memberships.delete(file);
    this.#order.delete(file);
    if (!entry) return;
    if (entry instanceof Set) {
      this.#unindexDirect(file, entry);
      return;
    }
    if (!entry.shared) this.#unindexDirect(file, entry.identities);
    entry.owners.delete(file);
    if (entry.owners.size !== 0) return;
    for (const identity of entry.shared ? entry.identities : []) {
      const groups = this.#sharedOwners.get(identity);
      groups?.delete(entry);
      if (groups?.size === 0) this.#sharedOwners.delete(identity);
    }
    const candidates = this.#groups.get(entry.fingerprint);
    candidates?.delete(entry);
    if (candidates?.size === 0) this.#groups.delete(entry.fingerprint);
    this.#groupCount--;
    this.#inputCount -= entry.inputs.length;
  }

  clear(): void {
    this.#directOwners.clear();
    this.#sharedOwners.clear();
    this.#memberships.clear();
    this.#groups.clear();
    this.#order.clear();
    this.#sequence = this.#groupCount = this.#inputCount = 0;
  }

  /** Preserve the direct index's registration order, including replacements. */
  ownersOf(identity: string): Iterable<string> {
    const direct = this.#directOwners.get(identity);
    const groups = this.#sharedOwners.get(identity);
    if (!groups?.size) return direct ?? [];
    if (!direct?.size && groups.size === 1) {
      for (const group of groups) return group.owners;
    }

    const shared: string[] = [];
    for (const group of groups) for (const owner of group.owners) shared.push(owner);
    const order = (file: string) => this.#order.get(file) ?? 0;
    shared.sort((left, right) => order(left) - order(right));
    if (!direct?.size) return shared;
    const result: string[] = [];
    let index = 0;
    for (const owner of direct) {
      while (index < shared.length && order(shared[index]) < order(owner)) {
        result.push(shared[index++]);
      }
      result.push(owner);
    }
    while (index < shared.length) result.push(shared[index++]);
    return result;
  }

  #indexDirect(file: string, identities: Iterable<string>): void {
    for (const identity of identities) {
      let owners = this.#directOwners.get(identity);
      if (!owners) this.#directOwners.set(identity, (owners = new Set()));
      owners.add(file);
    }
  }

  #unindexDirect(file: string, identities: Iterable<string>): void {
    for (const identity of identities) {
      const owners = this.#directOwners.get(identity);
      owners?.delete(file);
      if (owners?.size === 0) this.#directOwners.delete(identity);
    }
  }

  #promote(group: SharedDependencies): void {
    // A unique list uses only the ordinary direct index. Create the shared
    // reverse edges when a second owner actually needs the same snapshot.
    for (const owner of group.owners) this.#unindexDirect(owner, group.identities);
    for (const identity of group.identities) {
      let groups = this.#sharedOwners.get(identity);
      if (!groups) this.#sharedOwners.set(identity, (groups = new Set()));
      groups.add(group);
    }
    group.shared = true;
  }

  #sharedGroup(inputs: readonly string[]): SharedDependencies | undefined {
    // Standalone caches ask the filesystem afresh on every set. Relative paths
    // depend on cwd as well, so neither may share an observed identity list.
    if (!this.#generation || inputs.length < MIN_SHARED_DEPENDENCIES) return undefined;
    const generation = this.#generation();
    const fingerprint = dependencyFingerprint(inputs);
    const candidates = this.#groups.get(fingerprint);
    for (const group of candidates ?? []) {
      if (group.generation === generation && sameInputs(group.inputs, inputs)) {
        if (!group.shared) this.#promote(group);
        return group;
      }
    }
    if (
      this.#groupCount >= MAX_SHARED_GROUPS ||
      this.#inputCount + inputs.length > MAX_SHARED_INPUTS ||
      (candidates?.size ?? 0) >= MAX_FINGERPRINT_CANDIDATES ||
      !inputs.every(isAbsolute)
    )
      return undefined;

    const identities = new Set<string>();
    for (const input of inputs)
      for (const identity of this.#identities(input)) identities.add(identity);
    const group: SharedDependencies = {
      inputs: inputs.slice(),
      identities,
      owners: new Set(),
      generation,
      fingerprint,
      shared: false,
    };
    if (candidates) candidates.add(group);
    else this.#groups.set(fingerprint, new Set([group]));
    this.#groupCount++;
    this.#inputCount += inputs.length;
    return group;
  }
}

/** A bounded candidate lookup; equality always checks every input. */
function dependencyFingerprint(inputs: readonly string[]): number {
  let hash = inputs.length;
  for (const input of [inputs[0], inputs[inputs.length - 1]]) {
    for (let index = 0; index < input.length; index++) {
      hash = Math.imul(hash ^ input.charCodeAt(index), 16_777_619);
    }
  }
  return hash;
}

function sameInputs(left: readonly string[], right: readonly string[]): boolean {
  if (left.length !== right.length) return false;
  for (let index = 0; index < left.length; index++) if (left[index] !== right[index]) return false;
  return true;
}
