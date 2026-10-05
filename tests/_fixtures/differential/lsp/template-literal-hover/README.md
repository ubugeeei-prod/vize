# Native empty hover preserves literal ownership

The complete StatusBadge source is copied byte-for-byte from reporter #7916.
The real typed stdio fixture owns its temporary URI and uses the retained Vue
runtime and strict bundler configuration. The input is never reconstructed from
an implementation output.

Literal text and `${` delimiters must return complete JSON `null`; native types
and exact authored ranges remain required for every interpolated `tone`.
The additional same-spelling strings, comments, LF/CRLF, astral prefix and
versioned unsaved edits retain the same independent contract. Backend empty
quick info must not fall through to a fabricated template-word hover.
