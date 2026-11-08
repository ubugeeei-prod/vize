import { defineFrescoProvider } from "@vizejs/fresco";

export interface ExternalMeterTheme {
  filled: string;
  empty: string;
  foreground: string;
}

export const externalMeterTheme = defineFrescoProvider<ExternalMeterTheme>("external-meter-theme", {
  filled: "█",
  empty: "░",
  foreground: "cyan",
});
