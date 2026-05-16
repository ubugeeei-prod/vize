import "monaco-editor/esm/vs/language/typescript/monaco.contribution";
import "monaco-editor/esm/vs/language/html/monaco.contribution";
import "monaco-editor/esm/vs/language/css/monaco.contribution";
import "monaco-editor/esm/vs/language/json/monaco.contribution";

import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import tsWorker from "monaco-editor/esm/vs/language/typescript/ts.worker?worker";
import htmlWorker from "monaco-editor/esm/vs/language/html/html.worker?worker";
import cssWorker from "monaco-editor/esm/vs/language/css/css.worker?worker";
import jsonWorker from "monaco-editor/esm/vs/language/json/json.worker?worker";

self.MonacoEnvironment = {
  getWorker(_, label) {
    if (label === "typescript" || label === "javascript") {
      return new tsWorker();
    }
    if (label === "html" || label === "handlebars" || label === "razor") {
      return new htmlWorker();
    }
    if (label === "css" || label === "scss" || label === "less") {
      return new cssWorker();
    }
    if (label === "json") {
      return new jsonWorker();
    }
    return new editorWorker();
  },
};
