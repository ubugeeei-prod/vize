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

Screenshot comparisons and baseline updates need Node and Playwright. Run
`vp exec musea-vrt` from your project or use **Run VRT** in the development
gallery; the static VRT panel shows these instructions.

