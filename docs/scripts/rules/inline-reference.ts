/** Embed an existing reference without changing any fenced source bytes. */
export function inlineReference(reference: string, id: string) {
  const slug = id.replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase();
  const lines = reference.split("\n");
  if (lines[0] !== "---") throw new Error(`Missing reference frontmatter: ${id}`);
  const end = lines.indexOf("---", 1);
  if (end < 0) throw new Error(`Unclosed reference frontmatter: ${id}`);
  const output = [];
  let fence;
  for (const line of lines.slice(end + 1)) {
    const marker = line.match(/^(`{3,}|~{3,})/);
    if (marker) {
      if (!fence) fence = marker[1];
      else if (marker[1][0] === fence[0] && marker[1].length >= fence.length) fence = undefined;
      output.push(line);
      continue;
    }
    if (fence) {
      output.push(line);
      continue;
    }
    if (line.startsWith("# ")) {
      output.push(`<span id="${slug}"></span>`, "", `### ${line.slice(2)}`);
      continue;
    }
    const heading = line.match(/^## (.+)$/);
    if (heading) {
      const key = { Bad: "bad", Good: "good", 悪い: "bad", 良い: "good" }[heading[1]];
      if (key) output.push(`<span id="${slug}-${key}"></span>`, "");
      output.push(`**${heading[1]}**`);
      continue;
    }
    // References are one directory below their catalogue; code is never rewritten.
    output.push(
      line
        .replace(/\]\(\.\.\//g, "](")
        .replace(/\]\(#(?:bad|悪い)\)/g, `](#${slug}-bad)`)
        .replace(/\]\(#(?:good|良い)\)/g, `](#${slug}-good)`),
    );
  }
  if (fence) throw new Error(`Unclosed reference code fence: ${id}`);
  return `${output.join("\n").trim()}\n`;
}
