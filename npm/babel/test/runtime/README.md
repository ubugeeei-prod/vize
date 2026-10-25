# Real renderer contracts

This strict matrix runs one authored CRLF/Unicode `Counter`/`App` TSX program through the actual packed ESM and CJS plugin exports under Babel 7.29.7 and 8.0.9. It uses Vue/server-renderer 3.5.44 for VDOM and 3.6.0-rc.9/rc.11 for Vapor, with real Playwright 1.62.1 Chromium.

Install the existing `../hosts/babel7` and `babel8` locks, then each of `stable`, `rc9`, and `rc11` with `npm ci --ignore-scripts`. The source Actions owner installs Chromium explicitly; this suite never downloads a browser implicitly. `infrastructure.mjs` checks only actual runtime exports, exact compiler graphs, independent reference SSR and real Babel TypeScript parsing. It loads no packed plugin/native addon or browser and grants no runtime producer credit.

The full command is `node --test npm/babel/test/runtime-renderers.test.mjs`, while the genuine source-native preparation owner is active, with:

- `SOURCE_SHA`: full actual integration revision.
- `VIZE_BABEL_PACKED_PLUGIN`: actual built dual-export npm tarball.
- `VIZE_BABEL_NATIVE_ROOT`: explicit matching source package and active preparation.
- `VIZE_BABEL_RUNTIME_OUTPUT`: fresh output directory (optional; the default is `.artifacts/SOURCE_SHA`).

All 12 source rows fail closed on missing prerequisites, source lease/image mismatch, unsupported producers, missing browser or runtime defects; none skip. Each private compiler host extracts only the packed public distribution and links explicit installed dependencies. Native fallback is forbidden. Client and server target phases retain the complete actual Babel packets and loaded addon identity. Emitted ESM bytes are written unchanged. Node resolves server imports through the exact runtime installation; Chromium uses a real import map for `vue`. No emitted import text is rewritten.

The independent control has separately authored real Vue setup objects and templates. Stable rendering uses `defineComponent`/`h`; Vapor client and server control templates use their runtime's own exact compiler graph and real component APIs. Entire generated reference code and physical compiler hashes are retained. The authored child owns its counter and click handler; the parent supplies reactive labels and fallthrough id/class/style/data/listener bindings. Setup counts remain one across parent update, real Playwright click and restore. Both child and parent listeners must run exactly once.

Node SSR retains and compares entire raw HTML. Actual Chromium checks both fresh mounting and hydration against the independent control, preserving all comment/text nodes and attributes, exact raw HTML, effects, diagnostics and screenshots. Hydration must adopt every original server node in order, including text/comments, before interaction. Raw serialization or node-shape differences remain failures for review; the harness never strips markers, combines text nodes or edits output to match an oracle. All counterpart packets are collected before the final comparisons. Warnings, console errors, page errors and request failures must be empty.

The Babel parent lane owns source qualification, workflow integration and product fixes. Missing or failing source behavior is not calibrated from generated output, replaced with a public binary or reported as a runtime pass.
