use crate::models::*;
use std::collections::HashMap;

const PAGES_PER_UNIT: u32 = 40;
const MAX_WEEKS: u32 = 500;

// ============================================================
// Income helper
// ============================================================

/// Pages a skill receives over `weeks` weeks from its own income schedule
fn skill_income_pages(skill: &CombatSkillInput, weeks: u32) -> u32 {
    if weeks == 0 || skill.income_cycle_weeks == 0 {
        return 0;
    }
    (weeks / skill.income_cycle_weeks) * skill.income_batch_count * PAGES_PER_UNIT
}

/// Does skill i receive income in week w?
fn skill_gets_income(skill: &CombatSkillInput, week: u32) -> u32 {
    if skill.income_cycle_weeks == 0 || week == 0 {
        return 0;
    }
    if week % skill.income_cycle_weeks == 0 {
        skill.income_batch_count * PAGES_PER_UNIT
    } else {
        0
    }
}

// ============================================================
// Phase 1: Feasibility check & binary search
//
// Model: each skill accumulates pages from its own income.
// 仙品 comes from OTHER skills' surplus (pages beyond own 本体 need).
// Same-shop skills can also convert surplus → 本体 for each other.
// Conservation law still holds per-shop.
// ============================================================

fn check_feasibility(input: &PlannerInput, weeks: u32) -> bool {
    let adv = &input.advanced;
    let n = input.combat_skills.len();

    // Per-skill totals
    let mut total_pages = vec![0u32; n];
    let mut self_needed = vec![0u32; n];
    let mut total_other: u32 = 0;
    let mut total_purple: u32 = 0;
    let mut total_blue: u32 = 0;

    for i in 0..n {
        let s = &input.combat_skills[i];
        total_pages[i] = s.remaining_pages + skill_income_pages(s, weeks);
        if s.current_level < s.target_level {
            let cost = total_cost_between(s.current_level, s.target_level, s.realm, s.skill_class);
            self_needed[i] = cost.self_pages;
            total_other += cost.other_pages;
            total_purple += cost.purple_pages;
            total_blue += cost.blue_pages;
        }
    }

    // Purple/blue check
    let avail_purple = input.purple_pages + adv.weekly_purple_income * weeks;
    let avail_blue = input.blue_pages + adv.weekly_blue_income * weeks;
    if avail_purple < total_purple || avail_blue < total_blue {
        return false;
    }

    // Per-shop: skill pages + fodder pool must cover total self needed
    // Fodder pool pages can be converted to same-shop skill 本体 OR used as 仙品
    let mut fodder_pool = vec![0u32; 5]; // per-shop fodder pool over W weeks
    for &shop in &Shop::ALL {
        fodder_pool[shop.index()] = adv.fodder_income.total_pages(shop, weeks);
    }

    for &shop in &Shop::ALL {
        let shop_pages: u32 = (0..n)
            .filter(|&i| input.combat_skills[i].shop == shop)
            .map(|i| total_pages[i]).sum();
        let shop_self: u32 = (0..n)
            .filter(|&i| input.combat_skills[i].shop == shop)
            .map(|i| self_needed[i]).sum();
        // Skill pages + fodder pool can cover 本体 via conversion
        if shop_pages + fodder_pool[shop.index()] < shop_self {
            return false;
        }
    }

    // Conversion capacity: deficits not covered by own skill pages need conversion
    // (from same-shop surplus skills or fodder pool)
    let total_conversions_needed: u32 = (0..n)
        .filter(|&i| total_pages[i] < self_needed[i])
        .map(|i| {
            let deficit = self_needed[i] - total_pages[i];
            (deficit + PAGES_PER_UNIT - 1) / PAGES_PER_UNIT
        })
        .sum();
    let conv_capacity = adv.free_conversions_per_week * weeks + adv.conversion_stones;
    if total_conversions_needed > conv_capacity {
        return false;
    }

    // 仙品 check: surplus from combat skills + remaining fodder pools
    let total_surplus: u32 = Shop::ALL.iter().map(|&shop| {
        let shop_pages: u32 = (0..n)
            .filter(|&i| input.combat_skills[i].shop == shop)
            .map(|i| total_pages[i]).sum();
        let shop_self: u32 = (0..n)
            .filter(|&i| input.combat_skills[i].shop == shop)
            .map(|i| self_needed[i]).sum();
        // Total available = skill pages + fodder pool - self needed
        (shop_pages + fodder_pool[shop.index()]).saturating_sub(shop_self)
    }).sum();

    if total_surplus < total_other {
        return false;
    }

    // Per-skill 仙品 check: skill i can't use its OWN surplus as its own 仙品.
    // Available 仙品 for skill i = total_surplus - skill_i's own surplus.
    for i in 0..n {
        if self_needed[i] == 0 { continue; } // no upgrades needed
        let s = &input.combat_skills[i];
        let cost = total_cost_between(
            s.current_level,
            s.target_level,
            s.realm,
            s.skill_class,
        );
        let own_surplus = total_pages[i].saturating_sub(self_needed[i]);
        let available_for_i = total_surplus.saturating_sub(own_surplus);
        if available_for_i < cost.other_pages {
            return false;
        }
    }

    true
}

