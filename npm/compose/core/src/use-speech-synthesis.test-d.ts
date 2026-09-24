/** Compile-only assertions for the `use-speech-synthesis` type contracts. */

import { useSpeechSynthesis } from "./use-speech-synthesis.ts";
import type {
  SpeechSynthesisErrorCode,
  SpeechSynthesisHost,
  SpeechSynthesisStatus,
} from "./use-speech-synthesis.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const speech = useSpeechSynthesis(() => "hello", { rate: 1.5 });
type _StatusIsClosed = Expect<Equal<typeof speech.status.value, SpeechSynthesisStatus>>;
type _ErrorIsClosed = Expect<
  Equal<typeof speech.error.value, SpeechSynthesisErrorCode | undefined>
>;

({
  synthesis: speechSynthesis,
  Utterance: SpeechSynthesisUtterance,
}) satisfies SpeechSynthesisHost;

// @ts-expect-error the text is a string source.
useSpeechSynthesis(42);

// @ts-expect-error the status is read-only.
speech.status.value = "speaking";
