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

## Reviewing hosted screenshots in the browser

Open a built gallery over HTTPS and start a local session on the machine where
Playwright and your baselines are installed:

```bash
vp exec musea-vrt serve --gallery-url https://example.com/site/__musea__/
```

Select an Art and open **VRT**. Paste the printed **VRT endpoint** and **Session
token**, then select **Connect VRT**. Chrome may ask to allow loopback access for
this gallery. If access is blocked, allow it in that site's settings or use the
CLI capture command above.

**Run VRT** shows the actual baseline, current capture and diff images in the
gallery. Review the change, select **Update snapshots**, and run again to accept
it. Clear the checkbox and repeat the capture to confirm the updated baseline.
You can download the complete JSON and HTML reports from the result pane.

The session listens only on your machine and accepts the gallery URL you started
it with. It keeps its token for that session; restarting prints a new token.
Snapshots and per-Art reports stay local under `.vize/snapshots` and `.vize/reports`,
or your configured `--output`/`--config` paths. Stop the companion with Ctrl-C.
The hosted site continues serving static files and needs no Node process.

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
The development gallery and local hosted-gallery sessions use this ownership
rule; the standalone hosted CLI remains available for CI and batch captures.
