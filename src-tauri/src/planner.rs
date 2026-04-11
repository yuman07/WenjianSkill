use crate::models::*;

const PAGES_PER_UNIT: u32 = 40;
const DEFAULT_FREE_CONVERSIONS: u32 = 3;
const MAX_WEEKS: u32 = 200;

/// 规划器状态
struct PlannerState {
    skill_levels: Vec<SkillLevel>,
    skill_pages: Vec<u32>,   // 每个战斗神通的剩余本体书页
    skill_shops: Vec<Shop>,  // 每个战斗神通的商店
    skill_targets: Vec<SkillLevel>,
    non_combat_pools: ShopMap,
    purple_pages: u32,
    blue_pages: u32,
    conversion_stones: u32,
    weekly_shop_income: ShopMap,
    baizu_cycle_weeks: u32,  // 百族每 N 周获取 1 本
    free_conversions_per_week: u32,
    weekly_purple_income: u32,
    weekly_blue_income: u32,
}

impl PlannerState {
    fn from_input(input: &PlannerInput) -> Self {
        PlannerState {
            skill_levels: input.combat_skills.iter().map(|s| s.current_level).collect(),
            skill_pages: input.combat_skills.iter().map(|s| s.remaining_pages).collect(),
            skill_shops: input.combat_skills.iter().map(|s| s.shop).collect(),
            skill_targets: input.combat_skills.iter().map(|s| s.target_level).collect(),
            non_combat_pools: input.advanced.non_combat_pools.clone(),
            purple_pages: input.purple_pages,
            blue_pages: input.blue_pages,
            conversion_stones: input.advanced.conversion_stones,
            weekly_shop_income: input.advanced.weekly_shop_income.clone(),
            baizu_cycle_weeks: input.advanced.baizu_cycle_weeks,
            free_conversions_per_week: input.advanced.free_conversions_per_week,
            weekly_purple_income: input.advanced.weekly_purple_income,
            weekly_blue_income: input.advanced.weekly_blue_income,
        }
    }

    fn snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            skill_levels: self.skill_levels.clone(),
            skill_pages: self.skill_pages.clone(),
            non_combat_pools: self.non_combat_pools.clone(),
            purple_pages: self.purple_pages,
            blue_pages: self.blue_pages,
            conversion_stones_left: self.conversion_stones,
        }
    }

    fn all_targets_met(&self) -> bool {
        self.skill_levels
            .iter()
            .zip(self.skill_targets.iter())
            .all(|(cur, tgt)| *cur >= *tgt)
    }

    /// 计算某个技能从当前等级升一级需要的材料
    fn next_upgrade_cost(&self, idx: usize) -> Option<(SkillLevel, &'static UpgradeCost)> {
        let cur = self.skill_levels[idx];
        let next = cur.next()?;
        let cost = &UPGRADE_COSTS[next.index()];
        Some((next, cost))
    }

    /// 尝试为技能 idx 执行一次升级，返回升级动作（如果材料充足）
    fn try_upgrade(&mut self, idx: usize) -> Option<UpgradeAction> {
        let (next_level, cost) = self.next_upgrade_cost(idx)?;
        let from_level = self.skill_levels[idx];

        // 检查本体书页
        if self.skill_pages[idx] < cost.self_pages {
            return None;
        }

        // 检查紫色、蓝色
        if self.purple_pages < cost.purple_pages || self.blue_pages < cost.blue_pages {
            return None;
        }

        // 检查仙品书页（从所有池中凑）
        let total_pool = self.non_combat_pools.total();
        if total_pool < cost.other_pages {
            return None;
        }

        // 执行：扣除本体
        self.skill_pages[idx] -= cost.self_pages;

        // 扣除仙品：按珍贵度从低到高消耗
        let mut other_consumed = ShopMap::zero();
        let mut remaining_other = cost.other_pages;
        let mut shops_by_rarity: Vec<Shop> = Shop::ALL.to_vec();
        shops_by_rarity.sort_by_key(|s| s.rarity());

        for &shop in &shops_by_rarity {
            if remaining_other == 0 {
                break;
            }
            let available = self.non_combat_pools.get(shop);
            let take = available.min(remaining_other);
            if take > 0 {
                self.non_combat_pools.sub_clamped(shop, take);
                other_consumed.add(shop, take);
                remaining_other -= take;
            }
        }

        // 扣除紫色、蓝色
        self.purple_pages -= cost.purple_pages;
        self.blue_pages -= cost.blue_pages;

        // 升级
        self.skill_levels[idx] = next_level;

        Some(UpgradeAction {
            skill_index: idx,
            from_level,
            to_level: next_level,
            self_pages_used: cost.self_pages,
            other_pages_consumed: other_consumed,
            purple_pages_used: cost.purple_pages,
            blue_pages_used: cost.blue_pages,
        })
    }
}

