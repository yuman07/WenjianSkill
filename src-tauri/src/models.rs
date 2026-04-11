use serde::{Deserialize, Serialize};

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

    pub fn display_name(self) -> &'static str {
        match self {
            SkillLevel::Star1 => "1星",
            SkillLevel::Star2 => "2星",
            SkillLevel::Star3 => "3星",
            SkillLevel::Xuan1 => "玄1",
            SkillLevel::Xuan2 => "玄2",
            SkillLevel::Xuan3 => "玄3",
            SkillLevel::Di1 => "地1",
            SkillLevel::Di2 => "地2",
            SkillLevel::Di3 => "地3",
            SkillLevel::Tian1 => "天1",
            SkillLevel::Tian2 => "天2",
            SkillLevel::Tian3 => "天3",
            SkillLevel::Tian4 => "天4",
            SkillLevel::Tian5 => "天5",
        }
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

/// 升级消耗表，索引 0 = 获取(1星), 索引 1 = 1星→2星, ..., 索引 13 = 天4→天5
pub const UPGRADE_COSTS: [UpgradeCost; 14] = [
    UpgradeCost { self_pages: 40,  other_pages: 0,   purple_pages: 0,    blue_pages: 0 },
    UpgradeCost { self_pages: 40,  other_pages: 0,   purple_pages: 100,  blue_pages: 200 },
    UpgradeCost { self_pages: 0,   other_pages: 120, purple_pages: 100,  blue_pages: 200 },
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

/// 计算从 from_level 升到 to_level 所需总材料
pub fn total_cost_between(from: SkillLevel, to: SkillLevel) -> UpgradeCost {
    let fi = from.index();
    let ti = to.index();
    let mut result = UpgradeCost {
        self_pages: 0,
        other_pages: 0,
        purple_pages: 0,
        blue_pages: 0,
    };
    for i in (fi + 1)..=ti {
        result.self_pages += UPGRADE_COSTS[i].self_pages;
        result.other_pages += UPGRADE_COSTS[i].other_pages;
        result.purple_pages += UPGRADE_COSTS[i].purple_pages;
        result.blue_pages += UPGRADE_COSTS[i].blue_pages;
    }
    result
}

// --- 规划输入/输出 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatSkillInput {
    pub shop: Shop,
    #[serde(rename = "currentLevel")]
    pub current_level: SkillLevel,
    #[serde(rename = "remainingPages")]
    pub remaining_pages: u32,
    #[serde(rename = "targetLevel")]
    pub target_level: SkillLevel,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopMap {
    #[serde(rename = "论剑")]
    pub lun_jian: u32,
    #[serde(rename = "诸天")]
    pub zhu_tian: u32,
    #[serde(rename = "宗门")]
    pub zong_men: u32,
    #[serde(rename = "道蕴")]
    pub dao_yun: u32,
    #[serde(rename = "百族")]
    pub bai_zu: u32,
}

impl ShopMap {
    pub fn get(&self, shop: Shop) -> u32 {
        match shop {
            Shop::LunJian => self.lun_jian,
            Shop::ZhuTian => self.zhu_tian,
            Shop::ZongMen => self.zong_men,
            Shop::DaoYun => self.dao_yun,
            Shop::BaiZu => self.bai_zu,
        }
    }

    pub fn set(&mut self, shop: Shop, val: u32) {
        match shop {
            Shop::LunJian => self.lun_jian = val,
            Shop::ZhuTian => self.zhu_tian = val,
            Shop::ZongMen => self.zong_men = val,
            Shop::DaoYun => self.dao_yun = val,
            Shop::BaiZu => self.bai_zu = val,
        }
    }

    pub fn add(&mut self, shop: Shop, val: u32) {
        self.set(shop, self.get(shop) + val);
    }

    pub fn sub_clamped(&mut self, shop: Shop, val: u32) {
        let cur = self.get(shop);
        self.set(shop, cur.saturating_sub(val));
    }

    pub fn zero() -> Self {
        ShopMap {
            lun_jian: 0,
            zhu_tian: 0,
            zong_men: 0,
            dao_yun: 0,
            bai_zu: 0,
        }
    }

    pub fn total(&self) -> u32 {
        self.lun_jian + self.zhu_tian + self.zong_men + self.dao_yun + self.bai_zu
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedSettings {
    #[serde(rename = "conversionStones")]
    pub conversion_stones: u32,
    #[serde(rename = "nonCombatPools")]
    pub non_combat_pools: ShopMap,
    #[serde(rename = "weeklyShopIncome")]
    pub weekly_shop_income: ShopMap,
    #[serde(rename = "daoyunCycleWeeks")]
    pub daoyun_cycle_weeks: u32,
    #[serde(rename = "baizuCycleWeeks")]
    pub baizu_cycle_weeks: u32,
    #[serde(rename = "freeConversionsPerWeek")]
    pub free_conversions_per_week: u32,
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
    #[serde(rename = "otherPagesConsumed")]
    pub other_pages_consumed: ShopMap,
    #[serde(rename = "purplePagesUsed")]
    pub purple_pages_used: u32,
    #[serde(rename = "bluePagesUsed")]
    pub blue_pages_used: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopAcquisition {
    pub shop: Shop,
    #[serde(rename = "targetSkillIndex")]
    pub target_skill_index: Option<usize>,
    pub pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    #[serde(rename = "skillLevels")]
    pub skill_levels: Vec<SkillLevel>,
    #[serde(rename = "skillPages")]
    pub skill_pages: Vec<u32>,
    #[serde(rename = "nonCombatPools")]
    pub non_combat_pools: ShopMap,
    #[serde(rename = "purplePages")]
    pub purple_pages: u32,
    #[serde(rename = "bluePages")]
    pub blue_pages: u32,
    #[serde(rename = "conversionStonesLeft")]
    pub conversion_stones_left: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekPlan {
    pub week: u32,
    pub acquisitions: Vec<ShopAcquisition>,
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
