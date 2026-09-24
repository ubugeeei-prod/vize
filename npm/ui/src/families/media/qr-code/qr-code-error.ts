import type { QrCodeEncodeErrorCode } from "./qr-code-types.ts";

/** Typed encoder diagnostic. `message` starts with the stable {@link code}. */
export class QrCodeEncodeError extends Error {
  /** Stable machine-readable diagnostic code. */
  readonly code: QrCodeEncodeErrorCode;

  constructor(code: QrCodeEncodeErrorCode, detail: string) {
    super(`${code}: ${detail}`);
    this.name = "QrCodeEncodeError";
    this.code = code;
  }
}
