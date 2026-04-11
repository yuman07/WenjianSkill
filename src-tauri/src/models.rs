use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Realm {
    #[serde(rename = "人界一")]
    RenJie1,
    #[serde(rename = "人界二")]
    RenJie2,
    #[serde(rename = "返虚")]
    FanXu,
    #[serde(rename = "合体")]
    HeTi,
    #[serde(rename = "大乘")]
    DaCheng,
    #[serde(rename = "渡劫")]
    DuJie,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillClass {
    #[serde(rename = "剑")]
    Jian,
    #[serde(rename = "火")]
    Huo,
    #[serde(rename = "雷")]
    Lei,
    #[serde(rename = "百族")]
    BaiZu,
}

impl SkillClass {
    pub fn is_triple(self) -> bool {
        matches!(self, SkillClass::Jian | SkillClass::Huo | SkillClass::Lei)
    }
}

/// 消耗分类（由境界+职业决定）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostCategory {
    RenjieTriple,
    FanxuTriple,
    FanxuBaizu,
    HetiTriple,
    HetiBaizu,
    DachengBaizu,
}

pub fn cost_category(realm: Realm, skill_class: SkillClass) -> CostCategory {
    match (realm, skill_class.is_triple()) {
        (Realm::RenJie1 | Realm::RenJie2, _) => CostCategory::RenjieTriple,
        (Realm::FanXu, true)  => CostCategory::FanxuTriple,
        (Realm::FanXu, false) => CostCategory::FanxuBaizu,
        (Realm::HeTi | Realm::DuJie, true)  => CostCategory::HetiTriple,
        (Realm::DaCheng, true) => CostCategory::HetiTriple,
        (Realm::DaCheng, false) => CostCategory::DachengBaizu,
        (Realm::HeTi | Realm::DuJie, false) => CostCategory::HetiBaizu,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Shop {
    #[serde(rename = "论剑")]
    LunJian,
    #[serde(rename = "诸天")]
    ZhuTian,
    #[serde(rename = "宗门")]
    ZongMen,
    #[serde(rename = "道蕴")]
    DaoYun,
    #[serde(rename = "百族")]
    BaiZu,
}

impl Shop {
    pub const ALL: [Shop; 5] = [
        Shop::LunJian,
        Shop::ZhuTian,
        Shop::ZongMen,
        Shop::DaoYun,
        Shop::BaiZu,
    ];

    /// 珍贵度权重（越低越优先消耗）
    pub fn rarity(self) -> u32 {
        match self {
            Shop::LunJian => 1,
            Shop::ZhuTian => 2,
            Shop::ZongMen => 2,
            Shop::DaoYun => 3,
            Shop::BaiZu => 4,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Shop::LunJian => 0,
            Shop::ZhuTian => 1,
            Shop::ZongMen => 2,
            Shop::DaoYun => 3,
            Shop::BaiZu => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SkillLevel {
    #[serde(rename = "1星")]
    Star1,
    #[serde(rename = "2星")]
    Star2,
    #[serde(rename = "3星")]
    Star3,
    #[serde(rename = "玄1")]
    Xuan1,
    #[serde(rename = "玄2")]
    Xuan2,
    #[serde(rename = "玄3")]
    Xuan3,
    #[serde(rename = "地1")]
    Di1,
    #[serde(rename = "地2")]
    Di2,
    #[serde(rename = "地3")]
    Di3,
    #[serde(rename = "天1")]
    Tian1,
    #[serde(rename = "天2")]
    Tian2,
    #[serde(rename = "天3")]
    Tian3,
    #[serde(rename = "天4")]
    Tian4,
    #[serde(rename = "天5")]
    Tian5,
}

impl SkillLevel {
    pub const ALL: [SkillLevel; 14] = [
        SkillLevel::Star1,
        SkillLevel::Star2,
        SkillLevel::Star3,
        SkillLevel::Xuan1,
        SkillLevel::Xuan2,
        SkillLevel::Xuan3,
        SkillLevel::Di1,
        SkillLevel::Di2,
        SkillLevel::Di3,
        SkillLevel::Tian1,
        SkillLevel::Tian2,
        SkillLevel::Tian3,
        SkillLevel::Tian4,
        SkillLevel::Tian5,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn next(self) -> Option<SkillLevel> {
        let i = self.index();
        if i + 1 < Self::ALL.len() {
            Some(Self::ALL[i + 1])
        } else {
            None
        }
    }
}

/// 升级消耗（从前一等级升到该等级）
#[derive(Debug, Clone, Copy)]
pub struct UpgradeCost {
    pub self_pages: u32,
    pub other_pages: u32,
    pub purple_pages: u32,
    pub blue_pages: u32,
}

/// Table 1: 人界 三系 (人界一/人界二 + 剑/火/雷, max 天3 = 11 levels)
/// 索引 0 = 1星→2星, ..., 索引 10 = 天2→天3
pub const COSTS_RENJIE_TRIPLE: [UpgradeCost; 11] = [
    UpgradeCost { self_pages: 40,  other_pages: 0,   purple_pages: 0,   blue_pages: 0 },
    UpgradeCost { self_pages: 40,  other_pages: 0,   purple_pages: 30,  blue_pages: 100 },
    UpgradeCost { self_pages: 40,  other_pages: 0,   purple_pages: 60,  blue_pages: 150 },
    UpgradeCost { self_pages: 40,  other_pages: 40,  purple_pages: 60,  blue_pages: 150 },
    UpgradeCost { self_pages: 40,  other_pages: 40,  purple_pages: 90,  blue_pages: 220 },
    UpgradeCost { self_pages: 40,  other_pages: 40,  purple_pages: 90,  blue_pages: 220 },
    UpgradeCost { self_pages: 40,  other_pages: 40,  purple_pages: 90,  blue_pages: 220 },
    UpgradeCost { self_pages: 80,  other_pages: 80,  purple_pages: 200, blue_pages: 500 },
    UpgradeCost { self_pages: 80,  other_pages: 80,  purple_pages: 200, blue_pages: 500 },
    UpgradeCost { self_pages: 120, other_pages: 120, purple_pages: 300, blue_pages: 750 },
    UpgradeCost { self_pages: 120, other_pages: 120, purple_pages: 300, blue_pages: 750 },
];

/// Table 2: 返虚 三系 (返虚 + 剑/火/雷, max 天3 = 11 levels)
pub const COSTS_FANXU_TRIPLE: [UpgradeCost; 11] = [
    UpgradeCost { self_pages: 0,   other_pages: 80,  purple_pages: 100,  blue_pages: 200 },
    UpgradeCost { self_pages: 0,   other_pages: 80,  purple_pages: 100,  blue_pages: 200 },
    UpgradeCost { self_pages: 40,  other_pages: 80,  purple_pages: 150,  blue_pages: 350 },
    UpgradeCost { self_pages: 80,  other_pages: 80,  purple_pages: 200,  blue_pages: 500 },
    UpgradeCost { self_pages: 80,  other_pages: 120, purple_pages: 250,  blue_pages: 650 },
    UpgradeCost { self_pages: 80,  other_pages: 160, purple_pages: 350,  blue_pages: 900 },
    UpgradeCost { self_pages: 120, other_pages: 200, purple_pages: 500,  blue_pages: 1200 },
    UpgradeCost { self_pages: 160, other_pages: 240, purple_pages: 600,  blue_pages: 1500 },
    UpgradeCost { self_pages: 200, other_pages: 320, purple_pages: 700,  blue_pages: 1800 },
    UpgradeCost { self_pages: 240, other_pages: 400, purple_pages: 800,  blue_pages: 2100 },
    UpgradeCost { self_pages: 280, other_pages: 480, purple_pages: 1000, blue_pages: 2400 },
];

/// Table 3: 返虚 百族 (返虚 + 百族, max 天3 = 11 levels)
pub const COSTS_FANXU_BAIZU: [UpgradeCost; 11] = [
    UpgradeCost { self_pages: 0,   other_pages: 120, purple_pages: 100,  blue_pages: 300 },
    UpgradeCost { self_pages: 0,   other_pages: 120, purple_pages: 150,  blue_pages: 350 },
    UpgradeCost { self_pages: 40,  other_pages: 160, purple_pages: 250,  blue_pages: 600 },
    UpgradeCost { self_pages: 80,  other_pages: 160, purple_pages: 300,  blue_pages: 800 },
    UpgradeCost { self_pages: 80,  other_pages: 240, purple_pages: 450,  blue_pages: 1100 },
    UpgradeCost { self_pages: 80,  other_pages: 320, purple_pages: 500,  blue_pages: 1300 },
    UpgradeCost { self_pages: 120, other_pages: 400, purple_pages: 600,  blue_pages: 1600 },
    UpgradeCost { self_pages: 160, other_pages: 480, purple_pages: 750,  blue_pages: 1900 },
    UpgradeCost { self_pages: 200, other_pages: 560, purple_pages: 900,  blue_pages: 2200 },
    UpgradeCost { self_pages: 240, other_pages: 640, purple_pages: 1000, blue_pages: 2500 },
    UpgradeCost { self_pages: 280, other_pages: 720, purple_pages: 1200, blue_pages: 2800 },
];

/// Table 4: 合体/大乘/渡劫 三系 (合体/大乘/渡劫 + 剑/火/雷, max 天5 = 13 levels)
pub const COSTS_HETI_TRIPLE: [UpgradeCost; 13] = [
    UpgradeCost { self_pages: 40,  other_pages: 120, purple_pages: 100,  blue_pages: 200 },
    UpgradeCost { self_pages: 40,  other_pages: 120, purple_pages: 100,  blue_pages: 200 },
    UpgradeCost { self_pages: 40,  other_pages: 120, purple_pages: 150,  blue_pages: 350 },
    UpgradeCost { self_pages: 80,  other_pages: 120, purple_pages: 200,  blue_pages: 500 },
    UpgradeCost { self_pages: 80,  other_pages: 160, purple_pages: 250,  blue_pages: 650 },
    UpgradeCost { self_pages: 120, other_pages: 200, purple_pages: 350,  blue_pages: 900 },
    UpgradeCost { self_pages: 120, other_pages: 240, purple_pages: 500,  blue_pages: 1200 },
    UpgradeCost { self_pages: 160, other_pages: 280, purple_pages: 600,  blue_pages: 1500 },
    UpgradeCost { self_pages: 200, other_pages: 360, purple_pages: 700,  blue_pages: 1800 },
    UpgradeCost { self_pages: 240, other_pages: 440, purple_pages: 800,  blue_pages: 2100 },
    UpgradeCost { self_pages: 280, other_pages: 520, purple_pages: 1000, blue_pages: 2400 },
    UpgradeCost { self_pages: 320, other_pages: 600, purple_pages: 1000, blue_pages: 2400 },
    UpgradeCost { self_pages: 360, other_pages: 680, purple_pages: 1000, blue_pages: 2400 },
];

/// Table 5: 合体/大乘 百族 (合体/大乘 + 百族, max 天3 = 11 levels)
pub const COSTS_HETI_BAIZU: [UpgradeCost; 11] = [
    UpgradeCost { self_pages: 0,   other_pages: 160, purple_pages: 100,  blue_pages: 300 },
    UpgradeCost { self_pages: 0,   other_pages: 160, purple_pages: 150,  blue_pages: 350 },
    UpgradeCost { self_pages: 40,  other_pages: 200, purple_pages: 250,  blue_pages: 600 },
    UpgradeCost { self_pages: 80,  other_pages: 200, purple_pages: 300,  blue_pages: 800 },
    UpgradeCost { self_pages: 80,  other_pages: 280, purple_pages: 450,  blue_pages: 1100 },
    UpgradeCost { self_pages: 80,  other_pages: 360, purple_pages: 500,  blue_pages: 1300 },
    UpgradeCost { self_pages: 120, other_pages: 440, purple_pages: 600,  blue_pages: 1600 },
    UpgradeCost { self_pages: 160, other_pages: 520, purple_pages: 750,  blue_pages: 1900 },
    UpgradeCost { self_pages: 200, other_pages: 600, purple_pages: 900,  blue_pages: 2200 },
    UpgradeCost { self_pages: 240, other_pages: 680, purple_pages: 1000, blue_pages: 2500 },
    UpgradeCost { self_pages: 280, other_pages: 760, purple_pages: 1200, blue_pages: 2800 },
];

/// Table 6: 大乘 百族 (大乘 + 百族, max 天5 = 13 levels)
pub const COSTS_DACHENG_BAIZU: [UpgradeCost; 13] = [
    UpgradeCost { self_pages: 0,   other_pages: 160, purple_pages: 100,  blue_pages: 300 },
    UpgradeCost { self_pages: 0,   other_pages: 160, purple_pages: 150,  blue_pages: 350 },
    UpgradeCost { self_pages: 40,  other_pages: 200, purple_pages: 250,  blue_pages: 600 },
    UpgradeCost { self_pages: 80,  other_pages: 200, purple_pages: 300,  blue_pages: 800 },
    UpgradeCost { self_pages: 80,  other_pages: 280, purple_pages: 450,  blue_pages: 1100 },
    UpgradeCost { self_pages: 80,  other_pages: 360, purple_pages: 500,  blue_pages: 1300 },
    UpgradeCost { self_pages: 120, other_pages: 440, purple_pages: 600,  blue_pages: 1600 },
    UpgradeCost { self_pages: 160, other_pages: 520, purple_pages: 750,  blue_pages: 1900 },
    UpgradeCost { self_pages: 200, other_pages: 600, purple_pages: 900,  blue_pages: 2200 },
    UpgradeCost { self_pages: 240, other_pages: 680, purple_pages: 1000, blue_pages: 2500 },
    UpgradeCost { self_pages: 280, other_pages: 760, purple_pages: 1200, blue_pages: 2800 },
    UpgradeCost { self_pages: 320, other_pages: 840, purple_pages: 1400, blue_pages: 3100 },
    UpgradeCost { self_pages: 360, other_pages: 920, purple_pages: 1600, blue_pages: 3400 },
];

/// 根据分类获取消耗表
pub fn upgrade_costs_for_category(cat: CostCategory) -> &'static [UpgradeCost] {
    match cat {
        CostCategory::RenjieTriple  => &COSTS_RENJIE_TRIPLE,
        CostCategory::FanxuTriple   => &COSTS_FANXU_TRIPLE,
        CostCategory::FanxuBaizu    => &COSTS_FANXU_BAIZU,
        CostCategory::HetiTriple    => &COSTS_HETI_TRIPLE,
        CostCategory::HetiBaizu     => &COSTS_HETI_BAIZU,
        CostCategory::DachengBaizu  => &COSTS_DACHENG_BAIZU,
    }
}

/// 根据境界+职业获取最大等级
pub fn max_level(realm: Realm, skill_class: SkillClass) -> SkillLevel {
    match cost_category(realm, skill_class) {
        CostCategory::HetiTriple | CostCategory::DachengBaizu => SkillLevel::Tian5,
        _ => SkillLevel::Tian3,
    }
}

/// 计算从 from_level 升到 to_level 所需总材料（根据境界+职业选择消耗表）
pub fn total_cost_between(from: SkillLevel, to: SkillLevel, realm: Realm, skill_class: SkillClass) -> UpgradeCost {
    let costs = upgrade_costs_for_category(cost_category(realm, skill_class));
    let fi = from.index();
    let ti = to.index();
    let mut result = UpgradeCost {
        self_pages: 0,
        other_pages: 0,
        purple_pages: 0,
        blue_pages: 0,
    };
    // costs[0] = 1星→2星 corresponds to SkillLevel index 1
    // So cost table index = SkillLevel index - 1
    for i in (fi + 1)..=ti {
        let cost_idx = i - 1; // 0-based into cost table
        if cost_idx >= costs.len() { break; }
        result.self_pages += costs[cost_idx].self_pages;
        result.other_pages += costs[cost_idx].other_pages;
        result.purple_pages += costs[cost_idx].purple_pages;
        result.blue_pages += costs[cost_idx].blue_pages;
    }
    result
}

// --- 规划输入/输出 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatSkillInput {
    pub realm: Realm,
    #[serde(rename = "skillClass")]
    pub skill_class: SkillClass,
    pub shop: Shop,
    #[serde(rename = "currentLevel")]
    pub current_level: SkillLevel,
    #[serde(rename = "remainingPages")]
    pub remaining_pages: u32,
    #[serde(rename = "targetLevel")]
    pub target_level: SkillLevel,
    pub label: String,
    #[serde(rename = "incomeCycleWeeks")]
    pub income_cycle_weeks: u32,
    #[serde(rename = "incomeBatchCount")]
    pub income_batch_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FodderIncomeEntry {
    #[serde(rename = "initialPages")]
    pub initial_pages: u32,
    #[serde(rename = "cycleWeeks")]
    pub cycle_weeks: u32,
    #[serde(rename = "batchCount")]
    pub batch_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FodderIncomeMap {
    #[serde(rename = "论剑")]
    pub lun_jian: FodderIncomeEntry,
    #[serde(rename = "诸天")]
    pub zhu_tian: FodderIncomeEntry,
    #[serde(rename = "宗门")]
    pub zong_men: FodderIncomeEntry,
    #[serde(rename = "道蕴")]
    pub dao_yun: FodderIncomeEntry,
    #[serde(rename = "百族")]
    pub bai_zu: FodderIncomeEntry,
}

impl FodderIncomeMap {
    pub fn get(&self, shop: Shop) -> &FodderIncomeEntry {
        match shop {
            Shop::LunJian => &self.lun_jian,
            Shop::ZhuTian => &self.zhu_tian,
            Shop::ZongMen => &self.zong_men,
            Shop::DaoYun => &self.dao_yun,
            Shop::BaiZu => &self.bai_zu,
        }
    }

    pub fn initial_pages(&self, shop: Shop) -> u32 {
        self.get(shop).initial_pages
    }

    /// Total pages available over W weeks = initial + income
    pub fn total_pages(&self, shop: Shop, weeks: u32) -> u32 {
        let e = self.get(shop);
        let income = if weeks == 0 || e.cycle_weeks == 0 { 0 }
        else { (weeks / e.cycle_weeks) * e.batch_count * 40 };
        e.initial_pages + income
    }

    pub fn pages_in_week(&self, shop: Shop, week: u32) -> u32 {
        let e = self.get(shop);
        if e.cycle_weeks == 0 || week == 0 { return 0; }
        if week % e.cycle_weeks == 0 { e.batch_count * 40 } else { 0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedSettings {
    #[serde(rename = "conversionStones")]
    pub conversion_stones: u32,
    #[serde(rename = "freeConversionsPerWeek")]
    pub free_conversions_per_week: u32,
    #[serde(rename = "fodderIncome")]
    pub fodder_income: FodderIncomeMap,
    #[serde(rename = "weeklyPurpleIncome")]
    pub weekly_purple_income: u32,
    #[serde(rename = "weeklyBlueIncome")]
    pub weekly_blue_income: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerInput {
    #[serde(rename = "combatSkills")]
    pub combat_skills: Vec<CombatSkillInput>,
    #[serde(rename = "purplePages")]
    pub purple_pages: u32,
    #[serde(rename = "bluePages")]
    pub blue_pages: u32,
    pub advanced: AdvancedSettings,
}

// --- 规划输出 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionAction {
    pub shop: Shop,
    #[serde(rename = "targetSkillIndex")]
    pub target_skill_index: usize,
    #[serde(rename = "fromSkillIndex")]
    pub from_skill_index: usize,
    #[serde(rename = "usedStone")]
    pub used_stone: bool,
    pub pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeAction {
    #[serde(rename = "skillIndex")]
    pub skill_index: usize,
    #[serde(rename = "fromLevel")]
    pub from_level: SkillLevel,
    #[serde(rename = "toLevel")]
    pub to_level: SkillLevel,
    #[serde(rename = "selfPagesUsed")]
    pub self_pages_used: u32,
    /// key = donor skill index as string, value = pages consumed
    #[serde(rename = "otherPagesConsumed")]
    pub other_pages_consumed: std::collections::HashMap<String, u32>,
    #[serde(rename = "purplePagesUsed")]
    pub purple_pages_used: u32,
    #[serde(rename = "bluePagesUsed")]
    pub blue_pages_used: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillIncome {
    #[serde(rename = "skillIndex")]
    pub skill_index: usize,
    pub pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FodderPoolIncome {
    pub shop: Shop,
    pub pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    #[serde(rename = "skillLevels")]
    pub skill_levels: Vec<SkillLevel>,
    #[serde(rename = "skillPages")]
    pub skill_pages: Vec<u32>,
    #[serde(rename = "purplePages")]
    pub purple_pages: u32,
    #[serde(rename = "bluePages")]
    pub blue_pages: u32,
    #[serde(rename = "conversionStonesLeft")]
    pub conversion_stones_left: u32,
    /// Per-shop fodder pool remaining pages [论剑, 诸天, 宗门, 道蕴, 百族]
    #[serde(rename = "fodderPools")]
    pub fodder_pools: [u32; 5],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekPlan {
    pub week: u32,
    pub incomes: Vec<SkillIncome>,
    #[serde(rename = "fodderIncomes")]
    pub fodder_incomes: Vec<FodderPoolIncome>,
    pub conversions: Vec<ConversionAction>,
    pub upgrades: Vec<UpgradeAction>,
    pub snapshot: StateSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerOutput {
    pub feasible: bool,
    pub weeks: Vec<WeekPlan>,
    #[serde(rename = "unreachableReasons")]
    pub unreachable_reasons: Vec<String>,
    #[serde(rename = "finalLevels")]
    pub final_levels: Vec<SkillLevel>,
}
