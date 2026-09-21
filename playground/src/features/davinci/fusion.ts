// The S2 transform plan's walks (C-3), read off the compiler's
// `[fusion-plan-folio]` page: which passes share one traversal. The pass
// manager fuses adjacent fusable passes into one walk and times the walk,
// not its passes, so a timing view needs this page to read the profile
// honestly - nothing here infers a walk from names or timings.

export interface PlanPass {
  /** 0-based walk (fusion group) index. */
  walk: number;
  pass: string;
  /** `mandatory-diagnostic` / `mandatory-lowering` / `optional`. */
  kind: string;
  fusability: "fusable" | "barrier";
}

export interface FusionPlan {
  stage: string;
  walks: number;
  passes: PlanPass[];
}

export interface TimelineWalk {
  /** 0-based walk index. */
  index: number;
  /** Member passes, in run order; the first leads the walk. */
  passes: string[];
  /** Whether the walk is a fusable group rather than a lone barrier. */
  fusable: boolean;
  /** Measured walk time (profile key `davinci.pass.walk`), when profiled. */
  nanos: number | null;
}

const PASS_LINE = /^walk=(\d+) pass=(\S+) kind=(\S+) fusability=(fusable|barrier)$/;

/** Parse a `[fusion-plan-folio]` page, or `null` when the text is not one. */
export function parseFusionPlan(text: string): FusionPlan | null {
  const lines = text.split("\n");
  if (lines[0] !== "[fusion-plan-folio]") return null;
  const stage = /^stage=(\S+)$/.exec(lines[1] ?? "")?.[1];
  const walks = /^walks=(\d+)$/.exec(lines[2] ?? "")?.[1];
  if (stage === undefined || walks === undefined) return null;
  const passes: PlanPass[] = [];
  const start = lines.indexOf("[fusion-plan-folio.passes]");
  for (const line of start === -1 ? [] : lines.slice(start + 1)) {
    if (line === "") break;
    const match = PASS_LINE.exec(line);
    if (!match) return null;
    passes.push({
      walk: Number(match[1]),
      pass: match[2],
      kind: match[3],
      fusability: match[4] as PlanPass["fusability"],
    });
  }
  return { stage, walks: Number(walks), passes };
}

/**
 * The plan's walks in run order, each with its measured time: the timing
 * observer attributes a walk to its lead pass, so `walkTimings` is keyed
 * `stage/lead`.
 */
export function planWalks(
  plan: FusionPlan,
  walkTimings: ReadonlyMap<string, number>,
): TimelineWalk[] {
  const walks: TimelineWalk[] = [];
  for (const pass of plan.passes) {
    const last = walks[walks.length - 1];
    if (last && last.index === pass.walk) {
      last.passes.push(pass.pass);
      continue;
    }
    walks.push({
      index: pass.walk,
      passes: [pass.pass],
      fusable: pass.fusability === "fusable",
      nanos: walkTimings.get(`${plan.stage}/${pass.pass}`) ?? null,
    });
  }
  return walks;
}

/** How a walk ran, in words: a barrier alone, or how many passes it fused. */
export function describeWalk(walk: TimelineWalk): string {
  if (!walk.fusable) return "barrier";
  return walk.passes.length > 1 ? `${walk.passes.length} passes fused` : "fusable";
}
