# QR Code Behavior Contract

Normative state x input -> outcome table for `qr-code.vue` and the pure encoder
exported from `@vizejs/ui/qr-code` (`encodeQrCode`, `qrCodeToSvgPath`,
`qrCodeCapacity`, `isQrCodeModuleDark`). Every row is proven by the named test.

The encoder implements ISO/IEC 18004 model 2 symbols without dependencies:
numeric, alphanumeric, and UTF-8 byte segments (whole-value single mode, no ECI
header), versions 1-40, levels L/M/Q/H with optional boost, Reed-Solomon over
GF(256) with block interleaving, finder/timing/alignment/format/version
patterns, and all eight masks with the standard penalty score. Output is
identical to Project Nayuki's reference generator for the same inputs.

| ID  | State                  | Input                                   | Outcome                                                                                                     | Evidence                                                                    |
| --- | ---------------------- | --------------------------------------- | ----------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| Q1  | byte text, level L     | `encodeQrCode("Hello, world!")`         | produces the reference version 1, mask 2 module grid                                                        | `matches the reference byte-mode symbol for Hello, world! at level L`       |
| Q2  | numeric / alphanumeric | auto mode                               | selects the compact mode and produces the reference grids                                                   | `matches reference numeric and alphanumeric symbols`                        |
| Q3  | multi-block, v7+       | long URL at level H                     | interleaves blocks, draws version information, and matches the reference version 9 grid                     | `matches a multi-block reference symbol with version information`           |
| Q4  | spec worked example    | `01234567` at 1-M                       | emits the standard data codewords, Reed-Solomon codewords, and interleaved stream                           | `reproduces the ISO/IEC 18004 worked example codewords for 01234567 at 1-M` |
| Q5  | alphanumeric segment   | `AC-42`                                 | packs character pairs into 11 bits and a trailing character into 6 bits                                     | `encodes alphanumeric pairs from the standard AC-42 example`                |
| Q6  | Reed-Solomon           | field multiply, generator degree        | GF(256) arithmetic and generator polynomials match the standard; invalid degrees throw                      | `implements GF(256) arithmetic and Reed-Solomon generators`                 |
| Q7  | format information     | every level x mask                      | emits the 32 masked BCH format strings from table C.1                                                       | `computes format information for every level and mask`                      |
| Q8  | version information    | versions 7, 21, 40                      | emits the standard 18-bit BCH version strings                                                               | `computes version information for versions 7 through 40`                    |
| Q9  | alignment patterns     | versions 1-40                           | centers match the standard table, including the irregular version 32                                        | `places alignment patterns at the standard centers`                         |
| Q10 | capacity               | version x level x mode                  | codeword and character capacities match the standard tables                                                 | `matches the standard codeword capacity tables`                             |
| Q11 | auto version           | data at and over each capacity boundary | picks the smallest fitting version and moves up exactly one byte past capacity                              | `selects the smallest version at the exact capacity boundary`               |
| Q12 | invalid input          | overflow, forced mode, bad options      | throws `QrCodeEncodeError` with `VIZE_UI_QR_DATA_TOO_LONG`, `VIZE_UI_QR_INVALID_MODE`, or `…INVALID_OPTION` | `throws typed diagnostics for data overflow and invalid options`            |
| Q13 | explicit options       | fixed version/mask, forced mode, boost  | honors each option; boost raises the level only while the data still fits                                   | `honors fixed version, fixed mask, forced mode, and error correction boost` |
| Q14 | text encodings         | Unicode text, bytes, empty string       | Unicode text equals its UTF-8 bytes; empty values encode a version 1 symbol with `mode: null`               | `encodes UTF-8 text and empty values`                                       |
| Q15 | auto mask              | encode                                  | applies the lowest-penalty mask; output is frozen and deterministic                                         | `picks the mask with the lowest penalty and returns frozen output`          |
| Q16 | penalty rules          | synthetic grids                         | runs, 2x2 blocks, finder-like patterns, and balance follow the standard weights                             | `scores the four standard penalty rules`                                    |
| Q17 | SVG path               | module rows                             | merges each horizontal dark run into one rectangle                                                          | `merges horizontal dark runs into one rectangle per run`                    |
| Q18 | SVG path, quiet zone   | `quietZone`                             | offsets every coordinate by the quiet zone                                                                  | `offsets every coordinate by the quiet zone`                                |
| Q19 | SVG path coverage      | encoded symbol                          | rasterized path covers exactly the dark modules with no overlap                                             | `covers exactly the dark modules of an encoded symbol`                      |
| Q20 | SVG path options       | invalid quiet zone                      | rejects negative, fractional, or oversized quiet zones                                                      | `rejects invalid quiet zones`                                               |
| Q21 | ready                  | render                                  | renders `<svg role="img">` named by label or text, `<title>`, viewBox with quiet zone, and data hooks       | `renders an accessible SVG symbol with a quiet zone and data hooks`         |
| Q22 | ready                  | prop change                             | forwards encoder props and re-encodes reactively                                                            | `forwards encoder options and re-encodes when props change`                 |
| Q23 | labelled / decorative  | render                                  | byte values use `label`; decorative symbols drop role, name, and title and set `aria-hidden`; fills apply   | `supports explicit labels, byte values, decorative symbols, and colors`     |
| Q24 | ready + overlay        | render                                  | overlay slot renders inside the SVG with dimension and matrix slot state                                    | `renders the overlay slot inside the SVG with module coordinates`           |
| Q25 | error                  | unencodable value or invalid quiet zone | renders `<span data-state="error" data-error>` with the fallback slot, and recovers when props become valid | `renders the fallback slot with a typed diagnostic when encoding fails`     |
| Q26 | exposed instance       | ref access                              | exposes element, matrix, state, error, quiet zone, and dimension                                            | `exposes the encoded matrix, state, and element`                            |
| Q27 | SSR                    | two isolated requests                   | byte-identical markup, computed purely from props                                                           | `renders byte-identical QR code markup across isolated SSR requests`        |
| Q28 | SSR + hydration        | mount over server HTML                  | keeps server nodes and emits no warnings                                                                    | `hydrates the QR code without replacing server nodes or warning`            |

Colors are SVG presentation attributes (`foreground` defaults to
`currentColor`, `background` to `none`), so consumer CSS on
`[part="modules"]` and `[part="background"]` always wins. Scanners need a light
background and a 4-module quiet zone; use level `H` when an overlay covers the
center.
