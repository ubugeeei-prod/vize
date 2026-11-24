// Lists the comment blocks the ratchet's restating counter flags, with the next code line.
import fs from "node:fs";
import { countFile } from "../rd-l0/tools/commands/ci/readability/counters.ts";
import { lexRust } from "../rd-l0/tools/support/readability/rust-tokens.ts";

for (const file of process.argv.slice(2)) {
  const source = fs.readFileSync(file, "utf8");
  if (countFile(file, source, { lines: false })["restating-comments"] === 0) continue;
  const lines = source.split("\n");
  // Probe each comment block by deleting it and seeing whether the count drops.
  const lexed = lexRust(source);
  const blocks: { line: number; end: number }[] = [];
  for (const c of lexed.comments) {
    const end = c.line + c.lines.length - 1;
    const last = blocks.at(-1);
    if (last && last.end + 1 === c.line) last.end = end;
    else blocks.push({ line: c.line, end });
  }
  const total = countFile(file, source, { lines: false })["restating-comments"];
  for (const b of blocks) {
    const without = [...lines.slice(0, b.line - 1), ...lines.slice(b.end)].join("\n");
    if (countFile(file, without, { lines: false })["restating-comments"] < total)
      console.log(`${file}:${b.line}\n  ${lines.slice(b.line - 1, b.end).join("\n  ")}\n  >> ${lines[b.end]?.trim()}`);
  }
}
