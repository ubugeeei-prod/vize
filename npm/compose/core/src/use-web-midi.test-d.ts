/** Compile-only assertions for the `use-web-midi` type contracts. */

import { useWebMIDI } from "./use-web-midi.ts";
import type { MIDIAccessLike, MIDIHost, MIDIInputMessage, MIDIPortInfo } from "./use-web-midi.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const midi = useWebMIDI({
  onMessage: (message) => {
    type _Message = Expect<Equal<typeof message, MIDIInputMessage>>;
    message.data satisfies Uint8Array;
  },
});
type _Request = Expect<Equal<Awaited<ReturnType<typeof midi.request>>, boolean>>;
type _Inputs = Expect<Equal<(typeof midi.inputs.value)[number], MIDIPortInfo>>;

midi.send("out", [0x90, 60, 127]);
midi.send("out", Uint8Array.of(0xf8), 10);

// @ts-expect-error messages are bytes, not strings.
midi.send("out", "note-on");

// @ts-expect-error the port list is read-only.
midi.inputs.value = [];

// Real browser objects satisfy the host contracts.
navigator satisfies MIDIHost;
declare const access: MIDIAccess;
access satisfies MIDIAccessLike;
