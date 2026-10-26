// Make the optional `oxlint-plugin-vize` peer unresolvable, as in an app that never installed it.
import { registerHooks } from "node:module";

registerHooks({
  resolve(specifier, context, nextResolve) {
    if (specifier === "oxlint-plugin-vize" || specifier.startsWith("oxlint-plugin-vize/")) {
      const error = new Error(`Cannot find package '${specifier}'`);
      error.code = "ERR_MODULE_NOT_FOUND";
      throw error;
    }
    return nextResolve(specifier, context);
  },
});
