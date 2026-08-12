import "monaco-editor/language/typescript/monaco.contribution";
import "monaco-editor/language/html/monaco.contribution";
import "monaco-editor/language/css/monaco.contribution";
import "monaco-editor/language/json/monaco.contribution";

import editorWorker from "monaco-editor/editor/editor.worker?worker";
import tsWorker from "monaco-editor/language/typescript/ts.worker?worker";
import htmlWorker from "monaco-editor/language/html/html.worker?worker";
import cssWorker from "monaco-editor/language/css/css.worker?worker";
import jsonWorker from "monaco-editor/language/json/json.worker?worker";

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
