# Diff View behavior contract

Normative state × input → outcome table for `src/components/diff-view/diff-view.vue`.
Every row is proven by `src/components/diff-view/tests/behavior.test.ts`.

| #   | State            | Input  | Outcome                                                              | Proven by                                                 |
| --- | ---------------- | ------ | -------------------------------------------------------------------- | --------------------------------------------------------- |
| F1  | unified patch    | parse  | headers are meta, space lines are context, `+` adds, `-` removes     | `parses a unified diff without treating headers as edits` |
| F2  | patch prop       | render | table named by `label`; each line is a row whose `data-kind` matches | `renders a patch as a named table`                        |
| F3  | structured lines | render | `lines` wins over `patch`                                            | `prefers structured lines over a patch`                   |
| F4  | empty patch      | render | a table with no cells                                                | `renders an empty patch as a table without cells`         |
