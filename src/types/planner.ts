import type { Shop, SkillLevel } from "./game";

/** 单个战斗神通输入 */
export interface CombatSkillInput {
  shop: Shop;
  currentLevel: SkillLevel;
  remainingPages: number; // 当前剩余本体书页（40 的整数倍）
  targetLevel: SkillLevel;
  label: string; // 备注名称
}

/** 高级设置 */
export interface AdvancedSettings {
  conversionStones: number;
  nonCombatPools: Record<Shop, number>; // 每商店非战斗书页池
  weeklyShopIncome: Record<Shop, number>; // 每商店每周获取次数（默认 1）
  weeklyPurpleIncome: number;
  weeklyBlueIncome: number;
}

/** 完整的规划输入 */
export interface PlannerInput {
  combatSkills: CombatSkillInput[];
  purplePages: number;
  bluePages: number;
  advanced: AdvancedSettings;
}

/** 默认高级设置 */
export function defaultAdvancedSettings(): AdvancedSettings {
  return {
    conversionStones: 0,
    nonCombatPools: { 论剑: 0, 诸天: 0, 宗门: 0, 道蕴: 0, 百族: 0 },
    weeklyShopIncome: { 论剑: 1, 诸天: 1, 宗门: 1, 道蕴: 1, 百族: 1 },
    weeklyPurpleIncome: 0,
    weeklyBlueIncome: 0,
  };
}

/** 默认战斗神通 */
export function defaultCombatSkill(): CombatSkillInput {
  return {
    shop: "论剑",
    currentLevel: "1星",
    remainingPages: 0,
    targetLevel: "天1",
    label: "",
  };
}

/** 单次转换操作 */
export interface ConversionAction {
  shop: Shop;
  targetSkillIndex: number;
  usedStone: boolean;
  pages: number;
}

/** 单次升级操作 */
export interface UpgradeAction {
  skillIndex: number;
  fromLevel: SkillLevel;
  toLevel: SkillLevel;
  selfPagesUsed: number;
  otherPagesConsumed: Record<Shop, number>;
  purplePagesUsed: number;
  bluePagesUsed: number;
}

/** 每周的商店获取推荐 */
export interface ShopAcquisition {
  shop: Shop;
  targetSkillIndex: number | null; // null 表示进入非战斗池
  pages: number; // 获取的书页数量（通常 40 × 次数）
}

/** 单周规划 */
export interface WeekPlan {
  week: number;
  acquisitions: ShopAcquisition[];
  conversions: ConversionAction[];
  upgrades: UpgradeAction[];
  /** 本周结束后的状态快照 */
  snapshot: {
    skillLevels: SkillLevel[];
    skillPages: number[];
    nonCombatPools: Record<Shop, number>;
    purplePages: number;
    bluePages: number;
    conversionStonesLeft: number;
  };
}

/** 规划输出 */
export interface PlannerOutput {
  feasible: boolean;
  weeks: WeekPlan[];
  unreachableReasons: string[]; // 不可达时的原因说明
  finalLevels: SkillLevel[];
}
