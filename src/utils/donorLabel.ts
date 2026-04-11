import { SHOPS } from "../types/game";
import type { CombatSkillInput } from "../types/planner";
import { skillDisplayName } from "../types/planner";

const SHOP_INDEX_TO_NAME = SHOPS; // index 0=论剑, 1=诸天, 2=宗门, 3=道蕴, 4=百族

/**
 * 解析 otherPagesConsumed 的 key，返回可读的来源描述
 * key 格式：
 *   "pool_0" ~ "pool_4" → "论剑狗粮池" 等
 *   "0" ~ "5" → 战斗神通的显示名
 */
export function donorLabel(key: string, skills: CombatSkillInput[]): string {
  if (key.startsWith("pool_")) {
    const idx = parseInt(key.substring(5));
    const shopName = SHOP_INDEX_TO_NAME[idx] ?? `商店${idx}`;
    return `「${shopName}」狗粮池`;
  }
  const idx = parseInt(key);
  if (idx >= 0 && idx < skills.length) {
    return skillDisplayName(skills[idx]);
  }
  return key;
}

/** 格式化仙品消耗明细 */
export function formatDonors(
  consumed: Record<string, number>,
  skills: CombatSkillInput[]
): string[] {
  return Object.entries(consumed)
    .filter(([, v]) => (v as number) > 0)
    .map(([key, v]) => `${donorLabel(key, skills)} ${v}张`);
}
