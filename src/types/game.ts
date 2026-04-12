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

/** 商店默认收入周期 */
export interface ShopIncomeDefault {
  cycleWeeks: number;
  batchCount: number;
}

export function defaultIncomeForShop(shop: Shop): ShopIncomeDefault {
  switch (shop) {
    case "论剑": return { cycleWeeks: 2, batchCount: 2 };
    case "诸天": return { cycleWeeks: 2, batchCount: 2 };
    case "宗门": return { cycleWeeks: 1, batchCount: 1 };
    case "道蕴": return { cycleWeeks: 3, batchCount: 1 };
    case "百族": return { cycleWeeks: 4, batchCount: 1 };
  }
}

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
  goldPages: number;  // 金色书页（任意其他神通的本体书页）
  purplePages: number; // 紫色书页
  bluePages: number;   // 蓝色书页
}

/** 消耗分类（由境界+职业决定） */
export type CostCategory =
  | "renjie_triple"
  | "fanxu_triple"
  | "fanxu_baizu"
  | "heti_triple"
  | "heti_baizu"
  | "dacheng_baizu";

/** 根据境界+职业确定消耗分类 */
export function getCostCategory(realm: Realm, cls: SkillClass): CostCategory {
  const isTriple = cls === "剑" || cls === "火" || cls === "雷";
  if (realm === "人界一" || realm === "人界二") {
    return "renjie_triple"; // 人界只有三系
  }
  if (realm === "返虚") {
    return isTriple ? "fanxu_triple" : "fanxu_baizu";
  }
  // 合体/大乘/渡劫
  if (isTriple) return "heti_triple";
  if (realm === "大乘") return "dacheng_baizu";
  return "heti_baizu"; // 合体百族（渡劫没有百族，防御性归入此处）
}

/**
 * Table 1: 人界 三系 (人界一/人界二 + 剑/火/雷, max 天3 = 11 levels)
 * 索引 0 = 1星→2星, ..., 索引 10 = 天2→天3
 */
const COSTS_RENJIE_TRIPLE: UpgradeCost[] = [
  { selfPages: 40,  goldPages: 0,   purplePages: 0,   bluePages: 0 },
  { selfPages: 40,  goldPages: 0,   purplePages: 30,  bluePages: 100 },
  { selfPages: 40,  goldPages: 0,   purplePages: 60,  bluePages: 150 },
  { selfPages: 40,  goldPages: 40,  purplePages: 60,  bluePages: 150 },
  { selfPages: 40,  goldPages: 40,  purplePages: 90,  bluePages: 220 },
  { selfPages: 40,  goldPages: 40,  purplePages: 90,  bluePages: 220 },
  { selfPages: 40,  goldPages: 40,  purplePages: 90,  bluePages: 220 },
  { selfPages: 80,  goldPages: 80,  purplePages: 200, bluePages: 500 },
  { selfPages: 80,  goldPages: 80,  purplePages: 200, bluePages: 500 },
  { selfPages: 120, goldPages: 120, purplePages: 300, bluePages: 750 },
  { selfPages: 120, goldPages: 120, purplePages: 300, bluePages: 750 },
];

/**
 * Table 2: 返虚 三系 (返虚 + 剑/火/雷, max 天3 = 11 levels)
 * 索引 0 = 1星→2星, ..., 索引 10 = 天2→天3
 */
const COSTS_FANXU_TRIPLE: UpgradeCost[] = [
  { selfPages: 0,   goldPages: 80,  purplePages: 100,  bluePages: 200 },
  { selfPages: 0,   goldPages: 80,  purplePages: 100,  bluePages: 200 },
  { selfPages: 40,  goldPages: 80,  purplePages: 150,  bluePages: 350 },
  { selfPages: 80,  goldPages: 80,  purplePages: 200,  bluePages: 500 },
  { selfPages: 80,  goldPages: 120, purplePages: 250,  bluePages: 650 },
  { selfPages: 80,  goldPages: 160, purplePages: 350,  bluePages: 900 },
  { selfPages: 120, goldPages: 200, purplePages: 500,  bluePages: 1200 },
  { selfPages: 160, goldPages: 240, purplePages: 600,  bluePages: 1500 },
  { selfPages: 200, goldPages: 320, purplePages: 700,  bluePages: 1800 },
  { selfPages: 240, goldPages: 400, purplePages: 800,  bluePages: 2100 },
  { selfPages: 280, goldPages: 480, purplePages: 1000, bluePages: 2400 },
];

