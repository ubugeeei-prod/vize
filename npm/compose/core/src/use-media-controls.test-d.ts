/** Compile-only assertions for the `use-media-controls` type contracts. */

import type { Ref } from "vue";

import { useMediaControls } from "./use-media-controls.ts";
import type { MediaElementLike, PictureInPictureDocumentLike } from "./use-media-controls.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const video: HTMLVideoElement;
declare const audio: HTMLAudioElement;
video satisfies MediaElementLike;
audio satisfies MediaElementLike;
document satisfies PictureInPictureDocumentLike;

const media = useMediaControls(() => video, { src: [{ src: "/a.mp4", type: "video/mp4" }] });
type _PlayingIsWritable = Expect<Equal<typeof media.playing, Ref<boolean>>>;
type _BufferedIsPairs = Expect<
  Equal<typeof media.buffered.value, readonly (readonly [number, number])[]>
>;

media.currentTime.value = 10;

// @ts-expect-error the duration is read-only.
media.duration.value = 1;

// @ts-expect-error sources are URLs or candidates.
useMediaControls(video, { src: 1 });
