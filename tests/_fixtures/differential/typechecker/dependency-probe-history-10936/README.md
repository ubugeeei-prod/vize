# Dependency probe histories

Independent authored input for performance issue #10936. The reference revision is the
pre-optimization implementation, not a claim that this revision introduced a defect.
Runtime `.js` imports must prefer the authored `.ts` module, and `paths` fallback
must retain target precedence. Valid, invalid and repaired SFCs preserve complete
production diagnostics when reachability stops collecting discarded probe histories.
The separate Rust input-history tests pin every consulted candidate for freshness callers.
