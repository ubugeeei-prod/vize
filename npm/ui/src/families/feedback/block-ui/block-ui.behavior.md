# BlockUI Behavior Contract

Normative state x input -> outcome table for `block-ui.vue` (`@vizejs/ui/block-ui`).
Every row is proven by the named mounted-DOM, SSR, type, renderer, size, and
tree-shaking gates.

| ID  | State                       | Input / action     | Required outcome                                                                                                            | Evidence                                                                          |
| --- | --------------------------- | ------------------ | --------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| U1  | default idle                | render / Tab       | renders `<section data-vize-ui="block-ui">`, `part="root"`, idle/loading/none/off hooks, and no busy, inert, or ARIA policy | `renders an idle section by default without styling or accessibility policy`      |
| U2  | blocked + inert + polite    | render             | renders requested host with `aria-busy="true"`, native `inert`, `role="status"`, polite live region, and label              | `marks blocked inert regions busy and announces politely when labelled`           |
| U3  | attrs + blocked interaction | render / prop diff | passes unrelated attrs while component state owns `aria-busy` and native `inert`                                            | `owns busy and inert while leaving unrelated fallthrough attrs consumer owned`    |
| U4  | assertive / empty label     | prop diff          | uses `role="alert"` only with a non-empty label, then omits announcement attrs when the label is empty                      | `uses assertive announcement attrs only while announce and label are present`     |
| U5  | any                         | slot/expose        | passes blocked, state, reason, interaction, and announcement to the slot and exposes them live with the rendered element    | `passes slot state and exposes live block-ui state`                               |
| U6  | SSR blocked                 | isolated requests  | renders byte-identical blocked markup without request-global state                                                          | `renders byte-identical blocked markup across isolated SSR requests`              |
| U7  | SSR idle                    | isolated request   | renders idle markup without intrinsic busy, inert, or announcement attrs while preserving consumer attrs                    | `renders idle server markup without intrinsic busy, inert, or announcement attrs` |

## Props

| Prop          | Type                                                         | Behavior                                                         | Default     |
| ------------- | ------------------------------------------------------------ | ---------------------------------------------------------------- | ----------- |
| `as`          | `PrimitiveAs`                                                | Native element, custom element, or component to render.          | `"section"` |
| `blocked`     | `boolean`                                                    | Controls `data-state` and intrinsic busy/inert policy.           | `false`     |
| `reason`      | `"loading" \| "saving" \| "syncing" \| "stale" \| "offline"` | Consumer styling/status token mirrored to `data-reason`.         | `"loading"` |
| `interaction` | `"none" \| "inert"`                                          | Native inert policy applied only when `blocked` is true.         | `"none"`    |
| `announce`    | `"off" \| "polite" \| "assertive"`                           | Live-region policy applied only when `label` is non-empty.       | `"off"`     |
| `label`       | `string`                                                     | Accessible announcement label used when `announce` is not `off`. | `undefined` |

## Slots

| Slot      | Props                                                                                                                                  | Behavior                   | Default |
| --------- | -------------------------------------------------------------------------------------------------------------------------------------- | -------------------------- | ------- |
| `default` | `{ blocked: boolean; state: BlockUIState; reason: BlockUIReason; interaction: BlockUIInteraction; announcement: BlockUIAnnouncement }` | Render blocked UI content. | none    |

## Expose

| Name           | Type                     | Behavior                                     | Default     |
| -------------- | ------------------------ | -------------------------------------------- | ----------- |
| `element`      | `BlockUIElement \| null` | Rendered host element or component instance. | `null`      |
| `blocked`      | `boolean`                | Whether the region currently blocks work.    | `false`     |
| `state`        | `BlockUIState`           | Stable blocking token.                       | `"idle"`    |
| `reason`       | `BlockUIReason`          | Current reason token.                        | `"loading"` |
| `interaction`  | `BlockUIInteraction`     | Current interaction policy.                  | `"none"`    |
| `announcement` | `BlockUIAnnouncement`    | Current announcement policy.                 | `"off"`     |

## Attributes

| Attribute           | Value                     | Behavior                                                                                                   | Default     |
| ------------------- | ------------------------- | ---------------------------------------------------------------------------------------------------------- | ----------- |
| `part`              | `"root"`                  | Stable styling part.                                                                                       | always      |
| `data-vize-ui`      | `"block-ui"`              | Stable family selector.                                                                                    | always      |
| `data-state`        | `"blocked"`, `"idle"`     | Blocking state styling hook.                                                                               | `"idle"`    |
| `data-reason`       | `BlockUIReason`           | Reason styling/status hook.                                                                                | `"loading"` |
| `data-interaction`  | `BlockUIInteraction`      | Interaction policy hook.                                                                                   | `"none"`    |
| `data-announcement` | `BlockUIAnnouncement`     | Announcement policy hook.                                                                                  | `"off"`     |
| `aria-busy`         | `"true"`                  | Present only while blocked.                                                                                | `undefined` |
| `inert`             | native boolean attribute  | Present only while blocked and `interaction` is `"inert"`.                                                 | `undefined` |
| `role`              | `"status"`, `"alert"`     | Bound only when `announce` and a non-empty `label` request it; ordinary fallthrough attrs may override it. | `undefined` |
| `aria-live`         | `"polite"`, `"assertive"` | Mirrors active announcement politeness; ordinary fallthrough attrs may override it.                        | `undefined` |
| `aria-label`        | `string`                  | Uses the active announcement label; ordinary fallthrough attrs may override it.                            | `undefined` |

BlockUI emits no visual CSS, generates no ids, owns no focus trap, and stores no
request-global state.
