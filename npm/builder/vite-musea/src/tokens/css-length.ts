export function cssLengthToPx(value: string | number, baseFontSize = 16): number | null {
  if (typeof value === "number") return value;

  const match = value.trim().match(/^(-?(?:\d+|\d*\.\d+))([a-z%]*)$/i);
  if (!match) return null;

  const amount = Number.parseFloat(match[1]);
  if (!Number.isFinite(amount)) return null;

  switch (match[2]?.toLowerCase()) {
    case "":
    case "px":
      return amount;
    case "rem":
    case "em":
      return amount * baseFontSize;
    default:
      return amount;
  }
}