fn find_minimum_weeks(input: &PlannerInput) -> Option<u32> {
    if check_feasibility(input, 0) { return Some(0); }
    if !check_feasibility(input, MAX_WEEKS) { return None; }
    let (mut lo, mut hi) = (0u32, MAX_WEEKS);
    while lo < hi {
        let mid = (lo + hi) / 2;
        if check_feasibility(input, mid) { hi = mid; } else { lo = mid + 1; }
    }
    Some(lo)
}

// ============================================================
// Phase 2: Bonus levels
// ============================================================

fn find_bonus_levels(input: &PlannerInput, min_weeks: u32) -> Vec<SkillLevel> {
    let n = input.combat_skills.len();
    let mut max_possible: Vec<SkillLevel> = input.combat_skills.iter().map(|s| s.target_level).collect();
    let mut improved = true;
    while improved {
        improved = false;
        for i in 0..n {
            if let Some(next_level) = max_possible[i].next() {
                let mut test = input.clone();
                for j in 0..n { test.combat_skills[j].target_level = max_possible[j]; }
                test.combat_skills[i].target_level = next_level;
                if check_feasibility(&test, min_weeks) {
                    max_possible[i] = next_level;
                    improved = true;
                }
            }
        }
    }
    max_possible
}

// ============================================================
// Phase 3: Weekly simulation
// ============================================================

struct SimState {
    levels: Vec<SkillLevel>,
    pages: Vec<u32>,
    shops: Vec<Shop>,
    realms: Vec<Realm>,
    skill_classes: Vec<SkillClass>,
    targets: Vec<SkillLevel>,
    fodder_pools: [u32; 5], // per-shop fodder pool
    purple: u32,
    blue: u32,
    stones: u32,
    free_conv: u32,
}

impl SimState {
    fn from_input(input: &PlannerInput, targets: &[SkillLevel]) -> Self {
        SimState {
            levels: input.combat_skills.iter().map(|s| s.current_level).collect(),
            pages: input.combat_skills.iter().map(|s| s.remaining_pages).collect(),
            shops: input.combat_skills.iter().map(|s| s.shop).collect(),
            realms: input.combat_skills.iter().map(|s| s.realm).collect(),
            skill_classes: input.combat_skills.iter().map(|s| s.skill_class).collect(),
            targets: targets.to_vec(),
            fodder_pools: {
                let mut p = [0u32; 5];
                for &shop in &Shop::ALL {
                    p[shop.index()] = input.advanced.fodder_income.initial_pages(shop);
                }
                p
            },
            purple: input.purple_pages,
            blue: input.blue_pages,
            stones: input.advanced.conversion_stones,
            free_conv: input.advanced.free_conversions_per_week,
        }
    }

