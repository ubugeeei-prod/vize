import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type {
  MediaPlayerMediaKind,
  MediaPlayerPlaybackState,
  MediaPlayerResolvedMessages,
  MediaPlayerSeekReason,
  MediaPlayerSlotState,
} from "./media-player-types.ts";

/** Shared state and actions for the MediaPlayer compound parts. */
export interface MediaPlayerContextValue {
  readonly id: ComputedRef<string>;
  readonly getPartId: (part: string) => string;
  readonly media: Readonly<ShallowRef<HTMLMediaElement | null>>;
  readonly kind: Readonly<ShallowRef<MediaPlayerMediaKind | null>>;
  readonly state: ComputedRef<MediaPlayerPlaybackState>;
  readonly slotState: ComputedRef<MediaPlayerSlotState>;
  readonly messages: ComputedRef<MediaPlayerResolvedMessages>;
  readonly dir: ComputedRef<"ltr" | "rtl">;
  readonly muted: ComputedRef<boolean>;
  readonly seekStep: ComputedRef<number>;
  readonly volumeStep: ComputedRef<number>;
  readonly fullscreenSupported: Readonly<ShallowRef<boolean>>;
  readonly pictureInPictureSupported: Readonly<ShallowRef<boolean>>;
  readonly registerMedia: (media: HTMLMediaElement, kind: MediaPlayerMediaKind) => () => void;
  readonly play: () => Promise<boolean>;
  readonly pause: () => void;
  readonly togglePlay: () => Promise<boolean>;
  readonly seek: (time: number, reason: MediaPlayerSeekReason) => void;
  readonly setScrubbing: (scrubbing: boolean) => void;
  readonly setVolume: (volume: number) => boolean;
  readonly setMuted: (muted: boolean) => boolean;
  readonly setPlaybackRate: (rate: number) => boolean;
  readonly setCaptionTrack: (index: number) => boolean;
  readonly toggleCaptions: () => boolean;
  readonly toggleFullscreen: () => Promise<boolean>;
  readonly togglePictureInPicture: () => Promise<boolean>;
}

export const mediaPlayerContext = createContext<MediaPlayerContextValue>("MediaPlayer");
