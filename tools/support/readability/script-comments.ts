// Comment and literal masking for TypeScript, JavaScript and Vue `<script>`
// blocks, for line-based source checks that need no parser.
//
// `maskScript` returns the comments and a copy of the source in which every
// comment and the contents of every string, template and regular-expression
// literal are spaces. Newlines survive, so line numbers stay valid. Template
// substitutions (`${…}`) are code and stay visible. A `/` starts a regular
// expression when the previous significant token cannot end an expression;
// that is the usual heuristic and it is deterministic, which is what a
// ratchet needs.

import { parseSync, Visitor } from "vite";

import { vueScripts } from "./vue-scripts.ts";

export interface ScriptComment {
  /** `doc` is `/** … *\/`; `line` is `//`; `block` is any other `/* … *\/`. */
  readonly kind: "line" | "block" | "doc";
  readonly lines: readonly string[];
  readonly line: number;
}

export interface MaskedScript {
  readonly comments: readonly ScriptComment[];
  /** The source with comments and literal contents blanked. */
  readonly code: string;
}

const expressionKeywords = new Set([
  "return",
  "typeof",
  "instanceof",
  "case",
  "in",
  "of",
  "new",
  "delete",
  "void",
  "throw",
  "else",
  "do",
  "yield",
  "await",
]);

const controlKeywords = new Set(["if", "while", "for", "with", "switch", "catch"]);

const isWordChar = (char: string) => /[\p{L}\p{N}_$]/u.test(char);

class ScriptMasker {
  readonly comments: ScriptComment[] = [];
  private readonly source: string;
  private readonly out: string[];
  private index = 0;
  private line = 1;
  /** `true` when the last significant token ends an expression, so `/` divides. */
  private afterExpression = false;
  private pendingControl = false;
  private readonly controlParens: boolean[] = [];
  /** Brace depth at each open template substitution. */
  private readonly templates: number[] = [];
  private braces = 0;

  constructor(source: string) {
    this.source = source;
    this.out = source.split("");
  }

  run(): MaskedScript {
    while (this.index < this.source.length) this.step();
    return { comments: this.comments, code: this.out.join("") };
  }

  private step(): void {
    const char = this.source[this.index];
    const next = this.source[this.index + 1] ?? "";
    if (char === "\n") {
      this.line++;
      this.index++;
    } else if (char === " " || char === "\t" || char === "\r") this.index++;
    else if (char === "/" && next === "/") this.lineComment();
    else if (char === "/" && next === "*") this.blockComment();
    else if (char === '"' || char === "'") this.string(char);
    else if (char === "`") this.template(this.index + 1);
    else if (char === "/" && !this.afterExpression && this.regex()) return;
    else if (char === "}" && this.templates.at(-1) === this.braces - 1) {
      this.templates.pop();
      this.braces--;
      this.template(this.index + 1);
    } else if (isWordChar(char)) this.word();
    else if ((char === "+" || char === "-") && next === char) {
      this.index += 2;
      this.pendingControl = false;
    } else {
      if (char === "(") this.controlParens.push(this.pendingControl);
      this.pendingControl = false;
      if (char === "{") this.braces++;
      if (char === "}") this.braces--;
      this.afterExpression =
        char === ")" ? !this.controlParens.pop() : char === "]" || char === "}";
      this.index++;
    }
  }

  private blank(start: number, end: number): void {
    for (let at = start; at < end; at++) if (this.out[at] !== "\n") this.out[at] = " ";
  }

  /** Blank `start..end` and move past it, counting the newlines it spans. */
  private consume(start: number, end: number, blankFrom = start, blankTo = end): void {
    this.blank(blankFrom, blankTo);
    for (let at = start; at < end; at++) if (this.source[at] === "\n") this.line++;
    this.index = end;
  }

  private lineComment(): void {
    const newline = this.source.indexOf("\n", this.index);
    const end = newline < 0 ? this.source.length : newline;
    this.comments.push({
      kind: "line",
      lines: [this.source.slice(this.index + 2, end)],
      line: this.line,
    });
    this.consume(this.index, end);
  }

  private blockComment(): void {
    const close = this.source.indexOf("*/", this.index + 2);
    const end = close < 0 ? this.source.length : close + 2;
    const body = this.source.slice(this.index, end);
    const doc = body.startsWith("/**") && body !== "/**/" && !body.startsWith("/***");
    const inner = body.slice(doc ? 3 : 2, body.endsWith("*/") ? -2 : undefined);
    this.comments.push({ kind: doc ? "doc" : "block", lines: inner.split("\n"), line: this.line });
    this.consume(this.index, end);
  }

