#!/usr/bin/env bash
vp exec tsx --test gallery/components/home-layout.browser.test.ts
vp exec tsx --test src/a11y/browser-contracts.test.ts src/a11y/node-report.test.ts gallery/components/code-display.browser.test.ts gallery/components/global-toolbar.browser.test.ts gallery/components/public-browser-observation.browser.test.ts gallery/components/hosted-tests.browser.test.ts gallery/components/snapshot-collisions.browser.test.ts
vp exec tsx --test gallery/components/viewport-presets.browser.test.ts
vp exec tsx --test src/props-permalink.test.ts gallery/components/props-permalink.browser.test.ts
vp exec tsx --test gallery/components/sidebar-rotation.browser.test.ts
vp exec tsx --test src/vrt/mask-capture.browser.test.ts
vp exec tsx --test src/vrt/diff-style.browser.test.ts
vp exec tsx --test src/vrt/before-capture.browser.test.ts
vp exec tsx --test gallery/components/content-layout.browser.test.ts
