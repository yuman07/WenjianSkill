import type { Realm, Shop, SkillClass, SkillLevel } from "./game";
import { defaultIncomeForShop } from "./game";

/** 单个战斗神通输入（前端用） */
export interface CombatSkillInput {
  realm: Realm;
  skillClass: SkillClass;
  shop: Shop;
  currentLevel: SkillLevel;
  remainingPages: number;
  targetLevel: SkillLevel;
  incomeCycleWeeks: number; // 每 N 周
  incomeBatchCount: number; // 获取 M 本
}

/** 生成显示名称 */
export function skillDisplayName(s: CombatSkillInput): string {
  return `${s.realm}·${s.skillClass}·${s.shop}`;
}

/** 发给 Rust 后端的输入 */
export interface PlannerBackendSkill {
  realm: Realm;
  skillClass: SkillClass;
  shop: Shop;
  currentLevel: SkillLevel;
  remainingPages: number;
  targetLevel: SkillLevel;
  label: string;
  incomeCycleWeeks: number;
  incomeBatchCount: number;
}

/** 单商店狗粮设置 */
export interface FodderIncome {
  initialPages: number; // 当前剩余书页
  cycleWeeks: number;
  batchCount: number;
}

/** 设置 */
export interface AdvancedSettings {
  conversionStones: number;
  freeConversionsPerWeek: number;
  fodderIncome: Record<Shop, FodderIncome>; // 每商店狗粮收入周期
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

/** 默认设置 */
export function defaultAdvancedSettings(): AdvancedSettings {
  const f = (): FodderIncome => ({ initialPages: 0, cycleWeeks: 2, batchCount: 1 });
  return {
    conversionStones: 0,
    freeConversionsPerWeek: 3,
    fodderIncome: { 论剑: f(), 诸天: f(), 宗门: f(), 道蕴: f(), 百族: f() },
    weeklyPurpleIncome: 0,
    weeklyBlueIncome: 0,
  };
}

/** 默认战斗神通 */
export function defaultCombatSkill(): CombatSkillInput {
  const income = defaultIncomeForShop("论剑");
  return {
    realm: "人界一",
    skillClass: "剑",
    shop: "论剑",
    currentLevel: "1星",
    remainingPages: 0,
    targetLevel: "天1",
    incomeCycleWeeks: income.cycleWeeks,
    incomeBatchCount: income.batchCount,
  };
}

/** 单次转换操作 */
export interface ConversionAction {
  shop: Shop;
  targetSkillIndex: number;
  fromSkillIndex: number;
  usedStone: boolean;
  pages: number;
}

/** 单次升级操作 */
export interface UpgradeAction {
  skillIndex: number;
  fromLevel: SkillLevel;
  toLevel: SkillLevel;
  selfPagesUsed: number;
  otherPagesConsumed: Record<string, number>; // skill index → pages taken
  purplePagesUsed: number;
  bluePagesUsed: number;
}

/** 每周的收入 */
export interface SkillIncome {
  skillIndex: number;
  pages: number;
}

/** 狗粮池收入 */
export interface FodderPoolIncome {
  shop: Shop;
  pages: number;
}

/** 单周规划 */
export interface WeekPlan {
  week: number;
  incomes: SkillIncome[];
  fodderIncomes: FodderPoolIncome[];
  conversions: ConversionAction[];
  upgrades: UpgradeAction[];
  snapshot: {
    skillLevels: SkillLevel[];
    skillPages: number[];
    purplePages: number;
    bluePages: number;
    conversionStonesLeft: number;
    fodderPools: [number, number, number, number, number];
  };
}

/** 规划输出 */
export interface PlannerOutput {
  feasible: boolean;
  weeks: WeekPlan[];
  unreachableReasons: string[];
  finalLevels: SkillLevel[];
}
