# Patina reference provider resolution

Full Check [38993084178](https://github.com/ubugeeei-prod/vize/actions/runs/38993084178)
failed five legacy-instance reference files after #9251. Their compiler lookup
started at the benchmark manifest, which declares Vue 3.6.0-beta.10. The frozen
reference requires 3.6.0-rc.9, already installed by the full worker through the
exact `vue-legacy-instance-api-oracle` alias in `tests/package.json` and its lock.

Resolve Vue through that alias, compiler sources through the selected compiler's
own manifest, and TypeScript through the selected script parser's declared peer.
Retain the explicit external-provider manifest override and all version checks.

The correct dependency lookup exposed incorrect historical provider metadata:
the frozen compiler-core source hash identifies a hoisted 3.5.35 implementation,
although the selected compiler-dom/SFC files are rc.9. Both public tarballs were
downloaded independently and their SHA-512 integrity matched `pnpm-lock.yaml`:

| compiler-core version                                                  | `dist/compiler-core.cjs.js` SHA-256                                |
| ---------------------------------------------------------------------- | ------------------------------------------------------------------ |
| [3.5.35](https://registry.npmjs.org/@vue/compiler-core/3.5.35)         | `820601aa19727baa3de3b6eea199d6dfb547911afdb7b22d8081d3c81eff284e` |
| [3.6.0-rc.9](https://registry.npmjs.org/@vue/compiler-core/3.6.0-rc.9) | `61aa2191bdde10d4e7e9f8224f47913350de3941cf54b2c2430ccdb8f7d1c08f` |

Keep every original JSON and compressed proof byte unchanged. The family-specific
verifier asserts the historical metadata separately, then requires the actual
loaded rc.9 dependency, exact package versions, declared dependency, source paths
and implementation hashes. The historical hash cannot authenticate the current
loaded compiler-core. All other frozen provider fields and whole observations
remain exact; the verifier does not normalize compiler or runtime output.

With a frozen, script-free install of the tests and benchmark workspaces, all
95 original tests pass against the corrected provider resolution. A focused
negative control additionally refuses benchmark Vue and altered source hashes.
The original compiler/AST/map, linter, TypeScript erasure and runtime packets,
case counts and original-source custody checks all remain unchanged. Product
builds and hosted qualification remain the parent workflow's responsibility.
