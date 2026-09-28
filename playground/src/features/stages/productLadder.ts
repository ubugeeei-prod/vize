import type { SfcCompileResult } from "../../wasm/index";
import type { ProductCaptureFeed } from "../../wasm/types/productCapture";
import type { StageFeed } from "../../wasm/types/stages";
import { buildLadder, type StageLadder } from "./ladder";

/** Place only observed product pages on the rail. L4 is the module output. */
export function buildProductLadder(
  feed: ProductCaptureFeed,
  result: SfcCompileResult,
): StageLadder {
  const stageFeed: StageFeed = {
    schema_version: 1,
    command: feed.command,
    pages: feed.pages
      .filter(({ level }) => level !== "l4")
      .map(({ level, step, text }) => ({ path: null, stage: level, pass: step, text })),
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
  ladder.rungs = ladder.rungs.filter(({ pages }) => pages.length > 0);
  // The source editor speaks in the authored SFC's byte frame, never in the
  // rendered L1 page's text frame.
  ladder.template = result.descriptor.template?.content ?? "";
  return ladder;
}
