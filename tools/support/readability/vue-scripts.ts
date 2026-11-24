import { createRequire } from "node:module";

const fromTests = createRequire(new URL("../../../tests/package.json", import.meta.url));
interface ScriptBlock {
  readonly lang?: string;
  readonly content: string;
  readonly loc: { start: { offset: number }; end: { offset: number } };
}

// Use the workspace's pinned Vue SFC parser for authored block boundaries.
export function vueScripts(source: string) {
  const { parse } = fromTests("@vue/compiler-sfc") as {
    parse(
      source: string,
      options: { ignoreEmpty: boolean },
    ): {
      descriptor: { script: ScriptBlock | null; scriptSetup: ScriptBlock | null };
      errors: readonly unknown[];
    };
  };
  const { descriptor, errors } = parse(source, { ignoreEmpty: false });
  const blocks = [descriptor.script, descriptor.scriptSetup]
    .filter((block): block is ScriptBlock => block !== null)
    .sort((a, b) => a.loc.start.offset - b.loc.start.offset);
  return { blocks, errors };
}
