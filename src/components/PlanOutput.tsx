import { useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import type { PlannerOutput, WeekPlan, CombatSkillInput } from "../types/planner";
import { skillDisplayName } from "../types/planner";
import { SHOPS } from "../types/game";
import { generatePlanText } from "../utils/exportText";
import { formatDonors } from "../utils/donorLabel";

interface Props {
  output: PlannerOutput;
  skills: CombatSkillInput[];
}

function name(skills: CombatSkillInput[], idx: number): string {
  return skillDisplayName(skills[idx]);
}

function WeekCard({ week, skills }: { week: WeekPlan; skills: CombatSkillInput[] }) {
  const [open, setOpen] = useState(week.week <= 3);

  const hasContent = week.incomes.length > 0 || week.fodderIncomes.length > 0 || week.conversions.length > 0 || week.upgrades.length > 0;
  const isInitial = week.week === 0;
  const isBonus = !isInitial && week.incomes.length === 0 && week.fodderIncomes.length === 0 && week.conversions.length === 0 && week.upgrades.length > 0;

  let title: string;
  if (isInitial) title = "立即可做";
  else if (isBonus) title = "目标达成后 · 剩余资源分配";
  else if (!hasContent) title = `第 ${week.week} 周 · 积累资源`;
  else title = `第 ${week.week} 周`;

  return (
    <div className="border border-gray-200 rounded-lg bg-white shadow-sm overflow-hidden">
      <button
        onClick={() => setOpen(!open)}
        className="w-full flex items-center justify-between px-4 py-3 text-left hover:bg-gray-50 transition-colors"
      >
        <div className="flex items-center gap-2 flex-wrap">
          <span className="text-sm font-medium text-gray-800">{title}</span>
          {week.upgrades.length > 0 && (
            <span className="text-xs bg-green-100 text-green-700 px-2 py-0.5 rounded-full">升级 ×{week.upgrades.length}</span>
          )}
          {week.conversions.length > 0 && (
            <span className="text-xs bg-blue-100 text-blue-700 px-2 py-0.5 rounded-full">转换 ×{week.conversions.length}</span>
          )}
        </div>
        <span className={`text-gray-400 text-xs transition-transform ${open ? "rotate-180" : ""}`}>▼</span>
      </button>

      {open && (
        <div className="px-4 pb-4 border-t border-gray-100">
          <div className="mt-3 space-y-3">
            {/* 收入 */}
            {(week.incomes.length > 0 || week.fodderIncomes.length > 0) && (
              <div>
                <div className="text-xs font-medium text-gray-500 mb-1.5">第一步：兑换书页</div>
                {week.incomes.map((inc, i) => (
                  <div key={i} className="text-sm text-gray-700 ml-3 leading-relaxed">
                    <span className="font-medium">{name(skills, inc.skillIndex)}</span> +{inc.pages} 张本体书页
                  </div>
                ))}
                {week.fodderIncomes.map((fi, i) => (
                  <div key={`f${i}`} className="text-sm text-gray-700 ml-3 leading-relaxed">
                    <span className="font-medium">「{fi.shop}」狗粮池</span> +{fi.pages} 张书页
                  </div>
                ))}
              </div>
            )}

            {/* 转换 */}
            {week.conversions.length > 0 && (
              <div>
                <div className="text-xs font-medium text-gray-500 mb-1.5">
                  {week.incomes.length > 0 || week.fodderIncomes.length > 0 ? "第二步" : "第一步"}：转换书页
                </div>
                {week.conversions.map((c, i) => (
                  <div key={i} className="text-sm text-gray-700 ml-3 leading-relaxed">
                    从 <span className="font-medium">{c.fromSkillIndex < skills.length ? name(skills, c.fromSkillIndex) : `「${c.shop}」狗粮池`}</span> 取 {c.pages} 张，转给{" "}
                    <span className="font-medium">{name(skills, c.targetSkillIndex)}</span>
                    {c.usedStone && <span className="ml-1 text-xs text-amber-600 bg-amber-50 px-1.5 py-0.5 rounded">消耗转换石</span>}
                  </div>
                ))}
              </div>
            )}

            {/* 升级 */}
            {week.upgrades.length > 0 && (
              <div>
                <div className="text-xs font-medium text-gray-500 mb-1.5">
                  {(week.incomes.length > 0 || week.fodderIncomes.length > 0) && week.conversions.length > 0 ? "第三步"
                    : (week.incomes.length > 0 || week.fodderIncomes.length > 0) || week.conversions.length > 0 ? "第二步"
                    : "操作"}：升级神通
                </div>
                {week.upgrades.map((u, i) => {
                  const donorList = formatDonors(u.otherPagesConsumed, skills);
                  const isLast = i === week.upgrades.length - 1;
                  const nextSkillDiff = !isLast && week.upgrades[i + 1].skillIndex !== u.skillIndex;
                  return (
                    <div key={i} className="ml-3">
                      <div className="text-sm text-gray-800">
                        <span className="font-medium">{name(skills, u.skillIndex)}</span>{" "}
                        <span className="text-gray-400">{u.fromLevel} → </span>
                        <span className="font-medium text-green-600">{u.toLevel}</span>
                      </div>
                      <div className="text-xs text-gray-400 mt-0.5 leading-relaxed">
                        消耗：本体 {u.selfPagesUsed} 张
                        {donorList.length > 0 && <>，仙品：{donorList.join("、")}</>}
                        {u.purplePagesUsed > 0 && <>，紫色 {u.purplePagesUsed}</>}
                        {u.bluePagesUsed > 0 && <>，蓝色 {u.bluePagesUsed}</>}
                      </div>
                      {!isLast && (nextSkillDiff
                        ? <hr className="my-3 border-gray-300" />
                        : <hr className="my-2 border-gray-100" />
                      )}
                    </div>
                  );
                })}
              </div>
            )}

            {!hasContent && (
              <div className="text-sm text-gray-400 italic mt-2">本周无操作，等待下周资源积累</div>
            )}
          </div>

          {/* 状态快照 */}
          <div className="mt-3 pt-3 border-t border-gray-100">
            <div className="text-xs font-medium text-gray-500 mb-2">本周结束后的状态</div>
            <div className="grid grid-cols-2 sm:grid-cols-3 gap-x-4 gap-y-1">
              {week.snapshot.skillLevels.map((lv, i) => (
                <div key={i} className="text-xs text-gray-600">
                  {name(skills, i)}：<span className="font-medium text-gray-800">{lv}</span>
                  {week.snapshot.skillPages[i] > 0 && (
                    <span className="text-gray-400">（余 {week.snapshot.skillPages[i]} 页）</span>
                  )}
                </div>
              ))}
            </div>
            <div className="mt-1.5 text-xs text-gray-400">
              紫色 {week.snapshot.purplePages} · 蓝色 {week.snapshot.bluePages}
              {week.snapshot.conversionStonesLeft > 0 && ` · 转换石 ${week.snapshot.conversionStonesLeft}`}
            </div>
            {week.snapshot.fodderPools.some(p => p > 0) && (
              <div className="mt-1 text-xs text-gray-400">
                狗粮池：{SHOPS.map((shop, i) =>
                  week.snapshot.fodderPools[i] > 0 ? `${shop} ${week.snapshot.fodderPools[i]}` : null
                ).filter(Boolean).join(" · ")}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

export default function PlanOutput({ output, skills }: Props) {
  const totalWeeks = output.weeks.length > 0 ? output.weeks[output.weeks.length - 1].week : 0;

  const handleExport = async () => {
    const text = generatePlanText(output, skills);
    const path = await save({
      defaultPath: "神通规划方案.txt",
      filters: [{ name: "文本文件", extensions: ["txt"] }],
    });
    if (path) { await writeTextFile(path, text); }
  };

  return (
    <div className="space-y-3">
      {!output.feasible && output.unreachableReasons.length > 0 && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <div className="text-sm font-medium text-red-800 mb-2">目标无法达成</div>
          {output.unreachableReasons.map((r, i) => (
            <p key={i} className="text-sm text-red-700 leading-relaxed mt-1">{r}</p>
          ))}
          <p className="text-xs text-red-500 mt-3">请调整目标等级或补充对应材料后重试</p>
        </div>
      )}

      {output.feasible && (
        <>
          <div className="bg-green-50 border border-green-200 rounded-lg p-4">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-green-800">规划完成，共需 {totalWeeks} 周</span>
              <button onClick={handleExport}
                className="text-xs px-3 py-1 border border-green-300 rounded-lg hover:bg-green-100 transition-colors text-green-700 cursor-pointer">
                导出 TXT
              </button>
            </div>
            <div className="flex flex-wrap gap-x-4 gap-y-1">
              {output.finalLevels.map((lv, i) => (
                <span key={i} className="text-sm text-green-700">
                  {name(skills, i)}：<span className="font-bold">{lv}</span>
                </span>
              ))}
            </div>
          </div>
          {output.weeks.map((w) => <WeekCard key={w.week} week={w} skills={skills} />)}
        </>
      )}
    </div>
  );
}
