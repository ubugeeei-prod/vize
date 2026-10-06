## Summary

The `--fix` for `nuxt/nuxt-config-keys-order` adds a trailing comma after the last property when the object had none. The reorder itself is right (and since #7260 the comments move with their keys), but the fix also changes the punctuation of the file.

## Reproduction

`nuxt.config.ts`

```ts
export default defineNuxtConfig({
  ssr: false,
  modules: ["@pinia/nuxt"]
});
```

```sh
vize lint --no-config --preset nuxt --fix nuxt.config.ts
```

```ts
export default defineNuxtConfig({
  modules: ["@pinia/nuxt"],
  ssr: false,
});
```

## Expected

```ts
export default defineNuxtConfig({
  modules: ["@pinia/nuxt"],
  ssr: false
});
```

The fix should only move properties. Whether the last one ends with a comma should follow the original object: keep it when there was one, leave it out when there was not.

## Why it matters

Projects that format with `trailingComma: "none"` get a second diff (and a formatter failure in CI) after every `--fix`, unrelated to the key order. Running the formatter afterwards hides it, but `lint --fix` on its own should leave a file that the project's formatter accepts if it did before.

## Environment

- vize 0.432.0
- node 26.8.1, macOS arm64
