import {
  mdiLanguageTypescript,
  mdiVuejs,
  mdiFile,
  mdiClose,
  mdiAlert,
  mdiInformation,
} from "@mdi/js";

export function getFileIcon(filename: string): string {
  if (filename.endsWith(".vue")) return mdiVuejs;
  if (filename.endsWith(".ts")) return mdiLanguageTypescript;
  return mdiFile;
}

export function getSeverityIcon(severity: string): string {
  return severity === "error" ? mdiClose : severity === "warning" ? mdiAlert : mdiInformation;
}

export function getTypeLabel(type: string): string {
  const labels: Record<string, string> = {
    "provide-inject": "Provide/Inject",
    "component-emit": "Component Emit",
    "event-bubbling": "Event Bubbling",
    "fallthrough-attrs": "Fallthrough Attrs",
    reactivity: "Reactivity",
    "reactivity-loss": "Reactivity Loss",
    "reference-escape": "Reference Escape",
    "unique-id": "Unique ID",
    "unique-ids": "Unique IDs",
    "ssr-boundary": "SSR Boundary",
    "setup-context": "Setup Context",
    "component-resolution": "Component Resolution",
    "props-validation": "Props Validation",
    "circular-dependency": "Circular Dependency",
  };
  return labels[type] || type;
}

export function getTypeColor(type: string): string {
  const colors: Record<string, string> = {
    "provide-inject": "#8b5cf6",
    "component-emit": "#f59e0b",
    "event-bubbling": "#f97316",
    "fallthrough-attrs": "#06b6d4",
    reactivity: "#ef4444",
    "reactivity-loss": "#dc2626",
    "reference-escape": "#db2777",
    "unique-id": "#10b981",
    "unique-ids": "#10b981",
    "ssr-boundary": "#3b82f6",
    "setup-context": "#64748b",
    "component-resolution": "#14b8a6",
    "props-validation": "#84cc16",
    "circular-dependency": "#a855f7",
  };
  return colors[type] || "#6b7280";
}
