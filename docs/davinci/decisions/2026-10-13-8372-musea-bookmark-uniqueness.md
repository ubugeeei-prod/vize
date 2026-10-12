# Keep Musea options bookmarks unique

Issues: [#8372](https://github.com/ubugeeei-prod/vize/issues/8372) and
[#8365](https://github.com/ubugeeei-prod/vize/issues/8365).

The actual publication of `b87aff1d` from Docs run `38274286005` contained
both a span and a generated H2 with `id="musea-prefer-design-tokens"`.
The public reader check passed both English viewports, then rejected the
Japanese desktop page because that bookmark resolved to two elements.
The failed native receipt and original HTML remain evidence of the defect;
neither the successful whole-site build nor partial views prove delivery.

Remove only the redundant Japanese span. The existing H2 keeps the same
English bookmark. Retain the explicit English alias of the translated H1,
every paragraph, link, inline token and all three complete recipes.
The existing authored-source test keeps regeneration and recipe custody,
and the original rendered navigation job now checks both bookmarks for
uniqueness on the English and Japanese Musea options routes.

This corrects the duplicate-span implementation of the English bookmark
preservation required by the
[original translation decision](./2026-10-12-8372-japanese-musea-rule-options.md).
Other translation and CLI boundaries remain unchanged. Exact source checks,
protected merge, ordinary Docs publication and all four native public reader
views remain required. The broad documentation issues remain unfinished.
