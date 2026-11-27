import { dumpTokens, type DumpLine, type PageKind, type Token } from "./dumpLines";

/**
 * Private diagnostic presentation, deliberately not a stable language or API.
 * Original row indices and spans remain the mapping authority. Unknown records
 * stay visible; consumers must not parse or depend on this display notation.
 */
export function irNotation(kind: PageKind, lines: DumpLine[]): DumpLine[] {
  if (kind === "surface") return lines;
  let synthetic = -1;
  const result: DumpLine[] = [];
  let section = "";
  let moduleOpen = false;
  const add = (text: string, original?: DumpLine) =>
    result.push({
      index: original?.index ?? synthetic--,
      text,
      tokens: irTokens(text),
      span: original?.span ?? null,
      depth: original?.depth ?? 0,
      node: original?.node ?? null,
    });
  for (const line of lines) {
    if (!line.text.trim()) continue;
    const header = /^\[([\w.-]+)\]$/.exec(line.text);
    if (header) {
      if (!moduleOpen) {
        add(`module @${kind} {`, line);
        moduleOpen = true;
      } else {
        if (section) add("  }");
        section = header[1].split(".").at(-1)!;
        add(`  ${section} {`, line);
      }
      continue;
    }
    const indent = section ? "    " : "  ";
    const raw = line.text;
    if (kind === "disegno" && line.node !== null) {
      const body = fields(raw.trim()).replace(/@(\d+):(\d+)$/, "loc($1..$2)");
      add(`${indent}${"  ".repeat(line.depth)}%${line.node} = ${body}`, line);
    } else if (kind === "impeto") {
      const op = /^id=(\d+) kind=(\S+) region=(\d+) effect=(\d+|-) span=(\d+):(\d+)$/.exec(raw);
      const region = /^id=(\d+) parent=(\d+|-) owner=(\d+|-) span=(\d+):(\d+)$/.exec(raw);
      const effect = /^id=(\d+) owner=(\d+) region=(\d+) span=(\d+):(\d+)$/.exec(raw);
      const edge = /^from=(\d+) to=(\d+) kind=(\S+) effect=(\d+|-)$/.exec(raw);
      if (section === "ops" && op) {
        add(
          `${indent}%${op[1]} = ${op[2]} in ^r${op[3]}${op[4] === "-" ? "" : ` !e${op[4]}`} loc(${op[5]}..${op[6]})`,
          line,
        );
      } else if (section === "regions" && region) {
        add(
          `${indent}^r${region[1]}${region[2] === "-" ? " root" : ` within ^r${region[2]}`}${region[3] === "-" ? "" : ` owner %${region[3]}`} loc(${region[4]}..${region[5]})`,
          line,
        );
      } else if (section === "effects" && effect) {
        add(
          `${indent}!e${effect[1]} tracks %${effect[2]} in ^r${effect[3]} loc(${effect[4]}..${effect[5]})`,
          line,
        );
      } else if (section === "edges" && edge) {
        add(
          `${indent}%${edge[1]} -[${edge[3]}${edge[4] === "-" ? "" : ` !e${edge[4]}`}]-> %${edge[2]}`,
          line,
        );
      } else add(`${indent}${fields(raw)}`, line);
    } else if (kind === "partition") {
      const op = /^op=(\d+) kind=(static|dynamic) span=(\d+):(\d+)$/.exec(raw);
      add(
        op ? `${indent}%${op[1]} : ${op[2]} loc(${op[3]}..${op[4]})` : `${indent}${fields(raw)}`,
        line,
      );
    } else {
      add(`${indent}${fields(raw).replace(/@(\d+):(\d+)$/, "loc($1..$2)")}`, line);
    }
  }
  if (section) add("  }");
  if (moduleOpen) add("}");
  return result;
}

/** Do not rewrite field-like text inside quoted literals or JS expressions. */
function fields(text: string): string {
  return text.replace(/"(?:[^"\\]|\\.)*"|\b([a-z][\w-]*)=/g, (match, key: string | undefined) =>
    key ? `${key}: ` : match,
  );
}

export function irTokens(text: string): Token[] {
  const tokens: Token[] = [];
  const pattern =
    /"(?:[^"\\]|\\.)*"|loc\(\d+\.\.\d+\)|@\d+:\d+|[%^!]\w+|\b(?:module|in|within|root|owner|tracks|loc)\b|\b(?:l3|ui)\.[\w.-]+|@[\w-]+/g;
  let end = 0;
  for (const match of text.matchAll(pattern)) {
    if (match.index > end) tokens.push(...dumpTokens(text.slice(end, match.index)));
    tokens.push({
      type: match[0].startsWith('"')
        ? "string"
        : /^(loc\(|@\d)/.test(match[0])
          ? "span"
          : match[0].startsWith("^")
            ? "tag"
            : match[0].startsWith("!")
              ? "directive"
              : match[0].startsWith("%")
                ? "number"
                : "mnemonic",
      text: match[0],
    });
    end = match.index + match[0].length;
  }
  if (end < text.length) tokens.push(...dumpTokens(text.slice(end)));
  return tokens;
}
