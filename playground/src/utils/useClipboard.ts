export function useClipboard() {
  function copyToClipboard(text: string) {
    void navigator.clipboard.writeText(text);
  }

  return { copyToClipboard };
}
