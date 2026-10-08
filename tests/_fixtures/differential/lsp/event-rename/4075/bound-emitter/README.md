# Bound event casing editor contracts (#4075)

This test-only corpus preserves 20 independently authored cases across record,
call-signature, runtime-array, runtime-object and generic `defineEmits` forms.
Each form combines LF/CRLF with `nextEvent`/`next-event` replacements. The exact
33 cursor positions per case cover the declaration, the genuine bound emitter
call, and both camel/kebab listeners: 660 complete cursor triples in total.

`cases.json.txt` retains the complete source, references, WorkspaceEdit,
cursor and golden vectors from the pre-query independent oracle with SHA-256
`b334437925b1b18b863f70f2b01dccc7f8788769c5dd7b174547cce2f6be0db2`.
The readable LF inputs and goldens in each form directory are compiled into
the test and checked against all corresponding LF/CRLF case vectors.

The native stdio test requires the actual workspace runtime and frozen Vue
dependency. An independent TS2322 diagnostic and its repair prove the native
runtime is active before each case. All references and rename responses are
compared as complete envelopes, including request IDs. Definitions have 500
explicit contracts for Parent listeners and bound calls; the 160 declaration
origin definitions have no independently authored expected vector and remain
uncontracted observations. Neither URI aliases nor quoted ranges are accepted
by altering the supplied vectors.

The declaration-origin rename at query index 0 is selected before execution.
The test applies that genuine whole response to all three owned files and disk,
checks complete version-2 diagnostics, then independently installs the full
authored goldens and checks version-3 diagnostics. The unrelated component,
shadowed emitter, local handlers and native button listener remain unchanged.
Foreign or unsupported edit transactions are refused as a whole, without
filtering them into an apparent success.

Complete responses and all diagnostic publications, including additional
publications for the same URI/version, are retained under the separate
`bound-event-casing-transactions` capture namespace when capture is configured.
Existing original-report and library-report capture contexts are unchanged.

The historical candidate-H probe reproduced missing bound-call edits and
version-2 TS2345 in every case, while the independent version-3 goldens were
clean. That result applies only to H, the version-only child of signed parent
`177933f33e75af1f696fdc468665e7850025d28b`. This integration draft has not yet
executed against its current producer. It grants no current-source, protected
candidate, stock ContentMapper host, upstream fix or release qualification.
