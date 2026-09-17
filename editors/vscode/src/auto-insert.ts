import {
  SnippetString,
  window,
  workspace,
  type Disposable,
  type Position,
  type TextDocumentChangeEvent,
  type TextEditor,
} from "vscode";
import type { LanguageClient, Middleware } from "vscode-languageclient/node.js";
import type { VizeConfigurationLike } from "./extension-core.js";
import { trackAutoInsertForHostTest } from "./auto-insert-test-state.js";

export const AUTO_INSERT_METHOD = "volar/client/autoInsert";

type AutoInsertionProvider = {
  triggerCharacters?: string[];
  configurationSections?: Array<string[] | null>;
};

export function createAutoInsertMiddleware(
  getClient: () => LanguageClient | undefined,
  config: VizeConfigurationLike,
): Middleware {
  let applyingSnippet = false;

  const middleware: Middleware = {
    async didChange(event, next): Promise<void> {
      const documentVersion = event.document.version;
      const editor = window.activeTextEditor;
      const client = getClient();
      const wasApplyingSnippet = applyingSnippet;
      await next(event);
      if (wasApplyingSnippet || !config.get<boolean>("autoInsert.enable", false)) {
        return;
      }

      if (
        !client ||
        !editor ||
        getClient() !== client ||
        window.activeTextEditor !== editor ||
        event.document.version !== documentVersion ||
        !supportsAutoInsert(client) ||
        event.contentChanges.length !== 1 ||
        !isTriggerEnabled(event.contentChanges[0].text, config)
      ) {
        return;
      }

      const selection = await waitForAuthoredSelection(event, editor, documentVersion);
      if (
        !selection ||
        getClient() !== client ||
        window.activeTextEditor !== editor ||
        !config.get<boolean>("autoInsert.enable", false) ||
        event.document.version !== documentVersion ||
        !shouldRequest(event, editor, config) ||
        !editor.selection.active.isEqual(selection)
      ) {
        return;
      }
      const [change] = event.contentChanges;
      const snippet = await client
        .sendRequest<string | null>(AUTO_INSERT_METHOD, {
          textDocument: { uri: event.document.uri.toString() },
          selection: { line: selection.line, character: selection.character },
          change: {
            rangeOffset: change.rangeOffset,
            rangeLength: change.rangeLength,
            text: change.text,
          },
        })
        .catch(() => null);
      if (
        !snippet ||
        getClient() !== client ||
        window.activeTextEditor !== editor ||
        !config.get<boolean>("autoInsert.enable", false) ||
        !shouldRequest(event, editor, config) ||
        editor.document.version !== documentVersion ||
        !editor.selection.active.isEqual(selection)
      ) {
        return;
      }

      applyingSnippet = true;
      try {
        await editor.insertSnippet(new SnippetString(snippet), selection, {
          undoStopBefore: false,
          undoStopAfter: false,
        });
      } finally {
        applyingSnippet = false;
      }
    },
  };
  return {
    didChange: (event, next) =>
      trackAutoInsertForHostTest(Promise.resolve(middleware.didChange!(event, next))),
  };
}

async function waitForAuthoredSelection(
  event: TextDocumentChangeEvent,
  editor: TextEditor,
  version: number,
): Promise<Position | undefined> {
  const { document, contentChanges } = event;
  if (contentChanges.length !== 1 || editor.document !== document || document.isClosed) return;
  const [change] = contentChanges;
  // VS Code delivers document and selection updates separately. The paired
  // brace edit leaves the caret inside the pair; ordinary edits leave it at the end.
  const expected = document.positionAt(
    change.rangeOffset + (change.text === "{}" ? 1 : change.text.length),
  );
  const currentSelection = () =>
    window.activeTextEditor === editor &&
    !document.isClosed &&
    document.version === version &&
    editor.selection.isEmpty &&
    editor.selections.length === 1 &&
    editor.selection.active.isEqual(expected)
      ? editor.selection.active
      : undefined;
  const settled = currentSelection();
  if (settled) return settled;

  return new Promise((resolve) => {
    const subscriptions: Disposable[] = [];
    const finish = (position?: Position) => {
      clearTimeout(timeout);
      for (const subscription of subscriptions) subscription.dispose();
      resolve(position);
    };
    // A programmatic edit need not move the caret. Bound the wait and skip it,
    // never send the old caret just because a timer elapsed.
    const timeout = setTimeout(() => finish(), 1_000);
    subscriptions.push(
      window.onDidChangeTextEditorSelection((change) => {
        if (change.textEditor === editor) finish(currentSelection());
      }),
      window.onDidChangeActiveTextEditor((active) => {
        if (active !== editor) finish();
      }),
      workspace.onDidChangeTextDocument((change) => {
        if (change.document === document && document.version !== version) finish();
      }),
      workspace.onDidCloseTextDocument((closed) => {
        if (closed === document) finish();
      }),
    );
  });
}

function supportsAutoInsert(client: LanguageClient): boolean {
  const experimental = client.initializeResult?.capabilities.experimental as
    | { autoInsertionProvider?: AutoInsertionProvider }
    | undefined;
  return experimental?.autoInsertionProvider != null;
}

export function shouldRequest(
  event: TextDocumentChangeEvent,
  editor: TextEditor,
  config: VizeConfigurationLike,
): boolean {
  if (
    event.contentChanges.length !== 1 ||
    event.document.isClosed ||
    editor.document !== event.document ||
    !editor.selection.isEmpty ||
    editor.selections.length !== 1
  ) {
    return false;
  }
  return isTriggerEnabled(event.contentChanges[0].text, config);
}

function isTriggerEnabled(text: string, config: VizeConfigurationLike): boolean {
  if (text === "{}") {
    return config.get<boolean>("autoInsert.bracketSpacing", true);
  }
  if (text === "=") {
    return config.get<boolean>("autoInsert.autoCreateQuotes", true);
  }
  if (text === ">" || text === "/") {
    return config.get<boolean>("autoInsert.autoClosingTags", true);
  }
  return (
    text.length > 0 &&
    !text.includes("\n") &&
    /\w/u.test(text.at(-1) ?? "") &&
    config.get<boolean>("autoInsert.dotValue", true)
  );
}
