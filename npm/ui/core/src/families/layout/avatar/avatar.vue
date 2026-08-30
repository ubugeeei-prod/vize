<template>
  <component
    :is="as"
    ref="element"
    part="root"
    data-vize-ui="avatar"
    :data-state="state"
    :data-status="statusState"
    :data-image="imageState"
    :data-name="nameState"
    :data-fallback="fallbackState"
  >
    <img
      v-if="state === 'image'"
      ref="imageElement"
      part="image"
      data-vize-ui="avatar-image"
      :data-status="statusState"
      :alt="altState"
      v-bind="imageProps"
      @load="onImageLoad"
      @error="onImageError"
    />
    <span
      v-else
      ref="fallbackElement"
      part="fallback"
      data-vize-ui="avatar-fallback"
      :data-status="statusState"
      :data-name="nameState"
      :data-fallback="fallbackState"
    >
      <slot name="fallback" v-bind="slotState">
        <slot v-bind="slotState">
          {{ fallbackText }}
        </slot>
      </slot>
    </span>
  </component>
</template>

<script setup lang="ts">
import { computed, ref, useSlots, useTemplateRef, watch } from "vue";

import type {
  AvatarElement,
  AvatarExpose,
  AvatarFallbackElement,
  AvatarImageCrossOrigin,
  AvatarImageDecoding,
  AvatarImageElement,
  AvatarImageFetchPriority,
  AvatarImageLoading,
  AvatarImageReferrerPolicy,
  AvatarPresence,
  AvatarSlotState,
  AvatarState,
  AvatarStatus,
} from "./avatar-types.ts";
import type { PrimitiveAs } from "../../../primitive.ts";

const IMAGE_SOURCE_SCHEME = /^([a-z][a-z\d+.-]*):/i;
const DATA_IMAGE_SOURCE = /^data:image\/(?:avif|gif|jpeg|jpg|png|webp);base64,[a-z\d+/]+={0,2}$/i;

const {
  as = "span",
  src = undefined,
  alt = "",
  name = undefined,
  fallback = undefined,
  status = "none",
  loading = undefined,
  decoding = undefined,
  fetchPriority = undefined,
  crossOrigin = undefined,
  referrerPolicy = undefined,
} = defineProps<{
  /**
   * Native element, custom element, or component to render as the root.
   *
   * @default "span"
   */
  readonly as?: PrimitiveAs;

  /**
   * Native image source. Missing or failed sources render the fallback part.
   *
   * @default undefined
   */
  readonly src?: string;

  /**
   * Native image alternative text. Avatar does not infer this from `name`.
   *
   * @default ""
   */
  readonly alt?: string;

  /**
   * Consumer-owned display name exposed to slots and data presence hooks.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Consumer-owned fallback text rendered when no fallback slot is provided.
   *
   * @default undefined
   */
  readonly fallback?: string;

  /**
   * Consumer presence token mirrored to `data-status`; no CSS is emitted.
   *
   * @default "none"
   */
  readonly status?: AvatarStatus;

  /**
   * Native image loading policy.
   *
   * @default undefined
   */
  readonly loading?: AvatarImageLoading;

  /**
   * Native image decoding policy.
   *
   * @default undefined
   */
  readonly decoding?: AvatarImageDecoding;

  /**
   * Native image fetch-priority hint.
   *
   * @default undefined
   */
  readonly fetchPriority?: AvatarImageFetchPriority;

  /**
   * Native image CORS policy.
   *
   * @default undefined
   */
  readonly crossOrigin?: AvatarImageCrossOrigin;

  /**
   * Native image referrer policy.
   *
   * @default undefined
   */
  readonly referrerPolicy?: AvatarImageReferrerPolicy;
}>();

defineSlots<{
  /** Renders fallback content with the current avatar hooks. */
  default(props: AvatarSlotState): unknown;

  /** Renders named fallback content with the current avatar hooks. */
  fallback(props: AvatarSlotState): unknown;
}>();

