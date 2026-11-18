// Resolve the documented-extensions x imported-prop-contracts records.ts conflict.
import fs from "node:fs";
const file = process.argv[2];
const lines = fs.readFileSync(file, "utf8").split("\n");
const out = [];
let hunk = 0;
for (let i = 0; i < lines.length; i++) {
  if (!lines[i].startsWith("<<<<<<< ")) { out.push(lines[i]); continue; }
  const ours = [], theirs = [];
  let j = i + 1;
  for (; !lines[j].startsWith("======="); j++) ours.push(lines[j]);
  for (j++; !lines[j].startsWith(">>>>>>> "); j++) theirs.push(lines[j]);
  i = j;
  hunk++;
  if (hunk === 1) out.push(...ours, "import {", ...theirs);
  else if (hunk === 2) {
    // ours opens `if (` for custom-event... and continues into duplicate-class; theirs is the slots condition.
    out.push(...ours);
    globalThis.slotCondition = theirs;
  } else if (hunk === 3) {
    out.push(...ours);
    out.push("  if (", ...globalThis.slotCondition, "    source !== undefined &&", "    source !== null", "  ) {", "    const offset = findingOffset(finding, source);", ...theirs);
  } else if (hunk === 4) out.push(...ours, "}", "", ...theirs);
  else throw new Error(`unexpected hunk ${hunk}`);
}
fs.writeFileSync(file, out.join("\n"));
console.log(`resolved ${hunk} hunks`);
