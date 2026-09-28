import { computed, type Ref, type ShallowRef } from "vue";
import type { SfcCompileResult } from "../../wasm/index";
import type { ProductCaptureNegotiation, ProductTarget } from "../../wasm/types/productCapture";
import type { CodeOutputs } from "../atelier/codeOutputs";
import { utf8ToUtf16Offset } from "./offsets";
import { buildProductLadder } from "./productLadder";

export interface ProductCapturedResult {
  result: SfcCompileResult;
  negotiation: ProductCaptureNegotiation;
}

/** Selected target's verified capture, status, and authored-source mapping. */
export function useProductCaptureState(
  captures: ShallowRef<Partial<Record<ProductTarget, ProductCapturedResult>>>,
  compiledSource: Ref<string | null>,
  source: Ref<string>,
  outputs: ShallowRef<CodeOutputs>,
  outputTarget: Ref<ProductTarget>,
) {
  const capture = computed(() => captures.value[outputTarget.value] ?? null);
  const feed = computed(() => {
    const negotiated = capture.value?.negotiation;
    return negotiated?.ok ? negotiated.feed : null;
  });
  const captureError = computed(() => {
    const negotiated = capture.value?.negotiation;
    if (negotiated && !negotiated.ok) return negotiated.error;
    if (!negotiated && compiledSource.value !== null) {
      return (
        outputs.value[outputTarget.value].error ??
        `${outputTarget.value} compile returned no result.`
      );
    }
    return null;
  });
  const captureStatus = computed(() => {
    const current = feed.value;
    if (!current) return null;
    const { kind, reason } = current.outcome;
    return `${current.target.toUpperCase()} ${kind}${reason ? `: ${reason}` : ""}`;
  });
  const captureLabel = computed(() => {
    const current = feed.value;
    return current ? `${current.target.toUpperCase()} ${current.outcome.kind}` : null;
  });
  const remarksNote = computed(() =>
    feed.value?.outcome.kind === "accepted" && !feed.value.observed.remarks
      ? "Optimization remarks were not observed in this product run."
      : null,
  );
  const profileNote = computed(() =>
    feed.value?.outcome.kind === "accepted"
      ? feed.value.observed.timings
        ? "Flame profile export is unavailable for this product run."
        : "Step timings and flame profile were not observed in this product run."
      : null,
  );
  const ladder = computed(() => {
    const current = capture.value;
    if (!current?.negotiation.ok || current.negotiation.feed.outcome.kind !== "accepted") {
      return null;
    }
    return buildProductLadder(current.negotiation.feed, current.result);
  });
  const templateStart = computed(() => {
    const current = feed.value;
    const block = capture.value?.result.descriptor.template;
    const authored = block?.content;
    const span = current?.source.template_span;
    const compiled = compiledSource.value;
    if (
      !current ||
      !span ||
      !authored ||
      !block ||
      !compiled ||
      source.value !== compiled ||
      current.source.authored_syntax !== current.source.compiled_syntax ||
      span.start !== block.loc.start ||
      span.end !== block.loc.end
    ) {
      return null;
    }
    const start = utf8ToUtf16Offset(compiled, span.start);
    const end = utf8ToUtf16Offset(compiled, span.end);
    return compiled.slice(start, end) === authored ? start : null;
  });
  const syntaxNote = computed(() => {
    const current = feed.value;
    if (!current || current.outcome.kind !== "accepted" || templateStart.value !== null)
      return null;
    if (current.source.authored_syntax !== current.source.compiled_syntax) {
      return `${current.source.authored_syntax} source compiled as ${current.source.compiled_syntax}; source highlighting is unavailable.`;
    }
    return "The authored template span could not be verified; source highlighting is unavailable.";
  });

  return {
    captureError,
    captureStatus,
    captureLabel,
    remarksNote,
    profileNote,
    ladder,
    templateStart,
    syntaxNote,
  };
}
