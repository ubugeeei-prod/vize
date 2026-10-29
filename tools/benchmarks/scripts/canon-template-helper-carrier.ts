/** Original component-helper carrier layered over the unchanged admission generator. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import type { AdmissionAPI, ScaleOptions, ScaleProbe } from "./canon-helper-admission-api.ts";

const digest = (source: string): string => createHash("sha256").update(source).digest("hex");
const component = `import type { VNodeProps, AllowedComponentProps, ComponentCustomProps } from 'vue';
type Assert<T extends true> = T;
type Props = Readonly<{ value: number; label?: string; render?: (value: number) => string; onChange?: (value: number) => void }> & VNodeProps & AllowedComponentProps & ComponentCustomProps;
export declare const WitnessChild: {
  new(): {
    readonly $props: Props;
    readonly $slots: { default: (props: { item: number }) => unknown };
    focus(): void;
  };
};
export const componentValue: number = 1;
type ComponentProof = Assert<true>;
export default WitnessChild;
`;

function byFile<T extends { file: string }>(rows: T[]): Map<string, T[]> {
  const result = new Map<string, T[]>();
  for (const row of rows) {
    const entries = result.get(row.file) ?? [];
    entries.push(row);
    result.set(row.file, entries);
  }
  return result;
}

function at(source: string, line: number, column: number): number {
  const starts = [0];
  for (const match of source.matchAll(/\r\n|[\r\n\u2028\u2029]/gu))
    starts.push(match.index + match[0].length);
  const start = starts[line - 1];
  assert.ok(start !== undefined && column > 0);
  const end = starts[line] ?? source.length;
  assert.ok(start + column - 1 <= end);
  return start + column - 1;
}

function script(index: number, hasParent: boolean): string {
  return `import WitnessChild from '../component';
${hasParent ? "import DependencyChild from './Scale" + String(index + 1).padStart(5, "0") + ".vue';" : ""}
const componentProps = defineProps<{ value?: number; label?: string }>();
const componentEmit = defineEmits<{ change: [value: number] }>();
defineSlots<{ default: (props: { item: number }) => unknown }>();
function focus(): void {}
defineExpose({ focus });
const child = useTemplateRef<InstanceType<typeof WitnessChild>>('child');
type ChildExact = Assert<Exact<typeof child.value, InstanceType<typeof WitnessChild> | null>>;
type ChildNoAny = Assert<Exact<IsAny<InstanceType<typeof WitnessChild>>, false>>;
type ChildNoUnknown = Assert<Exact<IsUnknown<InstanceType<typeof WitnessChild>>, false>>;
type ChildNoNever = Assert<Exact<IsNever<InstanceType<typeof WitnessChild>>, false>>;
type FocusExact = Assert<Exact<InstanceType<typeof WitnessChild>['focus'], () => void>>;
function componentOnChange(value: number): void { componentEmit('change', value); }
child.value?.focus();
void componentProps;
type ComponentStartProof = Assert<true>;
type ComponentMiddleProof = Assert<true>;
type ComponentEndProof = Assert<true>;
`;
}

function template(hasParent: boolean): string {
  return `<WitnessChild ref="child" :value="row000000.score" :label="row000000.label" :render="(rendered: number) => rendered.toFixed()" @change="componentOnChange">
<template #default="{ item }"><span>{{ item.toFixed() }}</span></template>
</WitnessChild>
${hasParent ? '<DependencyChild :value="row000000.score" @change="componentOnChange" />' : ""}
`;
}

/** The original generator's oracle is relocated, never replaced by observed output. */
export function prepareHelperArchitectureCarrier(
  project: string,
  provider: string,
  options: ScaleOptions,
  api: AdmissionAPI,
) {
  const { prepareAdmissionScale, scaleSfc, scaleGraph, mutatedAdmission } = api;
  const corpus = prepareAdmissionScale(project, provider, options);
  const originalManifest = corpus.manifest.map((entry) => ({ ...entry }));
  const originalExpected = corpus.expected.map((entry) => ({ ...entry }));
  const originalByFile = byFile(originalExpected);
  const expectedByFile = byFile(corpus.expected);
  const manifestByFile = new Map(corpus.manifest.map((entry) => [entry.file, entry]));
  const sfcsByFile = new Map(corpus.sfcs.map((entry) => [entry.file, entry]));
  const mutationPlan: ScaleProbe[] = [];
  const repairedManifest: { file: string; bytes: number; sha256: string }[] = [];
  const preserveRepair = (file: string, source: string, expected: string, probes: ScaleProbe[]) => {
    let repaired = source;
    for (const probe of [...probes].reverse()) {
      assert.equal(repaired.split(probe.after).length, 2, "repair anchors must be unique");
      repaired = repaired.replace(probe.after, probe.before);
    }
    assert.equal(repaired, expected, "repair must restore the independent generator preimage");
    repairedManifest.push({ file, bytes: Buffer.byteLength(repaired), sha256: digest(repaired) });
    mutationPlan.push(...probes);
  };
  const additions: { file: string; scriptLines: number; originalSha256: string; sha256: string }[] =
    [];
  for (let index = 0; index < options.sfcs; index++) {
    const name = String(index).padStart(5, "0");
    const file = "src/Scale" + name + ".vue";
    const absolute = path.join(project, file);
    const original = fs.readFileSync(absolute, "utf8");
    const generated = scaleSfc(index, options);
    assert.equal(generated.file, file);
    const eol = original.includes("\r\n") ? "\r\n" : "\n";
    const hasParent = index + 1 < options.sfcs && index % 20 !== 19;
    const insertScript = script(index, hasParent).replaceAll("\n", eol);
    const insertTemplate = template(hasParent).replaceAll("\n", eol);
    const scriptAt = original.indexOf("</script>");
    const templateAt = original.lastIndexOf("</div>");
    assert.ok(scriptAt > 0 && templateAt > scriptAt);
    assert.ok(scriptAt === 0 || original[scriptAt - 1] === "\n");
    const scriptLines = insertScript.split(eol).length - 1;
    for (const expected of expectedByFile.get(file) ?? []) {
      const offset = at(original, expected.line, expected.column);
      assert.ok(offset < templateAt, "all original owned probes precede the added template");
      if (offset >= scriptAt) expected.line += scriptLines;
    }
    let source =
      original.slice(0, scriptAt) +
      insertScript +
      original.slice(scriptAt, templateAt) +
      insertTemplate +
      original.slice(templateAt);
    const probes: ScaleProbe[] =
      options.stage === "poisoned"
        ? ["Start", "Middle", "End"].map((part) => ({
            file,
            before: "type Component" + part + "Proof = Assert<true>;",
            after: "type Component" + part + "Proof = Assert<false>;",
            token: "false",
            code: 2344,
          }))
        : [
            {
              file,
              before: ':value="row000000.score" :label=',
              after: ':value="row000000.score.missingPropF' + name + '" :label=',
              token: "missingPropF" + name,
              code: 2339,
            },
            {
              file,
              before: "item.toFixed()",
              after: "item.missingSlotF" + name + "()",
              token: "missingSlotF" + name,
              code: 2339,
            },
            {
              file,
              before: "child.value?.focus();",
              after: "child.value?.missingRefF" + name + "();",
              token: "missingRefF" + name,
              code: 2339,
            },
          ];
    if (options.stage !== "valid") {
      for (const probe of probes) {
        const mutation = mutatedAdmission(source, { ...probe, name: file });
        source = mutation.source;
        corpus.expected.push({
          file,
          line: mutation.line,
          column: mutation.column,
          code: mutation.code,
        });
      }
    }
    for (const expected of originalByFile.get(file) ?? []) {
      const previous = at(original, expected.line, expected.column);
      const relocated = (expectedByFile.get(file) ?? []).find(
        (row) =>
          row.file === file &&
          row.code === expected.code &&
          row.column === expected.column &&
          row.line === expected.line + (previous >= scriptAt ? scriptLines : 0),
      );
      assert.ok(relocated, "the original oracle identity must survive composition");
      const current = at(source, relocated.line, relocated.column);
      const token = /^[A-Za-z_$][\w$]*/u.exec(original.slice(previous));
      assert.ok(token);
      assert.ok(
        source.slice(current).startsWith(token[0]),
        "the relocated original token is identical",
      );
    }
    const validScriptAt = generated.source.indexOf("</script>");
    const validTemplateAt = generated.source.lastIndexOf("</div>");
    const expectedRepair =
      generated.source.slice(0, validScriptAt) +
      insertScript +
      generated.source.slice(validScriptAt, validTemplateAt) +
      insertTemplate +
      generated.source.slice(validTemplateAt);
    preserveRepair(
      file,
      source,
      expectedRepair,
      options.stage === "valid" ? [] : [...generated.probes, ...probes],
    );
    fs.writeFileSync(absolute, source);
    const entry = manifestByFile.get(file);
    assert.ok(entry);
    entry.bytes = Buffer.byteLength(source);
    entry.sha256 = digest(source);
    entry.lines = source.split("\n").length;
    const sfc = sfcsByFile.get(file);
    assert.ok(sfc);
    sfc.bytes = entry.bytes;
    additions.push({ file, scriptLines, originalSha256: digest(original), sha256: entry.sha256 });
  }
  if (options.stage !== "valid") {
    const graph = scaleGraph(options);
    for (const originalProbe of graph.probes) {
      const probe =
        options.stage === "poisoned"
          ? {
              ...originalProbe,
              before: "type ProofLiveness = Assert<true>;",
              after: "type ProofLiveness = Assert<false>;",
              token: "false",
              code: 2344,
            }
          : originalProbe;
      const expected = graph.files[probe.file];
      assert.ok(expected);
      preserveRepair(
        probe.file,
        fs.readFileSync(path.join(project, probe.file), "utf8"),
        expected,
        [probe],
      );
    }
  }
  let componentSource = component;
  const componentProbes: ScaleProbe[] = [];
  if (options.stage !== "valid") {
    const probe =
      options.stage === "poisoned"
        ? {
            before: "type ComponentProof = Assert<true>;",
            after: "type ComponentProof = Assert<false>;",
            token: "false",
            code: 2344,
          }
        : {
            before: "export const componentValue: number = 1;",
            after: "export const componentValue: number = 'invalid';",
            token: "componentValue",
            code: 2322,
          };
    componentProbes.push({ ...probe, file: "component.ts" });
    const mutation = mutatedAdmission(componentSource, { ...probe, name: "component.ts" });
    componentSource = mutation.source;
    corpus.expected.push({
      file: "component.ts",
      line: mutation.line,
      column: mutation.column,
      code: mutation.code,
    });
  }
  preserveRepair("component.ts", componentSource, component, componentProbes);
  fs.writeFileSync(path.join(project, "component.ts"), componentSource);
  corpus.manifest.push({
    file: "component.ts",
    bytes: Buffer.byteLength(componentSource),
    sha256: digest(componentSource),
    lines: componentSource.split("\n").length,
  });
  assert.equal(corpus.sfcs.length, options.sfcs);
  assert.equal(
    corpus.expected.length,
    options.stage === "valid" ? 0 : 6 * options.sfcs + options.graphNodes + 1,
  );
  const composed = {
    ...corpus,
    originalManifest,
    originalExpected,
    additions,
    mutationPlan,
    repairedManifest,
    componentCarrier:
      "authored typed external constructor plus 20-component SFC chains; props/emits/slots/exposed and typed refs",
    sourceBytes: corpus.manifest.reduce((sum, entry) => sum + entry.bytes, 0),
    oracle:
      "original 3 probes per SFC relocated by exact inserted lines plus 3 independent component probes; graph unchanged; component TS also mutated",
  };
  fs.writeFileSync(path.join(project, "corpus.json"), JSON.stringify(composed, null, 2));
  return composed;
}

/** Revert only recorded authored mutations; preserve poison-stage row counts and all source bytes. */
export function repairHelperArchitectureCarrier(
  project: string,
  corpus: ReturnType<typeof prepareHelperArchitectureCarrier>,
) {
  for (const [file, probes] of byFile(corpus.mutationPlan)) {
    const absolute = path.join(project, file);
    let source = fs.readFileSync(absolute, "utf8");
    for (const probe of [...probes].reverse()) {
      assert.equal(source.split(probe.after).length, 2);
      source = source.replace(probe.after, probe.before);
    }
    fs.writeFileSync(absolute, source);
  }
  for (const entry of corpus.repairedManifest) {
    const bytes = fs.readFileSync(path.join(project, entry.file));
    assert.equal(bytes.length, entry.bytes);
    assert.equal(digest(bytes.toString("utf8")), entry.sha256);
  }
  const receipt = { stage: "repaired", expected: [], repairedManifest: corpus.repairedManifest };
  fs.writeFileSync(path.join(project, "repair-receipt.json"), JSON.stringify(receipt, null, 2));
  return receipt;
}
