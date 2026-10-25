// Keep the original production exception; Nitro otherwise logs a sanitized 500.
export default (nitro) => {
  nitro.hooks.hook("error", (error) => {
    console.error("[nuxt2-fixture:runtime-error]", error, error?.cause);
  });
};
