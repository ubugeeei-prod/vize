/** Compile-only assertions for the public command contract. */

import { ref } from "vue";
import type { ShallowRef } from "vue";

import { createCommandRouter } from "./command.ts";
import type { CommandDispatch, CommandExecution, CommandInfo } from "./command.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type EditorCommand = "editor.save" | "editor.undo";

export const router = createCommandRouter<EditorCommand>({
  isDisabled: ref(false),
  onDidExecute(dispatch: CommandDispatch<EditorCommand>) {
    const id: EditorCommand = dispatch.id;
    void id;
  },
});

export const release = router.register({
  id: "editor.save",
  title: "Save Document",
  keywords: ["persist"],
  group: "file",
  when: () => true,
  run(execution: CommandExecution<EditorCommand>) {
    const id: EditorCommand = execution.id;
    return id;
  },
});

export const dispatch = router.execute("editor.undo", { steps: 1 }, { source: "shortcut" });

type _CommandsAreReadonly = Expect<
  Equal<typeof router.commands, Readonly<ShallowRef<readonly CommandInfo<EditorCommand>[]>>>
>;
type _DispatchIdIsTyped = Expect<Equal<typeof dispatch.id, EditorCommand>>;
type _DispatchStatusIsClosed = Expect<
  Equal<typeof dispatch.status, "executed" | "disabled" | "not-found">
>;
type _RegisterReturnsReleaser = Expect<Equal<typeof release, () => void>>;

// @ts-expect-error identifiers outside the union cannot be registered.
router.register({ id: "editor.format", run: () => undefined });
// @ts-expect-error identifiers outside the union cannot be dispatched.
router.execute("editor.format");
// @ts-expect-error enablement can only be read for known identifiers.
router.isEnabled("editor.format");
// @ts-expect-error dispatch source is a closed union.
router.execute("editor.save", undefined, { source: "webhook" });
// @ts-expect-error dispatch results are immutable.
dispatch.status = "executed";
// @ts-expect-error run is required.
router.register({ id: "editor.undo" });