/// 执行规划
pub fn run_planner(input: &PlannerInput) -> PlannerOutput {
    let mut state = PlannerState::from_input(input);
    let mut weeks: Vec<WeekPlan> = Vec::new();
    let n = state.skill_levels.len();

    // 先检查：不进入周循环就能升级的（初始材料已足够）
    // Phase 0: 尝试用初始资源进行升级
    {
        let acquisitions = Vec::new();
        let conversions = Vec::new();
        let mut upgrades = Vec::new();
        loop {
            let mut any_upgrade = false;
            for idx in prioritized_skill_order(&state, true) {
                while let Some(action) = state.try_upgrade(idx) {
                    upgrades.push(action);
                    any_upgrade = true;
                }
            }
            if !any_upgrade {
                break;
            }
        }
        if !upgrades.is_empty() {
            weeks.push(WeekPlan {
                week: 0,
                acquisitions,
                conversions,
                upgrades,
                snapshot: state.snapshot(),
            });
        }
    }

    // Phase 1: 按周规划，直到所有最低目标达成
    let mut week_num = 1u32;
    while !state.all_targets_met() && week_num <= MAX_WEEKS {
        let week_plan = plan_one_week(&mut state, week_num, true);
        let has_actions = !week_plan.acquisitions.is_empty()
            || !week_plan.conversions.is_empty()
            || !week_plan.upgrades.is_empty();

        weeks.push(week_plan);

        // 如果一周内没有任何升级动作且目标未达成，检查是否还有进展可能
        if !has_actions && !state.all_targets_met() {
            // 检查是否有每周收入能最终解决问题
            let has_income = Shop::ALL.iter().any(|s| state.weekly_shop_income.get(*s) > 0)
                || state.baizu_cycle_weeks > 0
                || state.weekly_purple_income > 0
                || state.weekly_blue_income > 0;
            if !has_income {
                break; // 无收入且无法升级，终止
            }
        }

        week_num += 1;
    }

    // Phase 2: 最低目标达成后，一次性分配剩余资源继续提升
    if state.all_targets_met() {
        let mut final_upgrades = Vec::new();
        loop {
            let mut any_upgrade = false;
            for idx in prioritized_skill_order(&state, false) {
                while let Some(action) = state.try_upgrade(idx) {
                    final_upgrades.push(action);
                    any_upgrade = true;
                }
            }
            if !any_upgrade {
                break;
            }
        }
        if !final_upgrades.is_empty() {
            weeks.push(WeekPlan {
                week: week_num,
                acquisitions: Vec::new(),
                conversions: Vec::new(),
                upgrades: final_upgrades,
                snapshot: state.snapshot(),
            });
        }
    }

    // 生成不可达原因
    let mut unreachable_reasons = Vec::new();
    let feasible = state.all_targets_met();
    if !feasible {
        for i in 0..n {
            if state.skill_levels[i] < state.skill_targets[i] {
                let label = &input.combat_skills[i].label;
                let cur = state.skill_levels[i].display_name();
                let tgt = state.skill_targets[i].display_name();

                // 找到下一次升级卡在哪里
                let next_level = state.skill_levels[i].next();
                let bottleneck = if let Some(nl) = next_level {
                    let cost = &UPGRADE_COSTS[nl.index()];
                    let mut issues = Vec::new();
                    if state.skill_pages[i] < cost.self_pages {
                        issues.push(format!(
                            "本体书页不足（需要{}，当前{}）",
                            cost.self_pages, state.skill_pages[i]
                        ));
                    }
                    if cost.other_pages > 0 && state.non_combat_pools.total() < cost.other_pages {
                        issues.push(format!(
                            "狗粮不足（需要{}，当前{}）",
                            cost.other_pages, state.non_combat_pools.total()
                        ));
                    }
                    if cost.purple_pages > 0 && state.purple_pages < cost.purple_pages {
                        issues.push(format!(
                            "紫色书页不足（需要{}，当前{}）",
                            cost.purple_pages, state.purple_pages
                        ));
                    }
                    if cost.blue_pages > 0 && state.blue_pages < cost.blue_pages {
                        issues.push(format!(
                            "蓝色书页不足（需要{}，当前{}）",
                            cost.blue_pages, state.blue_pages
                        ));
                    }
                    if issues.is_empty() {
                        "资源不足".to_string()
                    } else {
                        issues.join("、")
                    }
                } else {
                    "已达最高等级".to_string()
                };

                unreachable_reasons.push(format!(
                    "{}: 当前 {}，目标 {}，卡在 {} → {} 升级 — {}",
                    label,
                    cur,
                    tgt,
                    cur,
                    next_level.map_or("?".to_string(), |l| l.display_name().to_string()),
                    bottleneck
                ));
            }
        }
    }

    PlannerOutput {
        feasible,
        weeks: if feasible { weeks } else { Vec::new() },
        unreachable_reasons,
        final_levels: state.skill_levels.clone(),
    }
}

