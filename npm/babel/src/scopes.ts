import type { BabelFile, Scope } from "./types.ts";

interface Reservations {
  names: string[];
  authoredBindings: Set<string>;
}

/** Earlier pre hooks can reserve identifiers before adding their declarations. */
export function captureReservations(file: BabelFile): Reservations | undefined {
  const names = Object.keys(file.scope.uids).filter((name) => file.scope.uids[name]);
  if (names.length === 0) return undefined;
  const authoredBindings = new Set(Object.keys(file.scope.bindings));
  const seen = new Set<Scope>([file.scope]);
  file.path.traverse({
    Scopable({ scope }) {
      if (seen.has(scope)) return;
      seen.add(scope);
      for (const name of Object.keys(scope.bindings)) authoredBindings.add(name);
    },
  });
  return { names, authoredBindings };
}

export function restoreReservations(file: BabelFile, reserved: Reservations | undefined): void {
  if (!reserved) return;
  const names = new Set(reserved.names);
  for (const name of names) file.scope.uids[name] = true;
  const seen = new Set<Scope>();
  const restore = (scope: Scope) => {
    if (seen.has(scope)) return;
    seen.add(scope);
    for (const name of Object.keys(scope.bindings)) {
      if (names.has(name) && !reserved.authoredBindings.has(name)) {
        // Alpha-renaming coordinates the compiler's new bindings with Babel's
        // pending identifiers. Authored bindings retain their existing names.
        scope.rename(name, scope.generateUid(name));
      }
    }
  };
  restore(file.scope);
  file.path.traverse({ Scopable: ({ scope }) => restore(scope) });
}
