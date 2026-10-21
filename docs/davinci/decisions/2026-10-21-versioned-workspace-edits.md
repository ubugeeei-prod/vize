# Versioned workspace edits

Issue: [#6876](https://github.com/ubugeeei-prod/vize/issues/6876).

The retained L1 edit provider now exposes a non-allocating current-snapshot
check for publication adapters. Maestro's experimental source-project adapter
uses that check under the actual open-document read guard, then converts the
same ordered `SpanEdit` set into one versioned LSP `TextDocumentEdit`.

Ranges are derived from the retained authored source in UTF-16 coordinates.
The adapter emits neither the unversioned `changes` form nor implicit change
annotations, and it does not mutate the document. Foreign documents, stale
revisions, reopened documents, equal copied buffers and invalid edit sets keep
the shared provider's existing refusals before any workspace edit is returned.

This is a provider-to-LSP boundary, not a claim that every code-action or
Patina fix producer has migrated. Existing product routes remain unchanged;
their migrations and fix-history gates still belong to #6876 and #6881.
