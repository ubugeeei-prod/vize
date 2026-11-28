# Watermark behavior contract

Normative state × input → outcome table for `src/components/watermark/watermark.vue`.
Every row is proven by `src/components/watermark/tests/behavior.test.ts`.

| #   | State          | Input  | Outcome                                                                              | Proven by                                  |
| --- | -------------- | ------ | ------------------------------------------------------------------------------------ | ------------------------------------------ |
| W1  | text `DRAFT`   | render | overlay is `aria-hidden`, ignores pointers, and exposes the SVG tile on `data-image` | `repeats text in an inert overlay`         |
| W2  | slotted button | click  | the button remains a button and receives the click                                   | `keeps slotted controls interactive`       |
| W3  | text `A&B<C>`  | format | markup is escaped before it enters the SVG                                           | `escapes markup before building the image` |
