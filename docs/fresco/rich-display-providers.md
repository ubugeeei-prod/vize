# Optional Rich Display Providers

Fresco keeps font data and QR encoders out of its core bundle. Two optional entry points provide
typed Vue renderers while the application chooses the data or encoder implementation.

## ASCII fonts

```ts
import { AsciiFont, createGlyphAsciiFont } from "@vizejs/fresco/ascii-font";

const font = createGlyphAsciiFont({
  name: "status-font",
  glyphs: {
    O: ["OOO", "O O", "OOO"],
    K: ["K K", "KK ", "K K"],
  },
});
```

Pass `font` to `<AsciiFont text="OK" :provider="font" />`. All glyphs must have the same row
height. Missing characters use the provider's `fallback` rows, and Unicode input is iterated by
code point rather than UTF-16 code unit.

## QR codes

```ts
import { defineQrCodeProvider, QrCode } from "@vizejs/fresco/qr-code";

const provider = defineQrCodeProvider((value, { errorCorrection }) => {
  return encoder(value, { errorCorrection }); // readonly boolean[][]
});
```

Pass the provider to `<QrCode :value="url" :provider="provider" />`. The component validates a
non-empty square matrix, then owns terminal cell width, quiet-zone, colors, and inversion. The
encoder remains an application dependency and is never pulled into `@vizejs/fresco` core imports.
