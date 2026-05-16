import { ref, computed, watch, nextTick } from "vue";
import type { CroquisResult } from "../../wasm/index";
import { PRESETS } from "../../shared/presets/crossfile";
import type { CrossFileIssue } from "./types";
import MonacoEditor from "../../shared/MonacoEditor.vue";

export function useFileManagement() {
  const currentPreset = ref<string>("default");
  const currentPresetData = computed(
    () => PRESETS.find((p) => p.id === currentPreset.value) || PRESETS[0],
  );
  const files = ref<Record<string, string>>({ ...currentPresetData.value.files });
  const activeFile = ref<string>(Object.keys(currentPresetData.value.files)[0]);

  // Monaco editor ref (for direct setValue calls - vite-plugin-vize workaround)
  const monacoEditorRef = ref<InstanceType<typeof MonacoEditor>>(null);

  // File names array for v-for (workaround for vite-plugin-vize object iteration issue)
  const fileNames = ref<string[]>(Object.keys(files.value));
  watch(
    files,
    (newFiles) => {
      fileNames.value = Object.keys(newFiles);
    },
    { deep: true },
  );

  const croquisResults = ref<Record<string, CroquisResult | null>>({});
  const crossFileIssues = ref<CrossFileIssue[]>([]);
  const analysisTime = ref<number>(0);
  const isAnalyzing = ref(false);
  const selectedIssue = ref<CrossFileIssue | null>(null);

  // Source State (workaround for vite-plugin-vize reactivity issue)
  const currentSource = ref(files.value[activeFile.value] || "");

  watch(
    activeFile,
    (newFile) => {
      currentSource.value = files.value[newFile] || "";
    },
    { immediate: false },
  );

  watch(
    currentSource,
    (newSource) => {
      files.value[activeFile.value] = newSource;
    },
    { immediate: false },
  );

  const editorLanguage = computed(() => {
    const ext = activeFile.value.split(".").pop()?.toLowerCase();
    switch (ext) {
      case "ts":
        return "typescript";
      case "js":
        return "javascript";
      case "css":
        return "css";
      case "scss":
        return "scss";
      case "json":
        return "json";
      case "vue":
      default:
        return "vue";
    }
  });

  const dependencyGraph = computed(() => {
    const graph: Record<string, string[]> = {};
    for (const [filename, source] of Object.entries(files.value)) {
      const imports: string[] = [];
      const importRegex = /import\s+[\w{}\s,*]+\s+from\s+['"]\.\/([^'"]+)['"]/g;
      let match;
      while ((match = importRegex.exec(source)) !== null) {
        let importFile = match[1];
        if (!importFile.endsWith(".vue")) importFile += ".vue";
        if (files.value[importFile]) {
          imports.push(importFile);
        }
      }
      graph[filename] = imports;
    }
    return graph;
  });

  function addFile() {
    const name = prompt("Enter file name (e.g., NewComponent.vue)");
    if (name && !files.value[name]) {
      files.value[name] =
        '<script setup lang="ts">\n// ' +
        name +
        "\n<" +
        "/script>\n\n<template>\n  <div></div>\n</template>";
      activeFile.value = name;
    }
  }

  function removeFile(name: string) {
    if (Object.keys(files.value).length > 1 && confirm(`Delete ${name}?`)) {
      delete files.value[name];
      if (activeFile.value === name) {
        activeFile.value = Object.keys(files.value)[0];
      }
    }
  }

  function resetProject() {
    const preset = currentPresetData.value;
    files.value = { ...preset.files };
    activeFile.value = Object.keys(preset.files)[0];
    crossFileIssues.value = [];
    selectedIssue.value = null;
  }

  function selectPreset(presetId: string, analyzeAll: () => void) {
    currentPreset.value = presetId;
    const preset = PRESETS.find((p) => p.id === presetId);
    if (preset) {
      const firstFile = Object.keys(preset.files)[0];
      const newSource = preset.files[firstFile] || "";
      // Update currentSource BEFORE files to prevent the currentSource watcher
      // from overwriting new files with stale editor content
      currentSource.value = newSource;
      files.value = { ...preset.files };
      activeFile.value = firstFile;
      crossFileIssues.value = [];
      selectedIssue.value = null;
      void nextTick(() => {
        monacoEditorRef.value?.setValue(newSource);
        analyzeAll();
      });
    }
  }

  function selectIssue(issue: CrossFileIssue) {
    selectedIssue.value = issue;
    activeFile.value = issue.file;
    const newSource = files.value[issue.file] || "";
    currentSource.value = newSource;
    monacoEditorRef.value?.setValue(newSource);
  }

  function selectFile(name: string) {
    activeFile.value = name;
    const newSource = files.value[name] || "";
    currentSource.value = newSource;
    monacoEditorRef.value?.setValue(newSource);
  }

  return {
    currentPreset,
    currentPresetData,
    files,
    activeFile,
    monacoEditorRef,
    fileNames,
    croquisResults,
    crossFileIssues,
    analysisTime,
    isAnalyzing,
    selectedIssue,
    currentSource,
    editorLanguage,
    dependencyGraph,
    addFile,
    removeFile,
    resetProject,
    selectPreset,
    selectIssue,
    selectFile,
  };
}
