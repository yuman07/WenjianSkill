/** 境界 */
export type Realm = "人界一" | "人界二" | "返虚" | "合体" | "大乘" | "渡劫";
export const REALMS: Realm[] = ["人界一", "人界二", "返虚", "合体", "大乘", "渡劫"];

/** 职业 */
export type SkillClass = "剑" | "火" | "雷" | "百族";
export const BASE_CLASSES: SkillClass[] = ["剑", "火", "雷"];

/** 拥有百族职业的境界 */
export const REALMS_WITH_BAIZU: Realm[] = ["返虚", "合体", "大乘"];

/** 根据境界返回可选职业 */
export function classesForRealm(realm: Realm): SkillClass[] {
  return REALMS_WITH_BAIZU.includes(realm) ? [...BASE_CLASSES, "百族"] : [...BASE_CLASSES];
}

/** 根据职业返回可选商店 */
export function shopsForClass(cls: SkillClass): Shop[] {
  return cls === "百族" ? ["百族"] : ["论剑", "诸天", "宗门", "道蕴"];
}

/** 商店类型 */
export type Shop = "论剑" | "诸天" | "宗门" | "道蕴" | "百族";

/** 所有商店（按珍贵度从低到高） */
export const SHOPS: Shop[] = ["论剑", "诸天", "宗门", "道蕴", "百族"];

/** 商店珍贵度权重（越高越珍贵，优先消耗低权重的） */
export const SHOP_RARITY: Record<Shop, number> = {
  论剑: 1,
  诸天: 2,
  宗门: 2,
  道蕴: 3,
  百族: 4,
};

/** 神通等级 */
export type SkillLevel =
  | "1星" | "2星" | "3星"
  | "玄1" | "玄2" | "玄3"
  | "地1" | "地2" | "地3"
  | "天1" | "天2" | "天3" | "天4" | "天5";

/** 等级顺序（索引即为等级序号） */
export const SKILL_LEVELS: SkillLevel[] = [
  "1星", "2星", "3星",
  "玄1", "玄2", "玄3",
  "地1", "地2", "地3",
  "天1", "天2", "天3", "天4", "天5",
];

/** 升级消耗：从前一等级升到该等级所需的材料 */
export interface UpgradeCost {
  selfPages: number;   // 本体书页
  otherPages: number;  // 仙品书页（任意其他神通的本体书页）
  purplePages: number; // 紫色书页
  bluePages: number;   // 蓝色书页
}

/**
 * 升级消耗表
 * 索引 0 = 获取（1星），索引 1 = 1星→2星，...，索引 13 = 天4→天5
 */
export const UPGRADE_COSTS: UpgradeCost[] = [
  { selfPages: 40,  otherPages: 0,   purplePages: 0,    bluePages: 0 },
  { selfPages: 40,  otherPages: 0,   purplePages: 100,  bluePages: 200 },
  { selfPages: 0,   otherPages: 120, purplePages: 100,  bluePages: 200 },
  { selfPages: 40,  otherPages: 120, purplePages: 150,  bluePages: 350 },
  { selfPages: 80,  otherPages: 120, purplePages: 200,  bluePages: 500 },
  { selfPages: 80,  otherPages: 160, purplePages: 250,  bluePages: 650 },
  { selfPages: 120, otherPages: 200, purplePages: 350,  bluePages: 900 },
  { selfPages: 120, otherPages: 240, purplePages: 500,  bluePages: 1200 },
  { selfPages: 160, otherPages: 280, purplePages: 600,  bluePages: 1500 },
  { selfPages: 200, otherPages: 360, purplePages: 700,  bluePages: 1800 },
  { selfPages: 240, otherPages: 440, purplePages: 800,  bluePages: 2100 },
  { selfPages: 280, otherPages: 520, purplePages: 1000, bluePages: 2400 },
  { selfPages: 320, otherPages: 600, purplePages: 1000, bluePages: 2400 },
  { selfPages: 360, otherPages: 680, purplePages: 1000, bluePages: 2400 },
];

/** 计算从 fromLevel 升到 toLevel 所需的总材料 */
export function totalCostBetween(from: SkillLevel, to: SkillLevel): UpgradeCost {
  const fromIdx = SKILL_LEVELS.indexOf(from);
  const toIdx = SKILL_LEVELS.indexOf(to);
  const result: UpgradeCost = { selfPages: 0, otherPages: 0, purplePages: 0, bluePages: 0 };
  for (let i = fromIdx + 1; i <= toIdx; i++) {
    result.selfPages += UPGRADE_COSTS[i].selfPages;
    result.otherPages += UPGRADE_COSTS[i].otherPages;
    result.purplePages += UPGRADE_COSTS[i].purplePages;
    result.bluePages += UPGRADE_COSTS[i].bluePages;
  }
  return result;
}
