import { createHash } from "node:crypto";

export function artScopeAttribute(filePath: string): string {
  const hash = createHash("sha256").update(filePath).digest("hex").slice(0, 8);
  return `data-v-${hash}`;
}

const TAG_NAME_START = /[A-Za-z]/;
const TAG_NAME_CHAR = /[\w:.-]/;
const SPACE = /\s/;

/**
 * Add `scopeAttr` to every start tag of a variant template.
 *
 * Tokenizes like Vue's template parser: quoted attribute values, comments and
 * `{{ }}` interpolations are opaque, so `@x="() => {}"` or `{{ a<b }}` is not
 * mistaken for tag syntax (#9752).
 */
export function stampScopeAttribute(template: string, scopeAttr: string): string {
  let out = "";
  let i = 0;
  while (i < template.length) {
    if (template.startsWith("{{", i)) {
      const end = template.indexOf("}}", i + 2);
      const stop = end === -1 ? template.length : end + 2;
      out += template.slice(i, stop);
      i = stop;
    } else if (template.startsWith("<!--", i)) {
      const end = template.indexOf("-->", i + 4);
      const stop = end === -1 ? template.length : end + 3;
      out += template.slice(i, stop);
      i = stop;
    } else if (template[i] === "<" && TAG_NAME_START.test(template[i + 1] ?? "")) {
      const tag = scanStartTag(template, i);
      out += tag.stamped(scopeAttr);
      i = tag.end;
    } else {
      out += template[i];
      i += 1;
    }
  }
  return out;
}

function scanStartTag(source: string, start: number) {
  let i = start + 1;
  while (i < source.length && TAG_NAME_CHAR.test(source[i]!)) i += 1;
  const nameEnd = i;
  const names: string[] = [];
  let attrsEnd = source.length;
  let selfClosing = false;
  let closed = false;

  while (i < source.length) {
    const ch = source[i]!;
    if (SPACE.test(ch)) {
      i += 1;
    } else if (ch === ">") {
      attrsEnd = i;
      i += 1;
      closed = true;
      break;
    } else if (ch === "/" && source[i + 1] === ">") {
      attrsEnd = i;
      i += 2;
      selfClosing = true;
      closed = true;
      break;
    } else if (ch === "/") {
      i += 1;
    } else {
      const nameStart = i;
      while (i < source.length && !/[\s=/>]/.test(source[i]!)) i += 1;
      // A name may start with `=` per the HTML tokenizer; never loop on it.
      if (i === nameStart) i += 1;
      names.push(source.slice(nameStart, i));
      let j = i;
      while (j < source.length && SPACE.test(source[j]!)) j += 1;
      if (source[j] !== "=") continue;
      j += 1;
      while (j < source.length && SPACE.test(source[j]!)) j += 1;
      const quote = source[j];
      if (quote === '"' || quote === "'") {
        const close = source.indexOf(quote, j + 1);
        i = close === -1 ? source.length : close + 1;
      } else {
        while (j < source.length && !/[\s>]/.test(source[j]!)) j += 1;
        i = j;
      }
    }
  }

  return {
    end: i,
    stamped(scopeAttr: string): string {
      const raw = source.slice(start, i);
      if (!closed || names.includes(scopeAttr)) return raw;
      const attrs = source.slice(nameEnd, attrsEnd).trimEnd();
      return `${source.slice(start, nameEnd)}${attrs} ${scopeAttr}${selfClosing ? " /" : ""}>`;
    },
  };
}
