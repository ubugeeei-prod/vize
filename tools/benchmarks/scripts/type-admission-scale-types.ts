import { typeProofs } from "../../../tests/tooling/support/type-admission-fixtures.ts";

export type ScaleShape = "chain" | "diamond" | "scc" | "crosspkg";
export type ScaleStage = "valid" | "invalid" | "poisoned";
export type ScaleOptions = {
  sfcs: number;
  sfcBytes: number;
  graphNodes: number;
  shape: ScaleShape;
  typeWidth: number;
  typeDepth: number;
  stage: ScaleStage;
  allowLarge?: boolean;
};
export type ScaleProbe = {
  file: string;
  before: string;
  after: string;
  token: string;
  code: number;
};

export function scaleTypes(width: number, depth: number): string {
  const variant = (index: number) =>
    `{ tag: 'v${index}'; id: N; tuple: [N, ${index}]; flags: { active: true; code: ${index} } }`;
  const expected = (index: number) =>
    `v${index}: { readonly tag: 'v${index}'; readonly id: N; readonly tuple: [N, ${index}]; readonly flags: { active: true; code: ${index} } };`;
  return `${typeProofs}
export type Variants<N extends number> = ${Array.from({ length: width }, (_, i) => variant(i)).join(" | ")};
type Buckets<N extends number> = {
  [V in Variants<N> as V['tag']]: { readonly [K in keyof V]: V[K] }
};
type Layers<T, D extends unknown[] = []> = D['length'] extends ${depth}
  ? T : { payload: Layers<T, [...D, unknown]>; level: D['length'] };
type Unwrap<T> = T extends { payload: infer P } ? Unwrap<P> : T;
export type Complexity<N extends number> = Unwrap<Layers<Buckets<N>>>;
export type Expected<N extends number> = {
  ${Array.from({ length: width }, (_, i) => expected(i)).join("\n  ")}
};
export declare function makeRow<const N extends number>(id: N): {
  id: N; label: \`row-\${N}\`; score: number; proof: Complexity<N>;
};
`;
}

export function validateScale(options: ScaleOptions): void {
  for (const [name, value, min, max] of [
    ["sfcs", options.sfcs, 1, 50_000],
    ["sfcBytes", options.sfcBytes, 2_048, 10 * 1024 * 1024],
    ["graphNodes", options.graphNodes, 0, 10_000],
    ["typeWidth", options.typeWidth, 1, 128],
    ["typeDepth", options.typeDepth, 0, 128],
  ] as const) {
    if (!Number.isSafeInteger(value) || value < min || value > max)
      throw new Error(`${name} must be an integer in ${min}..${max}`);
  }
  if (!["chain", "diamond", "scc", "crosspkg"].includes(options.shape))
    throw new Error("unknown dependency shape");
  if (!["valid", "invalid", "poisoned"].includes(options.stage))
    throw new Error("unknown admission stage");
  if (options.sfcs * options.sfcBytes > 256 * 1024 * 1024 && !options.allowLarge)
    throw new Error("over 256 MiB of original SFC source requires --allow-large");
}
