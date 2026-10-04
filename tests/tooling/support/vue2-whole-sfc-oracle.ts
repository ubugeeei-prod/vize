// Independent whole-SFC primary evidence over unchanged pinned package bodies.
import assert from "node:assert/strict";
import { compiler, plain } from "./vue2-pinned-oracle.ts";
import { loadRuntime } from "./vue2-pinned-runtime.ts";

type Template = { content: string; start: number; end: number };
type Parsed = {
  template: Template | null;
  script: unknown;
  styles: unknown[];
  customBlocks: unknown[];
  errors?: unknown[];
};
const sfcCompiler = compiler as typeof compiler & {
  parseComponent(source: string, options: object): Parsed;
};
export const frame = (source: string) => {
  const original = sfcCompiler.parseComponent(source, { deindent: false, pad: false });
  assert.ok(original.template, "whole authored SFC has a real primary template");
  const { start, end, content } = original.template;
  assert.equal(source.slice(start, end), content, "primary indices are original UTF16");
  const utf8 = {
    start: Buffer.byteLength(source.slice(0, start)),
    end: Buffer.byteLength(source.slice(0, end)),
  };
  assert.equal(Buffer.from(source).subarray(utf8.start, utf8.end).toString(), content);
  return { original: plain(original), utf8, byteLength: Buffer.byteLength(source) };
};
export const compileWhole = (source: string) => {
  const selected = sfcCompiler.parseComponent(source, { deindent: false, pad: false });
  assert.ok(selected.template);
  const actual = compiler.compile(selected.template.content) as ReturnType<
    typeof compiler.compile
  > & {
    staticRenderFns: string[];
    tips: unknown[];
  };
  return {
    render: actual.render,
    staticRenderFns: plain(actual.staticRenderFns),
    errors: plain(actual.errors),
    tips: plain(actual.tips),
  };
};
type Node = {
  tag?: string;
  data?: unknown;
  text?: string;
  key?: unknown;
  ns?: string;
  raw: boolean;
  isStatic: boolean;
  isRootInsert: boolean;
  isComment: boolean;
  isCloned: boolean;
  isOnce: boolean;
  children?: Node[];
};
type Snapshot = {
  tag: string | null;
  data: unknown;
  text: string | null;
  key: unknown;
  ns: string | null;
  raw: boolean;
  isStatic: boolean;
  isRootInsert: boolean;
  isComment: boolean;
  isCloned: boolean;
  isOnce: boolean;
  children: Snapshot[] | null;
};
const snapshot = (node: Node): Snapshot => ({
  tag: node.tag ?? null,
  data: plain(node.data),
  text: node.text ?? null,
  key: plain(node.key),
  ns: node.ns ?? null,
  raw: node.raw,
  isStatic: node.isStatic,
  isRootInsert: node.isRootInsert,
  isComment: node.isComment,
  isCloned: node.isCloned,
  isOnce: node.isOnce,
  children: node.children ? Array.from(node.children, snapshot) : null,
});
export const semantic = (node: Snapshot): unknown => ({
  tag: node.tag,
  data: node.data,
  text: node.text,
  children: node.children ? node.children.map(semantic) : null,
});
export const executeWhole = (source: string, environment: "test" | "production") => {
  const selected = sfcCompiler.parseComponent(source, { deindent: false, pad: false });
  assert.ok(selected.template);
  const calls: Array<{ name: string; args: unknown[] }> = [];
  const events: string[] = [];
  const implementations: Record<string, (...args: unknown[]) => unknown> = {
    add: (value, amount) => Number(value) + Number(amount),
    upper: (value) => `U:${String(value)}`,
    "upper ": (value) => `SP:${String(value)}`,
  };
  const Vue = loadRuntime(environment);
  const component = new Vue({
    data: () => ({ a: 6, b: 3, 日本: 4 }),
    filters: implementations,
    ...compiler.compileToFunctions(selected.template.content),
  }) as InstanceType<typeof Vue> & { $options: { filters: Record<string, unknown> } };
  for (const [name, implementation] of Object.entries(implementations)) {
    Object.defineProperty(component.$options.filters, name, {
      configurable: true,
      get() {
        events.push(`lookup:${name}`);
        return (...args: unknown[]) => {
          calls.push({ name, args: plain(args) as unknown[] });
          events.push(`call:${name}`);
          return implementation(...args);
        };
      },
    });
  }
  for (const [name, value] of Object.entries({ a: 6, b: 3, 日本: 4 })) {
    Object.defineProperty(component, name, {
      configurable: true,
      get() {
        events.push(`read:${name}`);
        return value;
      },
    });
  }
  return { vnode: snapshot(component._render() as Node), calls, events };
};
