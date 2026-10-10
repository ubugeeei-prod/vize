# Musea snapshot collision safety

Issue: [#8412](https://github.com/ubugeeei-prod/vize/issues/8412).

The legacy snapshot name includes only Art basename, variant name and viewport
name. Distinct `left/Button.art.vue` and `right/Button.art.vue` sources can
therefore read and write the same PNG. Sequential capture produces a false
visual difference against another component; concurrent capture can replace
that component's baseline or current image. Repeated viewport names also collide.

Enumerate the existing complete, filtered capture jobs and validate their output
names before any worker starts. Reject the batch with the filename and both
owners rather than creating or changing a shared PNG. No generated preview,
ordinary filename, viewport selection, capture concurrency, comparison, or
baseline approval behavior changes. Skipped variants retain their existing path.

The persisted left/right Art sources share only basename and variant name;
exact-source native package tests verify separate paths, titles, variant names
and blue/red templates. The real Chromium/HTTP contract first retains the legacy
shared-baseline comparison, then verifies whole-batch rejection before requests
or filesystem writes. The unguarded runner fails this latter assertion. Actions
retains both genuine source bytes, legacy PNGs and process observations. The
browser fixture isolates navigation and storage; native compilation is proved
separately and is not claimed for the fixture's explicit HTML renderer.

This safety refusal does not finish usable unique snapshot identities. Current
Art metadata exposes an absolute path only, and static preview IDs hash that
build-machine path. Next add stable project-relative identity/root metadata and
manifest support, preserve names for noncolliding captures, and document
migration of the ambiguous baseline. Do not silently hash an absolute path or
copy the ambiguous PNG into two trusted baselines. Explicitly reviewing and
recreating qualified baselines is necessary before claiming completion.
