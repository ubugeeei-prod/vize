## Summary

Since `oxlint-plugin-vize` 0.439.0, `oxlint-vize <dir>` exits 1 without linting anything when the directory contains both `.vue` files and a standalone `.html` file:

```text
Script-safe Vue transport unavailable: Scoped Vue transport cannot combine standalone HTML and Vue targets.
```

Every Vite app has this layout (an `index.html` at the project root next to `src/**/*.vue`), so `oxlint-vize apps/<app>` now fails in a monorepo where it passed on 0.435.0. `--ignore-pattern '**/*.html'` and `ignorePatterns` do not help.

I think this is the conservative refusal described in `docs/davinci/decisions/2026-10-08-oxlint-original-ignore-replay.md` ("mixed Vue/HTML discovery conservatively refuses transport, even when the HTML candidate is ignored"), which is being addressed in the #7903 HTML work. I'm filing it separately because the user-facing effect is a hard failure for a common project layout. That layout worked on 0.435.0, and I couldn't find it recorded as a regression.

## Reproduction

```sh
mkdir repro && cd repro && mkdir app
cat > package.json <<'JSON'
{ "name": "repro", "private": true, "type": "module",
  "devDependencies": { "oxlint": "1.81.0", "oxlint-plugin-vize": "0.441.0" } }
JSON
cat > .oxlintrc.json <<'JSON'
{ "plugins": ["vue"], "jsPlugins": ["oxlint-plugin-vize"],
  "settings": { "vize": { "preset": "recommended" } } }
JSON
printf '<!doctype html>\n<html><body><div id="app"></div></body></html>\n' > app/index.html
printf '<script setup lang="ts">\nconst msg = "hi"\n</script>\n\n<template>\n  <p>{{ msg }}</p>\n</template>\n' > app/App.vue
pnpm install
npx oxlint-vize app; echo "exit=$?"
```

## Actual / expected

| `oxlint-plugin-vize` | `oxlint-vize app` |
| --- | --- |
| 0.435.0 | exit 0 |
| 0.439.0 | exit 1, "cannot combine standalone HTML and Vue targets" |
| 0.441.0 | exit 1, same |

`oxlint-vize app/App.vue` passes on all three versions, so the failure comes from discovering the mixed directory.

Expected: the same behavior as 0.435.0. The `.vue` files are linted, and the standalone HTML is either skipped (stock Oxlint does not select it) or linted. At minimum, an ignored `.html` should not block the run.

## Environment

- `oxlint` 1.81.0, `oxlint-plugin-vize` 0.441.0 (also 0.439.0)
- macOS 26 (Darwin 25.6.0) arm64, Node v24.15.0, pnpm 11

Related: #7903
