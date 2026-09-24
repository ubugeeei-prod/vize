# AvatarGroup Behavior Contract

Normative state x input -> outcome table for `avatar-group.vue` and
`avatar-group-overflow.vue` (`@vizejs/ui/avatar-group`). Every row is proven by
the named test.

AvatarGroup renders a native `<ul>` of `<li>` tiles. The `item` slot renders each
visible person (typically an `Avatar`). `max` bounds every tile, including the
overflow tile. The overflow tile is `role="img"` with an accessible "N more"
name, and it wraps an `aria-hidden` `Avatar` showing "+N".

| ID   | State            | Input                     | Outcome                                                                                                   | Evidence                                                                  |
| ---- | ---------------- | ------------------------- | --------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| AG1  | no `max`         | render                    | a labelled list renders every item through the `item` slot, `data-state="expanded"`, and no overflow tile | `renders a labelled list of avatars without collapsing by default`        |
| AG2  | `max` exceeded   | render                    | `max - 1` items plus one overflow tile render; the tile is `role="img"` named "N more" showing "+N"       | `max counts the overflow tile and announces hidden people`                |
| AG3  | `total`          | render                    | people beyond `items` join the overflow count, with or without `max`                                      | `total adds server-side people to the overflow count`                     |
| AG4  | messages/spacing | render                    | overflow name and text come from typed `messages`; `spacing` publishes `--vize-ui-avatar-group-spacing`   | `localized messages, spacing, and empty groups`                           |
| AG5  | empty            | render                    | `data-state="empty"` and no tiles                                                                         | `localized messages, spacing, and empty groups`                           |
| AG6  | custom overflow  | `overflow` slot           | the slot receives typed hidden items, count, label, and text                                              | `the overflow slot receives hidden items for custom tiles`                |
| AG7  | reactive props   | `max`/`items` change      | the split re-computes; the instance exposes state, visible/hidden items, overflow count, and element      | `items and max are reactive and the instance exposes the split`           |
| AG8  | pure helpers     | `splitAvatarGroup()` etc. | invalid `max`/`total` never collapse; spacing rejects declaration injection                               | `splits groups, resolves messages, and sanitizes spacing`                 |
| AG9  | SSR              | isolated requests         | markup is byte-identical                                                                                  | `renders byte-identical avatar group markup across isolated SSR requests` |
| AG10 | SSR / hydration  | hydrate                   | server markup hydrates without warnings or node replacement                                               | `hydrates avatar group markup without warnings or node replacement`       |
| AG11 | types            | compile                   | item types flow from `items` into slots and exposes; states and messages are closed                       | `avatar-group.types.test-d.ts`                                            |
