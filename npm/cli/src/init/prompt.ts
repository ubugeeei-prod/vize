import readline from "node:readline";

import type { FeatureId, FeatureOffer, FeatureSelection } from "./select.js";

/**
 * Interactive multi-select for the five features.
 *
 * Implemented on `node:readline` rather than a prompt package: `vize` is a
 * published CLI whose install cost every user pays, and a numbered toggle list
 * needs no raw-mode handling, no terminal restore path, and no dependency in the
 * runtime path. It is a real multi-select -- numbers toggle, Enter accepts.
 */

export interface PromptIo {
  readonly input: NodeJS.ReadableStream;
  readonly output: NodeJS.WritableStream;
}

export interface PromptDeps extends PromptIo {
  /** Resolves to `null` when the input ended before an answer arrived. */
  readonly question: (query: string) => Promise<string | null>;
  /**
   * Releases the terminal. Required for readline-backed deps: an open interface
   * keeps stdin referenced and the process never exits.
   */
  readonly close?: () => void;
}

/** True when stdin cannot answer a prompt, so `init` must not ask one. */
export function isNonInteractive(stream: NodeJS.ReadableStream): boolean {
  return (stream as NodeJS.ReadStream).isTTY !== true;
}

/**
 * Wraps `node:readline` so a closed input resolves instead of hanging.
 *
 * `rl.question` never invokes its callback when stdin reaches EOF first. Left
 * alone that leaves `init`'s promise permanently pending, and the process exits
 * `0` having written nothing -- a silent no-op that looks like success. Resolving
 * to `null` on close turns that into an explicit cancellation.
 */
export function createPromptDeps(io: PromptIo): PromptDeps {
  const rl = readline.createInterface({ input: io.input, output: io.output });
  let closed = false;
  rl.on("close", () => {
    closed = true;
  });
  return {
    ...io,
    question: (query) =>
      new Promise<string | null>((resolve) => {
        if (closed) {
          resolve(null);
          return;
        }
        let settled = false;
        const onClose = (): void => {
          if (!settled) {
            settled = true;
            resolve(null);
          }
        };
        rl.once("close", onClose);
        rl.question(query, (answer) => {
          if (settled) {
            return;
          }
          settled = true;
          rl.removeListener("close", onClose);
          resolve(answer);
        });
      }),
    close: () => {
      rl.close();
    },
  };
}

/** Runs the checklist. Returns `null` when the input ended before confirmation. */
export async function selectFeatures(
  offers: readonly FeatureOffer[],
  initial: FeatureSelection,
  deps: PromptDeps,
): Promise<FeatureSelection | null> {
  const selection: Record<FeatureId, boolean> = { ...initial };
  const toggleable = offers.filter((offer) => offer.available);
  for (;;) {
    deps.output.write(renderChecklist(offers, selection));
    const raw = await deps.question("> ");
    if (raw === null) {
      return null;
    }
    const answer = raw.trim();
    if (answer === "") {
      return selection;
    }
    const indexes = parseIndexes(answer, toggleable.length);
    if (indexes === null) {
      deps.output.write(
        `Enter numbers between 1 and ${toggleable.length}, or press Enter to accept.\n`,
      );
      continue;
    }
    for (const index of indexes) {
      const offer = toggleable[index]!;
      selection[offer.id] = !selection[offer.id];
    }
  }
}

/** Yes/no confirmation. A closed input counts as "no", never as "yes". */
export async function confirm(query: string, deps: PromptDeps): Promise<boolean> {
  const raw = await deps.question(`${query} [Y/n] `);
  if (raw === null) {
    return false;
  }
  const answer = raw.trim().toLowerCase();
  return answer === "" || answer === "y" || answer === "yes";
}

function renderChecklist(
  offers: readonly FeatureOffer[],
  selection: Readonly<Record<FeatureId, boolean>>,
): string {
  const lines = [
    "",
    "Select the features to configure.",
    "Type the numbers to toggle (space or comma separated), then press Enter.",
    "",
  ];
  let position = 0;
  for (const offer of offers) {
    if (!offer.available) {
      lines.push(`     -  ${offer.label}${offer.note === "" ? "" : ` (${offer.note})`}`);
      continue;
    }
    position += 1;
    const mark = selection[offer.id] ? "x" : " ";
    const note = offer.note === "" ? "" : ` (${offer.note})`;
    lines.push(`  ${position}. [${mark}] ${offer.label}${note}`);
  }
  lines.push("");
  return `${lines.join("\n")}\n`;
}

/** Parses a toggle answer into zero-based indexes, or `null` when any entry is out of range. */
function parseIndexes(answer: string, count: number): readonly number[] | null {
  const tokens = answer.split(/[\s,]+/u).filter((token) => token !== "");
  const indexes: number[] = [];
  for (const token of tokens) {
    if (!/^\d+$/u.test(token)) {
      return null;
    }
    const value = Number.parseInt(token, 10);
    if (value < 1 || value > count) {
      return null;
    }
    indexes.push(value - 1);
  }
  return indexes.length === 0 ? null : indexes;
}
