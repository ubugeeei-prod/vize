# Japanese Markdown emphasis

Issue: [#8373](https://github.com/ubugeeei-prod/vize/issues/8373).

Japanese translations joined body text to punctuation at the end of an
emphasis span, for example `**⚠️ 進行中の作業:**Vize`. The native Ox Content
renderer correctly treats this as literal text under Markdown's delimiter
rules. Add a space only at invalid punctuation boundaries in translated
prose. Preserve fenced and inline source, HTML attributes, URL destinations,
and authored links. Apply the same normalization after every translation
provider and in the existing normalization-only generation command.

The audit covers all 394 shipped Japanese Markdown pages. Fifteen pages
require boundary corrections; two old escaped, malformed emphasis spans in
Comment Annotations and JSX are corrected at their authored inputs. The
native-renderer regression checks every Japanese page for unrendered strong
delimiters outside literal source, and verifies representative headings,
warnings, inline strong/emphasis, and unchanged supported-locale controls.

This fixes rendering, not the separate site-wide Japanese wording review
in [#8372](https://github.com/ubugeeei-prod/vize/issues/8372). Exact-head
Actions, docs SSG, and deployed representative reading checks remain
required before treating the issue as delivered.