/**
 * Table 3: 返虚 百族 (返虚 + 百族, max 天3 = 11 levels)
 * 索引 0 = 1星→2星, ..., 索引 10 = 天2→天3
 */
const COSTS_FANXU_BAIZU: UpgradeCost[] = [
  { selfPages: 0,   goldPages: 120, purplePages: 100,  bluePages: 300 },
  { selfPages: 0,   goldPages: 120, purplePages: 150,  bluePages: 350 },
  { selfPages: 40,  goldPages: 160, purplePages: 250,  bluePages: 600 },
  { selfPages: 80,  goldPages: 160, purplePages: 300,  bluePages: 800 },
  { selfPages: 80,  goldPages: 240, purplePages: 450,  bluePages: 1100 },
  { selfPages: 80,  goldPages: 320, purplePages: 500,  bluePages: 1300 },
  { selfPages: 120, goldPages: 400, purplePages: 600,  bluePages: 1600 },
  { selfPages: 160, goldPages: 480, purplePages: 750,  bluePages: 1900 },
  { selfPages: 200, goldPages: 560, purplePages: 900,  bluePages: 2200 },
  { selfPages: 240, goldPages: 640, purplePages: 1000, bluePages: 2500 },
  { selfPages: 280, goldPages: 720, purplePages: 1200, bluePages: 2800 },
];

/**
 * Table 4: 合体/大乘/渡劫 三系 (合体/大乘/渡劫 + 剑/火/雷, max 天5 = 13 levels)
 * 索引 0 = 1星→2星, ..., 索引 12 = 天4→天5
 */
const COSTS_HETI_TRIPLE: UpgradeCost[] = [
  { selfPages: 40,  goldPages: 120, purplePages: 100,  bluePages: 200 },
  { selfPages: 40,  goldPages: 120, purplePages: 100,  bluePages: 200 },
  { selfPages: 40,  goldPages: 120, purplePages: 150,  bluePages: 350 },
  { selfPages: 80,  goldPages: 120, purplePages: 200,  bluePages: 500 },
  { selfPages: 80,  goldPages: 160, purplePages: 250,  bluePages: 650 },
  { selfPages: 120, goldPages: 200, purplePages: 350,  bluePages: 900 },
  { selfPages: 120, goldPages: 240, purplePages: 500,  bluePages: 1200 },
  { selfPages: 160, goldPages: 280, purplePages: 600,  bluePages: 1500 },
  { selfPages: 200, goldPages: 360, purplePages: 700,  bluePages: 1800 },
  { selfPages: 240, goldPages: 440, purplePages: 800,  bluePages: 2100 },
  { selfPages: 280, goldPages: 520, purplePages: 1000, bluePages: 2400 },
  { selfPages: 320, goldPages: 600, purplePages: 1000, bluePages: 2400 },
  { selfPages: 360, goldPages: 680, purplePages: 1000, bluePages: 2400 },
];

/**
 * Table 5: 合体/大乘 百族 (合体/大乘 + 百族, max 天3 = 11 levels)
 * 索引 0 = 1星→2星, ..., 索引 10 = 天2→天3
 */
const COSTS_HETI_BAIZU: UpgradeCost[] = [
  { selfPages: 0,   goldPages: 160, purplePages: 100,  bluePages: 300 },
  { selfPages: 0,   goldPages: 160, purplePages: 150,  bluePages: 350 },
  { selfPages: 40,  goldPages: 200, purplePages: 250,  bluePages: 600 },
  { selfPages: 80,  goldPages: 200, purplePages: 300,  bluePages: 800 },
  { selfPages: 80,  goldPages: 280, purplePages: 450,  bluePages: 1100 },
  { selfPages: 80,  goldPages: 360, purplePages: 500,  bluePages: 1300 },
  { selfPages: 120, goldPages: 440, purplePages: 600,  bluePages: 1600 },
  { selfPages: 160, goldPages: 520, purplePages: 750,  bluePages: 1900 },
  { selfPages: 200, goldPages: 600, purplePages: 900,  bluePages: 2200 },
  { selfPages: 240, goldPages: 680, purplePages: 1000, bluePages: 2500 },
  { selfPages: 280, goldPages: 760, purplePages: 1200, bluePages: 2800 },
];

