export function exampleLinks(path, ja) {
  return ja
    ? `[悪い例](${path}#悪い) · [良い例](${path}#良い)`
    : `[Bad](${path}#bad) · [Good](${path}#good)`;
}
