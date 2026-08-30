# Cluster behavior contract

Normative state x input -> outcome table for `cluster.vue` (`@vizejs/ui/cluster`).
Every row is proven by the named mounted-DOM or SSR test. A row without a
passing test is a contract violation.

| #   | State         | Input             | Outcome                                                                                     | Proven by                                                                     |
| --- | ------------- | ----------------- | ------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| C1  | default       | resolve           | maps default inline flow to wrapping flexbox with `gap` resolved to `0` and no CSS classes  | `resolves a wrapping inline cluster with no authored CSS classes`             |
| C2  | nowrap        | resolve           | maps reversed nowrap flow to `row-reverse`, `nowrap`, and numeric px gap values             | `resolves reversed nowrap flow with native logical alignment values`          |
| C3  | default       | render            | renders a non-focusable `<div>` cluster with part, data hooks, default gap, and slots       | `renders a non-focusable cluster by default while preserving child semantics` |
| C4  | custom host   | render            | forwards host attributes and renders nowrap reversed cluster data hooks                     | `renders nowrap reversed flow on a custom semantic host`                      |
| C5  | any           | slot/expose       | exposes `element`, `wrap`, `reversed`, `direction`, `wrapMode`, `gap`, alignment, and state | `passes slot state and exposes live resolved layout state`                    |
| C6  | SSR           | isolated requests | renders byte-identical flex cluster markup without request-global state                     | `renders byte-identical cluster markup across isolated SSR requests`          |
| C7  | DOM/SSR/Vapor | compile           | authored SFC compiles in every renderer lane without warnings or fallback                   | `scripts/check-renderers.ts`                                                  |
| C8  | root/subpath  | consumer bundle   | root and subpath consumers retain only Cluster, emit no CSS, and stay within gzip budget    | `scripts/check-tree-shaking.mjs`                                              |

## Props

| Prop       | Type                                                                                  | Purpose                                                 | Default     |
| ---------- | ------------------------------------------------------------------------------------- | ------------------------------------------------------- | ----------- |
| `as`       | `PrimitiveAs`                                                                         | Native element, custom element, or component host.      | `"div"`     |
| `gap`      | `string \| number`                                                                    | Native CSS `gap` value; numbers resolve to px lengths.  | `0`         |
| `align`    | `"stretch" \| "start" \| "center" \| "end" \| "baseline"`                             | Native CSS `align-items` value for the cross axis.      | `"stretch"` |
| `justify`  | `"start" \| "center" \| "end" \| "space-between" \| "space-around" \| "space-evenly"` | Native CSS `justify-content` value for the inline axis. | `"start"`   |
| `wrap`     | `boolean`                                                                             | Allow items to wrap onto additional lines.              | `true`      |
| `reversed` | `boolean`                                                                             | Reverse inline item flow without changing DOM order.    | `false`     |

## Slots

| Slot      | Props              | Purpose                          | Default |
| --------- | ------------------ | -------------------------------- | ------- |
| `default` | `ClusterSlotState` | Renders direct cluster children. | empty   |

## Expose

| Name        | Type                     | Purpose                                      | Default       |
| ----------- | ------------------------ | -------------------------------------------- | ------------- |
| `element`   | `ClusterElement \| null` | Rendered host element or component instance. | `null`        |
| `wrap`      | `boolean`                | Whether items can wrap.                      | `true`        |
| `reversed`  | `boolean`                | Whether inline flow is reversed.             | `false`       |
| `direction` | `ClusterFlexDirection`   | Resolved CSS flex direction.                 | `"row"`       |
| `wrapMode`  | `ClusterFlexWrap`        | Resolved CSS flex wrapping mode.             | `"wrap"`      |
| `gap`       | `ClusterResolvedGap`     | Resolved CSS gap value.                      | `"0"`         |
| `align`     | `ClusterAlign`           | Resolved cross-axis alignment.               | `"stretch"`   |
| `justify`   | `ClusterJustify`         | Resolved inline-axis distribution.           | `"start"`     |
| `state`     | `"clustered"`            | Stable layout state token.                   | `"clustered"` |

## Data Attributes

| Attribute                     | Values                 | Purpose                       | Default       |
| ----------------------------- | ---------------------- | ----------------------------- | ------------- |
| `data-vize-ui`                | `"cluster"`            | Stable family selector.       | always        |
| `data-state`                  | `"clustered"`          | Cluster layout state.         | `"clustered"` |
| `data-wrap`                   | `"true"`, `"false"`    | Wrapping styling hook.        | `"true"`      |
| `data-reversed`               | `"true"`, `"false"`    | Reverse-flow styling hook.    | `"false"`     |
| `data-align`                  | `ClusterAlign`         | Resolved alignment hook.      | `"stretch"`   |
| `data-justify`                | `ClusterJustify`       | Resolved justification hook.  | `"start"`     |
| `data-vize-cluster-direction` | `ClusterFlexDirection` | Resolved flex direction hook. | `"row"`       |
| `data-vize-cluster-gap`       | `ClusterResolvedGap`   | Resolved gap hook.            | `"0"`         |

## ARIA Attributes

Cluster never sets `role`, `aria-hidden`, `aria-label`, `aria-labelledby`, or
`tabindex`. Consumers may pass semantic attributes to the host when the chosen
element requires them.

## CSS Custom Properties

| Property                    | Purpose                                   | Default     |
| --------------------------- | ----------------------------------------- | ----------- |
| `--vize-ui-cluster-gap`     | Value read by the host `gap`.             | `"0"`       |
| `--vize-ui-cluster-align`   | Value read by the host `align-items`.     | `"stretch"` |
| `--vize-ui-cluster-justify` | Value read by the host `justify-content`. | `"start"`   |

## Parts

| Part   | Purpose               | Default |
| ------ | --------------------- | ------- |
| `root` | Single rendered host. | always  |