  private string(quote: string): void {
    let at = this.index + 1;
    while (at < this.source.length) {
      const char = this.source[at];
      if (char === "\\") at += this.source.startsWith("\r\n", at + 1) ? 3 : 2;
      else if (char === quote || char === "\n") break;
      else at++;
    }
    const end = Math.min(at + 1, this.source.length);
    this.consume(this.index, end, this.index + 1, end - 1);
    this.afterExpression = true;
  }

  /** Template text from `start` to the closing backtick or the next `${`. */
  private template(start: number): void {
    let at = start;
    let substitution = false;
    while (at < this.source.length) {
      const char = this.source[at];
      if (char === "\\") at += this.source.startsWith("\r\n", at + 1) ? 3 : 2;
      else if (char === "`") break;
      else if (char === "$" && this.source[at + 1] === "{") {
        substitution = true;
        break;
      } else at++;
    }
    const end = Math.min(at + (substitution ? 2 : 1), this.source.length);
    this.consume(start, end, start, substitution ? end - 2 : end - 1);
    if (substitution) this.templates.push(this.braces++);
    this.afterExpression = !substitution;
  }

  private regex(): boolean {
    let at = this.index + 1;
    let inClass = false;
    while (at < this.source.length && this.source[at] !== "\n") {
      const char = this.source[at];
      if (char === "\\") {
        at += 2;
        continue;
      }
      if (char === "[") inClass = true;
      else if (char === "]") inClass = false;
      else if (char === "/" && !inClass) {
        at++;
        while (/[a-z]/iu.test(this.source[at] ?? "")) at++;
        this.consume(this.index, at, this.index + 1, at);
        this.afterExpression = true;
        return true;
      }
      at++;
    }
    return false;
  }

  private word(): void {
    let end = this.index + 1;
    while (end < this.source.length && isWordChar(this.source[end])) end++;
    const word = this.source.slice(this.index, end);
    this.pendingControl = controlKeywords.has(word);
    this.afterExpression = !expressionKeywords.has(word);
    this.index = end;
  }
}

/** JSX children are text until an expression container enters JavaScript again. */
function maskJsx(source: string, lang: "jsx" | "tsx"): MaskedScript {
  const parsed = parseSync(`source.${lang}`, source, { lang, sourceType: "unambiguous" });
  if (parsed.errors.length) return new ScriptMasker(source).run();
  const out = source.split("");
  const blank = (start: number, end: number) => {
    for (let at = start; at < end; at++) if (out[at] !== "\n" && out[at] !== "\r") out[at] = " ";
  };
  new Visitor({
    JSXText(node) {
      blank(node.start, node.end);
    },
    TemplateElement(node) {
      blank(node.start + 1, node.end - (node.tail ? 1 : 2));
    },
    Literal(node) {
      if (typeof node.value === "string") blank(node.start + 1, node.end - 1);
      else if ("regex" in node) blank(node.start, node.end);
    },
  }).visit(parsed.program);
  let cursor = 0;
  let line = 1;
  const comments = parsed.comments.map((comment): ScriptComment => {
    while (cursor < comment.start) if (source[cursor++] === "\n") line++;
    const kind =
      comment.type === "Line" ? "line" : source.startsWith("/**", comment.start) ? "doc" : "block";
    blank(comment.start, comment.end);
    return {
      kind,
      line,
      lines: (kind === "doc" ? comment.value.replace(/^\*/u, "") : comment.value).split("\n"),
    };
  });
  return { code: out.join(""), comments };
}

export function maskScript(source: string, lang?: string): MaskedScript {
  return lang === "jsx" || lang === "tsx" ? maskJsx(source, lang) : new ScriptMasker(source).run();
}

/** Preserve SFC offsets while giving each script its own lexical state. */
export function maskVueScripts(source: string): MaskedScript & { readonly raw: string } {
  const raw: string[] = [];
  const code: string[] = [];
  const comments: ScriptComment[] = [];
  const gap = (text: string) => text.replace(/[^\r\n]/g, " ");
  let end = 0;
  for (const block of vueScripts(source).blocks) {
    const blank = gap(source.slice(end, block.loc.start.offset));
    const masked = maskScript(block.content, block.lang);
    raw.push(blank, block.content);
    code.push(blank, masked.code);
    for (const comment of masked.comments) {
      comments.push({ ...comment, line: block.loc.start.line + comment.line - 1 });
    }
    end = block.loc.end.offset;
  }
  const tail = gap(source.slice(end));
  return { raw: raw.join("") + tail, code: code.join("") + tail, comments };
}
