# `@vizejs/wasm`

Vize's WebAssembly compiler bindings support browsers and Cloudflare Workers.

## Browser

```js
import init, { compileSfc } from "@vizejs/wasm";

await init();
const result = compileSfc("<template><main /></template>", { filename: "App.vue" });
```

## Cloudflare Workers

Import the focused compiler artifact as a compiled Wasm module and initialize it from inside the
request handler. Cache the promise so warm requests reuse the same binding.

```js
import vizeWasm from "@vizejs/wasm/wasm.wasm";
import { instantiate } from "@vizejs/wasm/workerd";

let bindingPromise;

function getBinding() {
  bindingPromise ??= instantiate(vizeWasm);
  return bindingPromise;
}

export default {
  async fetch() {
    const vize = await getBinding();
    const result = vize.compileSfc("<template><main /></template>", {
      filename: "App.vue",
    });
    return Response.json({ ok: true, result });
  },
};
```

Wrangler loads `.wasm` imports as `CompiledWasm` modules. The workerd artifact deliberately exposes
the compiler-only API rather than the browser package's linting, formatting, cross-file analysis,
Musea, and inspector bindings, keeping its gzip size below the Workers free-plan limit.
