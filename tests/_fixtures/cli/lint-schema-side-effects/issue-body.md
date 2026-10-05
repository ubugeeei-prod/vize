## Area

`vize lint` (CLI side effect)

## Version

`vize` 0.432.0 (macOS arm64, Node 26.6.0)

## Minimal reproduction

An empty directory with one file:

`MyText.vue`

```vue
<template>
  <p>x</p>
</template>
```

```sh
find . | sort
vize lint --no-config -f plain MyText.vue
find . | sort
```

## Actual

```
.
./MyText.vue
Patina lint report: No problems found in 1 file(s)
.
./MyText.vue
./node_modules
./node_modules/.vize
./node_modules/.vize/vize.config.schema.json
```

A read-only lint run creates `node_modules/.vize/vize.config.schema.json` (≈38 KB) in the cwd,
creating `node_modules/` itself when the directory has none. It happens with `--no-config` and
with a `vize.config.json`; `vize fmt --check` on the same directory does not write anything.

## Expected

`vize lint` does not write into the project. If the schema copy is needed (for editor completion
of `vize.config.json` via `"$schema"`), it could be written by `vize setup` / `vize init`, or only
when a config file references it, and never by creating `node_modules/` in a directory that does
not have one (a fresh checkout before install, a non-npm project, a directory linted from CI with a
read-only tree, or a temp directory used for a repro).
