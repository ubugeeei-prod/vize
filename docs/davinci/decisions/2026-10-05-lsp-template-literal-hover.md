# Preserve native empty hover answers in template expressions

Decision for [#7916](https://github.com/ubugeeei-prod/vize/issues/7916).

The native checker returns empty quick info for string literal and template
static text, but canonical hover discarded that response as unavailability.
The existing template-word fallback then fabricated expression documentation,
including a `${` dollar delimiter in its word range. Preserve successful empty
quick info as an explicit native answer. Template hover returns null for that
answer; native typed hover and documentation surfaces keep their current paths.
Backend or mapping unavailability retains the existing fallback policy.

No spelling scan, new parser, legacy AST fallback, extra backend request or
edit-time discovery is introduced. The complete reporter StatusBadge source
and strict configuration remain a current-product corpus. Real typed stdio
asserts complete null at all four original coordinates and exact native type
and ranges at every `${tone}`. Positive and negative controls retain same-name
strings, comments, `${` delimiters, LF/CRLF, astral positions and complete
versioned empty diagnostic publications after unsaved changes and restore.

Exact-source Actions, unchanged protected instruction ceilings, actual merge,
release and installed editor proof remain required. This current legacy repair
claims no native stage, complete fix-history, instant response or 10x gain.

Replay on literal actual main after the first protected prefix merge preserves
every incoming canonical clause plus the complete reviewed production, original
corpus and typed stdio source vectors. Only canonical placement and this receipt
change. Fresh current-head source/full Actions and actual protected delivery
remain required; earlier successful original/extension stdio and reviewed source
receipts stay historical.
