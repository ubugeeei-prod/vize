# Vitest persistent Vue module regression

Reproduces [Andrei Elkin's report, #9363](https://github.com/ubugeeei-prod/vize/issues/9363)
with the exact Vitest 5.0.5, Vite 8.3.4 and Vue 3.5.42 versions. The locked
0.458.0 Vize package remains the failing baseline; the candidate comes from a
separate source build or an installed release entrypoint.

After installing this fixture with `npm ci --ignore-scripts --no-audit --no-fund`,
run from the repository root:

```sh
node tools/support/compat/vitest-module-cache.ts \
  --candidate npm/builder/vite/dist/index.mjs \
  --output target/vitest-module-cache
```

Use a new output directory for each campaign. Each campaign retains the same
physical filesystem cache across distinct Vitest CLI processes. The harness
requires the published baseline's exact warm resolution failure, successful
plugin-vue and cache-disabled controls, and successful candidate cold/warm/warm
again mounts with filesystem cache hits. Candidate warm runs preserve every
cache file's bytes and timestamp. Real assertion failures and an SFC-only change
prove the tests execute and the changed component receives a new cache identity.

The candidate keeps `fsModuleCache: true`. It resolves Vite's root-relative
module URLs back to the same absolute source identity established by a cold
import; it does not use a Vitest-specific cache opt-out. See the
[Vitest filesystem cache contract](https://vitest.dev/config/fsmodulecache).
