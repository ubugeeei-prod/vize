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

/** Visual defaults for each alert type. */
export const alertConfigs: Record<AlertType, { icon: string; fg: string; title: string }> = {
  info: { icon: "ℹ", fg: "cyan", title: "Info" },
  success: { icon: "✓", fg: "green", title: "Success" },
  warning: { icon: "⚠", fg: "yellow", title: "Warning" },
  error: { icon: "✗", fg: "red", title: "Error" },
};

export type AvatarStatus = "online" | "offline" | "away" | "busy";

export interface AvatarProps {
  /** User name used to generate initials */
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
  status?: AvatarStatus;
}

/** Foreground colors for avatar statuses. */
export const avatarStatusColors: Record<AvatarStatus, string> = {
  online: "green",
  offline: "gray",
  away: "yellow",
  busy: "red",
};

/** Glyphs for avatar statuses. */
export const avatarStatusIcons: Record<AvatarStatus, string> = {
  online: "●",
  offline: "○",
  away: "◐",
  busy: "⊘",
};

export interface BreadcrumbItem {
  /** Stable item key */
  key: string;
  /** Visible label */
  label: string;
  /** Optional leading icon */
  icon?: string;
}

export interface BreadcrumbProps {
  /** Breadcrumb items */
  items: BreadcrumbItem[];
  /** Separator */
  separator?: string;
  /** Foreground color */
  fg?: string;
  /** Active item foreground color */
  activeFg?: string;
  /** Separator foreground color */
  separatorFg?: string;
}

export interface CardProps {
  /** Card title */
  title?: string;
  /** Card subtitle */
  subtitle?: string;
  /** Footer text */
  footer?: string;
  /** Border style */
  border?: "single" | "double" | "rounded" | "heavy" | "none";
  /** Padding */
  padding?: number;
  /** Title foreground color */
  titleFg?: string;
  /** Border foreground color */
  borderFg?: string;
  /** Background color */
  bg?: string;
}

export interface HeaderProps {
  /** Header title */
  title: string;
  /** Subtitle */
  subtitle?: string;
  /** Left fallback content */
  left?: string;
  /** Right fallback content */
  right?: string;
  /** Background color */
  bg?: string;
  /** Title foreground color */
  titleFg?: string;
  /** Subtitle foreground color */
  subtitleFg?: string;
  /** Show a bottom border */
  borderBottom?: boolean;
}

export type TooltipPosition = "top" | "bottom" | "left" | "right";

export interface TooltipProps {
  /** Tooltip text */
  text: string;
  /** Whether the tooltip is visible */
  visible?: boolean;
  /** Tooltip position */
  position?: TooltipPosition;
  /** Gap in terminal cells between the anchor and tooltip. */
  offset?: number;
  /** Border style */
  border?: "single" | "rounded" | "none";
  /** Background color */
  bg?: string;
  /** Foreground color */
  fg?: string;
}
