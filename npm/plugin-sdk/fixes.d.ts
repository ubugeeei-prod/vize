/** Host-owned UTF-8 replacement proposed by a plugin rule. */
export interface PluginFix {
  ruleId?: string;
  plugin?: string;
  start: number;
  end: number;
  text: string;
}
/** Apply all nonoverlapping host-owned UTF-8 ranges, or throw without editing. */
export declare function applyFixes(source: string, fixes: readonly PluginFix[]): string;
