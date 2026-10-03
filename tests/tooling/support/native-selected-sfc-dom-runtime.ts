import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
export const fromVue = createRequire(fromUi.resolve("vue/package.json"));
export const compiler = fromVue("@vue/compiler-sfc");
export const runtime = fromVue("vue");
const { parse } = createRequire(fromVue.resolve("@vue/compiler-sfc/package.json"))("@babel/parser");
export const hash = (text: string) => createHash("sha256").update(text).digest("hex");
const dataUrl = (text: string) =>
  `data:text/javascript;base64,${Buffer.from(text).toString("base64")}`;
const runtimeUrl = dataUrl(
  `import runtime from ${JSON.stringify(pathToFileURL(fromVue.resolve("vue")).href)};\n` +
    [
      "openBlock",
      "createElementBlock",
      "createElementVNode",
      "Fragment",
      "toDisplayString",
      "createTextVNode",
      "defineComponent",
    ]
      .map((name) => `export const ${name} = runtime.${name};`)
      .join("\n"),
);

export function runtimeModuleSource(code: string): string {
  const imports = parse(code, { sourceType: "module" }).program.body.filter(
    (node: any) => node.type === "ImportDeclaration" && node.source.value === "vue",
  );
  assert(imports.length > 0, "the complete component imports the actual Vue runtime");
  let rewritten = code;
  for (const declaration of imports.toReversed()) {
    const { start, end } = declaration.source;
    assert.equal(declaration.source.type, "StringLiteral");
    assert(Number.isInteger(start) && Number.isInteger(end) && start >= 0 && end <= code.length);
    // Babel's UTF-16 literal range excludes all authored strings/comments and
    // every other module byte. Reverse order retains the original offsets.
    rewritten = rewritten.slice(0, start) + JSON.stringify(runtimeUrl) + rewritten.slice(end);
  }
  return rewritten;
}

function vlq(segment: string): number[] {
  const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  const fields: number[] = [];
  let value = 0,
    shift = 0;
  for (const character of segment) {
    const digit = alphabet.indexOf(character);
    assert(digit >= 0);
    value += (digit & 31) * 2 ** shift;
    if (digit & 32) shift += 5;
    else {
      fields.push(value & 1 ? -(value >> 1) : value >> 1);
      value = 0;
      shift = 0;
    }
  }
  assert.equal(shift, 0);
  return fields;
}
function position(text: string, offset: number): [number, number] {
  assert(offset >= 0 && offset <= text.length);
  const prefix = text.slice(0, offset).split(/\r\n|[\n\r\u2028\u2029]/);
  return [prefix.length - 1, prefix.at(-1)!.length];
}
function anchors(map: any, code: string, source: string) {
  const generatedLines = code.split(/\r\n|[\n\r\u2028\u2029]/);
  const sourceLines = source.split(/\r\n|[\n\r\u2028\u2029]/);
  const result: { generated: number[]; authored: number[]; name: string | null }[] = [];
  let sourceIndex = 0,
    sourceLine = 0,
    sourceColumn = 0,
    nameIndex = 0;
  map.mappings.split(";").forEach((line: string, generatedLine: number) => {
    let column = 0;
    for (const segment of line.split(",").filter(Boolean)) {
      const fields = vlq(segment);
      assert(fields.length === 4 || fields.length === 5);
      column += fields[0];
      sourceIndex += fields[1];
      sourceLine += fields[2];
      sourceColumn += fields[3];
      assert.equal(sourceIndex, 0);
      assert(generatedLines[generatedLine] !== undefined && sourceLines[sourceLine] !== undefined);
      assert(column >= 0 && column <= generatedLines[generatedLine].length);
      assert(sourceColumn >= 0 && sourceColumn <= sourceLines[sourceLine].length);
      let name = null;
      if (fields.length === 5) {
        nameIndex += fields[4];
        assert(map.names[nameIndex] !== undefined);
        name = map.names[nameIndex];
      }
      result.push({
        generated: [generatedLine, column],
        authored: [sourceLine, sourceColumn],
        name,
      });
    }
  });
  assert(result.length > 0);
  return result;
}