    fn snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            skill_levels: self.levels.clone(),
            skill_pages: self.pages.clone(),
            purple_pages: self.purple,
            blue_pages: self.blue,
            conversion_stones_left: self.stones,
        }
    }

    fn all_done(&self) -> bool {
        self.levels.iter().zip(self.targets.iter()).all(|(c, t)| *c >= *t)
    }

    fn self_remaining_need(&self, i: usize) -> u32 {
        if self.levels[i] >= self.targets[i] { return 0; }
        let cost = total_cost_between(self.levels[i], self.targets[i], self.realms[i], self.skill_classes[i]);
        cost.self_pages.saturating_sub(self.pages[i])
    }

    /// Surplus pages available as 仙品 donor (pages beyond own remaining 本体 need)
    fn donatable(&self, i: usize) -> u32 {
        if self.levels[i] >= self.targets[i] {
            return self.pages[i]; // All pages are surplus
        }
        let cost = total_cost_between(self.levels[i], self.targets[i], self.realms[i], self.skill_classes[i]);
        self.pages[i].saturating_sub(cost.self_pages)
    }

    /// Try upgrade skill i, consuming 仙品 from other skills' surplus
    fn try_upgrade(&mut self, i: usize, n: usize) -> Option<UpgradeAction> {
        if self.levels[i] >= self.targets[i] { return None; }
        let next = self.levels[i].next()?;
        let costs = upgrade_costs_for_category(cost_category(self.realms[i], self.skill_classes[i]));
        let cost_idx = next.index() - 1; // cost table: 0 = 1星→2星
        if cost_idx >= costs.len() { return None; }
        let cost = &costs[cost_idx];
        let from = self.levels[i];

        if self.pages[i] < cost.self_pages { return None; }
        if self.purple < cost.purple_pages || self.blue < cost.blue_pages { return None; }

        // Check 仙品: other skills' surplus + fodder pools
        let total_donatable: u32 = (0..n).filter(|&j| j != i).map(|j| self.donatable(j)).sum();
        let total_fodder: u32 = self.fodder_pools.iter().sum();
        if total_donatable + total_fodder < cost.other_pages { return None; }

        // Execute
        self.pages[i] -= cost.self_pages;

        // Consume 仙品: fodder pools first (low rarity), then other skills' surplus
        let mut consumed: HashMap<String, u32> = HashMap::new();
        let mut remaining = cost.other_pages;
        let mut shop_order: Vec<Shop> = Shop::ALL.to_vec();
        shop_order.sort_by_key(|s| s.rarity());

        // From fodder pools first
        for &shop in &shop_order {
            if remaining == 0 { break; }
            let pool = &mut self.fodder_pools[shop.index()];
            let take = (*pool).min(remaining);
            if take > 0 {
                *pool -= take;
                consumed.insert(format!("pool_{}", shop.index()), take);
                remaining -= take;
            }
        }
        // Then from other combat skills' surplus
        let mut donors: Vec<usize> = (0..n).filter(|&j| j != i).collect();
        donors.sort_by_key(|&j| self.shops[j].rarity());
        for &j in &donors {
            if remaining == 0 { break; }
            let give = self.donatable(j).min(remaining);
            if give > 0 {
                self.pages[j] -= give;
                consumed.insert(j.to_string(), give);
                remaining -= give;
            }
        }

        self.purple -= cost.purple_pages;
        self.blue -= cost.blue_pages;
        self.levels[i] = next;

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

/// Interleave conversions and upgrades until no more progress.
/// `free_conv` is the number of free conversions available this cycle.
fn do_conversions_and_upgrades(
    state: &mut SimState,
    free_conv: u32,
    conversions: &mut Vec<ConversionAction>,
    upgrades: &mut Vec<UpgradeAction>,
) {
    let n = state.levels.len();
    let mut free_left = free_conv;
    loop {
        // Try upgrading all possible
        // Priority: (1) upgrades needing less 仙品 first (avoid competing for shared resource),
        //           (2) among equal 仙品 cost, smaller gap first (finish sooner → free surplus)
        let mut upgraded = false;
        loop {
            let mut order: Vec<usize> = (0..n).filter(|&i| state.levels[i] < state.targets[i]).collect();
            order.sort_by_key(|&i| {
                let next = state.levels[i].next().unwrap();
                let costs = upgrade_costs_for_category(cost_category(state.realms[i], state.skill_classes[i]));
                let other_cost = costs.get(next.index() - 1).map_or(0, |c| c.other_pages);
                let gap = state.targets[i].index() as i32 - state.levels[i].index() as i32;
                (other_cost, gap)
            });
            let mut any = false;
            for i in order {
                if let Some(action) = state.try_upgrade(i, n) {
                    upgrades.push(action);
                    any = true;
                    upgraded = true;
                    break;
                }
            }
            if !any { break; }
        }

        // Try one conversion for the most urgent skill needing 本体
        if free_left == 0 && state.stones == 0 { break; }
        let mut candidates: Vec<usize> = (0..n)
            .filter(|&i| state.levels[i] < state.targets[i] && state.self_remaining_need(i) > 0)
            .collect();
        candidates.sort_by_key(|&i| state.targets[i].index() as i32 - state.levels[i].index() as i32);

        let mut converted = false;
        for &i in &candidates {
            let shop = state.shops[i];

            // Source 1: same-shop fodder pool
            if state.fodder_pools[shop.index()] >= PAGES_PER_UNIT {
                if free_left > 0 {
                    free_left -= 1;
                    state.fodder_pools[shop.index()] -= PAGES_PER_UNIT;
                    state.pages[i] += PAGES_PER_UNIT;
                    conversions.push(ConversionAction {
                        shop, target_skill_index: i, from_skill_index: usize::MAX, // pool
                        used_stone: false, pages: PAGES_PER_UNIT,
                    });
                    converted = true;
                    break;
                } else if state.stones > 0 {
                    state.stones -= 1;
                    state.fodder_pools[shop.index()] -= PAGES_PER_UNIT;
                    state.pages[i] += PAGES_PER_UNIT;
                    conversions.push(ConversionAction {
                        shop, target_skill_index: i, from_skill_index: usize::MAX,
                        used_stone: true, pages: PAGES_PER_UNIT,
                    });
                    converted = true;
                    break;
                }
            }

            // Source 2: same-shop combat skill surplus
            let mut donors: Vec<usize> = (0..n)
                .filter(|&j| j != i && state.shops[j] == shop && state.donatable(j) >= PAGES_PER_UNIT)
                .collect();
            donors.sort_by(|a, b| state.donatable(*b).cmp(&state.donatable(*a)));
            if let Some(&donor) = donors.first() {
                if free_left > 0 {
                    free_left -= 1;
                    state.pages[donor] -= PAGES_PER_UNIT;
                    state.pages[i] += PAGES_PER_UNIT;
                    conversions.push(ConversionAction {
                        shop, target_skill_index: i, from_skill_index: donor,
                        used_stone: false, pages: PAGES_PER_UNIT,
                    });
                    converted = true;
                    break;
                } else if state.stones > 0 {
                    state.stones -= 1;
                    state.pages[donor] -= PAGES_PER_UNIT;
                    state.pages[i] += PAGES_PER_UNIT;
                    conversions.push(ConversionAction {
                        shop, target_skill_index: i, from_skill_index: donor,
                        used_stone: true, pages: PAGES_PER_UNIT,
                    });
                    converted = true;
                    break;
                }
            }
        }
        if !converted && !upgraded { break; }
        if !converted { break; }
    }
}

fn simulate_week(state: &mut SimState, input: &PlannerInput, week: u32) -> WeekPlan {
    let n = state.levels.len();
    let mut incomes = Vec::new();
    let mut conversions = Vec::new();
    let mut upgrades = Vec::new();

    // Step 1: Income (combat skills + fodder pools + purple/blue)
    state.purple += input.advanced.weekly_purple_income;
    state.blue += input.advanced.weekly_blue_income;
    for i in 0..n {
        let pages = skill_gets_income(&input.combat_skills[i], week);
        if pages > 0 {
            state.pages[i] += pages;
            incomes.push(SkillIncome { skill_index: i, pages });
        }
    }
    for &shop in &Shop::ALL {
        let pages = input.advanced.fodder_income.pages_in_week(shop, week);
        if pages > 0 {
            state.fodder_pools[shop.index()] += pages;
        }
    }

    // Step 2 & 3: Interleave conversions and upgrades
    let free_conv = state.free_conv;
    do_conversions_and_upgrades(state, free_conv, &mut conversions, &mut upgrades);

    WeekPlan {
        week,
        incomes,
        conversions,
        upgrades,
        snapshot: state.snapshot(),
    }
}

// ============================================================
// Entry point
// ============================================================

pub fn run_planner(input: &PlannerInput) -> PlannerOutput {
    let n = input.combat_skills.len();

    let min_weeks = match find_minimum_weeks(input) {
        Some(w) => w,
        None => {
            return PlannerOutput {
                feasible: false,
                weeks: Vec::new(),
                unreachable_reasons: generate_reasons(input),
                final_levels: input.combat_skills.iter().map(|s| s.current_level).collect(),
            };
        }
    };

    let final_targets = find_bonus_levels(input, min_weeks);
    let mut state = SimState::from_input(input, &final_targets);
    let mut weeks = Vec::new();

    // Week 0: conversions + upgrades with initial resources
    {
        let mut conversions = Vec::new();
        let mut upgrades = Vec::new();
        let free_conv = state.free_conv;
        do_conversions_and_upgrades(&mut state, free_conv, &mut conversions, &mut upgrades);
        if !conversions.is_empty() || !upgrades.is_empty() {
            weeks.push(WeekPlan {
                week: 0, incomes: Vec::new(), conversions, upgrades,
                snapshot: state.snapshot(),
            });
        }
    }

    for w in 1..=min_weeks {
        if state.all_done() { break; }
        weeks.push(simulate_week(&mut state, input, w));
    }

    PlannerOutput {
        feasible: true,
        weeks,
        unreachable_reasons: Vec::new(),
        final_levels: state.levels.clone(),
    }
}

fn generate_reasons(input: &PlannerInput) -> Vec<String> {
    let mut reasons = Vec::new();
    let adv = &input.advanced;
    let n = input.combat_skills.len();

    let mut total_purple_need: u32 = 0;
    let mut total_blue_need: u32 = 0;
    let mut total_other_need: u32 = 0;
    for s in &input.combat_skills {
        if s.current_level >= s.target_level { continue; }
        let cost = total_cost_between(s.current_level, s.target_level, s.realm, s.skill_class);
        total_purple_need += cost.purple_pages;
        total_blue_need += cost.blue_pages;
        total_other_need += cost.other_pages;
    }

    let max_purple = input.purple_pages + adv.weekly_purple_income * MAX_WEEKS;
    let max_blue = input.blue_pages + adv.weekly_blue_income * MAX_WEEKS;
    if max_purple < total_purple_need {
        reasons.push(format!(
            "紫色书页不足：合计需要 {}，当前 {}{}",
            total_purple_need, input.purple_pages,
            if adv.weekly_purple_income > 0 {
                format!("（每周+{}，仍不够）", adv.weekly_purple_income)
            } else { "，且无每周收入".to_string() }
        ));
    }
    if max_blue < total_blue_need {
        reasons.push(format!(
            "蓝色书页不足：合计需要 {}，当前 {}{}",
            total_blue_need, input.blue_pages,
            if adv.weekly_blue_income > 0 {
                format!("（每周+{}，仍不够）", adv.weekly_blue_income)
            } else { "，且无每周收入".to_string() }
        ));
    }

    // Per-shop 本体 check (combat skill pages + fodder pool must cover self needs)
    for &shop in &Shop::ALL {
        let skills_in_shop: Vec<usize> = (0..n).filter(|&i| input.combat_skills[i].shop == shop).collect();
        if skills_in_shop.is_empty() { continue; }
        let shop_pages: u32 = skills_in_shop.iter().map(|&i| {
            input.combat_skills[i].remaining_pages + skill_income_pages(&input.combat_skills[i], MAX_WEEKS)
        }).sum();
        let fodder = adv.fodder_income.total_pages(shop, MAX_WEEKS);
        let shop_self: u32 = skills_in_shop.iter().map(|&i| {
            let s = &input.combat_skills[i];
            if s.current_level >= s.target_level { 0 }
            else { total_cost_between(s.current_level, s.target_level, s.realm, s.skill_class).self_pages }
        }).sum();
        if shop_pages + fodder < shop_self {
            reasons.push(format!(
                "「{}」商店本体书页不足：该商店技能合计需要 {}，技能收入最多 {} + 狗粮池最多 {}",
                match shop {
                    Shop::LunJian => "论剑", Shop::ZhuTian => "诸天", Shop::ZongMen => "宗门",
                    Shop::DaoYun => "道蕴", Shop::BaiZu => "百族",
                },
                shop_self, shop_pages, fodder
            ));
        }
    }

    // Global 仙品 check (combat skill surplus + fodder pool income)
    let total_surplus: u32 = Shop::ALL.iter().map(|&shop| {
        let sp: u32 = (0..n).filter(|&i| input.combat_skills[i].shop == shop)
            .map(|i| input.combat_skills[i].remaining_pages + skill_income_pages(&input.combat_skills[i], MAX_WEEKS)).sum();
        let ss: u32 = (0..n).filter(|&i| input.combat_skills[i].shop == shop)
            .map(|i| {
                let s = &input.combat_skills[i];
                if s.current_level >= s.target_level { 0 }
                else { total_cost_between(s.current_level, s.target_level, s.realm, s.skill_class).self_pages }
            }).sum();
        let fodder = adv.fodder_income.total_pages(shop, MAX_WEEKS);
        (sp + fodder).saturating_sub(ss)
    }).sum();
    if total_surplus < total_other_need {
        reasons.push(format!(
            "仙品（狗粮）不足：合计需要 {}，所有来源最多可提供 {}",
            total_other_need, total_surplus
        ));
    }

    if reasons.is_empty() {
        reasons.push("转换次数不足，无法在合理时间内完成本体书页的跨技能调配".to_string());
    }
    reasons
}
