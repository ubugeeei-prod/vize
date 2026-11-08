import type { FrescoStyle } from "../protocol.js";

/** Public props for the Static output component. */
export interface StaticProps<T = unknown> {
  /** Items to render. */
  items: T[];
  /** Optional container style. */
  style?: FrescoStyle;
  /** Render item callback retained for Ink API parity. */
  children?: (item: T, index: number) => unknown;
}

/** Public props for the terminal Link component. */
export interface LinkProps {
  /** Link text. */
  text: string;
  /** URL displayed alongside the text when requested. */
  url?: string;
  /** Foreground color. Defaults to blue. */
  fg?: string;
  /** Whether to underline the link text. Defaults to true. */
  underline?: boolean;
  /** Whether to show the URL in parentheses. Defaults to false. */
  showUrl?: boolean;
}

/** Visual variant supported by Badge. */
export type BadgeVariant = "default" | "success" | "warning" | "error" | "info";

/** Public props for the Badge component. */
export interface BadgeProps {
  /** Badge text. */
  label: string;
  /** Badge variant. Defaults to default. */
  variant?: BadgeVariant;
  /** Custom foreground color that overrides the variant. */
  fg?: string;
  /** Custom background color that overrides the variant. */
  bg?: string;
  /** Whether to show a border. Defaults to false. */
  border?: boolean;
}

/** Default foreground and background colors for each Badge variant. */
export const badgeVariantColors: Record<BadgeVariant, { fg: string; bg?: string }> = {
  default: { fg: "white" },
  success: { fg: "green" },
  warning: { fg: "yellow" },
  error: { fg: "red" },
  info: { fg: "cyan" },
};

/** Built-in animation frame sets for Spinner. */
export const spinnerTypes = {
  dots: ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
  dots2: ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
  line: ["-", "\\", "|", "/"],
  arc: ["◜", "◠", "◝", "◞", "◡", "◟"],
  circle: ["◐", "◓", "◑", "◒"],
  bounce: ["⠁", "⠂", "⠄", "⡀", "⢀", "⠠", "⠐", "⠈"],
  box: ["▖", "▘", "▝", "▗"],
  arrow: ["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
  clock: ["🕛", "🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🕚"],
  moon: ["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"],
  earth: ["🌍", "🌎", "🌏"],
} as const;

/** Name of a built-in Spinner frame set. */
export type SpinnerType = keyof typeof spinnerTypes;

/** Public props for the Spinner component. */
export interface SpinnerProps {
  /** Built-in frame set. Defaults to dots. */
  type?: SpinnerType;
  /** Custom animation frames. */
  frames?: string[];
  /** Animation interval in milliseconds. Defaults to 80. */
  interval?: number;
  /** Optional label rendered after the frame. */
  label?: string;
  /** Foreground color. */
  fg?: string;
}
