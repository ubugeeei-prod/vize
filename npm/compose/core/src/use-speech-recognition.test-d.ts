/** Compile-only assertions for the `use-speech-recognition` type contracts. */

import { useSpeechRecognition } from "./use-speech-recognition.ts";
import type {
  SpeechRecognitionErrorCode,
  SpeechRecognitionHost,
  SpeechRecognitionLike,
} from "./use-speech-recognition.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const speech = useSpeechRecognition({ lang: () => "en-GB" });
type _ErrorCodeIsClosed = Expect<
  Equal<NonNullable<typeof speech.error.value>["code"], SpeechRecognitionErrorCode>
>;
type _ResultIsText = Expect<Equal<typeof speech.result.value, string>>;

declare const Recognizer: new () => SpeechRecognitionLike;
useSpeechRecognition({ host: Recognizer }).start() satisfies boolean;
Recognizer satisfies SpeechRecognitionHost;

// @ts-expect-error hosts are constructors, not instances.
useSpeechRecognition({ host: {} as SpeechRecognitionLike });

// @ts-expect-error transcripts are read-only.
speech.result.value = "";
