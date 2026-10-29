These are complete, unmodified `pnpm audit --prod --json` producer packets captured on 2026-10-27 with pnpm 12.1.0 in fresh temporary projects outside the workspace. Neither project has overrides or patches.

- `public-3.11.2-audit.json.txt`: the exact runtime Kit edge extracted from the integrity-verified public `@vizejs/nuxt@0.597.0` archive. The archive SHA-512 integrity is `sha512-dGX28fJWPDgKIqGz0DbTagxXAyxSe5K6NRClDgaaCcnoOT5Gi6o3SxG7w83u8wIyPwfy4ms7C7k+dAkHcZrBsw==`. The new verifier refuses this real before case: 12 findings, including critical GHSA-23hp-3jrh-7fpw.
- `public-3.21.8-audit.json.txt`: an independent candidate Kit project, with no findings. The workflow additionally packs the current module and resolves the Kit edge from its actual publishable manifest; this stored packet does not replace that live check.

This isolates the module's Kit dependency closure. Full Nuxt hosts and their separate upstream Nitro/listhen/Forge dependency are not represented as clean by this proof.

The root-only `chokidar-braces-oracle` dev dependency explicitly preserves the historical Chokidar 3.6.0 consumer after it stops being installed through Kit 3.11.2. It keeps the existing whole-source census and real Braces positive/negative runtime laws unchanged. It is not added to the published Nuxt module or to this public Kit closure.
