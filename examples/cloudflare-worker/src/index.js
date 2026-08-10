import vizeWasm from "@vizejs/wasm/wasm.wasm";
import { instantiate } from "@vizejs/wasm/workerd";

import { handleRequest } from "./handler.js";

let bindingPromise;

function getBinding() {
  bindingPromise ??= instantiate(vizeWasm);
  return bindingPromise;
}

export default {
  async fetch(request) {
    try {
      return await handleRequest(request, getBinding);
    } catch (error) {
      return Response.json(
        { ok: false, error: error instanceof Error ? error.message : String(error) },
        { status: 500 },
      );
    }
  },
};
