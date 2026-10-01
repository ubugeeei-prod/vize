# Fallible stage inspection

Decision for #6832, 2026-10-02. Issue comment is paired at publication.

An admitted native binding need not have a compatibility dump representation.
Inspection must retain that typed refusal without panicking, inventing a valid
empty page, or changing the compiled module. L0 owns the generic sink contract;
it cannot import the native L2 dump error.

`CaptureSink::try_page` accepts a lazy `Result<String, E>` renderer whose error
implements `Display`. The existing infallible `page` API stays available.
`NoCapture` invokes neither renderer nor formatter and remains zero size. The
observing `StageCapture` calls the renderer once: success enters its existing
page sequence, while failure enters a separate normally owned
`StageCaptureUnavailable` sequence containing level, step and complete reason.
The original error is dropped normally after formatting. Neither these records
nor the capture sidecar enter an arena or travel between semantic levels.

Both page kinds remain provisional until the actual native module is accepted.
Every other final outcome clears failed inspections with successful pages,
timings and remarks. An accepted module can retain valid later pages beside a
failed earlier inspection. Recording no inspection failure leaves the new
sequence empty. This provider changes no host serialization or successful dump
bytes and adds no compiler stage, reparsing, extra scan or dependency edge.

The production source adds one ordinary owned failure-vector field and its
initializer to the existing contract inventory (alloc `Vec` bound uses 9 → 11).
Reasons use the foundation's compact string through generic formatting. No
storage or instruction ceiling is waived; the unchanged protected 100-probe
gate remains mandatory before merge.

Seven actual capture-module laws cover the old infallible API, lazy no-capture
rendering and formatting, exactly-once renderer/error drop, complete Unicode
failure reasons, later valid pages and all four nonaccepted outcomes. A direct
Rust harness compiles the actual capture source and test module against cached
real L0 exported primitives; strict production Clippy passes. This scoped
evidence does not claim a current whole-workspace Cargo build. Exact-head
Actions and terminal protected-queue results are publication prerequisites.

## Remaining work

- Native L2 compatibility dump conversion returns its actual typed error and
  implements `Display`; legacy expression aliases keep byte-exact dumps.
- Product and inspector host feeds serialize failed inspections separately,
  omit the field when empty and preserve existing successful feed bytes.
- Curator's independent ladder reports fallible boundary and pass observations
  without treating them as valid pages or production-stage evidence.
- Native For construction and component consumption keep their own #6838 and
  #6836 prerequisites. This capture leaf does not complete them or #6832.
