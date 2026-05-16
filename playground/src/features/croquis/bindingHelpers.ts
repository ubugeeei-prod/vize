/** Map a binding source to a human-readable label for display. */
export function getSourceLabel(source: string): string {
  const labels: Record<string, string> = {
    props: "Props",
    emits: "Emits",
    model: "Models",
    slots: "Slots",
    ref: "Refs",
    reactive: "Reactive",
    computed: "Computed",
    import: "Imports",
    local: "Local",
    function: "Functions",
    class: "Classes",
    templateRef: "Template Refs",
    unknown: "Other",
  };
  return labels[source] || source;
}

/** Map a binding source to a CSS class name for styling. */
export function getSourceClass(source: string): string {
  const classes: Record<string, string> = {
    props: "src-props",
    emits: "src-emits",
    model: "src-model",
    slots: "src-slots",
    ref: "src-ref",
    reactive: "src-reactive",
    computed: "src-computed",
    import: "src-import",
    local: "src-local",
    function: "src-function",
    class: "src-class",
  };
  return classes[source] || "src-default";
}
