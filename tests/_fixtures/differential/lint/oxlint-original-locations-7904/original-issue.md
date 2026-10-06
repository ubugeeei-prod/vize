## Summary

`oxlint-vize` is meant to report original SFC positions ("`-f json` retains those original positions" in the README), but:

1. In `-f json`, `labels[].span.offset` is an offset into the temporary bridged file, not the original file. It is larger than the file itself (offset 368 in a 108-byte file). `line` / `column` are right in most cases, so tools that use the offset (code frames, reviewdog-style annotators, editor integrations) point at the wrong place or fail.
2. In an SFC without `<script>`, a diagnostic anchored at the start of the file (`vue/multi-word-component-names`) is printed as `Static.vue:1:301` in a 3-line, 44-byte file. `vize lint` reports `1:11`. The line/column seem to be recomputed from the shifted offset.

This is different from #7004 (raw `oxlint` anchors template diagnostics at `<script>`) and #7005 (raw `oxlint` skips script-less SFCs). Both happen with `oxlint-vize`, the documented workaround for those.

## Environment

- `oxlint-plugin-vize` 0.432.0, `oxlint` 1.86.0
- macOS 26.4.1 arm64, Node v26.10.0, npm 11.19.1

## Reproduction

```sh
mkdir -p repro-oxlint-vize-offset && cd repro-oxlint-vize-offset
npm init -y > /dev/null
npm install -D oxlint@1.86.0 oxlint-plugin-vize@0.432.0 > /dev/null 2>&1
cat > .oxlintrc.json <<'JSON'
{
  "jsPlugins": ["oxlint-plugin-vize"],
  "settings": { "vize": { "preset": "essential" } },
  "rules": {
    "vize/vue/no-v-html": "warn",
    "vize/vue/multi-word-component-names": "error"
  }
}
JSON
printf '<script setup lang="ts">\nconst html = "<b>x</b>";\n</script>\n\n<template>\n  <div v-html="html" />\n</template>\n' > Panel.vue
printf '<template>\n  <div v-html="x" />\n</template>\n' > Static.vue
wc -c Panel.vue Static.vue
npx oxlint-vize -f json Panel.vue Static.vue \
  | node -e 'let s="";process.stdin.on("data",c=>s+=c).on("end",()=>{for (const m of JSON.parse(s).diagnostics) console.log(m.filename, m.code, JSON.stringify(m.labels[0].span))})'
npx oxlint-vize Static.vue | grep -o '^Static.vue:[0-9]*:[0-9]*: [a-z]* vize([^)]*)'
npx vize@0.432.0 lint --no-config --preset essential -f plain --help-level none Static.vue | grep multi-word
```

## Actual

```text
     108 Panel.vue
      44 Static.vue
Static.vue vize(vue/no-v-html) {"offset":308,"length":10,"line":2,"column":8}
Static.vue vize(vue/multi-word-component-names) {"offset":300,"length":0,"line":1,"column":301}
Panel.vue vize(vue/no-v-html) {"offset":368,"length":13,"line":6,"column":8}
Panel.vue vize(vue/multi-word-component-names) {"offset":360,"length":0,"line":5,"column":11}
Static.vue:1:301: error vize(vue/multi-word-component-names)
  Static.vue:1:11 error vue/multi-word-component-names Component name "Static" should be multi-word to avoid conflicts with HTML elements
```

## Expected

Offsets into the original files (`Static.vue` `v-html`: 18, `multi-word-component-names`: 10; `Panel.vue` `v-html`: 79, `multi-word-component-names`: 71), and `Static.vue:1:11` for `vue/multi-word-component-names`, matching `vize lint`.
