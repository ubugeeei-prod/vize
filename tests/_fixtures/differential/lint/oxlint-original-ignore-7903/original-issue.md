## Summary

`oxlint-vize` (the wrapper the README recommends for original SFC locations) lints Vue and HTML files that Oxlint itself skips:

- files under a directory listed in `.gitignore` (e.g. `dist/`),
- files matched by `ignorePatterns` in the Oxlint config,
- files matched by `--ignore-pattern` on the command line.

Plain `oxlint` with the same config and arguments skips all of them. `.ts` files keep their ignores under `oxlint-vize` (a `debugger;` in `dist/built.ts` is not reported), so this looks specific to the temporary location bridge copies of `.vue` / `.html` files: the bridged paths no longer match the ignore rules.

In practice, `oxlint-vize .` lints build output: built HTML in a `dist/` directory produced thousands of diagnostics in a docs site, from files that `oxlint .` never opens.

## Environment

- `oxlint-plugin-vize` 0.432.0, `oxlint` 1.86.0
- macOS 26.4.1 arm64, Node v26.10.0, npm 11.19.1

## Reproduction

```sh
mkdir -p repro-oxlint-vize-ignore && cd repro-oxlint-vize-ignore
git init -q
npm init -y > /dev/null
npm install -D oxlint@1.86.0 oxlint-plugin-vize@0.432.0 > /dev/null 2>&1
printf 'node_modules/\ndist/\n' > .gitignore
mkdir -p src dist vendor
cat > .oxlintrc.json <<'JSON'
{
  "jsPlugins": ["oxlint-plugin-vize"],
  "ignorePatterns": ["vendor/**"],
  "rules": { "vize/vue/no-v-html": "warn" }
}
JSON
printf '<script setup lang="ts">\nconst html = "<b>x</b>";\n</script>\n\n<template>\n  <div v-html="html" />\n</template>\n' > src/AppPanel.vue
cp src/AppPanel.vue dist/BuiltPanel.vue
cp src/AppPanel.vue vendor/VendorPanel.vue
cp src/AppPanel.vue src/SkippedPanel.vue
echo '--- oxlint'
npx oxlint --ignore-pattern 'src/SkippedPanel.vue' .
echo '--- oxlint-vize'
npx oxlint-vize --ignore-pattern 'src/SkippedPanel.vue' .
```

## Actual

Help text shortened:

```text
--- oxlint
src/AppPanel.vue:2:2: warning vize(vue/no-v-html): v-html can lead to XSS attacks. …
--- oxlint-vize
src/SkippedPanel.vue:6:8: warning vize(vue/no-v-html): v-html can lead to XSS attacks. …
src/AppPanel.vue:6:8: warning vize(vue/no-v-html): v-html can lead to XSS attacks. …
vendor/VendorPanel.vue:6:8: warning vize(vue/no-v-html): v-html can lead to XSS attacks. …
dist/BuiltPanel.vue:6:8: warning vize(vue/no-v-html): v-html can lead to XSS attacks. …
```

(`oxlint`'s `2:2` position is the known limitation in #7004. This issue is about the file set.)

## Expected

`oxlint-vize` lints the same files as `oxlint`: only `src/AppPanel.vue` here. `dist/` (`.gitignore`), `vendor/**` (`ignorePatterns`) and `src/SkippedPanel.vue` (`--ignore-pattern`) are skipped.
