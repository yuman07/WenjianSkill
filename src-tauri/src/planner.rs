use crate::models::*;
use std::collections::HashMap;

const PAGES_PER_UNIT: u32 = 40;
const MAX_WEEKS: u32 = 500;
/// First N free conversions per week are used before conversion stones;
/// any remaining free conversions beyond this threshold are used after stones.
const PRIORITY_FREE_CONV: u32 = 3;

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
// 金色 comes from OTHER skills' surplus (pages beyond own 本体 need).
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
            total_other += cost.gold_pages;
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
    // Fodder pool pages can be converted to same-shop skill 本体 OR used as 金色
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
    // weeks+1 because the simulation grants free_conv at week 0 (initial resources)
    // plus at each of weeks 1..=W, totalling (W+1) batches of free conversions.
    let conv_capacity = adv.free_conversions_per_week * (weeks + 1) + adv.conversion_stones;
    if total_conversions_needed > conv_capacity {
        return false;
    }

    // Precompute per-shop aggregates for 金色 checks
    let mut shop_pages_sum = [0u32; 5];
    let mut shop_self_sum = [0u32; 5];
    for i in 0..n {
        let si = input.combat_skills[i].shop.index();
        shop_pages_sum[si] += total_pages[i];
        shop_self_sum[si] += self_needed[i];
    }
    let mut shop_surplus = [0u32; 5];
    for &shop in &Shop::ALL {
        let si = shop.index();
        shop_surplus[si] = (shop_pages_sum[si] + fodder_pool[si]).saturating_sub(shop_self_sum[si]);
    }
    let total_surplus: u32 = shop_surplus.iter().sum();

    if total_surplus < total_other {
        return false;
    }

    // Per-skill 金色 check: skill i can't use its OWN surplus as its own 金色.
    // Recompute i's shop surplus EXCLUDING skill i, to get the true available.
    // (Naive "total_surplus - own_surplus" overcounts when intra-shop conversions
    // consume part of the individual surplus.)
    // NOTE: must NOT skip on self_needed==0 — some realm/class combos (返虚, 合体百族
    // etc.) have self_pages=0 at low levels but still need gold_pages.
    for i in 0..n {
        if input.combat_skills[i].current_level >= input.combat_skills[i].target_level { continue; }
        let s = &input.combat_skills[i];
        let cost = total_cost_between(s.current_level, s.target_level, s.realm, s.skill_class);
        let si = s.shop.index();

        let shop_surplus_without_i = (shop_pages_sum[si] - total_pages[i] + fodder_pool[si])
            .saturating_sub(shop_self_sum[si] - self_needed[i]);
        let available_for_i = (total_surplus - shop_surplus[si]) + shop_surplus_without_i;

        if available_for_i < cost.gold_pages {
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
// Phase 2: Bonus levels — exhaustive search (provably optimal)
//
// Search space: each skill can gain 0..K bonus levels beyond its
// target, where K = max_level - target (up to 13 if target is 1星
// and max is 天5). Branch-and-bound pruning (trying high bonuses
// first for early good baselines) keeps the effective search small.
// Each feasibility check is O(n), so the total runs in microseconds.
// ============================================================

fn find_bonus_levels(input: &PlannerInput, min_weeks: u32) -> Vec<SkillLevel> {
    let n = input.combat_skills.len();
    let targets: Vec<SkillLevel> = input.combat_skills.iter().map(|s| s.target_level).collect();
    let max_levels: Vec<SkillLevel> = input.combat_skills.iter()
        .map(|s| max_level(s.realm, s.skill_class))
        .collect();
    // How many bonus levels each skill can potentially gain
    let bonus_range: Vec<usize> = (0..n).map(|i| {
        max_levels[i].index().saturating_sub(targets[i].index())
    }).collect();
    let max_possible_total: usize = bonus_range.iter().sum();
    if max_possible_total == 0 {
        return targets;
    }

    let mut best_total: usize = 0;
    let mut best = targets.clone();

    // Recursive search with pruning
    fn search(
        skill: usize,
        bonus: &mut Vec<usize>,
        current_total: usize,
        n: usize,
        bonus_range: &[usize],
        targets: &[SkillLevel],
        input: &PlannerInput,
        min_weeks: u32,
        best_total: &mut usize,
        best: &mut Vec<SkillLevel>,
    ) {
        if skill == n {
            if current_total > *best_total {
                let mut test = input.clone();
                for i in 0..n {
                    test.combat_skills[i].target_level = SkillLevel::ALL[targets[i].index() + bonus[i]];
                }
                if check_feasibility(&test, min_weeks) {
                    *best_total = current_total;
                    *best = (0..n).map(|i| SkillLevel::ALL[targets[i].index() + bonus[i]]).collect();
                }
            }
            return;
        }

        // Upper bound: current total + max from remaining skills
        let max_remaining: usize = bonus_range[skill..].iter().sum();
        if current_total + max_remaining <= *best_total {
            return; // Prune: can't beat current best
        }

        // Try from highest bonus down (find good solutions early for better pruning)
        for b in (0..=bonus_range[skill]).rev() {
            bonus[skill] = b;
            search(skill + 1, bonus, current_total + b, n, bonus_range, targets,
                   input, min_weeks, best_total, best);
        }
    }

    let mut bonus = vec![0usize; n];
    search(0, &mut bonus, 0, n, &bonus_range, &targets, input, min_weeks, &mut best_total, &mut best);
    best
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
            fodder_pools: self.fodder_pools,
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

    /// Surplus pages available as 金色 donor (pages beyond own remaining 本体 need)
    fn donatable(&self, i: usize) -> u32 {
        if self.levels[i] >= self.targets[i] {
            return self.pages[i]; // All pages are surplus
        }
        let cost = total_cost_between(self.levels[i], self.targets[i], self.realms[i], self.skill_classes[i]);
        self.pages[i].saturating_sub(cost.self_pages)
    }

    /// Fodder pool pages in `shop` that are safe to use as 金色 without starving
    /// same-shop 本体 conversions. `exclude`: the skill being upgraded (can't
    /// donate to itself, so excluded from intra-shop surplus calculation).
    fn fodder_available_as_other(&self, shop: Shop, exclude: usize, n: usize) -> u32 {
        let si = shop.index();
        let pool = self.fodder_pools[si];
        // Total 本体 deficit for all skills in this shop
        let deficit: u32 = (0..n)
            .filter(|&j| self.shops[j] == shop && self.levels[j] < self.targets[j])
            .map(|j| self.self_remaining_need(j))
            .sum();
        // Intra-shop skill surplus that can cover part of the deficit via conversion
        let skill_surplus: u32 = (0..n)
            .filter(|&j| j != exclude && self.shops[j] == shop)
            .map(|j| self.donatable(j))
            .sum();
        let needed_from_pool = deficit.saturating_sub(skill_surplus);
        pool.saturating_sub(needed_from_pool)
    }

    /// Per-shop gold budget: max gold extractable without starving same-shop conversions.
    /// Uses total self-cost (not deficit) to correctly reserve pages each skill needs for
    /// its own upgrades, matching the Phase 1 feasibility check formula.
    fn shop_gold_budgets(&self, exclude: usize, n: usize) -> [u32; 5] {
        let mut budgets = [0u32; 5];
        for &shop in &Shop::ALL {
            let si = shop.index();
            let total_pages: u32 = (0..n)
                .filter(|&j| self.shops[j] == shop)
                .map(|j| self.pages[j])
                .sum();
            let total_self_cost: u32 = (0..n)
                .filter(|&j| self.shops[j] == shop && self.levels[j] < self.targets[j])
                .map(|j| {
                    total_cost_between(self.levels[j], self.targets[j], self.realms[j], self.skill_classes[j]).self_pages
                })
                .sum();
            // Shop surplus: total pages + fodder beyond all self-costs
            let total_surplus = (total_pages + self.fodder_pools[si]).saturating_sub(total_self_cost);
            // Excluded skill's surplus can't be used as its own gold
            let exclude_surplus = if self.shops[exclude] == shop {
                self.donatable(exclude)
            } else {
                0
            };
            budgets[si] = total_surplus.saturating_sub(exclude_surplus);
        }
        budgets
    }

    /// Try upgrade skill i, consuming 金色 from other skills' surplus
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

        // Check 金色: per-shop gold budget ensures we don't starve same-shop conversions
        let mut gold_budgets = self.shop_gold_budgets(i, n);
        let total_gold_available: u32 = gold_budgets.iter().sum();
        if total_gold_available < cost.gold_pages { return None; }

        // Execute
        self.pages[i] -= cost.self_pages;

        // Consume 金色: fodder pools first (low rarity), then skills' surplus.
        // Respect per-shop gold budget to preserve conversion capacity.
        let mut consumed: HashMap<String, u32> = HashMap::new();
        let mut remaining = cost.gold_pages;
        let mut shop_order: Vec<Shop> = Shop::ALL.to_vec();
        shop_order.sort_by_key(|s| s.rarity());

        // From fodder pools (limited by shop gold budget)
        for &shop in &shop_order {
            if remaining == 0 { break; }
            let si = shop.index();
            let safe = self.fodder_available_as_other(shop, i, n);
            let take = safe.min(remaining).min(gold_budgets[si]);
            if take > 0 {
                self.fodder_pools[si] -= take;
                gold_budgets[si] -= take;
                consumed.insert(format!("pool_{}", si), take);
                remaining -= take;
            }
        }
        // Then from other combat skills' surplus (limited by shop gold budget)
        let mut donors: Vec<usize> = (0..n).filter(|&j| j != i).collect();
        donors.sort_by_key(|&j| self.shops[j].rarity());
        for &j in &donors {
            if remaining == 0 { break; }
            let si = self.shops[j].index();
            let give = self.donatable(j).min(remaining).min(gold_budgets[si]);
            if give > 0 {
                self.pages[j] -= give;
                gold_budgets[si] -= give;
                consumed.insert(j.to_string(), give);
                remaining -= give;
            }
        }

        // Safety: if actual sources couldn't cover the gold cost, abort
        if remaining > 0 {
            // Undo consumed gold
            for (key, amount) in &consumed {
                if let Some(si_str) = key.strip_prefix("pool_") {
                    if let Ok(si) = si_str.parse::<usize>() {
                        self.fodder_pools[si] += amount;
                    }
                } else if let Ok(j) = key.parse::<usize>() {
                    self.pages[j] += amount;
                }
            }
            self.pages[i] += cost.self_pages;
            return None;
        }

        self.purple -= cost.purple_pages;
        self.blue -= cost.blue_pages;
        self.levels[i] = next;

        Some(UpgradeAction {
            skill_index: i,
            from_level: from,
            to_level: next,
            self_pages_used: cost.self_pages,
            gold_pages_consumed: consumed,
            purple_pages_used: cost.purple_pages,
            blue_pages_used: cost.blue_pages,
        })
    }
}

/// Interleave conversions and upgrades until no more progress.
/// `free_conv` is the number of free conversions available this cycle.
///
/// Conversion slot priority: first 3 free → stones → remaining free.
fn do_conversions_and_upgrades(
    state: &mut SimState,
    free_conv: u32,
    conversions: &mut Vec<ConversionAction>,
    upgrades: &mut Vec<UpgradeAction>,
) {
    let n = state.levels.len();
    // Split free conversions into priority tier (first 3) and extra tier (rest).
    // Priority free is used before stones; extra free is used after stones.
    let mut priority_left = free_conv.min(PRIORITY_FREE_CONV);
    let mut extra_left = free_conv - priority_left;
    loop {
        // Try upgrading all possible
        // Priority: (1) upgrades needing less 金色 first (avoid competing for shared resource),
        //           (2) among equal 金色 cost, smaller gap first (finish sooner → free surplus)
        let mut upgraded = false;
        loop {
            let mut order: Vec<usize> = (0..n).filter(|&i| state.levels[i] < state.targets[i]).collect();
            order.sort_by_key(|&i| {
                let next = state.levels[i].next().unwrap();
                let costs = upgrade_costs_for_category(cost_category(state.realms[i], state.skill_classes[i]));
                let other_cost = costs.get(next.index() - 1).map_or(0, |c| c.gold_pages);
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

        // Try one conversion for the skill that most immediately benefits.
        // Only consider skills whose NEXT level actually needs more self-pages than
        // they currently have — skills blocked on gold/purple/blue (not self-pages)
        // gain nothing from conversion and would waste the slot.
        if priority_left == 0 && state.stones == 0 && extra_left == 0 { break; }
        let mut candidates: Vec<usize> = (0..n)
            .filter(|&i| {
                if state.levels[i] >= state.targets[i] { return false; }
                let next = state.levels[i].next().unwrap();
                let costs = upgrade_costs_for_category(cost_category(state.realms[i], state.skill_classes[i]));
                let self_for_next = costs.get(next.index() - 1).map_or(0, |c| c.self_pages);
                state.pages[i] < self_for_next
            })
            .collect();
        // Prioritize by how few pages are still needed for the NEXT level-up
        // (smallest deficit = one conversion has maximum impact)
        candidates.sort_by_key(|&i| {
            let next = state.levels[i].next().unwrap();
            let costs = upgrade_costs_for_category(cost_category(state.realms[i], state.skill_classes[i]));
            let self_for_next = costs.get(next.index() - 1).map_or(0, |c| c.self_pages);
            self_for_next.saturating_sub(state.pages[i])
        });

        let mut converted = false;
        for &i in &candidates {
            let shop = state.shops[i];

            // Try to acquire a conversion slot: priority free → stone → extra free
            macro_rules! try_use_slot {
                () => {
                    if priority_left > 0 { priority_left -= 1; Some(false) }
                    else if state.stones > 0 { state.stones -= 1; Some(true) }
                    else if extra_left > 0 { extra_left -= 1; Some(false) }
                    else { None }
                };
            }

            // Source 1: same-shop fodder pool
            if state.fodder_pools[shop.index()] >= PAGES_PER_UNIT {
                if let Some(used_stone) = try_use_slot!() {
                    state.fodder_pools[shop.index()] -= PAGES_PER_UNIT;
                    state.pages[i] += PAGES_PER_UNIT;
                    conversions.push(ConversionAction {
                        shop, target_skill_index: i, from_skill_index: usize::MAX,
                        used_stone, pages: PAGES_PER_UNIT,
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
                if let Some(used_stone) = try_use_slot!() {
                    state.pages[donor] -= PAGES_PER_UNIT;
                    state.pages[i] += PAGES_PER_UNIT;
                    conversions.push(ConversionAction {
                        shop, target_skill_index: i, from_skill_index: donor,
                        used_stone, pages: PAGES_PER_UNIT,
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
    let mut fodder_incomes = Vec::new();
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
            fodder_incomes.push(FodderPoolIncome { shop, pages });
        }
    }

    // Step 2 & 3: Interleave conversions and upgrades
    let free_conv = state.free_conv;
    do_conversions_and_upgrades(state, free_conv, &mut conversions, &mut upgrades);

    WeekPlan {
        week,
        incomes,
        fodder_incomes,
        conversions,
        upgrades,
        snapshot: state.snapshot(),
    }
}

// ============================================================
// Entry point
// ============================================================

/// Run the weekly simulation with the given targets.
/// Returns (week_plans, last_simulated_week, final_levels).
fn simulate_plan(
    input: &PlannerInput,
    targets: &[SkillLevel],
) -> (Vec<WeekPlan>, u32, Vec<SkillLevel>) {
    let mut state = SimState::from_input(input, targets);
    let mut weeks = Vec::new();
    let mut last_week = 0u32;

    // Week 0: conversions + upgrades with initial resources
    {
        let mut conversions = Vec::new();
        let mut upgrades = Vec::new();
        let free_conv = state.free_conv;
        do_conversions_and_upgrades(&mut state, free_conv, &mut conversions, &mut upgrades);
        if !conversions.is_empty() || !upgrades.is_empty() {
            weeks.push(WeekPlan {
                week: 0, incomes: Vec::new(), fodder_incomes: Vec::new(),
                conversions, upgrades, snapshot: state.snapshot(),
            });
        }
    }

    let mut w = 1u32;
    while !state.all_done() && w <= MAX_WEEKS * 2 {
        weeks.push(simulate_week(&mut state, input, w));
        last_week = w;
        w += 1;
    }

    (weeks, last_week, state.levels.clone())
}

pub fn run_planner(input: &PlannerInput) -> PlannerOutput {
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

    let mut effective_weeks = min_weeks;
    let mut final_targets = find_bonus_levels(input, effective_weeks);

    // Iteratively re-optimize bonus levels when simulation extends past expected weeks.
    // The feasibility check uses aggregate conversion capacity but the simulation enforces
    // a per-week limit, so extra weeks may be needed. Those extra weeks provide additional
    // income that may enable higher bonus levels — re-run Phase 2 to capture this.
    for _ in 0..10 {
        let (weeks, actual_weeks, final_levels) = simulate_plan(input, &final_targets);

        if actual_weeks <= effective_weeks {
            return PlannerOutput {
                feasible: true, weeks, unreachable_reasons: Vec::new(), final_levels,
            };
        }

        let new_targets = find_bonus_levels(input, actual_weeks);
        if new_targets == final_targets {
            return PlannerOutput {
                feasible: true, weeks, unreachable_reasons: Vec::new(), final_levels,
            };
        }

        effective_weeks = actual_weeks;
        final_targets = new_targets;
    }

    // Convergence cap reached — return the last simulation result
    let (weeks, _, final_levels) = simulate_plan(input, &final_targets);
    PlannerOutput {
        feasible: true, weeks, unreachable_reasons: Vec::new(), final_levels,
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
        total_other_need += cost.gold_pages;
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

    // Global 金色 check (combat skill surplus + fodder pool income)
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
