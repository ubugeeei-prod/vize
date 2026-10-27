import type { ScaleOptions, ScaleProbe } from "./type-admission-scale-types.ts";

const id = (index: number) => String(index).padStart(5, "0");

function parents(index: number, options: ScaleOptions): number[] {
  const count = options.graphNodes;
  if (options.shape === "scc") return [(index + 1) % count];
  if (options.shape === "chain") return index + 1 < count ? [index + 1] : [];
  if (options.shape === "crosspkg") return [index + 1, index + 2].filter((n) => n < count);
  const base = index - (index % 4);
  const next = index % 4 === 0 ? [base + 1, base + 2] : index % 4 === 3 ? [base + 4] : [base + 3];
  return next.filter((n) => n < count);
}

export function scaleGraph(options: ScaleOptions) {
  const files: Record<string, string> = {};
  const probes: ScaleProbe[] = [];
  const imports: string[] = [];
  const registry: string[] = [];
  const kinds: string[] = [];
  for (let index = 0; index < options.graphNodes; index++) {
    const name = id(index);
    const deps = parents(index, options);
    const file = `packages/node-${name}/index.ts`;
    const imported = deps.map((parent) => {
      const target =
        options.shape === "crosspkg"
          ? `@admission/node-${id(parent)}`
          : `../node-${id(parent)}/barrel`;
      return `import type { Node${id(parent)} } from '${target}';`;
    });
    files[file] = `import type { Assert, Exact } from '../../types';
${imported.join("\n")}
export interface Node${name} {
  readonly kind: 'node-${name}';
  readonly payload: { id: ${index}; label: string };
  readonly dependencies: readonly [${deps.map((n) => `Node${id(n)}`).join(", ")}];
}
export type PayloadWitness = Assert<Exact<Node${name}['payload']['id'], ${index}>>;
export type ProofLiveness = Assert<true>;
export const graphWitness: number = ${index};
`;
    files[`packages/node-${name}/barrel.ts`] = `export type { Node${name} } from './index';\n`;
    if (options.shape === "crosspkg")
      files[`packages/node-${name}/package.json`] = JSON.stringify({
        name: `@admission/node-${name}`,
        private: true,
        types: "barrel.ts",
      });
    probes.push({
      file,
      before: `export const graphWitness: number = ${index};`,
      after: `export const graphWitness: number = 'invalid-node-${name}';`,
      token: "graphWitness",
      code: 2322,
    });
    imports.push(`import type { Node${name} } from './node-${name}/barrel';`);
    registry.push(`  node${name}: Node${name};`);
    kinds.push(`'node-${name}'`);
  }
  files["packages/graph.ts"] = `${imports.join("\n")}
import type { Assert, Exact } from '../types';
export interface Registry {
${registry.join("\n")}
}
type NodeShape = { kind: string; dependencies: readonly unknown[] };
type Walk<T, D extends unknown[] = []> = T extends NodeShape
  ? T['kind'] | (D['length'] extends 4 ? never : Walk<T['dependencies'][number], [...D, unknown]>)
  : never;
type Seen = { [K in keyof Registry]: Walk<Registry[K]> }[keyof Registry];
type Expected = ${kinds.join(" | ") || "never"};
export type GraphProof = Exact<Seen, Expected>;
type GraphWitness = Assert<GraphProof>;
export declare const graph: Registry;
`;
  return { files, probes };
}
