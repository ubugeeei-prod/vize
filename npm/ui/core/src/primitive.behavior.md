# Primitive behavior contract

Normative state × input → outcome table for `primitive-element.vue` (`@vizejs/ui/primitive`).
Every row is proven by the named mounted-DOM test in `src/primitive.test.ts`.

| #   | State            | Input  | Outcome                                                     | Proven by                                            |
| --- | ---------------- | ------ | ----------------------------------------------------------- | ---------------------------------------------------- |
| P1  | `as="section"`   | render | renders `<section data-vize-ui="primitive">` with slot text | `renders the requested element with slotted content` |
| P2  | default          | render | renders `<div>`; exposed `element` is the rendered node     | `defaults to a div and exposes the rendered element` |
| P3  | `as` = component | render | every named slot is forwarded to the target component       | `forwards every named slot to a component target`    |
