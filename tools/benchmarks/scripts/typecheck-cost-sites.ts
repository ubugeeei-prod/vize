/**
 * The projection family of one 1 ms sample in a generated SFC: the innermost
 * recorded statement-level check (`checkExpression`, `checkVariableDeclaration`,
 * `checkDeferredNode`) when one covers the sample, else the statements between
 * the recorded checks before and after it in the same container, the sample
 * split evenly over them. tsgo frames no type alias declaration, so unframed
 * samples are mostly alias resolution; those families carry "(unframed)".
 */

import {
  type SiteFamily,
  type SourceText,
  lineOf,
  lineText,
  siteFamily,
  skipTrivia,
} from "./typecheck-cost-families.ts";
import { type Frame, isPositionFrame } from "./typecheck-cost-trace.ts";

export type LoadedText = { text: SourceText; sections: Array<[number, string]> };

export const familyLabel = ({ section, family }: SiteFamily): string =>
  section === family ? section : `${section}/${family}`;

/** Function bodies that hold a whole scope rather than one construct. */
const CONTAINER = /\(function __template\(|function __setup\(/u;
/** Statements tsgo checks without a recorded frame. */
const UNFRAMED_HEAD = /^\s*(?:export\s+)?(?:declare\s+)?(?:type|interface|function)\s/u;

/** Families of one sample with their shares (summing to 1). */
export type SiteShares = Array<[family: string, share: number]>;

export function siteResolver(
  textOf: (path: string) => LoadedText | null,
  children: ReadonlyMap<Frame, Frame[]>,
): (stack: readonly Frame[], tick: number) => SiteShares {
  const regions = new Map<string, SiteShares>();

  const isContainer = (loaded: LoadedText, frame: Frame) =>
    CONTAINER.test(
      lineText(loaded.text, lineOf(loaded.text, skipTrivia(loaded.text, frame.args.pos ?? 0))),
    );

  /** Families of the unframed statements in [from, to), by statement count. */
  const region = (path: string, loaded: LoadedText, from: number, to: number): SiteShares => {
    const key = `${path}:${from}:${to}`;
    const cached = regions.get(key);
    if (cached != null) return cached;
    const votes = new Map<string, number>();
    const fallback = new Map<string, number>();
    const last = lineOf(loaded.text, Math.max(from, to - 1));
    for (let line = lineOf(loaded.text, from); line <= last; line++) {
      const content = lineText(loaded.text, line);
      if (!content.trim() || content.trim().startsWith("//")) continue;
      const family = familyLabel(
        siteFamily(loaded.text, loaded.sections, loaded.text.lineStarts[line] as number),
      );
      const target = UNFRAMED_HEAD.test(content) ? votes : fallback;
      target.set(family, (target.get(family) ?? 0) + 1);
    }
    const counted = [...(votes.size > 0 ? votes : fallback)];
    const total = counted.reduce((sum, [, count]) => sum + count, 0);
    const shares: SiteShares =
      total > 0
        ? counted.map(([family, count]) => [`${family} (unframed)`, count / total])
        : [["(unframed)", 1]];
    regions.set(key, shares);
    return shares;
  };

  return (stack, tick) => {
    const file = stack[0] as Frame;
    let container = file;
    for (let index = stack.length - 1; index > 0; index--) {
      const frame = stack[index] as Frame;
      if (!isPositionFrame(frame)) continue;
      const loaded = textOf(frame.args.path as string);
      if (!loaded) continue;
      if (!isContainer(loaded, frame)) {
        return [
          [familyLabel(siteFamily(loaded.text, loaded.sections, frame.args.pos as number)), 1],
        ];
      }
      container = frame;
      break;
    }
    const path = container.args.path as string;
    const loaded = textOf(path);
    if (!loaded) return [["(no text)", 1]];
    const time = tick * 1000;
    const siblings = children.get(container) ?? [];
    // Last sibling that ended before the sample and first that starts after it.
    let low = 0;
    let high = siblings.length;
    while (low < high) {
      const mid = (low + high) >> 1;
      if ((siblings[mid] as Frame).start < time) low = mid + 1;
      else high = mid;
    }
    const next = siblings[low];
    const previous = siblings[low - 1];
    const from =
      previous && previous.end <= time ? (previous.args.end ?? 0) : (container.args.pos ?? 0);
    const to = next
      ? (next.args.pos ?? loaded.text.bytes.length)
      : (container.args.end ?? loaded.text.bytes.length);
    return to > from ? region(path, loaded, from, to) : [["(unframed)", 1]];
  };
}