/// 规划单周
fn plan_one_week(state: &mut PlannerState, week: u32, phase1: bool) -> WeekPlan {
    let mut acquisitions = Vec::new();
    let mut conversions = Vec::new();
    let mut upgrades = Vec::new();

    // Step 1: 接收每周收入
    state.purple_pages += state.weekly_purple_income;
    state.blue_pages += state.weekly_blue_income;

    for &shop in &Shop::ALL {
        // 百族使用周期制：每 N 周获取 1 本
        let income_count = if shop == Shop::BaiZu {
            if state.baizu_cycle_weeks > 0 && week % state.baizu_cycle_weeks == 0 {
                1
            } else {
                0
            }
        } else {
            state.weekly_shop_income.get(shop)
        };

        if income_count == 0 {
            continue;
        }
        let pages = income_count * PAGES_PER_UNIT;

        // 决定收入给谁：优先给该商店中本体书页最缺的战斗神通
        let best_target = find_best_income_target(state, shop, phase1);

        match best_target {
            Some(idx) => {
                state.skill_pages[idx] += pages;
                acquisitions.push(ShopAcquisition {
                    shop,
                    target_skill_index: Some(idx),
                    pages,
                });
            }
            None => {
                // 没有该商店的战斗神通需要本体，进入狗粮池
                state.non_combat_pools.add(shop, pages);
                acquisitions.push(ShopAcquisition {
                    shop,
                    target_skill_index: None,
                    pages,
                });
            }
        }
    }

    // Step 2: 执行转换（非战斗池 → 战斗神通本体）
    let mut free_conversions_left = state.free_conversions_per_week;

    // 收集需要本体书页的战斗神通，按紧急度排序
    let mut conversion_needs: Vec<(usize, u32)> = Vec::new();
    for idx in 0..state.skill_levels.len() {
        let target = if phase1 {
            state.skill_targets[idx]
        } else {
            SkillLevel::Tian5
        };
        if state.skill_levels[idx] >= target {
            continue;
        }
        let needed = total_cost_between(state.skill_levels[idx], target);
        if needed.self_pages > state.skill_pages[idx] {
            let deficit = needed.self_pages - state.skill_pages[idx];
            conversion_needs.push((idx, deficit));
        }
    }
    // 按本体缺口从大到小排序
    conversion_needs.sort_by(|a, b| b.1.cmp(&a.1));

    for (idx, _deficit) in &conversion_needs {
        let shop = state.skill_shops[*idx];
        let pool_available = state.non_combat_pools.get(shop);

        // 计算还需要多少本体
        let target = if phase1 {
            state.skill_targets[*idx]
        } else {
            SkillLevel::Tian5
        };
        let needed = total_cost_between(state.skill_levels[*idx], target);
        let self_deficit = needed.self_pages.saturating_sub(state.skill_pages[*idx]);
        let conversions_needed = (self_deficit + PAGES_PER_UNIT - 1) / PAGES_PER_UNIT;
        let conversions_possible = pool_available / PAGES_PER_UNIT;
        let conversions_to_do = conversions_needed.min(conversions_possible);

        for _ in 0..conversions_to_do {
            if free_conversions_left > 0 {
                free_conversions_left -= 1;
                state.non_combat_pools.sub_clamped(shop, PAGES_PER_UNIT);
                state.skill_pages[*idx] += PAGES_PER_UNIT;
                conversions.push(ConversionAction {
                    shop,
                    target_skill_index: *idx,
                    used_stone: false,
                    pages: PAGES_PER_UNIT,
                });
            } else if state.conversion_stones > 0 {
                state.conversion_stones -= 1;
                state.non_combat_pools.sub_clamped(shop, PAGES_PER_UNIT);
                state.skill_pages[*idx] += PAGES_PER_UNIT;
                conversions.push(ConversionAction {
                    shop,
                    target_skill_index: *idx,
                    used_stone: true,
                    pages: PAGES_PER_UNIT,
                });
            } else {
                break; // 没有转换次数了
            }
        }
    }

    // Step 3: 尝试升级
    loop {
        let mut any_upgrade = false;
        for idx in prioritized_skill_order(state, phase1) {
            if let Some(action) = state.try_upgrade(idx) {
                upgrades.push(action);
                any_upgrade = true;
                break; // 每次升级后重新排优先级
            }
        }
        if !any_upgrade {
            break;
        }
    }

    WeekPlan {
        week,
        acquisitions,
        conversions,
        upgrades,
        snapshot: state.snapshot(),
    }
}

