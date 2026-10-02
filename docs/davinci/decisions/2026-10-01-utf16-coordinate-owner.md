# UTF-16 coordinate ownership

Issue: [#6834](https://github.com/ubugeeei-prod/vize/issues/6834).

`LineIndex` belongs to L0 and must not import a host protocol module to return
its coordinates. L0 now owns `line_index::Position` and `line_index::Range`.
These retain the previous fields, derives, constructors, serialized field
names and UTF-16 units. The existing `lsp` path re-exports the same types, so
there is no conversion, new allocation, copied representation or changed
line/column calculation.

The migration law passes indexed positions through the old host path without
conversion, checks a non-ASCII/astral/CRLF source and pins the existing JSON
bytes for positions and ranges. Existing exhaustive byte-offset line-index
laws remain the oracle for mid-character and out-of-range behavior.

This is the provider slice for a dependent Carton host-module move. It does
not yet move LSP transport ownership, isolate all L0 platform support, switch
any product path or close #6834. Config, internationalization and profiler
host boundaries remain separate work; their shared vocabulary must not gain
a reverse dependency on legacy products.
