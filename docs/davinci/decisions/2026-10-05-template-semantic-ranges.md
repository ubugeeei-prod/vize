# Original template expression token and comment boundaries

Issue [#8014](https://github.com/ubugeeei-prod/vize/issues/8014) reports three
complete inputs at version 0.432.0: App.vue, Multi.vue and Child.art.vue. The
corpus retains the three entire code fences, with their byte lengths and SHA-256.
Authored complete token vectors are expectations, not old runtime observations.

Extend the existing structural expression tokenizer with disjoint original-byte
code, string/quasi and comment regions. Comments receive comment tokens rather
than identifier/operator tokens. Template substitutions remain code, including
nested template literals and object braces. Original string/comment spans are
split at physical LF/CRLF into nonempty UTF-16 tokens. Always emitting single-line
spans also supports clients that advertise multilineTokenSupport: false; the
[official LSP 3.17 semantic-token specification](https://github.com/microsoft/language-server-protocol/blob/gh-pages/_specifications/lsp/3.17/language/semanticTokens.md)
requires positions and lengths in the negotiated encoding and explains that an
unsupported multiline token is clipped at its starting line.

Definition and hover reject original comment positions before a checker or
structural fallback can resolve words from the comment. Both synchronous and
native async entry points use the same lexical authority. The existing directive
selector now returns its original expression start without changing its boolean
completion routing. Script navigation, provider maps and component hover
presentation remain with their current owners. Existing mustache/dynamic-argument
boundary heuristics remain bounded; this does not claim a complete JS parser or
replace the retained LSP with a native product. No AST parse per token, checker IPC,
new dependency version, serialization or compiler stage is added. The sole
manifest/lock edge is htmlize::Context from the already locked 1.1.0 dependency:
use the existing native L1 compatibility entity decoder to shield original
HTML-encoded quote data once, preserving the existing entity token wire. Plain
JSX and authored JavaScript regions do not decode HTML references. The existing public OXC
lexer constructor is private; its unsafe benchmarking backdoor is not used.

Art opening/closing tag tokens address only art or variant, preserving type,
modifiers and every other tuple. Regular and inline art share this collector.
Unrelated identifier, property, function, number, string and operator controls
retain exact full vectors. Lexer controls cover real comments, quoted comment
data, regexp classes/escaped slashes, postfix division, nested substitutions,
unclosed regions and astral text. This lexical structural path does not claim
full grammar-sensitive regular-expression classification.

The new PR-selected real stdio suite advertises multilineTokenSupport: false,
checks entire full/range token payloads for all three original documents under
both typecheck settings, and retains original plus LF/CRLF/astral whole vectors.
Comment punctuation/words must yield literal null hover and definition; adjacent
real identifiers retain exact definitions. Existing source-built session capture
retains whole frames and actual CLI/build/source hashes. Those observations remain
pending session evidence, with zero native whole-product/history closure credit.

Source review, original input custody and authored expectations do not certify
execution. Exact-head normal Actions and protected full Rust/104 ceilings are
required before actual signed merge, literal reporter trailer and release handoff.
No performance budget or frozen history expectation is changed. #6883 remains
open; this fixture is an additive legacy correction rather than native completion.
