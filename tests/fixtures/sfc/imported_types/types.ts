export const SELECT_SIZES = {
  sm: "text-xs",
  md: "text-sm",
} as const;

export type SelectBaseProps = {
  disabled?: boolean;
  size?: keyof typeof SELECT_SIZES;
};
