import { h, type FunctionalComponent, type VNodeChild } from "vue";
import { Marked, type Token, type Tokens } from "marked";
import hljs from "highlight.js/lib/core";
import xml from "highlight.js/lib/languages/xml";
import javascript from "highlight.js/lib/languages/javascript";
import typescript from "highlight.js/lib/languages/typescript";
import css from "highlight.js/lib/languages/css";
import bash from "highlight.js/lib/languages/bash";

hljs.registerLanguage("xml", xml);
hljs.registerLanguage("html", xml);
hljs.registerLanguage("vue", xml);
hljs.registerLanguage("javascript", javascript);
hljs.registerLanguage("js", javascript);
hljs.registerLanguage("typescript", typescript);
hljs.registerLanguage("ts", typescript);
hljs.registerLanguage("css", css);
hljs.registerLanguage("bash", bash);
hljs.registerLanguage("sh", bash);

const markedInstance = new Marked();
const URL_SCHEME_RE = /^[a-z][a-z\d+.-]*:/i;
const ALLOWED_URL_PROTOCOLS = new Set(["http:", "https:", "mailto:", "tel:"]);

function cleanUrl(value: string): string | null {
  const trimmed = value.trim();
  if (!trimmed || trimmed.startsWith("//") || trimmed.includes("\\")) return null;
  if (
    trimmed.startsWith("#") ||
    trimmed.startsWith("/") ||
    trimmed.startsWith("./") ||
    trimmed.startsWith("../")
  ) {
    return trimmed;
  }
  if (!URL_SCHEME_RE.test(trimmed)) return trimmed;

  try {
    return ALLOWED_URL_PROTOCOLS.has(new URL(trimmed).protocol) ? trimmed : null;
  } catch {
    return null;
  }
}

function renderHighlightedNode(node: Node): VNodeChild {
  if (node.nodeType === 3) return node.textContent ?? "";
  if (node instanceof Element && node.tagName === "SPAN") {
    const className = [...node.classList].filter((name) => /^hljs-[a-z_-]+$/.test(name)).join(" ");
    return h("span", { class: className }, [...node.childNodes].map(renderHighlightedNode));
  }
  return node.textContent ?? "";
}

function renderCode(code: string, language?: string): string | VNodeChild[] {
  const name = language?.split(/\s+/)[0];
  if (!name || !hljs.getLanguage(name) || typeof DOMParser === "undefined") return code;

  // Parse only highlight.js output, then rebuild Vue nodes with text and span classes.
  const highlighted = hljs.highlight(code, { language: name }).value;
  const document = new DOMParser().parseFromString("<code>" + highlighted + "</code>", "text/html");
  const codeElement = document.body.firstElementChild;
  return codeElement ? [...codeElement.childNodes].map(renderHighlightedNode) : code;
}

function renderTokens(tokens?: Token[]): VNodeChild[] {
  return tokens?.map(renderToken) ?? [];
}

function renderTableCell(cell: Tokens.TableCell, tag: "th" | "td"): VNodeChild {
  return h(
    tag,
    { style: cell.align ? { textAlign: cell.align } : undefined },
    renderTokens(cell.tokens),
  );
}

function renderToken(token: Token): VNodeChild {
  switch (token.type) {
    case "space":
    case "def":
      return null;
    case "text":
      return token.tokens?.length ? renderTokens(token.tokens) : token.text;
    case "escape":
      return token.text;
    case "html":
      // Source HTML is shown as text; documentation cannot inject arbitrary elements.
      return token.text;
    case "heading": {
      const level = Math.max(1, Math.min(6, token.depth));
      return h("h" + level, renderTokens(token.tokens));
    }
    case "paragraph":
      return h("p", renderTokens(token.tokens));
    case "blockquote":
      return h("blockquote", renderTokens(token.tokens));
    case "list":
      return h(
        token.ordered ? "ol" : "ul",
        token.ordered && token.start !== "" ? { start: token.start } : null,
        token.items.map(renderToken),
      );
    case "list_item":
      return h("li", [
        token.task
          ? h("input", { type: "checkbox", checked: token.checked, disabled: true })
          : null,
        ...renderTokens(token.tokens),
      ]);
    case "code":
      return h("pre", [
        h(
          "code",
          { class: token.lang ? "language-" + token.lang.split(/\s+/)[0] : undefined },
          renderCode(token.text, token.lang),
        ),
      ]);
    case "codespan":
      return h("code", token.text);
    case "strong":
      return h("strong", renderTokens(token.tokens));
    case "em":
      return h("em", renderTokens(token.tokens));
    case "del":
      return h("del", renderTokens(token.tokens));
    case "br":
      return h("br");
    case "hr":
      return h("hr");
    case "link": {
      const href = cleanUrl(token.href);
      return href
        ? h(
            "a",
            { href, title: token.title || undefined, rel: "noreferrer" },
            renderTokens(token.tokens),
          )
        : renderTokens(token.tokens);
    }
    case "image": {
      const src = cleanUrl(token.href);
      if (!src || src.startsWith("mailto:") || src.startsWith("tel:")) return token.text;
      return h("img", { src, alt: token.text, title: token.title || undefined });
    }
    case "table": {
      const table = token as Tokens.Table;
      return h("table", [
        h("thead", [
          h(
            "tr",
            table.header.map((cell) => renderTableCell(cell, "th")),
          ),
        ]),
        h(
          "tbody",
          table.rows.map((row) =>
            h(
              "tr",
              row.map((cell) => renderTableCell(cell, "td")),
            ),
          ),
        ),
      ]);
    }
    default:
      return "text" in token && typeof token.text === "string" ? token.text : null;
  }
}

const MarkdownContent: FunctionalComponent<{ markdown: string }> = (props) =>
  h("div", { class: "docs-markdown" }, renderTokens(markedInstance.lexer(props.markdown)));

export default MarkdownContent;
