import { SnippetString, window, type TextDocumentChangeEvent, type TextEditor } from "vscode";
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

      // VS Code updates the active selection immediately after emitting the
      // document change. Yield once so paired-character changes such as "{}"
      // report the caret between the braces, matching Volar's wire contract.
      await new Promise<void>((resolve) => setTimeout(resolve, 0));

      if (
        !client ||
        !editor ||
        getClient() !== client ||
        window.activeTextEditor !== editor ||
        event.document.version !== documentVersion ||
        !supportsAutoInsert(client) ||
        !shouldRequest(event, editor, config)
      ) {
        return;
      }

      const [change] = event.contentChanges;
      const selection = editor.selection.active;
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
  const text = event.contentChanges[0].text;
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
