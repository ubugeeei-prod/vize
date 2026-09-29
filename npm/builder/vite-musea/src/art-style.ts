import { createHash } from "node:crypto";

export function artScopeAttribute(filePath: string): string {
  const hash = createHash("sha256").update(filePath).digest("hex").slice(0, 8);
  return `data-v-${hash}`;
}

export function stampScopeAttribute(template: string, scopeAttr: string): string {
  return template.replace(/<([A-Za-z][\w:.-]*)([^<>]*?)>/g, (match, tag: string, rest: string) => {
    if (rest.includes(scopeAttr)) return match;
    const selfClosing = /\/\s*$/.test(rest);
    const attrs = selfClosing ? rest.replace(/\/\s*$/, "").trimEnd() : rest.trimEnd();
    const gap = attrs.length > 0 && !attrs.startsWith(" ") && !attrs.startsWith("\n") ? " " : "";
    const body = `${gap}${attrs}`.trimEnd();
    return `<${tag}${body} ${scopeAttr}${selfClosing ? " /" : ""}>`;
  });
}
