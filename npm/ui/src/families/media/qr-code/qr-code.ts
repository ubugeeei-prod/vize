/** Dependency-free QR Code encoder and headless SVG renderer. */
export { default as QrCode } from "./qr-code.vue";
export {
  QrCodeEncodeError,
  encodeQrCode,
  isQrCodeModuleDark,
  qrCodeCapacity,
} from "./qr-code-encoder.ts";
export { qrCodeToSvgPath } from "./qr-code-svg.ts";
export type {
  QrCodeEncodeErrorCode,
  QrCodeEncodeErrorLike,
  QrCodeEncodeOptions,
  QrCodeErrorCorrection,
  QrCodeExpose,
  QrCodeMask,
  QrCodeMatrix,
  QrCodeMode,
  QrCodeSlotState,
  QrCodeState,
  QrCodeSvgPathOptions,
  QrCodeValue,
  QrCodeVersion,
} from "./qr-code-types.ts";
