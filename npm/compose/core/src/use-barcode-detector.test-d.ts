/** Compile-only assertions for the `use-barcode-detector` type contracts. */

import { useBarcodeDetector } from "./use-barcode-detector.ts";
import type { BarcodeFormat, DetectedBarcode } from "./use-barcode-detector.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const detector = useBarcodeDetector({ formats: ["qr_code", "ean_13"] });

type _Detect = Expect<
  Equal<Awaited<ReturnType<typeof detector.detect>>, readonly DetectedBarcode[]>
>;
type _Format = Expect<Equal<DetectedBarcode["format"], BarcodeFormat>>;
type _Formats = Expect<
  Equal<Awaited<ReturnType<typeof detector.getSupportedFormats>>, BarcodeFormat[]>
>;

// @ts-expect-error formats are a closed union.
useBarcodeDetector({ formats: ["qr"] });

// @ts-expect-error strings are not image sources.
void detector.detect("image.png");

// @ts-expect-error barcodes are read-only.
detector.barcodes.value = [];
