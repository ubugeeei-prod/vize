# Configured-route and hosted Musea VRT

Issue: [#8397](https://github.com/ubugeeei-prod/vize/issues/8397).

The runner previously constructed `/__musea__/preview` for every capture, even
when Musea or Vite configured another route. Share one route-aware resolver
between screenshot and accompanying accessibility capture. The CLI resolves
Vite base plus Musea `basePath`; an explicit VRT `previewBasePath` overrides it.
Default preview URL encoding and generated modules remain unchanged.

A static host cannot run Node or update local snapshot files. Add `--gallery-url`
to the local CLI: read the existing `api/static.json`, validate required art and
variant fields, and resolve its exact preview URLs. Manifest entries must remain
within that gallery's origin and path; missing entries never fall back to a dev
server. No original source files, host-side service, or server credentials are
required. Run, approve, and clean share the same manifest and snapshot options.
The static panel supplies its usable command, with English and Japanese guides.

CLI and development-gallery captures wait for the existing `musea:ready`
message after async setup. Install the standalone same-window listener before
navigation. Low-level runner callers retain their historical selector-only
behavior unless they request `capture.waitForPreviewReady`. JSON reports retain
existing fields and add source identity, PNG paths, and pixel counts.

The persisted hosted VRT fixture delays setup 1200ms with network-idle waiting
and settle time disabled. Real emitted files run over HTTP at a subpath; the
source Art file is then deleted. A real CLI process creates six baselines,
repeats all six exactly, detects six changed previews with nonzero CI status,
approves them, repeats them successfully, and removes an orphan. An independent
Chromium screenshot of the mounted named button must equal the CLI's baseline
pixels. Physical served preview files produce the deliberate visual difference.
The legacy no-ready capture control fails before 1200ms; the ready successor
passes. Associated CLI audits run real axe on all six exact previews. Actions
retains reports, PNGs, and HTTP/process observations. Native Art
compilation remains outside this browser fixture; package tests cover it
separately. Source/protected checks, actual merge, and installed publication
acceptance remain distinct. Automatic global-combination capture remains open.
