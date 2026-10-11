# Native Musea failed-response evidence

Related issue: [#8482](https://github.com/ubugeeei-prod/vize/issues/8482).

The original native inline Art test rejected actual CSS, Bash and JSON dependency
responses with status 504 on multiple exact Actions sources. Existing receipts
retain status and URL, but lack the error body, reason and request timing.
Those failures remain failures; their underlying cause is unknown.

Keep the existing response ledger, every native rendering/document assertion,
the final status < 400 oracle, original deadlines and workflow unchanged.
An additive collector retains status text, event/body-read timestamps, request
timing and only content-type/content-length/etag/cache-control response headers.
It removes URL queries and never reads request headers, cookies or authorization.

Read the same failed Response body only after its actual requestfinished event,
with decimal content-length at most 16 KiB and at most 64 KiB reserved overall.
Unknown lengths, incomplete requests and oversize bodies retain explicit refusals.
Collection/API failures remain explicit errors. Base64, byte counts and SHA-256
preserve admitted whole bodies. Playwright buffers Response.body(): these are
retained-byte limits, not streaming or peak-allocation guarantees.
The additive JSON is capped at 256 KiB; a failure-only stdout summary is capped
at 8 KiB and labels every body preview as a prefix.

Start a read-only snapshot of resolved optimizeDeps inputs, optimizer cache
metadata and the two actual highlighting import producers. This snapshot neither
gates first navigation nor imports, fetches, prewarms or waits for an optimizer.
Record absent startup cache metadata.

Detach/drain owned event tasks before retaining failed-responses.json.
Preserve the original thrown assertion if collection also fails, log the additive
error explicitly with a failing process status, and attempt original cleanup.
If the original law passed, throw a deferred collection failure after cleanup. Pure real HTTP 503/504, cap and read-error
controls qualify this source helper; they do not reproduce a browser optimizer
cause or qualify a native renderer. Fresh Actions must run the unchanged native
law before runtime acceptance.
