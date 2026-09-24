/** Compile-only assertions for the `use-clipboard` type contracts. */

import { useClipboard } from "./use-clipboard.ts";
import type { ClipboardFailure, ClipboardHost, ClipboardResult } from "./use-clipboard.ts";
import type { PermissionQueryState } from "./use-permission.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const clipboard = useClipboard();
const copied = clipboard.copy("x");
type _CopyResolvesADiscriminatedResult = Expect<
  Equal<Awaited<typeof copied>, ClipboardResult<string>>
>;
type _ErrorIsAFailure = Expect<Equal<typeof clipboard.error.value, ClipboardFailure | undefined>>;
type _PermissionIsClosed = Expect<
  Equal<typeof clipboard.writePermission.value, PermissionQueryState>
>;

declare const result: ClipboardResult<string>;
if (result.status === "success") {
  result.method satisfies "clipboard" | "legacy";
} else {
  // @ts-expect-error failures carry no value.
  void result.value;
}

declare const navigatorClipboard: Clipboard;
({ clipboard: navigatorClipboard }) satisfies ClipboardHost;

// @ts-expect-error copy accepts text only.
void clipboard.copy(1);

// @ts-expect-error the copied flag is read-only.
clipboard.copied.value = true;
