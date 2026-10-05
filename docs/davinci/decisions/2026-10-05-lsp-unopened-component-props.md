# Public prop navigation includes unopened project consumers

Decision for [#7824](https://github.com/ubugeeei-prod/vize/issues/7824).

The current references default and canonical rename path open only reverse
importers already held in the editor. A public component prop can therefore
miss every unopened consumer even though the configured TS project includes it.
Use the existing on-demand configured-project Vue surface for public prop
references and rename, independently of the default `crossFile: false` policy.
Ordinary local bindings retain their current surface; script parameter shadows
are rejected by existing scope ownership. Imported component attributes enter
the same prop route. Existing TypeScript definition identity checks continue to
exclude unrelated same-spelling component props and governing tsconfig ownership
continues to exclude outside-project importers. Native HTML attributes stay local.

Discovery happens on explicit references/rename requests. No workspace rescan
is added to document edits, diagnostics, hover, completion or prepareRename.
This correction does not claim an instant response or a measured 10x gain;
query-wide project materialization cost remains a separately measured concern.

The paired corpus preserves complete reporter Child/Parent inputs and config
ownership. Fresh typed stdio tests compare complete ordered references and
WorkspaceEdit objects from Child declaration/interpolation and an opened parent
attribute, applying every edit to compare complete source bytes. The original
Child-only reproduction and Parent-open control stay separate. Negative and
positive cases retain aliases, static/bound props, foreign prop identity,
parameter shadows, non-code text, excluded importers, LF/CRLF, astral UTF-16
coordinates and unsaved Parent overlays. Complete initial diagnostic publications
must be empty. A pure admission law guards local/native attribute scope.

Exact-source Actions, unchanged protected instruction ceilings, actual merge,
release and installed editor proof remain pending. This is the current legacy
product correction without native or complete fix-history credit; #6883 stays open.
