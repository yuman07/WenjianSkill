use crate::models::*;

const PAGES_PER_UNIT: u32 = 40;
const MAX_WEEKS: u32 = 500;

/// Calculate total income batches for a shop over `weeks` weeks
/// 道蕴 and 百族 use cycle-based income, others use weekly count
fn shop_income_over_weeks(shop: Shop, adv: &AdvancedSettings, weeks: u32) -> u32 {
    if weeks == 0 { return 0; }
    match shop {
        Shop::DaoYun => {
            if adv.daoyun_cycle_weeks > 0 { weeks / adv.daoyun_cycle_weeks } else { 0 }
        }
        Shop::BaiZu => {
            if adv.baizu_cycle_weeks > 0 { weeks / adv.baizu_cycle_weeks } else { 0 }
        }
        _ => adv.weekly_shop_income.get(shop) * weeks,
    }
}

// ============================================================
// Phase 1: Find minimum weeks via binary search
// The key insight: for a given W, the optimal allocation is
// DETERMINISTIC — no search needed.
//
// Proof: skill 本体 can only come from its own shop (income or
// conversion). Directing income to skill is strictly better than
// pool→conversion (same 本体 gained, no conversion slot used,
// same 仙品 remaining). Therefore "income to skills first" is
// always optimal. The remaining allocation follows automatically.
// ============================================================

/// Per-shop aggregated resource info for a given W
struct ShopAllocation {
    /// Total income batches (units of 40) available for this shop over W weeks
    total_income: u32,
    /// Sum of 本体 deficit (in units of 40) for all skills in this shop
    total_skill_deficit: u32,
    /// Income batches used for skills (min of above two)
    income_to_skills: u32,
    /// Conversions needed from pool
    conversions_needed: u32,
    /// Total pool available (init + excess income)
    pool_available: u32,
    /// Pool remaining as 仙品 after conversions
    fodder_available: u32,
}

/// Check if all targets can be met in exactly `weeks` weeks.
/// Returns None if infeasible, Some(allocation) if feasible.
fn check_feasibility(input: &PlannerInput, weeks: u32) -> Option<Vec<ShopAllocation>> {
    let adv = &input.advanced;
    let n = input.combat_skills.len();

    // Compute total needs per skill
    let mut skill_self_deficit = vec![0u32; n]; // in pages
    let mut total_other: u32 = 0;
    let mut total_purple: u32 = 0;
    let mut total_blue: u32 = 0;

    for i in 0..n {
        let s = &input.combat_skills[i];
        if s.current_level >= s.target_level {
            continue;
        }
        let cost = total_cost_between(s.current_level, s.target_level);
        skill_self_deficit[i] = cost.self_pages.saturating_sub(s.remaining_pages);
        total_other += cost.other_pages;
        total_purple += cost.purple_pages;
        total_blue += cost.blue_pages;
    }

    // Check purple/blue (these have no allocation choices)
    let avail_purple = input.purple_pages + adv.weekly_purple_income * weeks;
    let avail_blue = input.blue_pages + adv.weekly_blue_income * weeks;
    if avail_purple < total_purple || avail_blue < total_blue {
        return None;
    }

    // Per-shop allocation
    let mut allocations = Vec::new();
    let mut total_conversions_needed: u32 = 0;

    for &shop in &Shop::ALL {
        let total_income = shop_income_over_weeks(shop, adv, weeks);

        // Sum deficits for skills in this shop (in units of 40)
        let total_skill_deficit: u32 = (0..n)
            .filter(|&i| input.combat_skills[i].shop == shop)
            .map(|i| (skill_self_deficit[i] + PAGES_PER_UNIT - 1) / PAGES_PER_UNIT) // ceil div
            .sum();

        let income_to_skills = total_income.min(total_skill_deficit);
        let conversions_needed = total_skill_deficit - income_to_skills;
        let pool_available_units =
            adv.non_combat_pools.get(shop) / PAGES_PER_UNIT + (total_income - income_to_skills);

        if conversions_needed > pool_available_units {
            return None; // Not enough pool for conversions
        }

        let fodder_available = (pool_available_units - conversions_needed) * PAGES_PER_UNIT;
        total_conversions_needed += conversions_needed;

        allocations.push(ShopAllocation {
            total_income,
            total_skill_deficit,
            income_to_skills,
            conversions_needed,
            pool_available: pool_available_units * PAGES_PER_UNIT,
            fodder_available,
        });
    }

    // Check conversion capacity
    let total_conv_capacity = adv.free_conversions_per_week * weeks + adv.conversion_stones;
    if total_conversions_needed > total_conv_capacity {
        return None;
    }

    // Check 仙品
    let total_fodder: u32 = allocations.iter().map(|a| a.fodder_available).sum();
    if total_fodder < total_other {
        return None;
    }

    Some(allocations)
}

