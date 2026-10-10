/** Compute documentation-only line metadata without modifying copyable source. */
export function changedCodeLines(before: string, after: string) {
  const left = before.split("\n");
  const right = after.split("\n");
  const width = right.length + 1;
  const lengths = new Uint32Array((left.length + 1) * width);
  for (let i = left.length - 1; i >= 0; i -= 1) {
    for (let j = right.length - 1; j >= 0; j -= 1) {
      lengths[i * width + j] =
        left[i] === right[j]
          ? 1 + lengths[(i + 1) * width + j + 1]
          : Math.max(lengths[(i + 1) * width + j], lengths[i * width + j + 1]);
    }
  }
  const removed: number[] = [];
  const added: number[] = [];
  let i = 0;
  let j = 0;
  while (i < left.length && j < right.length) {
    if (left[i] === right[j]) {
      i += 1;
      j += 1;
    } else if (lengths[(i + 1) * width + j] >= lengths[i * width + j + 1]) {
      removed.push(++i);
    } else {
      added.push(++j);
    }
  }
  while (i < left.length) removed.push(++i);
  while (j < right.length) added.push(++j);
  return { removed, added };
}

export function annotateExampleChanges(reference: string, bad: string, good: string) {
  const changes = changedCodeLines(bad, good);
  let section: "bad" | "good" | undefined;
  let inFence = false;
  return reference
    .split("\n")
    .map((line) => {
      if (!inFence && (line === "## Bad" || line === "## 悪い")) section = "bad";
      if (!inFence && (line === "## Good" || line === "## 良い")) section = "good";
      if (!line.startsWith("```")) return line;
      const opening = !inFence;
      inFence = !inFence;
      if (!opening || !section || !/^```(?:vue|ts|html)$/.test(line)) return line;
      const changed = section === "bad" ? changes.removed : changes.added;
      return changed.length
        ? `${line} annotate="${section === "bad" ? "remove" : "add"}:${changed.join(",")}"`
        : line;
    })
    .join("\n");
}
