import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, realpathSync, statSync } from "node:fs";
import path from "node:path";

export const sha256 = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
export type FileImage = { path: string; bytes: number; sha256: string };
type LocalEdge = {
  kind:
    | "reference"
    | "type-import"
    | "type-export"
    | "import"
    | "export"
    | "import-type"
    | "type-require";
  specifier: string;
  attributes: Record<string, string>;
};
export type DeclarationEdge = LocalEdge & { from: string; target: string };
export type DeclarationGraph = { files: FileImage[]; edges: DeclarationEdge[] };

function readImage(filename: string) {
  const actual = realpathSync(filename);
  assert(statSync(actual).isFile(), `regular file required: ${filename}`);
  const contents = readFileSync(actual);
  return {
    image: { path: actual, bytes: contents.length, sha256: sha256(contents) },
    contents,
  };
}
export function fileImage(filename: string): FileImage {
  return readImage(filename).image;
}
export function assertCustody(images: FileImage[]) {
  assert(images.length > 0, "custody must include actual installed files");
  assert.equal(new Set(images.map((image) => image.path)).size, images.length);
  for (const image of images) {
    assert.deepEqual(fileImage(image.path), image, `installed file changed: ${image.path}`);
  }
}

const declarationEntries = [
  "virtual-ts.d.ts",
  "virtual-ts.d.mts",
  "virtual-ts.d.cts",
  "content-mapper-helpers.d.ts",
  "content-mapper-helpers.d.mts",
  "content-mapper-helpers.d.cts",
  "generated/content-mapper-types.d.ts",
] as const;

export function inside(root: string, filename: string) {
  const relative = path.relative(root, filename);
  return (
    relative === "" ||
    (!relative.startsWith(`..${path.sep}`) && relative !== ".." && !path.isAbsolute(relative))
  );
}

