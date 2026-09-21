// The S2 provenance page (`[s2-provenance-folio]`) read back into records:
// which rule produced an op, from which authored bytes - the answer to "why
// is this here?" - plus the records of what was dropped and the facts passes
// attached. The grammar is `vize_s2::folio::provenance`'s; nothing is
// inferred beyond it.

import type { Range } from "./offsets";

export interface ProvenanceRecord {
  /** The deciding rule (`lower.element`, `drop.comment`, `pass.hoist-static.fact`). */
  rule: string;
  /** The produced op's page-order id, or null when nothing was produced. */
  node: number | null;
  /** The authored text the decision consumed. */
  before: string;
  /** The produced form; empty when nothing was produced. */
  after: string;
  /** Authored template bytes of `before`. */
  span: Range;
}

const RECORD =
  /^rule=(\S+) node=(\d+|-) before="((?:[^"\\]|\\.)*)" after="((?:[^"\\]|\\.)*)" @(\d+):(\d+)$/;

const ESCAPES: Record<string, string> = { n: "\n", r: "\r", t: "\t", '"': '"', "\\": "\\" };

function unquote(body: string): string {
  return body.replace(/\\(.)/g, (_, escaped: string) => ESCAPES[escaped] ?? escaped);
}

/** Every record on the page, in decision order; other lines are skipped. */
export function parseProvenance(text: string): ProvenanceRecord[] {
  const records: ProvenanceRecord[] = [];
  for (const line of text.split("\n")) {
    const match = RECORD.exec(line);
    if (!match) continue;
    records.push({
      rule: match[1],
      node: match[2] === "-" ? null : Number(match[2]),
      before: unquote(match[3]),
      after: unquote(match[4]),
      span: { start: Number(match[5]), end: Number(match[6]) },
    });
  }
  return records;
}

/** The records naming op `node`: its lowering first, then pass facts. */
export function recordsForNode(
  records: readonly ProvenanceRecord[],
  node: number,
): ProvenanceRecord[] {
  return records.filter((record) => record.node === node);
}

/** Whether a record was attached by a pass rather than by the lowering. */
export function isPassRecord(record: ProvenanceRecord): boolean {
  return record.rule.startsWith("pass.");
}
