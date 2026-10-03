import assert from "node:assert/strict";
import { createRequire } from "node:module";
import {
  compiler,
  fromVue,
  hash,
  runtime,
  runtimeModuleSource,
} from "./native-selected-sfc-dom-runtime.ts";

export { compiler, fromVue, hash };
const babel = createRequire(fromVue.resolve("@vue/compiler-sfc/package.json"))("@babel/parser");
const url = (code: string) => `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
export const decode = (raw: string) => raw.replaceAll("&quot;", '"');
export function bodyFacts(raw: string) {
  const decoded = decode(raw),
    prefix = "($event)=>{";
  const ast = babel.parseExpression(prefix + decoded + "}");
  const scopes: any[] = [],
    declarations: any[] = [],
    references: any[] = [];
  const walk = (node: any, parent: any, key: string, scope: number | null, kind: string | null) => {
    if (!node || typeof node !== "object") return;
    if (node.type === "BlockStatement") {
      const span =
        node === ast.body ? null : [node.start - prefix.length, node.end - prefix.length];
      scopes.push({ parent: scope, span });
      scope = scopes.length - 1;
    }
    if (node.type === "VariableDeclaration") kind = node.kind;
    if (node.type === "VariableDeclarator") {
      assert.equal(node.id.type, "Identifier");
      declarations.push({
        name: node.id.name,
        kind,
        scope,
        span: [node.id.start - prefix.length, node.id.end - prefix.length],
      });
    }
    if (
      node.type === "Identifier" &&
      node.start >= prefix.length &&
      !(parent?.type === "VariableDeclarator" && key === "id") &&
      !(parent?.type === "MemberExpression" && key === "property" && !parent.computed)
    ) {
      references.push({
        name: node.name,
        scope,
        span: [node.start - prefix.length, node.end - prefix.length],
      });
    }
    for (const [childKey, value] of Object.entries(node)) {
      if (["loc", "start", "end", "extra", "comments"].includes(childKey)) continue;
      if (Array.isArray(value)) for (const child of value) walk(child, node, childKey, scope, kind);
      else if (value && typeof value === "object") walk(value, node, childKey, scope, kind);
    }
  };
  walk(ast, null, "", null, null);
  let wrappedUnits = 0,
    word = false;
  for (const byte of Buffer.from("()=>{\n" + decoded + "\n}")) {
    const next =
      (byte >= 48 && byte <= 57) ||
      (byte >= 65 && byte <= 90) ||
      (byte >= 97 && byte <= 122) ||
      byte === 95 ||
      byte === 36;
    if ((next && !word) || (!next && ![9, 10, 11, 12, 13, 32].includes(byte))) wrappedUnits++;
    word = next;
  }
  return { decoded, utf16: decoded.length, wrappedUnits, scopes, declarations, references };
}

export function position(text: string, offset: number): number[] {
  assert(offset >= 0 && offset <= text.length);
  const lines = text.slice(0, offset).split(/\r\n|[\r\n\u2028\u2029]/);
  return [lines.length - 1, lines.at(-1)!.length];
}
export function mapAnchors(map: any, code: string, source: string) {
  const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  const generated = code.split(/\r\n|[\r\n\u2028\u2029]/),
    authored = source.split(/\r\n|[\r\n\u2028\u2029]/);
  const points: any[] = [];
  let sourceIndex = 0,
    sourceLine = 0,
    sourceColumn = 0,
    nameIndex = 0;
  map.mappings.split(";").forEach((line: string, index: number) => {
    let column = 0;
    for (const segment of line.split(",").filter(Boolean)) {
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
      assert(fields.length === 4 || fields.length === 5);
      column += fields[0];
      sourceIndex += fields[1];
      sourceLine += fields[2];
      sourceColumn += fields[3];
      assert.equal(sourceIndex, 0);
      assert(generated[index] !== undefined && authored[sourceLine] !== undefined);
      assert(column >= 0 && column <= generated[index].length);
      assert(sourceColumn >= 0 && sourceColumn <= authored[sourceLine].length);
      let name = null;
      if (fields.length === 5) {
        nameIndex += fields[4];
        name = map.names[nameIndex];
        assert(name);
      }
      points.push({ generated: [index, column], authored: [sourceLine, sourceColumn], name });
    }
  });
  assert(points.length > 0);
  return points;
}

type Host = {
  type: string;
  text: string;
  children: Host[];
  parent: Host | null;
  props: Record<string, any>;
};
const host = (type: string, text = ""): Host => ({
  type,
  text,
  children: [],
  parent: null,
  props: {},
});
function rendererHost() {
  const patches: { target: Host; previous: any; value: any }[] = [];
  const renderer = runtime.createRenderer({
    createElement: (type: string) => host(type),
    createText: (text: string) => host("text", text),
    createComment: (text: string) => host("comment", text),
    setText: (node: Host, text: string) => {
      node.text = text;
    },
    setElementText: (node: Host, text: string) => {
      node.text = text;
      node.children = [];
    },
    parentNode: (node: Host) => node.parent,
    nextSibling: (node: Host) =>
      node.parent?.children[node.parent.children.indexOf(node) + 1] ?? null,
    patchProp: (target: Host, key: string, previous: any, value: any) => {
      if (key === "onClick") patches.push({ target, previous, value });
      target.props[key] = value;
    },
    insert: (node: Host, parent: Host, anchor: Host | null) => {
      if (node.parent) node.parent.children.splice(node.parent.children.indexOf(node), 1);
      const index = anchor ? parent.children.indexOf(anchor) : -1;
      parent.children.splice(index < 0 ? parent.children.length : index, 0, node);
      node.parent = parent;
    },
    remove: (node: Host) => {
      assert(node.parent);
      node.parent.children.splice(node.parent.children.indexOf(node), 1);
      node.parent = null;
    },
  });
  return { renderer, patches, container: host("root") };
}
function buttons(node: Host): Host[] {
  return node.type === "button" ? [node] : node.children.flatMap(buttons);
}
function shape(node: Host): any {
  return {
    type: node.type,
    text: node.text,
    props: Object.fromEntries(
      Object.entries(node.props).map(([key, value]) => [
        key,
        typeof value === "function" ? "callback" : value,
      ]),
    ),
    children: node.children.map(shape),
  };
}

export async function executeLocalComponent(code: string, fixture: any) {
  const module = await import(url(runtimeModuleSource(code)));
  const component = module.default;
  assert.deepEqual(Object.keys(component), ["render"]);
  assert.equal(component.setup, undefined);
  const { renderer, container, patches } = rendererHost();
  const warnings: string[] = [],
    errors: unknown[] = [];
  const app = renderer.createApp(component);
  app.config.warnHandler = (warning: string) => warnings.push(warning);
  app.config.errorHandler = (error: unknown) => errors.push(error);
  const outcomes: unknown[] = [],
    shapes: unknown[] = [];
  let targets: Host[] = [],
    previous: any[] = [];
  app.mount(container);
  try {
    assert.equal(app._instance.type.render, component.render);
    assert.equal(app._instance.type.setup, undefined);
    for (let phase = 0; phase < fixture.events.length; phase++) {
      if (phase) {
        app._instance.proxy.$forceUpdate();
        await runtime.nextTick();
      }
      const found = buttons(container);
      assert.equal(found.length, fixture.bodies.length);
      assert.equal(new Set(found).size, found.length, "each original handler has its own element");
      const handlers = found.map((target) => target.props.onClick);
      assert.equal(
        new Set(handlers).size,
        handlers.length,
        "each original body creates its own callback",
      );
      assert.equal(patches.length, found.length * (phase + 1));
      const effects = [];
      for (let index = 0; index < found.length; index++) {
        const handler = handlers[index];
        assert.equal(typeof handler, "function");
        if (phase) {
          assert.equal(found[index], targets[index]);
          assert.notEqual(
            handler,
            previous[index],
            "real uncached PROPS update replaces each callback",
          );
        }
        const patch = patches.filter((entry) => entry.target === found[index]).at(-1)!;
        assert.equal(patch.previous ?? undefined, previous[index]);
        assert.equal(patch.value, handler);
        const event = structuredClone(fixture.events[phase][index]);
        let thrown: { name: string } | null = null;
        try {
          assert.equal(handler(event), undefined);
        } catch (error) {
          assert(
            error instanceof ReferenceError,
            "only the explicit authored TDZ error is expected",
          );
          assert.match(error.message, /before initialization/);
          thrown = { name: error.name };
        }
        const undefinedKeys = Object.keys(event).filter((key) => event[key] === undefined);
        const result = {
          event: Object.fromEntries(
            Object.entries(event).filter(([, value]) => value !== undefined),
          ),
          undefinedKeys,
          thrown,
        };
        assert.deepEqual(result, fixture.expected[phase][index]);
        for (const key of undefinedKeys) assert(Object.hasOwn(event, key));
        effects.push(result);
      }
      shapes.push(shape(container));
      outcomes.push(effects);
      targets = found;
      previous = handlers;
    }
  } finally {
    app.unmount();
  }
  assert.equal(app._instance, null);
  assert.deepEqual(container.children, []);
  assert.deepEqual(warnings, []);
  assert.deepEqual(errors, []);
  return {
    outcomes,
    shapes,
    replacedCallbacks: patches.length - fixture.bodies.length,
    unmounted: true,
    warnings,
    errors,
  };
}
