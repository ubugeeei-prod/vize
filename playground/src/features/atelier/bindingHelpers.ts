export function getBindingIcon(type: string): string {
  const icons: Record<string, string> = {
    "setup-const": "C",
    "setup-let": "L",
    "setup-reactive-const": "R",
    "setup-maybe-ref": "?",
    "setup-ref": "r",
    props: "P",
    "props-aliased": "A",
    data: "D",
    options: "O",
  };
  return icons[type] || type[0]?.toUpperCase() || "?";
}

export function getBindingLabel(type: string): string {
  const labels: Record<string, string> = {
    "setup-const": "Constants",
    "setup-let": "Let Variables",
    "setup-reactive-const": "Reactive",
    "setup-maybe-ref": "Maybe Ref",
    "setup-ref": "Refs",
    props: "Props",
    "props-aliased": "Aliased Props",
    data: "Data",
    options: "Options",
  };
  return labels[type] || type;
}
