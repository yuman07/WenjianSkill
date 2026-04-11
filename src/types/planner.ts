import type { Realm, Shop, SkillClass, SkillLevel } from "./game";

/** 单个战斗神通输入（前端用，包含境界和职业用于显示） */
export interface CombatSkillInput {
  realm: Realm;
  skillClass: SkillClass;
  shop: Shop;
  currentLevel: SkillLevel;
  remainingPages: number;
  targetLevel: SkillLevel;
}

/** 生成显示名称 */
export function skillDisplayName(s: CombatSkillInput): string {
  return `${s.realm}·${s.skillClass}·${s.shop}`;
}

/** 发给 Rust 后端的输入（label 由前端自动生成） */
export interface PlannerBackendSkill {
  shop: Shop;
  currentLevel: SkillLevel;
  remainingPages: number;
  targetLevel: SkillLevel;
  label: string;
}

/** 高级设置 */
export interface AdvancedSettings {
  conversionStones: number;
  nonCombatPools: Record<Shop, number>;
  weeklyShopIncome: Record<Shop, number>;
  weeklyPurpleIncome: number;
  weeklyBlueIncome: number;
}

/** 完整的规划输入（发给后端） */
export interface PlannerInput {
  combatSkills: PlannerBackendSkill[];
  purplePages: number;
  bluePages: number;
  advanced: AdvancedSettings;
}

/** 默认高级设置 */
export function defaultAdvancedSettings(): AdvancedSettings {
  return {
    conversionStones: 0,
    nonCombatPools: { 论剑: 0, 诸天: 0, 宗门: 0, 道蕴: 0, 百族: 0 },
    weeklyShopIncome: { 论剑: 1, 诸天: 1, 宗门: 1, 道蕴: 1, 百族: 0 },
    weeklyPurpleIncome: 0,
    weeklyBlueIncome: 0,
  };
}

/** 默认战斗神通 */
export function defaultCombatSkill(): CombatSkillInput {
  return {
    realm: "人界一",
    skillClass: "剑",
    shop: "论剑",
    currentLevel: "1星",
    remainingPages: 0,
    targetLevel: "天1",
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
  targetSkillIndex: number | null;
  pages: number;
}

/** 单周规划 */
export interface WeekPlan {
  week: number;
  acquisitions: ShopAcquisition[];
  conversions: ConversionAction[];
  upgrades: UpgradeAction[];
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
  unreachableReasons: string[];
  finalLevels: SkillLevel[];
}
