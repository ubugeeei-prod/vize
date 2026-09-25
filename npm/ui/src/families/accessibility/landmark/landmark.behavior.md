# Landmark behavior contract

Normative behavior for `@vizejs/ui/landmark`. `landmark.vue` renders native landmark
elements; `landmark-provider.vue` and `useLandmarkNavigation` add F6 / Shift+F6 landmark
cycling, the convention browsers and desktop apps use to move between page regions. Every
row is proven by the named test.

`role="search"` renders the native HTML `<search>` element (implicit `search` landmark);
older engines that do not map it still expose its `aria-label`. `region` and `form` are only
exposed as landmarks when named, and `header`/`footer` inside sectioning content are not
`banner`/`contentinfo`.

| State x input                                           | Observable outcome                                                                                                                                                              | Proven by                                                                       |
| ------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| `landmark.vue` render                                   | Renders `header`/`nav`/`main`/`aside`/`footer`/`section`/`form`/`search` with a deterministic id, `data-vize-ui="landmark"`, and `data-landmark`; no `tabindex`.                | `renders native landmark elements with names, ids, and data hooks`              |
| `region`, `search`, `form`, `complementary`             | Render `section`, `search`, `form`, and `aside`.                                                                                                                                | `region, search, and form landmarks render section, search, and form elements`  |
| repeatable role without `ariaLabel`/`ariaLabelledby`    | Development builds warn `VIZE_UI_LANDMARK_NAME`; `main` never warns.                                                                                                            | `unnamed repeatable landmarks warn in development`                              |
| F6 / Shift+F6 inside `LandmarkProvider`                 | Focus moves to the next/previous landmark in document order with wrapping, starting from the landmark containing focus; `navigate` fires.                                       | `F6 and Shift+F6 cycle focus through landmarks in document order with wrapping` |
| landmark focused by cycling                             | It gets `tabindex="-1"` and `data-focused="true"`; the temporary `tabindex` is removed when focus leaves.                                                                       | `F6 and Shift+F6 cycle focus through landmarks in document order with wrapping` |
| landmark inside `[hidden]`, `[inert]`, or `aria-hidden` | Skipped by cycling.                                                                                                                                                             | `hidden and inert landmarks are skipped`                                        |
| `discover`                                              | Native and `[role]` landmarks not rendered by `Landmark` join the cycle; unnamed sections and article headers do not.                                                           | `discovery includes native and role landmarks that Landmark did not render`     |
| `disabled`, remapped or `null` keys; expose             | F6 is ignored when disabled; custom bindings replace F6; `focusLandmark(id \| role)`, `focusNext`, and `focusPrevious` work programmatically.                                   | `expose focuses by id or role and disabled or remapped keys are respected`      |
| `Landmark` without a provider                           | Renders and focuses through its expose without registration.                                                                                                                    | `landmarks without a provider still render and focus through expose`            |
| runtime helpers and effect scopes                       | `landmarkRoleOf` follows HTML-AAM mappings; `useLandmarkNavigation` attaches in a client effect scope and detaches on stop; outside a scope it throws `VIZE_UI_LANDMARK_SETUP`. | `runtime helpers resolve roles and work in a plain effect scope`                |
| SSR                                                     | Byte-identical native landmark markup with no `tabindex` or listeners.                                                                                                          | `renders byte-identical native landmark markup across SSR requests`             |
| hydration                                               | Server elements are reused with zero warnings; keyboard cycling starts after mount.                                                                                             | `hydrates landmarks without diagnostics and cycles only after mount`            |
| public types                                            | Roles, named roles, element map, and key bindings are closed contracts.                                                                                                         | `src/families/accessibility/landmark/landmark.types.test-d.ts`                  |
| DOM/SSR/Vapor                                           | `landmark.vue` and `landmark-provider.vue` compile in every renderer lane.                                                                                                      | `scripts/check-renderers.ts`                                                    |

| Target   | Public contract                                                                                    |
| -------- | -------------------------------------------------------------------------------------------------- |
| Landmark | native landmark element, `part="root"`, `data-vize-ui="landmark"`, `data-landmark`, `data-focused` |
| Provider | renderless; publishes the landmark registry through `landmarkContext`                              |

The family ships no stylesheet.
