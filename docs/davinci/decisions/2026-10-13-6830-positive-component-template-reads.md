# Positive template component reads and editor completeness

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The release matrix reports used Tiny RDM and Frappe Builder imports as
unused. The existing lint path reuses a complete editor occurrence packet;
an unrelated `if`, `try`, class or throwing function refuses that packet,
which also discards separately proved runtime template reads.

Keep the editor packet's completeness contract unchanged. During the same
import and template AST walks, retain a separate positive channel for exact
runtime setup import declarations and complete runtime-only template reads.
Every read must retain its original declaration identity, source bytes and
lexical template owner. Parser failures, foreign or derived sources, external
blocks, unsupported languages, and setup generic metadata receive no credit.
Ordinary raw rule contexts cannot borrow SFC qualification.

This channel establishes only positive usage. It cannot establish an absence
of reads, provide editor rename/reference completeness, or accept type-only,
write-only, mixed or shadowed references. Native unused diagnostics and their
configuration keep their current behavior. The original editor refusal and
component runtime refusal corpora remain unchanged.

The new native differential corpus covers actual unchanged Tiny RDM and
Frappe Builder files, minimal unrelated script forms, and unread, property,
literal, object-key, loop, slot, callback, type and assignment controls.
Actions must verify these native cases and every existing refusal before
protected merge. Release and lint divergence budgets remain unchanged.

The finite current-source inventory updates only five changed owned rows and
its whole-manifest checksum. All 323 current paths, the 206-path union,
immutable original rows, historical laws and frames remain present. The
physical inventory check provides source identity only. Native Actions
independently executes the 17 cases, existing refusal corpora, and four
foreign/derived/generic/external qualification boundaries.

Before changing comparison scope, the complete 27-file native corpus executes
all 72 component-prop false positives from the original release matrix.
Its unchanged original files retain five genuinely unread setup imports and
one Pug registration warning. Every current public result field is written
under the existing Nextest artifact directory before assertions; no new CI
stage or source frame is introduced. The separate JSX PR executes the
remaining unchanged Douyin file.
