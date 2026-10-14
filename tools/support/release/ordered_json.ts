// Order-preserving JSON values with the MoonBit `@json` parse and stringify
// rules the release commands relied on. Unlike `JSON.parse`, object members
// keep their source order even for integer-like keys, and integer literals
// beyond the exact double range keep their source text when written back.

export interface JsonNumber {
  readonly kind: "number";
  readonly value: number;
  readonly repr: string | null;
}

export type JsonValue = null | boolean | string | JsonNumber | JsonValue[] | JsonObject;

export class JsonObject {
  readonly members = new Map<string, JsonValue>();

  get(key: string): JsonValue | undefined {
    return this.members.get(key);
  }

  set(key: string, value: JsonValue): void {
    this.members.set(key, value);
  }
}

export const jsonNumber = (value: number): JsonNumber => ({ kind: "number", value, repr: null });

export const isJsonObject = (value: JsonValue | undefined): value is JsonObject =>
  value instanceof JsonObject;

export const isJsonNumber = (value: JsonValue | undefined): value is JsonNumber =>
  typeof value === "object" && value !== null && !Array.isArray(value) && "kind" in value;

/** Returns a string member, or `undefined` when it is absent or not a string. */
export function stringMember(value: JsonValue | undefined, key: string): string | undefined {
  if (!isJsonObject(value)) return undefined;
  const member = value.get(key);
  return typeof member === "string" ? member : undefined;
}

const MAX_NESTING_DEPTH = 1024;
const SAFE_INTEGER_LIMIT = 9007199254740991n;

class JsonSyntaxError extends Error {}

class Parser {
  index = 0;
  readonly input: string;

  constructor(input: string) {
    this.input = input;
  }

  fail(): never {
    throw new JsonSyntaxError(`Invalid JSON at offset ${this.index}`);
  }

  skipWhitespace(): void {
    while (this.index < this.input.length) {
      const character = this.input[this.index];
      if (character !== " " && character !== "\t" && character !== "\n" && character !== "\r") {
        return;
      }
      this.index++;
    }
  }

  parseValue(depth: number): JsonValue {
    this.skipWhitespace();
    const character = this.input[this.index];
    if (character === "{") return this.parseObject(depth);
    if (character === "[") return this.parseArray(depth);
    if (character === '"') return this.parseString();
    if (character === "-" || (character !== undefined && character >= "0" && character <= "9")) {
      return this.parseNumber();
    }
    for (const [literal, value] of [
      ["true", true],
      ["false", false],
      ["null", null],
    ] as const) {
      if (this.input.startsWith(literal, this.index)) {
        this.index += literal.length;
        return value;
      }
    }
    return this.fail();
  }

  parseObject(depth: number): JsonObject {
    if (depth <= 0) this.fail();
    this.index++;
    const object = new JsonObject();
    this.skipWhitespace();
    if (this.input[this.index] === "}") {
      this.index++;
      return object;
    }
    for (;;) {
      this.skipWhitespace();
      if (this.input[this.index] !== '"') this.fail();
      const key = this.parseString();
      this.skipWhitespace();
      if (this.input[this.index] !== ":") this.fail();
      this.index++;
      object.set(key, this.parseValue(depth - 1));
      this.skipWhitespace();
      const separator = this.input[this.index];
      this.index++;
      if (separator === "}") return object;
      if (separator !== ",") this.fail();
    }
  }

  parseArray(depth: number): JsonValue[] {
    if (depth <= 0) this.fail();
    this.index++;
    const items: JsonValue[] = [];
    this.skipWhitespace();
    if (this.input[this.index] === "]") {
      this.index++;
      return items;
    }
    for (;;) {
      items.push(this.parseValue(depth - 1));
      this.skipWhitespace();
      const separator = this.input[this.index];
      this.index++;
      if (separator === "]") return items;
      if (separator !== ",") this.fail();
    }
  }

  parseString(): string {
    const start = this.index;
    this.index++;
    for (;;) {
      const character = this.input[this.index];
      if (character === undefined) this.fail();
      if (character === '"') {
        this.index++;
        break;
      }
      if (character < " ") this.fail();
      if (character === "\\") {
        const escape = this.input[this.index + 1];
        if (escape === "u") {
          if (!/^[0-9a-fA-F]{4}$/.test(this.input.slice(this.index + 2, this.index + 6))) {
            this.fail();
          }
          this.index += 6;
          continue;
        }
        if (escape === undefined || !'"\\/bfnrt'.includes(escape)) this.fail();
        this.index += 2;
        continue;
      }
      this.index++;
    }
    return JSON.parse(this.input.slice(start, this.index)) as string;
  }

  parseNumber(): JsonNumber {
    const match = /^-?(?:0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?/.exec(
      this.input.slice(this.index),
    );
    if (match === null) this.fail();
    const text = match[0];
    this.index += text.length;
    const value = Number(text);
    const integerLiteral = match[1] === undefined && match[2] === undefined;
    const magnitude = integerLiteral ? BigInt(text.startsWith("-") ? text.slice(1) : text) : 0n;
    const keepsSource =
      (integerLiteral && magnitude > SAFE_INTEGER_LIMIT) || !Number.isFinite(value);
    return { kind: "number", value, repr: keepsSource ? text : null };
  }
}

/**
 * Parses strict JSON with a nesting limit of 1024, returning `undefined` when
 * the input is not valid JSON.
 */
export function parseJson(input: string): JsonValue | undefined {
  const parser = new Parser(input);
  try {
    const value = parser.parseValue(MAX_NESTING_DEPTH);
    parser.skipWhitespace();
    if (parser.index < input.length) return undefined;
    return value;
  } catch (error) {
    if (error instanceof JsonSyntaxError) return undefined;
    throw error;
  }
}

function writeString(value: string): string {
  let output = '"';
  for (const character of value) {
    const code = character.codePointAt(0) ?? 0;
    if (character === '"') output += '\\"';
    else if (character === "\\") output += "\\\\";
    else if (character === "\n") output += "\\n";
    else if (character === "\r") output += "\\r";
    else if (character === "\b") output += "\\b";
    else if (character === "\t") output += "\\t";
    else if (code === 0x0c) output += "\\f";
    else if (code < 0x20) output += `\\u00${code.toString(16).padStart(2, "0")}`;
    else output += character;
  }
  return `${output}"`;
}

function newline(depth: number, indent: number): string {
  return indent === 0 ? "" : `\n${" ".repeat(indent * depth)}`;
}

function write(value: JsonValue, indent: number, depth: number): string {
  if (value === null) return "null";
  if (value === true) return "true";
  if (value === false) return "false";
  if (typeof value === "string") return writeString(value);
  if (Array.isArray(value)) {
    if (value.length === 0) return "[]";
    const items = value.map((item) => newline(depth + 1, indent) + write(item, indent, depth + 1));
    return `[${items.join(",")}${newline(depth, indent)}]`;
  }
  if (isJsonObject(value)) {
    if (value.members.size === 0) return "{}";
    const separator = indent > 0 ? ": " : ":";
    const members = [...value.members].map(
      ([key, member]) =>
        newline(depth + 1, indent) +
        writeString(key) +
        separator +
        write(member, indent, depth + 1),
    );
    return `{${members.join(",")}${newline(depth, indent)}}`;
  }
  return value.repr ?? String(value.value);
}

/** Serializes a value, compactly or with the given indentation width. */
export function stringifyJson(value: JsonValue, indent = 0): string {
  return write(value, indent, 0);
}
