// The S3 partition facts laid over the S3 graph page: each op line of
// `[s3-folio.ops]` gets the static/dynamic kind the `[s3-partition-folio]`
// page exported for that op id. Both pages come from the same lowering; this
// only joins them by op id.

export type PartitionKind = "static" | "dynamic";

/** `op id -> kind` from the partition page. */
export function partitionKinds(partitionPage: string): Map<number, PartitionKind> {
  const kinds = new Map<number, PartitionKind>();
  for (const match of partitionPage.matchAll(/^op=(\d+) kind=(static|dynamic) /gm)) {
    kinds.set(Number(match[1]), match[2] as PartitionKind);
  }
  return kinds;
}

/** `line index -> kind` for the graph page's op lines. */
export function graphLineKinds(
  graphPage: string,
  kinds: ReadonlyMap<number, PartitionKind>,
): Map<number, PartitionKind> {
  const marks = new Map<number, PartitionKind>();
  let inOps = false;
  graphPage.split("\n").forEach((line, index) => {
    if (line.startsWith("[")) {
      inOps = line === "[s3-folio.ops]";
      return;
    }
    const id = inOps ? /^id=(\d+) /.exec(line) : null;
    const kind = id ? kinds.get(Number(id[1])) : undefined;
    if (kind) marks.set(index, kind);
  });
  return marks;
}