/// 确定优先升级顺序：
/// Phase1: 优先升离目标最近的（容易达标的先完成）
/// Phase2: 优先升等级最低的（拉齐水平）
fn prioritized_skill_order(state: &PlannerState, phase1: bool) -> Vec<usize> {
    let n = state.skill_levels.len();
    let mut indices: Vec<usize> = (0..n).collect();

    if phase1 {
        // 优先升距离目标差距最小的（更容易达标）
        indices.sort_by_key(|&i| {
            let gap = state.skill_targets[i].index() as i32 - state.skill_levels[i].index() as i32;
            if gap <= 0 { i32::MAX } else { gap } // 已达标的排最后
        });
    } else {
        // 优先升等级最低的
        indices.sort_by_key(|&i| state.skill_levels[i].index());
    }

    indices
}

/// 找到该商店中最需要本体书页的战斗神通
fn find_best_income_target(state: &PlannerState, shop: Shop, phase1: bool) -> Option<usize> {
    let mut best: Option<(usize, u32)> = None;

    for (i, &s) in state.skill_shops.iter().enumerate() {
        if s != shop {
            continue;
        }
        let target = if phase1 {
            state.skill_targets[i]
        } else {
            SkillLevel::Tian5
        };
        if state.skill_levels[i] >= target {
            continue;
        }
        let needed = total_cost_between(state.skill_levels[i], target);
        let deficit = needed.self_pages.saturating_sub(state.skill_pages[i]);
        if deficit > 0 {
            match best {
                None => best = Some((i, deficit)),
                Some((_, bd)) if deficit > bd => best = Some((i, deficit)),
                _ => {}
            }
        }
    }

    best.map(|(i, _)| i)
}
