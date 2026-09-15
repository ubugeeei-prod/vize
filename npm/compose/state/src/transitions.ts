import type { AllowedTransition, StateTransitionTable } from "./types.ts";

/** Define a literal finite-state table for type-checked transitions. */
export function defineStateTransitions<const Table extends StateTransitionTable>(
  table: Table,
): Table {
  return table;
}

/** Runtime guard for finite-state transitions. */
export function canTransition<
  const Table extends StateTransitionTable,
  const From extends keyof Table & string,
>(table: Table, from: From, to: string): to is AllowedTransition<Table, From> {
  return table[from]?.includes(to) ?? false;
}

/** Assert and return a literal transition destination. */
export function transitionState<
  const Table extends StateTransitionTable,
  const From extends keyof Table & string,
  const To extends AllowedTransition<Table, From>,
>(table: Table, from: From, to: To): To {
  if (!canTransition(table, from, to)) {
    throw new Error(`Invalid state transition ${from} -> ${String(to)}`);
  }
  return to;
}
