# Docs navigation motion and readable surfaces

Issues: [#8376](https://github.com/ubugeeei-prod/vize/issues/8376),
[#8375](https://github.com/ubugeeei-prod/vize/issues/8375),
[#8368](https://github.com/ubugeeei-prod/vize/issues/8368),
[#8369](https://github.com/ubugeeei-prod/vize/issues/8369), and
[#8362](https://github.com/ubugeeei-prod/vize/issues/8362).

## Decision

Philosophy belongs after Getting Started in Start. Content Mapper belongs in
static analysis, beside comment directives and editor integration, because its
audience integrates Vue source with a native TypeScript host. The shared sitemap
applies both placements in every locale while retaining existing page URLs.
Each Content Mapper introduction links ordinary project checking and editor setup.
Localized CLI check sections retain an explicit `check` anchor: the prior mapper
links used that stable fragment while translated heading slugs did not provide it.

Keep native details/summary semantics for sidebar groups. Short CSS size and
chevron transitions respect reduced motion; browsers without intrinsic-size
interpolation retain functional native disclosure.
Each closed group immediately makes its list inert, so links painted during
the collapse transition cannot retain focus or re-enter the keyboard sequence.
The mobile sheet animates its existing transform. A closed sheet is inert immediately, its controls expose
the actual expanded state, and Escape closes it and returns focus to its trigger.
Escape belongs to the sidebar only when its event originates there or at its
active trigger and another widget has not consumed it. Closing search must keep
the sidebar open and preserve search's own focus behavior.

Deepen the warm light background and its alternate/border surfaces by a few RGB
steps. Dark page and alternate surfaces likewise deepen slightly. Syntax colors
remain distinct and reach at least 4.5:1 against the composited code background,
including native annotation backgrounds. Dim annotations lose blur and opacity
reduction; their source and annotation meaning remain intact.
Native dimmed syntax tokens inherit the readable muted line color while focused
tokens retain their syntax palette. The verifier builds its native annotation
fixture only when it checks theme readability, keeping unrelated imports inert.
Dark comments also need sufficient contrast. Benchmark provenance tables wrap hashes only inside
their dedicated disclosure, preserving ordinary table/code layout.

## Validation and delivery

The locale navigation test pins both placements and original hrefs. The existing
Docs browser gate visits both preserved routes in all five locales. It measures
real intermediate disclosure geometry, keyboard focus, mobile Escape/inert/ARIA,
reduced motion, and syntax contrast against computed composited backgrounds.
Representative entry, setup, Content Mapper and rule pages capture both palettes
on desktop/mobile. Existing source, links, anchors and screenshot checks remain.

Exact-head Check and Docs Actions, protected queue delivery, deployment and live
browser verification are required before closing these issues. Built previews
and source-only tests do not establish deployed acceptance.
