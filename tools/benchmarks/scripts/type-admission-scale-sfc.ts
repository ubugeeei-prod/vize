import type { ScaleOptions, ScaleProbe } from "./type-admission-scale-types.ts";

export function scaleSfc(index: number, options: ScaleOptions) {
  const name = String(index).padStart(5, "0");
  const file = `src/Scale${name}.vue`;
  const eol = index % 2 === 0 ? "\r\n" : "\n";
  const header = `<script setup lang="ts">
import { useTemplateRef } from 'vue';
import { makeRow } from '../types';
import type { Assert, Exact, IsAny, IsUnknown, IsNever, Complexity, Expected } from '../types';
import type { GraphProof } from '../packages/graph';
const root = useTemplateRef<HTMLDivElement>('root');
type RefWitness = Assert<Exact<typeof root.value, HTMLDivElement | null>>;
type GraphWitness = Assert<GraphProof>;
type TypeWitness = Assert<Exact<Complexity<${index}>, Expected<${index}>>>;
type AnyWitness = Assert<Exact<IsAny<Complexity<${index}>>, false>>;
type UnknownWitness = Assert<Exact<IsUnknown<Complexity<${index}>>, false>>;
type NeverWitness = Assert<Exact<IsNever<Complexity<${index}>>, false>>;
`;
  const middle = '</script>\n<template>\n<div ref="root">\n';
  const footer = "</div>\n</template>\n";
  const script: string[] = [];
  const template: string[] = [];
  let bytes = Buffer.byteLength((header + middle + footer).replaceAll("\n", eol));
  let rows = 0;
  // Poisoned exact-type proofs are shorter than the valid expressions. Reserve
  // their maximum shrinkage so every stage still exercises the requested size.
  const mutationReserve = options.stage === "poisoned" ? 3 * 256 : 0;
  // Count actual authored expressions/declarations, never comment padding.
  while (bytes < options.sfcBytes + mutationReserve || rows < 3) {
    const row = String(rows).padStart(6, "0");
    const declaration = `const row${row} = makeRow(${rows});
type Proof${row} = Assert<Exact<typeof row${row}.proof, Expected<${rows}>>>;
`;
    const use = `<div :data-row="row${row}.id">🌱日本語 {{ row${row}.score.toFixed() }}</div>\n`;
    script.push(declaration);
    template.push(use);
    bytes += Buffer.byteLength((declaration + use).replaceAll("\n", eol));
    rows++;
  }
  const source = (header + script.join("") + middle + template.join("") + footer).replaceAll(
    "\n",
    eol,
  );
  const probes: ScaleProbe[] = [];
  for (const position of [0, Math.floor(rows / 2), rows - 1]) {
    const row = String(position).padStart(6, "0");
    const token = `missingF${name}R${row}`;
    probes.push(
      options.stage === "poisoned"
        ? {
            file,
            before: `type Proof${row} = Assert<Exact<typeof row${row}.proof, Expected<${position}>>>;`,
            after: `type Proof${row} = Assert<false>;`,
            token: "false",
            code: 2344,
          }
        : {
            file,
            before: `row${row}.score.toFixed()`,
            after: `row${row}.score.${token}()`,
            token,
            code: 2339,
          },
    );
  }
  return { file, source, probes, rows, lineEndings: eol === "\r\n" ? "CRLF" : "LF" };
}
