/** Headless signature pad: pointer-captured, pressure-aware strokes rendered as SVG. */
export { default as SignaturePadCanvas } from "./signature-pad-canvas.vue";
export { default as SignaturePadClear } from "./signature-pad-clear.vue";
export { default as SignaturePadGuide } from "./signature-pad-guide.vue";
export { default as SignaturePadRedo } from "./signature-pad-redo.vue";
export { default as SignaturePad, default as SignaturePadRoot } from "./signature-pad-root.vue";
export { default as SignaturePadUndo } from "./signature-pad-undo.vue";
export {
  SIGNATURE_STROKE_DEFAULTS,
  SignaturePadError,
  isSignatureEmpty,
  parseSignature,
  serializeSignature,
  signatureToDataUrl,
  signatureToSvg,
  simulatePressure,
  strokeOutlinePath,
} from "./signature-pad-path.ts";
export type {
  SignatureDataUrlOptions,
  SignatureImageType,
  SignaturePadButtonExpose,
  SignaturePadCanvasExpose,
  SignaturePadChangeReason,
  SignaturePadErrorCode,
  SignaturePadGuideExpose,
  SignaturePadPressureMode,
  SignaturePadRootExpose,
  SignaturePadSlotState,
  SignaturePadState,
  SignaturePadValueFormat,
  SignaturePoint,
  SignatureStroke,
  SignatureStrokeOptions,
  SignatureSvgOptions,
  SignatureValue,
} from "./signature-pad-types.ts";
