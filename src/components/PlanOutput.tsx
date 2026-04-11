import { useState } from "react";
import type { PlannerOutput, WeekPlan } from "../types/planner";
import type { CombatSkillInput } from "../types/planner";

interface Props {
  output: PlannerOutput;
  skillLabels: CombatSkillInput[];
}

function skillName(skills: CombatSkillInput[], idx: number): string {
  const s = skills[idx];
  return s.label || `神通${idx + 1}(${s.shop})`;
}

function WeekCard({ week, skills }: { week: WeekPlan; skills: CombatSkillInput[] }) {
  const [open, setOpen] = useState(week.week <= 4);

  const title = week.week === 0
    ? "初始资源分配"
    : week.upgrades.length === 0 && week.conversions.length === 0 && week.acquisitions.length === 0
      ? `第 ${week.week} 周（积累资源）`
      : `第 ${week.week} 周`;

  const isBonus = week.acquisitions.length === 0 && week.conversions.length === 0 && week.upgrades.length > 0 && week.week > 0;

  return (
    <div className="border border-gray-200 rounded-lg bg-white shadow-sm">
      <button
        onClick={() => setOpen(!open)}
        className="w-full flex items-center justify-between px-4 py-3 text-sm font-medium text-gray-700 hover:bg-gray-50 transition-colors"
      >
        <div className="flex items-center gap-2">
          <span>{isBonus ? "剩余资源分配" : title}</span>
          {week.upgrades.length > 0 && (
            <span className="text-xs bg-green-100 text-green-700 px-1.5 py-0.5 rounded">
              {week.upgrades.length} 次升级
            </span>
          )}
          {week.conversions.length > 0 && (
            <span className="text-xs bg-blue-100 text-blue-700 px-1.5 py-0.5 rounded">
              {week.conversions.length} 次转换
            </span>
          )}
        </div>
        <span className={`transition-transform ${open ? "rotate-180" : ""}`}>▼</span>
      </button>
      {open && (
        <div className="px-4 pb-4 space-y-3 border-t border-gray-100 pt-3">
          {/* 商店获取 */}
          {week.acquisitions.length > 0 && (
            <div>
              <h4 className="text-xs font-medium text-gray-500 mb-1.5">📥 商店获取</h4>
              {week.acquisitions.map((a, i) => (
                <div key={i} className="text-sm text-gray-600 ml-2">
                  {a.shop}：+{a.pages} 书页 →{" "}
                  {a.targetSkillIndex !== null
                    ? skillName(skills, a.targetSkillIndex)
                    : "非战斗池"}
                </div>
              ))}
            </div>
          )}

          {/* 转换 */}
          {week.conversions.length > 0 && (
            <div>
              <h4 className="text-xs font-medium text-gray-500 mb-1.5">🔄 转换操作</h4>
              {week.conversions.map((c, i) => (
                <div key={i} className="text-sm text-gray-600 ml-2">
                  {c.shop}池 → {skillName(skills, c.targetSkillIndex)} ({c.pages}张)
                  {c.usedStone && <span className="text-amber-600 ml-1">⬥转换石</span>}
                </div>
              ))}
            </div>
          )}

          {/* 升级 */}
          {week.upgrades.length > 0 && (
            <div>
              <h4 className="text-xs font-medium text-gray-500 mb-1.5">⬆️ 升级操作</h4>
              {week.upgrades.map((u, i) => (
                <div key={i} className="text-sm ml-2 space-y-0.5">
                  <div className="text-gray-800 font-medium">
                    {skillName(skills, u.skillIndex)}: {u.fromLevel} → {u.toLevel}
                  </div>
                  <div className="text-xs text-gray-400">
                    本体 {u.selfPagesUsed}
                    {u.purplePagesUsed > 0 && ` · 紫 ${u.purplePagesUsed}`}
                    {u.bluePagesUsed > 0 && ` · 蓝 ${u.bluePagesUsed}`}
                    {Object.entries(u.otherPagesConsumed)
                      .filter(([, v]) => (v as number) > 0)
                      .map(([k, v]) => ` · ${k}池 ${v}`)
                      .join("")}
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* 状态快照 */}
          <div className="pt-2 border-t border-gray-100">
            <h4 className="text-xs font-medium text-gray-500 mb-1.5">📊 当前状态</h4>
            <div className="grid grid-cols-3 gap-2 text-xs text-gray-500">
              {week.snapshot.skillLevels.map((lv, i) => (
                <div key={i}>
                  {skillName(skills, i)}: <span className="font-medium text-gray-700">{lv}</span>
                  {week.snapshot.skillPages[i] > 0 && (
                    <span className="text-gray-400"> +{week.snapshot.skillPages[i]}页</span>
                  )}
                </div>
              ))}
            </div>
            <div className="mt-1 text-xs text-gray-400">
              紫 {week.snapshot.purplePages} · 蓝 {week.snapshot.bluePages}
              {week.snapshot.conversionStonesLeft > 0 &&
                ` · 转换石 ${week.snapshot.conversionStonesLeft}`}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default function PlanOutput({ output, skillLabels }: Props) {
  if (!output.feasible && output.unreachableReasons.length > 0) {
    return (
      <div className="space-y-4">
        <div className="bg-amber-50 border border-amber-200 rounded-lg p-4">
          <h3 className="text-sm font-medium text-amber-800 mb-2">⚠ 部分目标不可达</h3>
          {output.unreachableReasons.map((r, i) => (
            <p key={i} className="text-sm text-amber-700">{r}</p>
          ))}
        </div>
        <h3 className="text-sm font-medium text-gray-700">以下为最佳方案：</h3>
        {output.weeks.map((w) => (
          <WeekCard key={w.week} week={w} skills={skillLabels} />
        ))}
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {/* 总览 */}
      <div className="bg-green-50 border border-green-200 rounded-lg p-4">
        <h3 className="text-sm font-medium text-green-800 mb-2">
          ✓ 规划完成 — 共 {output.weeks.length > 0 ? output.weeks[output.weeks.length - 1].week : 0} 周
        </h3>
        <div className="flex flex-wrap gap-3">
          {output.finalLevels.map((lv, i) => (
            <span key={i} className="text-sm text-green-700">
              {skillName(skillLabels, i)}: <span className="font-bold">{lv}</span>
            </span>
          ))}
        </div>
      </div>

      {/* 每周详情 */}
      {output.weeks.map((w) => (
        <WeekCard key={w.week} week={w} skills={skillLabels} />
      ))}
    </div>
  );
}
