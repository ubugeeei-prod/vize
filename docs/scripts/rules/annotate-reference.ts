import { changedCodeLines } from "./inline-code-diff.ts";
import { inlineReference } from "./inline-reference.ts";

interface Block {
  line: number;
  key: string;
  source: string;
  section: "bad" | "good";
}

/** Match project files by path; shared files and every source byte remain unchanged. */
export function annotateReference(reference: string, project = false) {
  const lines = reference.split("\n");
  const blocks: Block[] = [];
  const anonymous = { bad: 0, good: 0 };
  let section: Block["section"] | undefined;
  let filename: string | undefined;
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    if (line === "## Bad" || line === "## 悪い") section = "bad";
    else if (line === "## Good" || line === "## 良い") section = "good";
    else if (line.startsWith("## ")) section = undefined;
    const path = line.match(/^`([^`]+)`$/)?.[1];
    if (path) filename = path;
    if (!/^```[\w+-]+$/.test(line)) continue;
    const opening = index;
    const source: string[] = [];
    while (++index < lines.length && lines[index] !== "```") source.push(lines[index]);
    if (index === lines.length) throw new Error("Unclosed reference code fence");
    if (section) {
      const key = project && filename ? filename : `anonymous:${anonymous[section]++}`;
      blocks.push({ line: opening, key, source: source.join("\n"), section });
    }
    filename = undefined;
  }
  const before = new Map(
    blocks.filter((block) => block.section === "bad").map((block) => [block.key, block]),
  );
  const after = new Map(
    blocks.filter((block) => block.section === "good").map((block) => [block.key, block]),
  );
  if (
    before.size !== blocks.filter((block) => block.section === "bad").length ||
    after.size !== blocks.filter((block) => block.section === "good").length
  )
    throw new Error("Ambiguous Bad/Good project file identity");
  for (const block of blocks) {
    const left = before.get(block.key);
    const right = after.get(block.key);
    const changes = left && right ? changedCodeLines(left.source, right.source) : undefined;
    const changed = changes
      ? block.section === "bad"
        ? changes.removed
        : changes.added
      : block.source.split("\n").map((_, index) => index + 1);
    if (changed.length)
      lines[block.line] +=
        ` annotate="${block.section === "bad" ? "remove" : "add"}:${changed.join(",")}"`;
  }
  return lines.join("\n");
}

/** The native heading owns its existing fragment; Bad/Good retain explicit fragments. */
export function nativeInlineReference(reference: string, id: string, project = false) {
  return inlineReference(annotateReference(reference, project), id).replace(
    /^<span id="[^"\n]+"><\/span>\n\n/,
    "",
  );
}
