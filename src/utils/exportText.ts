import type { PlannerOutput, WeekPlan, CombatSkillInput } from "../types/planner";
import { skillDisplayName } from "../types/planner";
import { SHOPS } from "../types/game";
import { formatDonors } from "./donorLabel";

function n(skills: CombatSkillInput[], idx: number): string {
  return skillDisplayName(skills[idx]);
}

function conversionSource(fromIdx: number, skills: CombatSkillInput[], shop: string): string {
  if (fromIdx >= skills.length) {
    return `「${shop}」狗粮池`;
  }
  return n(skills, fromIdx);
}

function divider(): string {
  return "─".repeat(40);
}

function formatWeek(week: WeekPlan, skills: CombatSkillInput[]): string {
  const lines: string[] = [];
  const hasContent = week.incomes.length > 0 || week.fodderIncomes.length > 0 || week.conversions.length > 0 || week.upgrades.length > 0;
  const isInitial = week.week === 0;
  const isBonus = !isInitial && week.incomes.length === 0 && week.fodderIncomes.length === 0 && week.conversions.length === 0 && week.upgrades.length > 0;

  if (isInitial) lines.push("【立即可做】");
  else if (isBonus) lines.push("【目标达成后 · 剩余资源分配】");
  else if (!hasContent) lines.push(`【第 ${week.week} 周 · 积累资源】`);
  else lines.push(`【第 ${week.week} 周】`);
  lines.push("");

  let step = 1;

  if (week.incomes.length > 0 || week.fodderIncomes.length > 0) {
    lines.push(`  ${step}. 兑换书页`);
    step++;
    for (const inc of week.incomes) {
      lines.push(`     - ${n(skills, inc.skillIndex)} +${inc.pages} 张本体书页`);
    }
    for (const fi of week.fodderIncomes) {
      lines.push(`     - 「${fi.shop}」狗粮池 +${fi.pages} 张书页`);
    }
    lines.push("");
  }

  if (week.conversions.length > 0) {
    lines.push(`  ${step}. 转换书页`);
    step++;
    for (const c of week.conversions) {
      const stone = c.usedStone ? "（消耗转换石）" : "";
      const from = conversionSource(c.fromSkillIndex, skills, c.shop);
      lines.push(`     - 从 ${from} 取 ${c.pages} 张 → ${n(skills, c.targetSkillIndex)}${stone}`);
    }
    lines.push("");
  }

  if (week.upgrades.length > 0) {
    lines.push(`  ${step}. 升级神通`);
    for (let ui = 0; ui < week.upgrades.length; ui++) {
      const u = week.upgrades[ui];
      lines.push(`     - ${n(skills, u.skillIndex)}: ${u.fromLevel} → ${u.toLevel}`);
      const costs: string[] = [`本体 ${u.selfPagesUsed}张`];
      const donorList = formatDonors(u.goldPagesConsumed, skills);
      if (donorList.length > 0) {
        costs.push(`金色: ${donorList.join("、")}`);
      }
      if (u.purplePagesUsed > 0) costs.push(`紫色 ${u.purplePagesUsed}`);
      if (u.bluePagesUsed > 0) costs.push(`蓝色 ${u.bluePagesUsed}`);
      lines.push(`       消耗: ${costs.join(", ")}`);
      if (ui < week.upgrades.length - 1) {
        const nextSkillDiff = week.upgrades[ui + 1].skillIndex !== u.skillIndex;
        lines.push(nextSkillDiff ? "     ════════════════════" : "     ──────");
      }
    }
    lines.push("");
  }

  if (!hasContent) {
    lines.push("  本周无操作，等待下周资源积累");
    lines.push("");
  }

  lines.push("  本周结束后:");
  for (let i = 0; i < week.snapshot.skillLevels.length; i++) {
    const extra = week.snapshot.skillPages[i] > 0 ? `（余${week.snapshot.skillPages[i]}页）` : "";
    lines.push(`    ${n(skills, i)}: ${week.snapshot.skillLevels[i]}${extra}`);
  }
  lines.push(`    紫色 ${week.snapshot.purplePages} / 蓝色 ${week.snapshot.bluePages}${week.snapshot.conversionStonesLeft > 0 ? ` / 转换石 ${week.snapshot.conversionStonesLeft}` : ""}`);
  lines.push(`    狗粮池: ${SHOPS.map((shop, i) => `${shop} ${week.snapshot.fodderPools[i]}`).join(" / ")}`);

  return lines.join("\n");
}

export function generatePlanText(output: PlannerOutput, skills: CombatSkillInput[]): string {
  const lines: string[] = [];
  lines.push("问剑长生 · 神通升级规划方案");
  lines.push(divider());
  lines.push("");
  lines.push("目标:");
  for (let i = 0; i < skills.length; i++) {
    lines.push(`  ${i + 1}. ${skillDisplayName(skills[i])}  当前 ${skills[i].currentLevel} → 目标 ${skills[i].targetLevel}`);
  }
  lines.push("");

  const totalWeeks = output.weeks.length > 0 ? output.weeks[output.weeks.length - 1].week : 0;
  if (output.feasible) {
    lines.push(`规划结果: 全部目标可达成，共需 ${totalWeeks} 周`);
  } else {
    lines.push("规划结果: 部分目标无法达成");
    for (const r of output.unreachableReasons) lines.push(`  ! ${r}`);
  }

  lines.push("");
  lines.push("最终等级:");
  for (let i = 0; i < output.finalLevels.length; i++) {
    lines.push(`  ${skillDisplayName(skills[i])}: ${output.finalLevels[i]}`);
  }

  lines.push("");
  lines.push(divider());
  lines.push("详细步骤");
  lines.push(divider());

  for (const w of output.weeks) {
    lines.push("");
    lines.push(formatWeek(w, skills));
    lines.push(divider());
  }
  return lines.join("\n");
}
