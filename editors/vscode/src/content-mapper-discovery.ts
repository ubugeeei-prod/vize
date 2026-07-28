import {
  type ExtensionContext,
  type OutputChannel,
  type Uri,
  commands,
  extensions,
  workspace,
} from "vscode";

function vueUri(document: { languageId: string; uri: Uri }): Uri | undefined {
  return document.uri.scheme === "file" &&
    (document.languageId === "vue" || document.languageId === "art-vue") &&
    document.uri.path.endsWith(".vue")
    ? document.uri
    : undefined;
}

export function registerTypeScriptContentMapperDiscovery(
  context: ExtensionContext,
  outputChannel: OutputChannel,
): void {
  const discover = (uris: readonly Uri[]) => {
    if (!workspace.isTrusted || uris.length === 0) {
      return;
    }
    const nativePreview = extensions.getExtension("TypeScriptTeam.native-preview");
    if (!nativePreview) {
      return;
    }
    void (async () => {
      try {
        await nativePreview.activate();
        const discovered = await commands.executeCommand(
          "typescript.native-preview.discoverContentMappers",
          { uris, extensions: [".vue"] },
        );
        if (Array.isArray(discovered) && discovered.includes(".vue")) {
          outputChannel.appendLine("TypeScript native preview discovered the Vize .vue mapper.");
        }
      } catch (error: unknown) {
        outputChannel.appendLine(
          `TypeScript content mapper discovery is unavailable: ${String(error)}`,
        );
      }
    })();
  };

  discover(workspace.textDocuments.map(vueUri).filter((uri): uri is Uri => uri !== undefined));
  context.subscriptions.push(
    workspace.onDidOpenTextDocument((document) => {
      const uri = vueUri(document);
      if (uri) {
        discover([uri]);
      }
    }),
  );
}
