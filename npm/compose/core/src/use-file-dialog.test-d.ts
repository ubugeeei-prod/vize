/** Compile-only assertions for the `use-file-dialog` type contracts. */

import { useFileDialog } from "./use-file-dialog.ts";
import type { FileDialogHost } from "./use-file-dialog.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const dialog = useFileDialog({ accept: "image/*" });
type _FilesAreReadonly = Expect<Equal<typeof dialog.files.value, readonly File[] | null>>;

document satisfies FileDialogHost;

// @ts-expect-error capture is a closed union.
useFileDialog({ capture: "front" });

// @ts-expect-error the selection is read-only.
dialog.files.value = [];
