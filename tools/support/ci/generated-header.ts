// Every Pkl output opens with a notice that names its source module, so a reader
// who lands in a generated file knows where to make the change instead.

const regenerate = "node tools/commands/generate/repository-config.ts";

export type CommentStyle = "hash" | "slash" | "batch" | "html";

function notice(source: string): [string, string] {
  return [
    `DO NOT EDIT. Generated from ${source}.`,
    `Edit the Pkl source and run \`${regenerate}\`.`,
  ];
}

function comment(style: CommentStyle, line: string): string {
  if (style === "hash") return `# ${line}`;
  if (style === "slash") return `// ${line}`;
  // `@` keeps the remark out of the console when echo is on.
  if (style === "batch") return `@REM ${line}`;
  return `<!-- ${line} -->`;
}

// An interpreter line must stay first, so the notice follows it.
function leadingLine(style: CommentStyle, text: string): string {
  const firstLine = /^[^\n]*\n/u.exec(text)?.[0] ?? "";
  return style === "hash" && firstLine.startsWith("#!") ? firstLine : "";
}

export function withGeneratedHeader(text: string, style: CommentStyle, source: string): string {
  const lead = leadingLine(style, text);
  const header = notice(source)
    .map((line) => comment(style, line) + "\n")
    .join("");
  // Markdown formatters keep a blank line between an HTML comment block and the next heading.
  const separator = style === "html" ? "\n" : "";
  return lead + header + separator + text.slice(lead.length);
}

export function withoutGeneratedHeader(text: string, style: CommentStyle): string {
  const lead = leadingLine(style, text);
  const lines = text.slice(lead.length).split("\n");
  const [first, second] = notice("").map((line) => comment(style, line));
  const marker = " Generated from ";
  const sourcePrefix = first!.slice(0, first!.indexOf(marker) + marker.length);
  if (!lines[0]?.startsWith(sourcePrefix) || lines[1] !== second) return text;
  const bodyStart = style === "html" && lines[2] === "" ? 3 : 2;
  return lead + lines.slice(bodyStart).join("\n");
}

export function commentStyle(target: string): CommentStyle {
  if (target.endsWith(".cmd")) return "batch";
  if (target.endsWith(".cjs")) return "slash";
  if (target.endsWith(".md")) return "html";
  return "hash";
}

/** Byte contracts pin the generated program, not the notice that marks it as generated. */
export function generatedBody(bytes: Buffer): Buffer {
  const text = bytes.toString("utf8");
  const body = withoutGeneratedHeader(text, "hash");
  return body === text ? bytes : Buffer.from(body, "utf8");
}
