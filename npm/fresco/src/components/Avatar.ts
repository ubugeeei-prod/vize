/**
 * Avatar Component - User avatar display
 */

import { defineComponent, h, type PropType } from "vue";

export interface AvatarProps {
  /** User name (used to generate initials) */
  name?: string;
  /** Custom initials */
  initials?: string;
  /** Avatar size */
  size?: "sm" | "md" | "lg";
  /** Background color */
  bg?: string;
  /** Foreground color */
  fg?: string;
  /** Show border */
  border?: boolean;
  /** Status indicator */
  status?: "online" | "offline" | "away" | "busy";
}

const STATUS_COLORS: Record<string, string> = {
  online: "green",
  offline: "gray",
  away: "yellow",
  busy: "red",
};

const STATUS_ICONS: Record<string, string> = {
  online: "●",
  offline: "○",
  away: "◐",
  busy: "⊘",
};

export const Avatar = defineComponent({
  name: "Avatar",
  props: {
    name: String,
    initials: String,
    size: {
      type: String as PropType<"sm" | "md" | "lg">,
      default: "md",
    },
    bg: {
      type: String,
      default: "blue",
    },
    fg: {
      type: String,
      default: "white",
    },
    border: {
      type: Boolean,
      default: true,
    },
    status: {
      type: String as PropType<AvatarProps["status"]>,
    },
  },
  setup(props) {
    return () => {
      // Generate initials from name
      let displayInitials = props.initials;
      if (!displayInitials && props.name) {
        const parts = props.name.split(" ").filter(Boolean);
        if (parts.length >= 2) {
          displayInitials = `${parts[0][0]}${parts[1][0]}`.toUpperCase();
        } else if (parts.length === 1) {
          displayInitials = parts[0].slice(0, 2).toUpperCase();
        }
      }
      displayInitials = displayInitials || "??";

      // Size-based padding
      const padding = props.size === "sm" ? 0 : props.size === "lg" ? 1 : 0;

      const children = [
        h(
          "text",
          {
            fg: props.fg,
            bg: props.bg,
            bold: true,
          },
          displayInitials,
        ),
      ];

      // Status indicator
      if (props.status) {
        children.push(
          h(
            "text",
            {
              fg: STATUS_COLORS[props.status],
            },
            STATUS_ICONS[props.status],
          ),
        );
      }

      if (props.border) {
        return h(
          "box",
          {
            border: "rounded",
            bg: props.bg,
            style: {
              flex_direction: "row",
              padding_left: padding,
              padding_right: padding,
            },
          },
          children,
        );
      }

      return h(
        "box",
        {
          bg: props.bg,
          style: {
            flex_direction: "row",
            padding_left: padding,
            padding_right: padding,
          },
        },
        children,
      );
    };
  },
});
