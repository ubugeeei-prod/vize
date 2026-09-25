/** Compile-only assertions for the public VideoPlayer contract. */

import { MediaPlayerRoot } from "../media-player/media-player.ts";
import {
  VideoPlayer,
  VideoPlayerFullscreenButton,
  VideoPlayerPictureInPictureButton,
  VideoPlayerPlayButton,
  VideoPlayerRoot,
  VideoPlayerVideo,
  type VideoPlayerUnsupportedBehavior,
} from "./video-player.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _UnsupportedIsLiteral = Expect<Equal<VideoPlayerUnsupportedBehavior, "disable" | "hide">>;
type _RootIsShared = Expect<Equal<typeof VideoPlayerRoot, typeof MediaPlayerRoot>>;
type _AliasIsRoot = Expect<Equal<typeof VideoPlayer, typeof VideoPlayerRoot>>;

void VideoPlayerFullscreenButton;
void VideoPlayerPictureInPictureButton;
void VideoPlayerPlayButton;
void VideoPlayerVideo;

// @ts-expect-error unsupported behavior is a closed union.
const _unknownBehavior: VideoPlayerUnsupportedBehavior = "remove";
