# Musea Hosting

If you install the `vize` npm package, `vp exec vize musea` is a convenience wrapper around Vite:

```bash
vp exec vize musea
vp exec vize musea --build
```

The built gallery can run accessibility checks in its preview iframes. Install
`axe-core` before building so the export includes the audit bundle. Asynchronous
`previewSetup` hooks are supported: checks wait for the mounted preview, and
loading or audit errors appear as failed tests. Host the complete output,
including its `vendor` directory, beneath the configured Vite base path.

## Visual regression testing a hosted gallery

Install Playwright in the environment that runs the CLI, then install Chromium:

```bash
vp add -D playwright
vp exec playwright install chromium
vp exec musea-vrt --gallery-url https://example.com/site/__musea__/ --json --ci
```

The static VRT panel shows a command for its own gallery URL. The CLI reads
`api/static.json` and captures the exact emitted preview URLs. You do not need
the original Art source files or a Node service on the host. Keep the gallery's
complete output available at that URL, including previews, assets, and the manifest.

The first run creates baselines in `.vize/snapshots`; subsequent runs compare
against them. A screenshot waits for asynchronous `previewSetup` and mounting.
A visual difference or capture error makes `--ci` exit unsuccessfully. JSON
reports include baseline, current, and diff paths, and changed pixel counts.
Use `--output path` to store reports and default snapshots elsewhere.

Review current and diff images before accepting a visual change:

```bash
vp exec musea-vrt approve --gallery-url https://example.com/site/__musea__/
vp exec musea-vrt clean --gallery-url https://example.com/site/__musea__/
```

`approve` captures the current gallery again and replaces failed baselines.
`clean` removes baselines for variants no longer present in the manifest. Pass
the same `--config` and `--output` options to every command when customizing
storage, viewport, threshold, or capture settings.

For a development server, run `vp exec musea-vrt` in the project or use **Run VRT**
in the gallery. The CLI honors the Vite base and Musea `basePath` from your config;
`--base-url` selects the server origin. Hosted capture uses `--gallery-url` instead.

See [snapshot identity and migration](./musea-snapshots.md) before reusing existing baselines or testing same-named Art files.

## Gallery VRT reports

**Run VRT** saves the selected Art's JSON and HTML reports in `.vize/reports`.
A unique safe basename keeps its familiar name, such as `vrt-Button-report.json`.
Same-named Arts receive separate deterministic report names, so capturing one
keeps the other Art's report and baseline intact. Use the report paths shown
by the gallery rather than constructing them from an Art basename.

If a report's owner cannot be established, capture stops before changing reports
or snapshots. Move both named JSON and HTML files to an archive directory and
retry; keep those historical files until you have reviewed their contents.
Removing one of two same-named Arts does not transfer its old report to the other.
This applies to the development gallery; hosted galleries use the CLI above.
