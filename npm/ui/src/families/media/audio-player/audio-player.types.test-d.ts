/** Compile-only assertions for the public AudioPlayer contract. */

import { MediaPlayerPlayButton, MediaPlayerRoot } from "../media-player/media-player.ts";
import {
  AudioPlayer,
  AudioPlayerAudio,
  AudioPlayerPlayButton,
  AudioPlayerRoot,
} from "./audio-player.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _RootIsShared = Expect<Equal<typeof AudioPlayerRoot, typeof MediaPlayerRoot>>;
type _AliasIsRoot = Expect<Equal<typeof AudioPlayer, typeof AudioPlayerRoot>>;
type _ControlsAreShared = Expect<Equal<typeof AudioPlayerPlayButton, typeof MediaPlayerPlayButton>>;

void AudioPlayerAudio;

// @ts-expect-error the audio player has no video part.
import { AudioPlayerVideo } from "./audio-player.ts";
void AudioPlayerVideo;