/// Binary search for minimum weeks
fn find_minimum_weeks(input: &PlannerInput) -> Option<u32> {
    // Check W=0 (can we do it with initial resources only?)
    if check_feasibility(input, 0).is_some() {
        return Some(0);
    }

    // Check if feasible at all (at MAX_WEEKS)
    if check_feasibility(input, MAX_WEEKS).is_none() {
        return None;
    }

    // Binary search
    let mut lo: u32 = 0;
    let mut hi: u32 = MAX_WEEKS;
    while lo < hi {
        let mid = (lo + hi) / 2;
        if check_feasibility(input, mid).is_some() {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    Some(lo)
}

// ============================================================
// Phase 2: Maximize total level with remaining resources
// Uses bounded enumeration (6 skills × ~14 levels each)
// ============================================================

/// Find the best additional upgrades beyond minimum targets
fn find_bonus_levels(
    input: &PlannerInput,
    min_weeks: u32,
) -> Vec<SkillLevel> {
    let n = input.combat_skills.len();
    let targets: Vec<SkillLevel> = input.combat_skills.iter().map(|s| s.target_level).collect();

    // For each skill, compute maximum possible level it could reach
    // (limited by 本体 availability from its shop)
    let mut max_possible = targets.clone();

    // We try to extend each skill's target and check if the combined
    // targets are still feasible at min_weeks
    // Use iterative improvement: try raising each skill one level at a time
    let mut improved = true;
    while improved {
        improved = false;
        for i in 0..n {
            let next = max_possible[i].next();
            if next.is_none() {
                continue;
            }
            let next_level = next.unwrap();
            let mut test_input = input.clone();
            for j in 0..n {
                test_input.combat_skills[j].target_level = max_possible[j];
            }
            test_input.combat_skills[i].target_level = next_level;

            if check_feasibility(&test_input, min_weeks).is_some() {
                max_possible[i] = next_level;
                improved = true;
            }
        }
    }

    max_possible
}

// ============================================================
// Phase 3: Generate the weekly plan
// Simulate week by week with provably optimal allocation
// ============================================================

struct SimState {
    skill_levels: Vec<SkillLevel>,
    skill_pages: Vec<u32>,
    skill_shops: Vec<Shop>,
    final_targets: Vec<SkillLevel>,
    non_combat_pools: ShopMap,
    purple_pages: u32,
    blue_pages: u32,
    conversion_stones: u32,
    free_conv_per_week: u32,
    weekly_shop_income: ShopMap,
    daoyun_cycle_weeks: u32,
    baizu_cycle_weeks: u32,
    weekly_purple: u32,
    weekly_blue: u32,
}

impl SimState {
    fn from_input(input: &PlannerInput, final_targets: &[SkillLevel]) -> Self {
        SimState {
            skill_levels: input.combat_skills.iter().map(|s| s.current_level).collect(),
            skill_pages: input.combat_skills.iter().map(|s| s.remaining_pages).collect(),
            skill_shops: input.combat_skills.iter().map(|s| s.shop).collect(),
            final_targets: final_targets.to_vec(),
            non_combat_pools: input.advanced.non_combat_pools.clone(),
            purple_pages: input.purple_pages,
            blue_pages: input.blue_pages,
            conversion_stones: input.advanced.conversion_stones,
            free_conv_per_week: input.advanced.free_conversions_per_week,
            weekly_shop_income: input.advanced.weekly_shop_income.clone(),
            daoyun_cycle_weeks: input.advanced.daoyun_cycle_weeks,
            baizu_cycle_weeks: input.advanced.baizu_cycle_weeks,
            weekly_purple: input.advanced.weekly_purple_income,
            weekly_blue: input.advanced.weekly_blue_income,
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

    fn all_done(&self) -> bool {
        self.skill_levels.iter().zip(self.final_targets.iter()).all(|(c, t)| *c >= *t)
    }

    /// Self deficit in pages for skill i
    fn self_deficit(&self, i: usize) -> u32 {
        if self.skill_levels[i] >= self.final_targets[i] {
            return 0;
        }
        let cost = total_cost_between(self.skill_levels[i], self.final_targets[i]);
        cost.self_pages.saturating_sub(self.skill_pages[i])
    }

    /// Try one upgrade for skill i
    fn try_upgrade(&mut self, i: usize) -> Option<UpgradeAction> {
        if self.skill_levels[i] >= self.final_targets[i] {
            return None;
        }
        let next = self.skill_levels[i].next()?;
        let cost = &UPGRADE_COSTS[next.index()];
        let from = self.skill_levels[i];

        if self.skill_pages[i] < cost.self_pages {
            return None;
        }
        if self.purple_pages < cost.purple_pages || self.blue_pages < cost.blue_pages {
            return None;
        }
        if self.non_combat_pools.total() < cost.other_pages {
            return None;
        }

        // Deduct self pages
        self.skill_pages[i] -= cost.self_pages;

        // Deduct 仙品 by rarity (low first)
        let mut consumed = ShopMap::zero();
        let mut remaining = cost.other_pages;
        let mut shops: Vec<Shop> = Shop::ALL.to_vec();
        shops.sort_by_key(|s| s.rarity());
        for &shop in &shops {
            if remaining == 0 { break; }
            let take = self.non_combat_pools.get(shop).min(remaining);
            if take > 0 {
                self.non_combat_pools.sub_clamped(shop, take);
                consumed.add(shop, take);
                remaining -= take;
            }
        }

        self.purple_pages -= cost.purple_pages;
        self.blue_pages -= cost.blue_pages;
        self.skill_levels[i] = next;

        Some(UpgradeAction {
            skill_index: i,
            from_level: from,
            to_level: next,
            self_pages_used: cost.self_pages,
            other_pages_consumed: consumed,
            purple_pages_used: cost.purple_pages,
            blue_pages_used: cost.blue_pages,
        })
    }
}

fn simulate_week(state: &mut SimState, week: u32) -> WeekPlan {
    let n = state.skill_levels.len();
    let mut acquisitions = Vec::new();
    let mut conversions = Vec::new();
    let mut upgrades = Vec::new();

    // Step 1: Weekly income
    state.purple_pages += state.weekly_purple;
    state.blue_pages += state.weekly_blue;

    for &shop in &Shop::ALL {
        let batches = match shop {
            Shop::DaoYun => {
                if state.daoyun_cycle_weeks > 0 && week % state.daoyun_cycle_weeks == 0 { 1 } else { 0 }
            }
            Shop::BaiZu => {
                if state.baizu_cycle_weeks > 0 && week % state.baizu_cycle_weeks == 0 { 1 } else { 0 }
            }
            _ => state.weekly_shop_income.get(shop),
        };
        if batches == 0 { continue; }
        let pages = batches * PAGES_PER_UNIT;

        // Find skill in this shop with most 本体 deficit
        let best = (0..n)
            .filter(|&i| state.skill_shops[i] == shop && state.self_deficit(i) > 0)
            .max_by_key(|&i| state.self_deficit(i));

        match best {
            Some(i) => {
                state.skill_pages[i] += pages;
                acquisitions.push(ShopAcquisition { shop, target_skill_index: Some(i), pages });
            }
            None => {
                state.non_combat_pools.add(shop, pages);
                acquisitions.push(ShopAcquisition { shop, target_skill_index: None, pages });
            }
        }
    }

    // Step 2 & 3: Interleave conversions and upgrades
    // Strategy: convert only what's needed for the NEXT upgrade of each skill,
    // then upgrade, then repeat. This preserves pool as 仙品 for current upgrades.
    let mut free_left = state.free_conv_per_week;

    loop {
        // First, try upgrading without any new conversions
        let mut upgraded_any = false;
        loop {
            let mut order: Vec<usize> = (0..n)
                .filter(|&i| state.skill_levels[i] < state.final_targets[i])
                .collect();
            order.sort_by_key(|&i| {
                state.final_targets[i].index() as i32 - state.skill_levels[i].index() as i32
            });
            let mut any = false;
            for i in order {
                if let Some(action) = state.try_upgrade(i) {
                    upgrades.push(action);
                    any = true;
                    upgraded_any = true;
                    break;
                }
            }
            if !any { break; }
        }

        // Then, try converting ONE batch for the most urgent skill that needs 本体
        // to unlock its next upgrade. Only convert if it enables a new upgrade.
        if free_left == 0 && state.conversion_stones == 0 {
            break;
        }

        let mut converted_any = false;
        // Sort skills: closest to next upgrade first
        let mut candidates: Vec<usize> = (0..n)
            .filter(|&i| state.skill_levels[i] < state.final_targets[i])
            .collect();
        candidates.sort_by_key(|&i| {
            state.final_targets[i].index() as i32 - state.skill_levels[i].index() as i32
        });

        for &i in &candidates {
            let next = state.skill_levels[i].next();
            if next.is_none() { continue; }
            let next_cost = &UPGRADE_COSTS[next.unwrap().index()];

            // Only convert if this skill needs more 本体 for its next upgrade
            if state.skill_pages[i] >= next_cost.self_pages {
                continue;
            }

            let shop = state.skill_shops[i];
            let pool_units = state.non_combat_pools.get(shop) / PAGES_PER_UNIT;
            if pool_units == 0 { continue; }

            // Convert one batch
            if free_left > 0 {
                free_left -= 1;
                state.non_combat_pools.sub_clamped(shop, PAGES_PER_UNIT);
                state.skill_pages[i] += PAGES_PER_UNIT;
                conversions.push(ConversionAction {
                    shop, target_skill_index: i, used_stone: false, pages: PAGES_PER_UNIT,
                });
                converted_any = true;
                break;
            } else if state.conversion_stones > 0 {
                state.conversion_stones -= 1;
                state.non_combat_pools.sub_clamped(shop, PAGES_PER_UNIT);
                state.skill_pages[i] += PAGES_PER_UNIT;
                conversions.push(ConversionAction {
                    shop, target_skill_index: i, used_stone: true, pages: PAGES_PER_UNIT,
                });
                converted_any = true;
                break;
            }
        }

        if !converted_any && !upgraded_any {
            break;
        }
        if !converted_any {
            break; // No more conversions possible/needed, done for this week
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

// ============================================================
// Public entry point
// ============================================================

pub fn run_planner(input: &PlannerInput) -> PlannerOutput {
    let n = input.combat_skills.len();

    // Phase 1: Find minimum weeks
    let min_weeks = match find_minimum_weeks(input) {
        Some(w) => w,
        None => {
            // Infeasible — generate reasons
            return PlannerOutput {
                feasible: false,
                weeks: Vec::new(),
                unreachable_reasons: generate_reasons(input),
                final_levels: input.combat_skills.iter().map(|s| s.current_level).collect(),
            };
        }
    };

    // Phase 2: Find bonus levels achievable with same min_weeks resources
    let final_targets = find_bonus_levels(input, min_weeks);

    // Phase 3: Simulate week by week
    let mut state = SimState::from_input(input, &final_targets);
    let mut weeks = Vec::new();

    // Week 0: upgrades with initial resources
    {
        let mut w0_upgrades = Vec::new();
        loop {
            let mut order: Vec<usize> = (0..n)
                .filter(|&i| state.skill_levels[i] < state.final_targets[i])
                .collect();
            order.sort_by_key(|&i| {
                state.final_targets[i].index() as i32 - state.skill_levels[i].index() as i32
            });
            let mut any = false;
            for i in order {
                if let Some(action) = state.try_upgrade(i) {
                    w0_upgrades.push(action);
                    any = true;
                    break;
                }
            }
            if !any { break; }
        }
        if !w0_upgrades.is_empty() {
            weeks.push(WeekPlan {
                week: 0,
                acquisitions: Vec::new(),
                conversions: Vec::new(),
                upgrades: w0_upgrades,
                snapshot: state.snapshot(),
            });
        }
    }

    // Weekly simulation
    for w in 1..=min_weeks {
        if state.all_done() { break; }
        let plan = simulate_week(&mut state, w);
        weeks.push(plan);
    }

    PlannerOutput {
        feasible: true,
        weeks,
        unreachable_reasons: Vec::new(),
        final_levels: state.skill_levels.clone(),
    }
}

/// Generate human-readable reasons for infeasibility
fn generate_reasons(input: &PlannerInput) -> Vec<String> {
    let mut reasons = Vec::new();
    let adv = &input.advanced;
    let n = input.combat_skills.len();

    // Check global bottlenecks first (purple/blue affect all skills)
    let mut total_purple_need: u32 = 0;
    let mut total_blue_need: u32 = 0;
    let mut total_other_need: u32 = 0;
    for s in &input.combat_skills {
        if s.current_level >= s.target_level { continue; }
        let cost = total_cost_between(s.current_level, s.target_level);
        total_purple_need += cost.purple_pages;
        total_blue_need += cost.blue_pages;
        total_other_need += cost.other_pages;
    }

    let max_purple = input.purple_pages + adv.weekly_purple_income * MAX_WEEKS;
    let max_blue = input.blue_pages + adv.weekly_blue_income * MAX_WEEKS;
    if max_purple < total_purple_need {
        reasons.push(format!(
            "紫色书页不足：所有神通合计需要 {}，当前 {}{}",
            total_purple_need,
            input.purple_pages,
            if adv.weekly_purple_income > 0 {
                format!("（每周+{}，但 {} 周也只有 {}）", adv.weekly_purple_income, MAX_WEEKS, max_purple)
            } else {
                "，且无每周收入".to_string()
            }
        ));
    }
    if max_blue < total_blue_need {
        reasons.push(format!(
            "蓝色书页不足：所有神通合计需要 {}，当前 {}{}",
            total_blue_need,
            input.blue_pages,
            if adv.weekly_blue_income > 0 {
                format!("（每周+{}，但 {} 周也只有 {}）", adv.weekly_blue_income, MAX_WEEKS, max_blue)
            } else {
                "，且无每周收入".to_string()
            }
        ));
    }

    // Check per-skill 本体 bottleneck
    for i in 0..n {
        let s = &input.combat_skills[i];
        if s.current_level >= s.target_level { continue; }

        let cost = total_cost_between(s.current_level, s.target_level);
        let shop = s.shop;
        let max_income = shop_income_over_weeks(shop, adv, MAX_WEEKS);
        let max_self = s.remaining_pages + max_income * PAGES_PER_UNIT
            + adv.non_combat_pools.get(shop);
        if max_self < cost.self_pages {
            reasons.push(format!(
                "{}: 本体书页不足（需要 {}，该商店最多可获得 {}）",
                s.label, cost.self_pages, max_self
            ));
        }
    }

    // Check global 仙品 (cross-shop)
    let max_fodder: u32 = Shop::ALL.iter().map(|&shop| {
        let income = shop_income_over_weeks(shop, adv, MAX_WEEKS);
        let deficit: u32 = (0..n)
            .filter(|&i| input.combat_skills[i].shop == shop)
            .map(|i| {
                let c = total_cost_between(input.combat_skills[i].current_level, input.combat_skills[i].target_level);
                let d = c.self_pages.saturating_sub(input.combat_skills[i].remaining_pages);
                (d + PAGES_PER_UNIT - 1) / PAGES_PER_UNIT
            })
            .sum();
        let pool = adv.non_combat_pools.get(shop) / PAGES_PER_UNIT + income;
        pool.saturating_sub(deficit) * PAGES_PER_UNIT
    }).sum();

    if max_fodder < total_other_need {
        reasons.push(format!(
            "狗粮不足：所有神通合计需要仙品 {}，所有商店狗粮池最多可提供 {}（扣除本体转换后）",
            total_other_need, max_fodder
        ));
    }

    if reasons.is_empty() {
        // Conversion capacity bottleneck
        reasons.push("每周转换次数不足，无法在合理时间内完成所有转换".to_string());
    }

    reasons
}
