import { SnippetString, window, type TextDocumentChangeEvent, type TextEditor } from "vscode";
import type { LanguageClient, Middleware } from "vscode-languageclient/node.js";
import type { VizeConfigurationLike } from "./extension-core.js";

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

  return {
    async didChange(event, next): Promise<void> {
      await next(event);
      if (applyingSnippet || !config.get<boolean>("autoInsert.enable", false)) {
        return;
      }

      // VS Code updates the active selection immediately after emitting the
      // document change. Yield once so paired-character changes such as "{}"
      // report the caret between the braces, matching Volar's wire contract.
      await new Promise<void>((resolve) => setTimeout(resolve, 0));

      const client = getClient();
      const editor = window.activeTextEditor;
      if (
        !client ||
        !editor ||
        !supportsAutoInsert(client) ||
        !shouldRequest(event, editor, config)
      ) {
        return;
      }

      const [change] = event.contentChanges;
      const selection = editor.selection.active;
      const documentVersion = event.document.version;
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
    editor.document !== event.document ||
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