/**
 * Table 6: 大乘 百族 (大乘 + 百族, max 天5 = 13 levels)
 * 1星→天3 同合体百族，新增天3→天4、天4→天5
 */
const COSTS_DACHENG_BAIZU: UpgradeCost[] = [
  { selfPages: 0,   goldPages: 160, purplePages: 100,  bluePages: 300 },
  { selfPages: 0,   goldPages: 160, purplePages: 150,  bluePages: 350 },
  { selfPages: 40,  goldPages: 200, purplePages: 250,  bluePages: 600 },
  { selfPages: 80,  goldPages: 200, purplePages: 300,  bluePages: 800 },
  { selfPages: 80,  goldPages: 280, purplePages: 450,  bluePages: 1100 },
  { selfPages: 80,  goldPages: 360, purplePages: 500,  bluePages: 1300 },
  { selfPages: 120, goldPages: 440, purplePages: 600,  bluePages: 1600 },
  { selfPages: 160, goldPages: 520, purplePages: 750,  bluePages: 1900 },
  { selfPages: 200, goldPages: 600, purplePages: 900,  bluePages: 2200 },
  { selfPages: 240, goldPages: 680, purplePages: 1000, bluePages: 2500 },
  { selfPages: 280, goldPages: 760, purplePages: 1200, bluePages: 2800 },
  { selfPages: 320, goldPages: 840, purplePages: 1400, bluePages: 3100 },
  { selfPages: 360, goldPages: 920, purplePages: 1600, bluePages: 3400 },
];

/** 根据分类获取消耗表 */
export function getUpgradeCosts(cat: CostCategory): UpgradeCost[] {
  switch (cat) {
    case "renjie_triple":  return COSTS_RENJIE_TRIPLE;
    case "fanxu_triple":   return COSTS_FANXU_TRIPLE;
    case "fanxu_baizu":    return COSTS_FANXU_BAIZU;
    case "heti_triple":    return COSTS_HETI_TRIPLE;
    case "heti_baizu":     return COSTS_HETI_BAIZU;
    case "dacheng_baizu":  return COSTS_DACHENG_BAIZU;
  }
}

/** 根据分类获取最大等级 */
export function maxLevelForCategory(cat: CostCategory): SkillLevel {
  return (cat === "heti_triple" || cat === "dacheng_baizu") ? "天5" : "天3";
}

/** 根据境界+职业获取最大等级 */
export function maxLevel(realm: Realm, cls: SkillClass): SkillLevel {
  return maxLevelForCategory(getCostCategory(realm, cls));
}

/** 根据境界+职业返回可用等级列表（1星 到 maxLevel） */
export function availableLevels(realm: Realm, cls: SkillClass): SkillLevel[] {
  const max = maxLevel(realm, cls);
  const maxIdx = SKILL_LEVELS.indexOf(max);
  return SKILL_LEVELS.slice(0, maxIdx + 1);
}

/** 计算从 fromLevel 升到 toLevel 所需的总材料 */
export function totalCostBetween(from: SkillLevel, to: SkillLevel, realm: Realm, cls: SkillClass): UpgradeCost {
  const costs = getUpgradeCosts(getCostCategory(realm, cls));
  const fromIdx = SKILL_LEVELS.indexOf(from);
  const toIdx = SKILL_LEVELS.indexOf(to);
  const result: UpgradeCost = { selfPages: 0, goldPages: 0, purplePages: 0, bluePages: 0 };
  // costs[0] = 1星→2星 corresponds to fromIdx=0(1星) → toIdx=1(2星), i.e. SKILL_LEVELS index 1
  // So cost table index = SKILL_LEVELS index - 1
  for (let i = fromIdx + 1; i <= toIdx; i++) {
    const costIdx = i - 1; // cost table index: 0 = 1星→2星
    if (costIdx < 0 || costIdx >= costs.length) break;
    result.selfPages += costs[costIdx].selfPages;
    result.goldPages += costs[costIdx].goldPages;
    result.purplePages += costs[costIdx].purplePages;
    result.bluePages += costs[costIdx].bluePages;
  }
  return result;
}
