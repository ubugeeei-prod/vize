/** Headless avatar stack with a bounded tile count and an accessible overflow tile. */
export { default as AvatarGroup } from "./avatar-group.vue";
export { default as AvatarGroupOverflow } from "./avatar-group-overflow.vue";
export {
  avatarGroupSpacing,
  resolveAvatarGroupMessages,
  splitAvatarGroup,
} from "./avatar-group-state.ts";
export type { AvatarGroupSplit } from "./avatar-group-state.ts";
export { defaultAvatarGroupMessages } from "./avatar-group-types.ts";
export type {
  AvatarGroupExpose,
  AvatarGroupItemSlotState,
  AvatarGroupMessageOverrides,
  AvatarGroupMessages,
  AvatarGroupOverflowExpose,
  AvatarGroupOverflowSlotState,
  AvatarGroupSlotState,
  AvatarGroupState,
} from "./avatar-group-types.ts";
