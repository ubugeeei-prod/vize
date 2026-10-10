# Authored variant binding collision (#8478)

`Host.vue` and `Host.art.vue` retain the byte-exact native public npm 0.440.0
macOS reproduction. `ordinary/` changes only `State-enabled` to `State disabled`.
Both variants intentionally render different Space/Dash labels. The collision
build fails with duplicate `StateEnabled` imports; the ordinary control builds
114 emitted files and renders both native previews with scoped CSS through
plain HTTP in actual Chrome controlled by Playwright.

`public-0.440-macos.json` retains the whole error, fixture/module hashes,
actual preview DOM/resources/screenshots and unchanged original-consumer checks.
`public-0.440-build.log` and `public-0.440-emitted.log` are byte-exact whole raw
artifacts. The PNGs are the actual ordinary HTTP controls. All 2,061 original
public consumer files/symlinks remain unchanged after browser verification.

This is a macOS product diagnostic, not official Linux release acceptance or
current-source proof. The separate native browser law builds both original
fixtures through real source Vize/Musea and checks all four authored previews.
No public package, compiler oracle or official acceptance receipt was patched.