const emit = defineEmits<{
  /** Fired after the image part dispatches a native load event. */
  load: [nativeEvent: Event];

  /** Fired after the image part dispatches a native error event and Avatar renders fallback. */
  error: [nativeEvent: Event];
}>();

const slots = useSlots();
const element = useTemplateRef<AvatarElement>("element");
const imageElement = useTemplateRef<AvatarImageElement>("imageElement");
const fallbackElement = useTemplateRef<AvatarFallbackElement>("fallbackElement");
const imageFailed = ref(false);

const srcState = computed(() => normalizeImageSource(src));
const altState = computed(() => alt);
const imageProps = computed(() => ({
  crossorigin: crossOrigin,
  decoding,
  fetchpriority: fetchPriority,
  loading,
  referrerpolicy: referrerPolicy,
  src: srcState.value,
}));
const nameValue = computed(() => name);
const fallbackValue = computed(() => fallback);
const statusState = computed(() => status);
const imageState = computed<AvatarPresence>(() =>
  srcState.value === undefined ? "missing" : "present",
);
const nameState = computed<AvatarPresence>(() =>
  nameValue.value !== undefined && nameValue.value.length > 0 ? "present" : "missing",
);
const fallbackText = computed(() => fallbackValue.value ?? "");
const fallbackState = computed<AvatarPresence>(() =>
  fallbackText.value.length > 0 || slots.default !== undefined || slots.fallback !== undefined
    ? "present"
    : "missing",
);
const state = computed<AvatarState>(() =>
  imageState.value === "present" && !imageFailed.value ? "image" : "fallback",
);
const slotState = computed<AvatarSlotState>(() => ({
  alt: altState.value,
  fallback: fallbackValue.value,
  fallbackState: fallbackState.value,
  image: imageState.value,
  name: nameValue.value,
  nameState: nameState.value,
  src: srcState.value,
  state: state.value,
  status: statusState.value,
}));

watch(srcState, () => {
  imageFailed.value = false;
});

function onImageLoad(event: Event): void {
  imageFailed.value = false;
  emit("load", event);
}

function onImageError(event: Event): void {
  imageFailed.value = true;
  emit("error", event);
}

function normalizeImageSource(value: string | undefined): string | undefined {
  if (typeof value !== "string") return undefined;

  const normalized = value.trim();
  if (normalized.length === 0 || containsControlCharacter(normalized)) return undefined;

  const scheme = IMAGE_SOURCE_SCHEME.exec(normalized)?.[1]?.toLowerCase();
  if (scheme === undefined || scheme === "https" || scheme === "blob") return normalized;
  if (scheme === "data" && DATA_IMAGE_SOURCE.test(normalized)) return normalized;
  return undefined;
}

function containsControlCharacter(value: string): boolean {
  for (const character of value) {
    const code = character.charCodeAt(0);
    if (code <= 0x1f || code === 0x7f) return true;
  }
  return false;
}

type AvatarSetupExpose = Omit<
  AvatarExpose,
  | "alt"
  | "element"
  | "fallback"
  | "fallbackElement"
  | "fallbackState"
  | "image"
  | "imageElement"
  | "name"
  | "nameState"
  | "src"
  | "state"
  | "status"
> & {
  readonly alt: typeof altState;
  readonly element: typeof element;
  readonly fallback: typeof fallbackValue;
  readonly fallbackElement: typeof fallbackElement;
  readonly fallbackState: typeof fallbackState;
  readonly image: typeof imageState;
  readonly imageElement: typeof imageElement;
  readonly name: typeof nameValue;
  readonly nameState: typeof nameState;
  readonly src: typeof srcState;
  readonly state: typeof state;
  readonly status: typeof statusState;
};

const exposed = {
  alt: altState,
  element,
  fallback: fallbackValue,
  fallbackElement,
  fallbackState,
  image: imageState,
  imageElement,
  name: nameValue,
  nameState,
  src: srcState,
  state,
  status: statusState,
} satisfies AvatarSetupExpose;

defineExpose(exposed);
</script>

<style scoped>
/* Headless by design. Image sizing, clipping, fallback layout, and status styling remain consumer-owned. */
</style>