export function checkNativeMap(fixture: any, filename: string, code: string, map: any) {
  assert(map, `${fixture.id}: complete native map must be frozen from actual Rust output`);
  assert.equal(map.version, 3);
  assert.equal(map.file, filename);
  assert.deepEqual(map.sources, [filename]);
  assert.deepEqual(map.sourcesContent, [fixture.source]);
  assert.deepEqual(map.names, ["$event"]);
  const measured = anchors(map, code, fixture.source);
  const raw = /@click="([^"]*)"/.exec(fixture.source)!;
  assert(raw);
  const authoredStart = raw.index + '@click="'.length;
  const decoded = raw[1].replaceAll("&quot;", '"');
  const generatedStart = code.indexOf("onClick: $event => {") + "onClick: $event => {".length;
  assert(generatedStart >= "onClick: $event => {".length);
  assert.equal(code.slice(generatedStart, generatedStart + decoded.length), decoded);
  assert.equal(code[generatedStart + decoded.length], "}");
  // Independently partition the authored body, retaining each entity and each
  // plain $event reference. JS offsets are UTF-16, including the snow/flower.
  const pieces: { raw: string; decoded: string; name: string | null }[] = [];
  let at = 0;
  for (const match of raw[1].matchAll(/&quot;|\$event/g)) {
    if (match.index > at) {
      const text = raw[1].slice(at, match.index);
      pieces.push({ raw: text, decoded: text, name: null });
    }
    pieces.push({
      raw: match[0],
      decoded: match[0] === "&quot;" ? '"' : match[0],
      name: match[0] === "$event" ? "$event" : null,
    });
    at = match.index + match[0].length;
  }
  if (at < raw[1].length) {
    const text = raw[1].slice(at);
    pieces.push({ raw: text, decoded: text, name: null });
  }
  assert.equal(pieces.map((piece) => piece.raw).join(""), raw[1]);
  assert.equal(pieces.map((piece) => piece.decoded).join(""), decoded);
  let originalOffset = authoredStart,
    generatedOffset = generatedStart;
  const expected = pieces.map((piece) => {
    const anchor = {
      generated: position(code, generatedOffset),
      authored: position(fixture.source, originalOffset),
      name: piece.name,
    };
    assert.equal(
      fixture.source.slice(originalOffset, originalOffset + piece.raw.length),
      piece.raw,
    );
    assert.equal(
      code.slice(generatedOffset, generatedOffset + piece.decoded.length),
      piece.decoded,
    );
    originalOffset += piece.raw.length;
    generatedOffset += piece.decoded.length;
    return anchor;
  });
  assert.equal(originalOffset, authoredStart + raw[1].length);
  assert.equal(generatedOffset, generatedStart + decoded.length);
  const [line, column] = position(code, generatedStart);
  assert.deepEqual(
    measured.filter(
      (point) =>
        point.generated[0] === line &&
        point.generated[1] >= column &&
        point.generated[1] < column + decoded.length,
    ),
    expected,
    `${fixture.id}: the entire decoded body retains exact original SFC anchors`,
  );
  assert.equal(
    measured.filter((point) => point.name === "$event").length,
    [...raw[1].matchAll(/\$event/g)].length,
  );
  const tag = fixture.source.indexOf("<button");
  assert(
    measured.some(
      (point) =>
        JSON.stringify(point.authored) === JSON.stringify(position(fixture.source, tag)) &&
        JSON.stringify(point.generated) ===
          JSON.stringify(position(code, code.indexOf('"button"'))),
    ),
  );
}