function referencePath(line: string, filename: string): string | undefined {
  if (!/^\/\/\/\s*<reference\b/u.test(line)) return;
  const tag = /^\/\/\/\s*<reference\s+(.+?)\s*\/>/u.exec(line);
  assert(tag, `malformed declaration reference in ${filename}: ${line}`);
  let remaining = tag[1]!;
  const attributes = new Map<string, string>();
  while (remaining.trim()) {
    const attribute = /^\s*([\w-]+)\s*=\s*(["'])(.*?)\2(?:\s+|$)/u.exec(remaining);
    assert(attribute, `malformed declaration reference in ${filename}: ${line}`);
    const [, name, , value] = attribute;
    assert(!attributes.has(name!), `duplicate declaration reference attribute: ${name}`);
    attributes.set(name!, value!);
    remaining = remaining.slice(attribute[0].length);
  }
  assert(
    ["path", "types", "lib", "no-default-lib"].some((name) => attributes.has(name)),
    `malformed declaration reference in ${filename}: ${line}`,
  );
  if (!attributes.has("path")) return; // External type/lib references are not local file paths.
  const value = attributes.get("path")!;
  assert(value && !value.includes("\0"), `empty or invalid declaration reference in ${filename}`);
  assert(
    !path.win32.isAbsolute(value) &&
      !path.posix.isAbsolute(value) &&
      !/^[a-z][a-z0-9+.-]*:/iu.test(value),
    `declaration reference must stay local: ${value}`,
  );
  return value.replace(/\\/gu, "/");
}

/** TypeScript recognizes reference directives only among a file's leading comments. */
function localReferences(contents: Buffer, filename: string): string[] {
  let remaining = contents.toString("utf8").trimStart();
  const references: string[] = [];
  while (remaining) {
    if (remaining.startsWith("/*")) {
      const end = remaining.indexOf("*/", 2);
      if (end < 0) break;
      remaining = remaining.slice(end + 2).trimStart();
    } else if (remaining.startsWith("//")) {
      const end = remaining.search(/[\r\n\u2028\u2029]/u);
      const line = end < 0 ? remaining : remaining.slice(0, end);
      const reference = referencePath(line, filename);
      if (reference !== undefined) references.push(reference);
      remaining = end < 0 ? "" : remaining.slice(end).trimStart();
    } else break;
  }
  return references;
}

type Token = { value: string; string: boolean };

function decodeString(raw: string): string {
  let value = "";
  for (let index = 1; index < raw.length - 1; index++) {
    let character = raw[index]!;
    if (character === "\\") {
      character = raw[++index]!;
      if (character === "u" || character === "x") {
        const braced = character === "u" && raw[index + 1] === "{";
        const end = braced ? raw.indexOf("}", index + 2) : index + (character === "u" ? 4 : 2);
        const digits = raw.slice(index + (braced ? 2 : 1), end + (braced ? 0 : 1));
        assert(/^[0-9a-f]+$/iu.test(digits), "malformed declaration module string escape");
        value += String.fromCodePoint(Number.parseInt(digits, 16));
        index = end;
        continue;
      }
      if (/[1-9]/u.test(character) || (character === "0" && /[0-9]/u.test(raw[index + 1] ?? "")))
        throw new Error("legacy octal escape in declaration module string");
      if (/[\r\n\u2028\u2029]/u.test(character)) {
        if (character === "\r" && raw[index + 1] === "\n") index++;
        continue;
      }
      character =
        (
          { "0": "\0", b: "\b", f: "\f", n: "\n", r: "\r", t: "\t", v: "\v" } as Record<
            string,
            string
          >
        )[character] ?? character;
    }
    value += character;
  }
  return value;
}

/** Lex strings/comments without evaluating declaration code; template type expressions remain visible. */
function tokens(contents: Buffer): Token[] {
  const source = contents.toString("utf8"),
    result: Token[] = [];
  let index = 0;
  const emit = (value: string, string = false) => result.push({ value, string });
  const scan = (closingBrace = false) => {
    while (index < source.length) {
      const character = source[index]!;
      if (/\s/u.test(character)) {
        index++;
        continue;
      }
      if (source.startsWith("//", index)) {
        const end = source.slice(index).search(/[\r\n\u2028\u2029]/u);
        index = end < 0 ? source.length : index + end;
      } else if (source.startsWith("/*", index)) {
        const end = source.indexOf("*/", index + 2);
        assert(end >= 0, "unterminated declaration comment");
        index = end + 2;
      } else if (character === '"' || character === "'") {
        const start = index++;
        while (index < source.length && source[index] !== character) {
          if (source[index] === "\\") index++;
          index++;
        }
        assert(index < source.length, "unterminated declaration module string");
        emit(decodeString(source.slice(start, ++index)), true);
      } else if (character === "`") {
        index++;
        while (index < source.length && source[index] !== "`") {
          if (source[index] === "\\") index += 2;
          else if (source.startsWith("${", index)) {
            emit("{");
            index += 2;
            scan(true);
          } else index++;
        }
        assert(index < source.length, "unterminated declaration template");
        index++;
      } else if (character === "{") {
        emit(character);
        index++;
        scan(true);
      } else if (character === "}") {
        emit(character);
        index++;
        if (closingBrace) return;
      } else if (/[a-z_$]/iu.test(character)) {
        const start = index++;
        while (/[\w$]/u.test(source[index] ?? "")) index++;
        emit(source.slice(start, index));
      } else {
        emit(character);
        index++;
      }
    }
  };
  scan();
  return result;
}

function importAttributes(
  list: Token[],
  start: number,
): { attributes: Record<string, string>; next: number } {
  const attributes = new Map<string, string>();
  assert.equal(list[start]?.value, "{", "malformed declaration import attributes");
  let index = start + 1;
  while (list[index]?.value !== "}") {
    const key = list[index++]?.value;
    assert(key && list[index++]?.value === ":", "malformed declaration import attribute key");
    const value = list[index++];
    assert(value?.string, "declaration import attribute value must be a string");
    assert(!attributes.has(key), `duplicate declaration import attribute: ${key}`);
    attributes.set(key, value.value);
    if (list[index]?.value === ",") index++;
    else assert.equal(list[index]?.value, "}", "malformed declaration import attributes");
  }
  const mode = attributes.get("resolution-mode");
  assert(
    mode === undefined || mode === "import" || mode === "require",
    "invalid declaration resolution-mode",
  );
  return { attributes: Object.fromEntries(attributes), next: index + 1 };
}

function moduleEdges(contents: Buffer): LocalEdge[] {
  const list = tokens(contents),
    edges: LocalEdge[] = [];
  const add = (kind: LocalEdge["kind"], specifier: Token | undefined, after: number) => {
    assert(specifier?.string, "declaration module specifier must be a string");
    const value = specifier.value.replace(/\\/gu, "/");
    assert(value && !value.includes("\0"), "invalid declaration module specifier");
    if (!value.startsWith(".")) {
      assert(
        !path.win32.isAbsolute(value) &&
          !path.posix.isAbsolute(value) &&
          (!/^[a-z][a-z0-9+.-]*:/iu.test(value) || value.startsWith("node:")),
        `declaration module reference must stay local: ${value}`,
      );
      return; // Bare Vue/package/builtin imports retain their ordinary external resolution.
    }
    let attributes: Record<string, string> = {};
    if (list[after]?.value === "with") attributes = importAttributes(list, after + 1).attributes;
    else if (list[after]?.value === ",") {
      assert(
        list[after + 1]?.value === "{" &&
          list[after + 2]?.value === "with" &&
          list[after + 3]?.value === ":",
        "malformed declaration import-type attributes",
      );
      const parsed = importAttributes(list, after + 4);
      assert(
        list[parsed.next]?.value === "}" && list[parsed.next + 1]?.value === ")",
        "malformed declaration import-type attributes",
      );
      attributes = parsed.attributes;
    }
    edges.push({ kind, specifier: value, attributes });
  };
  for (let index = 0; index < list.length; index++) {
    const current = list[index]!;
    if (current.string || !["import", "export"].includes(current.value)) continue;
    if (list[index - 1]?.value === ".") continue;
    if (current.value === "import" && list[index + 1]?.value === "(") {
      if (list[index + 2]?.string) add("import-type", list[index + 2], index + 3);
      continue;
    }
    if (current.value === "import" && list[index + 1]?.string) {
      add("import", list[index + 1], index + 2);
      continue;
    }
    const typeOnly = list[index + 1]?.value === "type";
    const start = index + (typeOnly ? 2 : 1);
    if (current.value === "export" && !["*", "{"].includes(list[start]?.value ?? "")) continue;
    if (
      current.value === "import" &&
      !typeOnly &&
      !["*", "{"].includes(list[start]?.value ?? "") &&
      !/^[a-z_$][\w$]*$/iu.test(list[start]?.value ?? "")
    )
      continue;
    let depth = 0;
    for (let end = start; end < list.length && list[end]?.value !== ";"; end++) {
      const token = list[end]!;
      if (token.value === "{") depth++;
      if (token.value === "}") depth--;
      if (!token.string && token.value === "from" && depth === 0 && list[end + 1]?.string) {
        add(
          typeOnly
            ? current.value === "import"
              ? "type-import"
              : "type-export"
            : (current.value as "import" | "export"),
          list[end + 1],
          end + 2,
        );
        break;
      }
      if (
        current.value === "import" &&
        token.value === "require" &&
        depth === 0 &&
        list[end - 1]?.value === "=" &&
        list[end + 1]?.value === "("
      ) {
        add("type-require", list[end + 2], end + 3);
        break;
      }
    }
  }
  return edges;
}

function declarationTarget(edge: LocalEdge): string {
  if (edge.kind === "reference") return edge.specifier;
  const specifier = edge.specifier;
  if (/\.d\.(?:ts|mts|cts)$/u.test(specifier)) return specifier;
  for (const [source, declaration] of [
    [".js", ".d.ts"],
    [".mjs", ".d.mts"],
    [".cjs", ".d.cts"],
  ])
    if (specifier.endsWith(source!)) return specifier.slice(0, -source!.length) + declaration;
  throw new Error(`unsupported local declaration module extension: ${specifier}`);
}

/** Capture actual files and module-kind attributes without converting imports into script references. */
export function captureDeclarationGraph(directory: string): DeclarationGraph {
  const root = realpathSync(directory);
  assert(statSync(root).isDirectory(), `installed package directory required: ${directory}`);
  const pending = declarationEntries.map((name) => path.join(root, name));
  const seen = new Set<string>();
  const images: FileImage[] = [];
  const edges: DeclarationEdge[] = [];
  for (let index = 0; index < pending.length; index++) {
    const filename = pending[index]!;
    assert(inside(root, filename), `declaration reference escapes installed package: ${filename}`);
    const actual = realpathSync(filename);
    assert(inside(root, actual), `declaration reference escapes installed package: ${filename}`);
    assert.match(
      actual,
      /\.d\.(?:ts|mts|cts)$/u,
      "declaration reference must name a declaration file",
    );
    if (seen.has(actual)) continue; // Legitimate reference cycles and shared leaves are finite.
    const { image, contents } = readImage(actual);
    assert(
      inside(root, image.path),
      `declaration reference escapes installed package: ${filename}`,
    );
    seen.add(image.path);
    images.push(image);
    const references: LocalEdge[] = localReferences(contents, image.path).map((specifier) => ({
      kind: "reference",
      specifier,
      attributes: {},
    }));
    for (const edge of [...references, ...moduleEdges(contents)]) {
      const target = path.resolve(path.dirname(image.path), declarationTarget(edge));
      assert(inside(root, target), `declaration reference escapes installed package: ${target}`);
      const actualTarget = realpathSync(target);
      assert(
        inside(root, actualTarget),
        `declaration reference escapes installed package: ${target}`,
      );
      edges.push({ ...edge, from: image.path, target: actualTarget });
      pending.push(actualTarget);
    }
  }
  return { files: images, edges };
}

export function captureDeclarationCustody(directory: string): FileImage[] {
  return captureDeclarationGraph(directory).files;
}

export function assertDeclarationCustody(
  directory: string,
  images: FileImage[],
  edges?: DeclarationEdge[],
) {
  const actual = captureDeclarationGraph(directory);
  assert.deepEqual(actual.files, images, "installed declaration closure changed");
  if (edges) assert.deepEqual(actual.edges, edges, "installed declaration module edges changed");
}
