import type { SfcCompileResult } from "../../wasm/index";
import type { ProductCaptureFeed } from "../../wasm/types/productCapture";
import type { StageFeed } from "../../wasm/types/stages";
import { buildLadder, type StageLadder } from "./ladder";

function displayStage(level: string, step: string): string {
  if (level === "l2" && ["plan", "provenance", "facts"].includes(step)) return `l2-${step}`;
  if (level === "l3" && ["partition", "values"].includes(step)) return `l3-${step}`;
  return level;
}

/** Place only observed product pages on the rail, keeping L4 emission separate. */
export function buildProductLadder(
  feed: ProductCaptureFeed,
  result: SfcCompileResult,
): StageLadder {
  const stageFeed: StageFeed = {
    schema_version: 1,
    command: feed.command,
    pages: feed.pages
      .filter(({ level }) => level !== "l4")
      .map(({ level, step, text }) => ({
        path: null,
        stage: displayStage(level, step),
        pass: step,
        text,
      })),
    remarks: feed.observed.remarks
      ? feed.remarks.map(({ level, pass, kind, name, span, args }) => ({
          path: null,
          stage: level,
          pass,
          kind,
          name,
          span,
          args: args.map(({ name: key, value }) => ({ key, value })),
        }))
      : [],
  };
  const timings = new Map(
    feed.observed.timings
      ? feed.timings.map(({ level, step, nanos }) => [`${level}/${step}`, nanos] as const)
      : [],
  );
  const ladder = buildLadder(stageFeed, undefined, timings);
  ladder.l4Pages = feed.pages
    .filter(({ level }) => level === "l4")
    .map(({ step, text }) => ({ step, text }));
  ladder.rungs = ladder.rungs.filter(({ pages }) => pages.length > 0);
  // The source editor speaks in the authored SFC's byte frame, never in the
  // rendered L1 page's text frame.
  ladder.template = result.descriptor.template?.content ?? "";
  return ladder;
}