type Host = {
  type: string;
  text: string;
  props: Record<string, any>;
  children: Host[];
  parent: Host | null;
};
const host = (type: string, text = ""): Host => ({
  type,
  text,
  props: {},
  children: [],
  parent: null,
});
function rendererHost() {
  const clicks: { target: Host; previous: any; value: any }[] = [];
  const renderer = runtime.createRenderer({
    createElement: (type: string) => host(type),
    createText: (text: string) => host("text", text),
    createComment: (text: string) => host("comment", text),
    setText: (target: Host, text: string) => {
      target.text = text;
    },
    setElementText: (target: Host, text: string) => {
      target.text = text;
      target.children = [];
    },
    parentNode: (target: Host) => target.parent,
    nextSibling: (target: Host) =>
      target.parent?.children[target.parent.children.indexOf(target) + 1] ?? null,
    patchProp: (target: Host, key: string, previous: any, value: any) => {
      if (key === "onClick") clicks.push({ target, previous, value });
      target.props[key] = value;
    },
    insert: (target: Host, parent: Host, anchor: Host | null) => {
      if (target.parent) target.parent.children.splice(target.parent.children.indexOf(target), 1);
      const index = anchor ? parent.children.indexOf(anchor) : -1;
      parent.children.splice(index < 0 ? parent.children.length : index, 0, target);
      target.parent = parent;
    },
    remove: (target: Host) => {
      assert(target.parent);
      target.parent.children.splice(target.parent.children.indexOf(target), 1);
      target.parent = null;
    },
  });
  return { renderer, clicks, container: host("root") };
}
function button(target: Host): Host | undefined {
  if (target.type === "button") return target;
  return target.children.map(button).find(Boolean);
}
function shape(target: Host): any {
  return {
    type: target.type,
    text: target.text,
    props: Object.fromEntries(
      Object.entries(target.props).map(([key, value]) => [
        key,
        typeof value === "function" ? "callback" : value,
      ]),
    ),
    children: target.children.map(shape),
  };
}

export async function executeComponent(code: string, fixture: any) {
  // Load the complete component unchanged except its runtime import address.
  // No setup/context/state is supplied by this host or the module loader.
  const loaded = await import(dataUrl(runtimeModuleSource(code)));
  const component = loaded.default;
  assert.deepEqual(Object.keys(component), ["render"]);
  assert.equal(component.setup, undefined);
  assert.equal(typeof component.render, "function");
  assert.equal(component.render.length, 6);
  const { renderer, clicks, container } = rendererHost();
  const warnings: string[] = [];
  const errors: unknown[] = [];
  const app = renderer.createApp(component);
  app.config.warnHandler = (warning: string) => warnings.push(warning);
  app.config.errorHandler = (error: unknown) => errors.push(error);
  app.mount(container);
  assert.equal(app._instance.type.render, component.render);
  assert.equal(app._instance.type.setup, undefined);
  const outcomes: unknown[] = [];
  const shapes: unknown[] = [];
  let actualButton: Host | undefined, previous: any;
  try {
    for (let phase = 0; phase < fixture.events.length; phase++) {
      if (phase) {
        app._instance.proxy.$forceUpdate();
        await runtime.nextTick();
      }
      const target = button(container);
      assert(target);
      if (actualButton) assert.equal(target, actualButton);
      actualButton = target;
      const handler = target.props.onClick;
      assert.equal(typeof handler, "function");
      if (previous) assert.notEqual(handler, previous, "uncached Vue update replaces onClick");
      assert.equal(clicks.length, phase + 1);
      assert.equal(clicks[phase].target, target);
      assert.equal(clicks[phase].previous ?? undefined, previous);
      assert.equal(clicks[phase].value, handler);
      previous = handler;
      shapes.push(shape(container));
      const event = structuredClone(fixture.events[phase]);
      assert.equal(handler(event), undefined);
      assert.deepEqual(event, fixture.expected[phase]);
      outcomes.push(event);
    }
  } finally {
    app.unmount();
  }
  assert(actualButton);
  assert.equal(app._instance, null);
  assert.equal(button(container), undefined);
  assert.deepEqual(container.children, []);
  assert.deepEqual(warnings, []);
  assert.deepEqual(errors, []);
  return {
    shapes,
    outcomes,
    replacedCallbacks: clicks.length - 1,
    unmounted: true,
    warnings,
    errors,
  };
}
