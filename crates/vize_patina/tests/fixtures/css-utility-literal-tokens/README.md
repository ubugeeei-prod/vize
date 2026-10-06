These ten complete SFC inputs cover #7980 through the public Rust API and the
source-built CLI. The two original Vue blocks and JSON configuration are
copied byte-exact from the retained complete issue body. The CRLF original
differs only by newline encoding. All inputs and complete independently
authored API/JSON/plain output vectors are pinned in `cases.json`.

The controls preserve real exact and digit-prefix utility findings, their
whole authored spans, existing pattern order, existing boundary nonmatches
and inline disable behavior. Strings with escaped quotes, comment-looking
string data, nested functions/blocks, URLs, Unicode and multiple style blocks
exercise CSS token ownership. Neither source linter execution nor its output
was used to generate these new expected vectors.

This remains the existing heuristic class rule. The fix excludes comments
and literal token bytes using the existing CSS tokenizer; it does not broaden
the class policy to compound/pseudo/escaped selectors. No original historical
corpus or expected bytes are edited. Hosted source, protected instruction and
release verification remain required; preparation itself earns no native or
performance credit.
