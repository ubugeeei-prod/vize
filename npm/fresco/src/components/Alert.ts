/**
 * Alert Component - Alert/notification box
 */

import { defineComponent, h, type PropType } from "vue";

export type AlertType = "info" | "success" | "warning" | "error";

export interface AlertProps {
  /** Alert message */
  message: string;
  /** Alert type */
  type?: AlertType;
  /** Alert title */
  title?: string;
  /** Show icon */
  showIcon?: boolean;
  /** Border style */
  border?: "single" | "double" | "rounded" | "none";
}

const ALERT_CONFIG: Record<AlertType, { icon: string; fg: string; title: string }> = {
  info: { icon: "ℹ", fg: "cyan", title: "Info" },
  success: { icon: "✓", fg: "green", title: "Success" },
  warning: { icon: "⚠", fg: "yellow", title: "Warning" },
  error: { icon: "✗", fg: "red", title: "Error" },
};

export const Alert = defineComponent({
  name: "Alert",
  props: {
    message: {
      type: String,
      required: true,
    },
    type: {
      type: String as PropType<AlertType>,
      default: "info",
    },
    title: String,
    showIcon: {
      type: Boolean,
      default: true,
    },
    border: {
      type: String as PropType<AlertProps["border"]>,
      default: "rounded",
    },
  },
  setup(props) {
    return () => {
      const config = ALERT_CONFIG[props.type];
      const title = props.title ?? config.title;

      const children = [];

      // Header with icon and title
      const headerParts = [];
      if (props.showIcon) {
        headerParts.push(h("text", { fg: config.fg }, `${config.icon} `));
      }
      headerParts.push(h("text", { fg: config.fg, bold: true }, title));

      children.push(h("box", { key: "header", style: { flex_direction: "row" } }, headerParts));

      // Message
      children.push(h("text", { key: "message", style: { margin_top: 1 } }, props.message));

      return h(
        "box",
        {
          border: props.border === "none" ? undefined : props.border,
          fg: config.fg,
          style: {
            flex_direction: "column",
            padding: 1,
          },
        },
        children,
      );
    };
  },
});
