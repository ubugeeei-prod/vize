# Complete Docs capture proof and operation timings

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The successful Full Docs run
[38113866192](https://github.com/ubugeeei-prod/vize/actions/runs/38113866192)
used source `2ea3fd78e0b0337b33d94fe6ac55fc72a655870a` and passed all
113 ordered routes / 226 default route-device jobs with two workers.
General navigation took 8457.812 seconds. Initial and theme captures used
15111.092 of 16848.069 job worker-seconds, 89.69%; worker-seconds sum
parallel elapsed operations and are not CPU measurements.

The original records only measure whole capture phases. They do not separate
snapshot collection, layout comparison, screenshot preparation, encoding,
transfer, file writes, and restoration. These measurements prove capture
dominates this exact source. They do not prove any internal cause or a speedup
for a different source or route manifest.

## Decision

The complete before/after node packets stay inside Chromium, owned by the
capture closure. Every node identity, source string, bounding rectangle and
text-range rectangle remains in the original comparison. Only primitive
LayoutProof fields return to Node; there is no packet structured clone into
the host realm.

The owned native DOM producer creates private plain records and dense number
arrays. With the native DOM/JSON/array intrinsics used by these first-party
Docs pages, page code cannot access or mutate the private packet graph during
the digest awaits. Serialize each complete packet once, at its original point:
before serialization, before digest await, after serialization, after digest
await. Reuse those exact two strings for the original complete equality check.
The SHA inputs, field/property/array order and returned proof remain exact.

This is an internal owned-page contract. A stateful inherited `toJSON` hook or
patched JSON/DOM intrinsic is ineligible: four observable serializations can
differ from two. The regression explicitly demonstrates that boundary instead
of claiming unrestricted scripted-document equivalence. The actual first-party
callers and page producers have no such hooks. Reassess eligibility before
introducing an untrusted scripted caller; use the original four-call path for
that caller if it can change serialization semantics.

Add optional host-only operation timings with `--capture-timings`. The default
is off. Existing page operations, awaits, failure precedence and finally
cleanup remain; timings never add browser waits or override either clock.
The optional `capture_timings` boolean dispatch input forwards this flag only
to the original general-navigation command. It defaults to false; the sole
Docs publisher enables it for the next already-required actual-head Full.
Primitive records accumulate in memory. A separate `capture-timings.json` is
written after the original receipt/timings. A sidecar write failure cannot
replace an original capture failure; when rendering succeeded it fails the
requested instrumentation rather than minting missing evidence.

Measure original preparation, complete layout-before/after proof, viewport
changes, fallback proof, every tile position, PNG validation/hash, complete
coverage and restoration. `screenshot-call-inclusive-write` remains the
original `page.screenshot({ path, fullPage, animations: "disabled" })` call.
Its duration includes Playwright preparation, encoding, transfer and PNG
writing. It cannot honestly isolate a PNG-write or font-settlement duration.
Failed cleanup leaves an incomplete metric marked failed with unknown duration.

Keep two workers, every route/device/theme/scope, complete corpus and control,
all screenshot paths/options, legal viewport proof, fallback, 128px overlap,
the original 1px budget and all deadlines. No sampling or capture caps.
Prior rejected four-worker PNG/resource evidence supplies no waiver.

## Validation and remaining evidence

The source-extracted browser proof regression is RED before reuse and PASS
afterward. It checks exact UTF-16/UTF-8 bytes and hashes, attribute and array
order, node identity, DOM mutation, text-only 1px negatives, and serial digest
failure order against the original literal implementation. The optional timing
regression verifies identical original page calls, options, receipts, rejection
identity and finally operations with instrumentation enabled or disabled.

Integrate this small private slice through the existing coherent Docs Stack,
then use its next already-required actual-head Full Docs execution. Authenticate
that literal complete route/device manifest and all original native/page/control
receipts before interpreting the new sidecar. No separate render campaign or
independent PR is needed. Hosted improvement and full-source acceptance remain
unmeasured until that execution supplies genuine terminal evidence.
