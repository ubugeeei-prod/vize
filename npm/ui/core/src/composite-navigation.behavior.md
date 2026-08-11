# Composite navigation behavior contract

`createCompositeNavigation` adapts one `CollectionRegistry` to either WAI-ARIA focus-management
strategy. It owns logical navigation and DOM focus representation; roles, labels, selection,
activation, and styling remain consumer-owned.

| Concern              | Contract                                                                                                                |
| -------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| Server rendering     | Construction and prop reads require no browser globals. IDs come from consumer data, not request-global counters.       |
| Hydration            | Server markup hydrates in place; stable prop handlers begin navigation without replacing the host.                      |
| Vapor                | The public composable fixture must compile in native DOM, SSR, and Vapor lanes without diagnostics.                     |
| Roving focus         | Exactly the effective active item receives `tabindex="0"`; all other registered items receive `-1`.                     |
| Active descendant    | The container receives `tabindex="0"` and the active item's validated, stable ID. Items keep DOM focus off themselves.  |
| Initial state        | Before explicit activation, the first navigable item is the effective tab stop or active descendant.                    |
| Disabled items       | Registry policy determines navigability. Default `skip` behavior omits disabled items from every command and typeahead. |
| Arrow keys           | Orientation gates horizontal and vertical arrows. Horizontal movement reverses under reactive RTL direction.            |
| Boundaries           | `loop` affects arrows only; Home, End, PageUp, and PageDown remain deterministic.                                       |
| Paging               | `pageSize` counts navigable items, never disabled records, and clamps at collection boundaries.                         |
| Editable descendants | Keyboard events from text-entry inputs, selects, textareas, or contenteditable descendants are not consumed.            |
| Modified input       | Composition and Alt, Control, or Meta shortcuts are not interpreted as navigation.                                      |
| Typeahead            | Optional Unicode-aware typeahead commits into the same registry state and synchronizes the configured focus strategy.   |
| Pointer and focus    | Item-owned pointerdown and focus handlers update logical state without duplicating an unchanged transition.             |
| Virtualization       | Null item elements are valid with active descendant; a custom reveal callback receives the logical item.                |
| Portals              | Active descendants must be contained, `aria-owns` related, or inside a controlled popup of a supported input role.      |
| Scrolling            | Roving focus may use `preventScroll`; custom reveal takes precedence over the nearest-block fallback.                   |
| Callback timing      | Logical state and DOM representation commit before `onNavigate`; snapshots are immutable and retain the native event.   |
| Failure atomicity    | Focus, reveal, and callback failures are surfaced together after committed state remains observable.                    |
| Reactivity           | Orientation, direction, loop, page size, disablement, and typeahead controls are read at event time.                    |
| Disposal             | Disposal is idempotent, releases timers and handler caches, makes container handlers inert, and spares the registry.    |
| Styling              | The module emits no CSS. Consumers freely apply classes, data attributes, CSS, or design-token presets.                 |
| Tree shaking         | Root and subpath consumers produce identical JavaScript, retain no unrelated family signatures, and emit zero CSS.      |
