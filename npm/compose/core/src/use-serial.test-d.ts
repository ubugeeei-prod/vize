/** Compile-only assertions for the `use-serial` type contracts. */

import { useSerial } from "./use-serial.ts";
import type { SerialChunk, SerialPortLike } from "./use-serial.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

useSerial({
  decode: "text",
  onData: (chunk) => {
    type _Text = Expect<Equal<typeof chunk, string>>;
  },
});
useSerial({
  onData: (chunk) => {
    type _Bytes = Expect<Equal<typeof chunk, Uint8Array>>;
  },
});
type _TextChunk = Expect<Equal<SerialChunk<"text">, string>>;

const serial = useSerial();
type _Request = Expect<
  Equal<Awaited<ReturnType<typeof serial.requestPort>>, SerialPortLike | undefined>
>;
declare const port: SerialPortLike;
void serial.open(port, { baudRate: 9600, parity: "even" });

// @ts-expect-error baudRate is required.
void serial.open(port, {});

// @ts-expect-error only bytes or text can be decoded.
useSerial({ decode: "json" });

// @ts-expect-error the connection flag is read-only.
serial.connected.value = true;
