# NotificationCenter behavior contract

Normative behavior for `@vizejs/ui/notification-center`: an SSR-safe notification history
store and an APG [feed](https://www.w3.org/WAI/ARIA/apg/patterns/feed/) inbox. It can serve
as toast history by feeding it through `connectNotificationSource`. Every row is proven by
the named test.

| State x input                                       | Observable outcome                                                                                                                                                                                               | Proven by                                                                       |
| --------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| store `add`                                         | Notifications are frozen snapshots, newest first, with `"{idPrefix}-{n}"` ids and timestamps from the injected `now`.                                                                                            | `adds notifications newest first with generated ids and injected timestamps`    |
| store `add` with an existing id                     | The notification is replaced in place.                                                                                                                                                                           | `re-adding an id replaces it in place`                                          |
| `markRead`/`markUnread`/`update`/`archive`/`remove` | Each reports whether anything changed; archived notifications leave the feed and the unread count.                                                                                                               | `read, archive, update, and remove report whether anything changed`             |
| `markAllRead`                                       | Only active unread notifications change; the count is returned.                                                                                                                                                  | `markAllRead only touches active unread notifications`                          |
| `groups`                                            | Active notifications grouped by key in first-appearance order with unread counts.                                                                                                                                | `groups preserve first appearance order with per-group unread counts`           |
| `maxLength` exceeded                                | The oldest notifications are evicted.                                                                                                                                                                            | `maxLength evicts the oldest notifications`                                     |
| `initial`, `clear`                                  | Seeds are sorted by `createdAt`; `clear` empties the store.                                                                                                                                                      | `initial notifications and clear`                                               |
| `connectNotificationSource`                         | Source emissions are added until the returned function unsubscribes.                                                                                                                                             | `connectNotificationSource feeds the store until unsubscribed`                  |
| invalid option or type                              | Throws `VIZE_UI_NOTIFICATION_OPTION`.                                                                                                                                                                            | `invalid options and inputs throw diagnostics`                                  |
| disclosure trigger click                            | The trigger toggles `aria-expanded`, controls the feed id, names the unread count, and the feed stops being `hidden`.                                                                                            | `trigger discloses the feed and announces the unread count`                     |
| rendered feed                                       | `role="feed"` with `aria-label`/`aria-busy`; articles are focusable with `aria-posinset`, `aria-setsize`, title/description wiring, `data-read`, `data-type`, `data-group`; inline centers omit `aria-expanded`. | `feed renders APG articles with position, size, labels, and read state`         |
| PageDown / PageUp inside an article                 | Focus moves to the next or previous article; at the edges focus stays and the key is not consumed.                                                                                                               | `PageDown and PageUp move focus between articles`                               |
| click or Enter on an article                        | The notification is marked read and `activate` fires unless `markReadOnActivate` is false; clicks on inner controls are ignored.                                                                                 | `activating an article marks it read unless opted out`                          |
| Escape inside a disclosure feed                     | The feed closes and focus returns to the trigger.                                                                                                                                                                | `Escape closes a disclosure feed and returns focus to the trigger`              |
| custom `item` slot                                  | Consumers render `NotificationCenterItem` with typed data and receive ids plus `markRead`/`archive`/`remove` actions.                                                                                            | `custom item slots receive typed actions and ids`                               |
| root-owned store                                    | The root creates, exposes, and injects its store; `useNotificationCenter()` reads it; the empty state shows when nothing remains.                                                                                | `root owns a store, exposes it, and useNotificationCenter reads it`             |
| parts or composable outside the root                | Throws `VIZE_UI_CONTEXT_MISSING`.                                                                                                                                                                                | `parts and the composable require a NotificationCenterRoot`                     |
| SSR                                                 | Isolated requests render byte-identical feeds with deterministic ids.                                                                                                                                            | `renders byte-identical feed markup across isolated SSR requests`               |
| hydration                                           | The feed hydrates without warnings or node replacement.                                                                                                                                                          | `hydrates the feed without diagnostics`                                         |
| public types                                        | `Data` flows through records, slots, expose, and `useNotificationCenter<Data>()`; types are closed unions.                                                                                                       | `src/families/feedback/notification-center/notification-center.types.test-d.ts` |

## Components

| Component                         | State x input                       | Outcome                                                                             |
| --------------------------------- | ----------------------------------- | ----------------------------------------------------------------------------------- |
| `notification-center-root.vue`    | `store`, `inline`, `open`           | Creates or adopts the store, owns disclosure state, provides context and the store. |
| `notification-center-trigger.vue` | click                               | Toggles the feed and labels itself with the unread count.                           |
| `notification-center-list.vue`    | notifications, PageUp/PageDown, Esc | Renders the `role="feed"` element and one article per active notification.          |
| `notification-center-item.vue`    | click, Enter                        | Renders one `article` with feed positions and marks it read on activation.          |
| `notification-center-empty.vue`   | no active notifications             | Shows the empty-state text; hidden otherwise.                                       |

## Parts And Data

| Target  | Public contract                                                                                            |
| ------- | ---------------------------------------------------------------------------------------------------------- |
| Root    | `data-vize-ui="notification-center-root"`, `part="root"`, `data-state`, `data-inline`, `data-unread-count` |
| Trigger | `data-vize-ui="notification-center-trigger"`, `part="trigger"`, `data-unread-count`, `data-has-unread`     |
| List    | `data-vize-ui="notification-center-list"`, `part="list"`, `role="feed"`, `data-count`                      |
| Item    | `data-vize-ui="notification-center-item"`, `part="item"`, `data-read`, `data-type`, `data-group`           |
| Empty   | `data-vize-ui="notification-center-empty"`, `part="empty"`                                                 |

The feed pattern's Control+Home / Control+End exits are left to the surrounding page.
NotificationCenter ships no stylesheet.
