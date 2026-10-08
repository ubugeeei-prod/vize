export function exampleLinks(path, ja, id) {
  if (id)
    return ja
      ? `[悪い例](${path}#${id}-bad) · [良い例](${path}#${id}-good)`
      : `[Bad](${path}#${id}-bad) · [Good](${path}#${id}-good)`;
  return ja
    ? `[悪い例](${path}#悪い) · [良い例](${path}#良い)`
    : `[Bad](${path}#bad) · [Good](${path}#good)`;
}
