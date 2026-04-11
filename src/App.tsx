import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import SkillCard from "./components/SkillCard";
import PlanOutput from "./components/PlanOutput";
import { SHOPS } from "./types/game";
import type {
  CombatSkillInput,
  PlannerInput,
  PlannerOutput,
  AdvancedSettings,
  FodderIncome,
} from "./types/planner";
import { defaultCombatSkill, defaultAdvancedSettings, skillDisplayName } from "./types/planner";

function createInitialSkills(): CombatSkillInput[] {
  return Array.from({ length: 6 }, () => defaultCombatSkill());
}

export default function App() {
  const [skills, setSkills] = useState<CombatSkillInput[]>(createInitialSkills);
  const [purplePages, setPurplePages] = useState(0);
  const [bluePages, setBluePages] = useState(0);
  const [advanced, setAdvanced] = useState<AdvancedSettings>(defaultAdvancedSettings);
  const [output, setOutput] = useState<PlannerOutput | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const updateSkill = (idx: number, skill: CombatSkillInput) => {
    const next = [...skills];
    next[idx] = skill;
    setSkills(next);
  };

  const handleGenerate = async () => {
    setLoading(true);
    setError(null);
    setOutput(null);

    const input: PlannerInput = {
      combatSkills: skills.map((s) => ({
        shop: s.shop,
        currentLevel: s.currentLevel,
        remainingPages: s.remainingPages,
        targetLevel: s.targetLevel,
        label: skillDisplayName(s),
        incomeCycleWeeks: s.incomeCycleWeeks,
        incomeBatchCount: s.incomeBatchCount,
      })),
      purplePages,
      bluePages,
      advanced,
    };

    try {
      const result = await invoke<PlannerOutput>("generate_plan", { input });
      setOutput(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-5xl mx-auto px-4 py-6">
        <h1 className="text-xl font-bold text-gray-800 mb-1">问剑长生 · 神通规划</h1>
        <p className="text-sm text-gray-500 mb-6">选择 6 个战斗神通，设定目标等级，生成最优升级路径</p>

        {/* 战斗神通 */}
        <section className="mb-6">
          <h2 className="text-sm font-medium text-gray-700 mb-3">战斗神通</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
            {skills.map((skill, i) => (
              <SkillCard key={i} index={i} skill={skill} onChange={(s) => updateSkill(i, s)} />
            ))}
          </div>
        </section>

        {/* 材料与设置 */}
        <section className="mb-6">
          <h2 className="text-sm font-medium text-gray-700 mb-3">材料与设置</h2>
          <div className="bg-white border border-gray-200 rounded-lg p-4 shadow-sm space-y-4">
            <div className="grid grid-cols-4 gap-4">
              <div>
                <label className="block text-xs text-purple-500 mb-1">紫色书页</label>
                <input
                  type="number" min={0} value={purplePages}
                  onChange={(e) => setPurplePages(Math.max(0, parseInt(e.target.value) || 0))}
                  className="w-full text-sm border border-gray-200 rounded px-2 py-1.5 focus:border-purple-400 outline-none"
                />
              </div>
              <div>
                <label className="block text-xs text-blue-500 mb-1">蓝色书页</label>
                <input
                  type="number" min={0} value={bluePages}
                  onChange={(e) => setBluePages(Math.max(0, parseInt(e.target.value) || 0))}
                  className="w-full text-sm border border-gray-200 rounded px-2 py-1.5 focus:border-blue-400 outline-none"
                />
              </div>
              <div>
                <label className="block text-xs text-amber-600 mb-1">转换石</label>
                <input
                  type="number" min={0} value={advanced.conversionStones}
                  onChange={(e) => setAdvanced({ ...advanced, conversionStones: Math.max(0, parseInt(e.target.value) || 0) })}
                  className="w-full text-sm border border-gray-200 rounded px-2 py-1.5 focus:border-amber-500 outline-none"
                />
              </div>
              <div>
                <label className="block text-xs text-gray-500 mb-1">每周转换次数</label>
                <input
                  type="number" min={3} max={10} value={advanced.freeConversionsPerWeek}
                  onChange={(e) => setAdvanced({ ...advanced, freeConversionsPerWeek: Math.min(10, Math.max(3, parseInt(e.target.value) || 3)) })}
                  className="w-full text-sm border border-gray-200 rounded px-2 py-1.5 focus:border-amber-500 outline-none"
                />
              </div>
            </div>
            {/* 狗粮池收入 */}
            <div>
              <label className="block text-xs text-gray-500 mb-2">狗粮池收入（非战斗神通的书页积累速度）</label>
              <div className="space-y-2">
                {SHOPS.map((shop) => {
                  const fi = advanced.fodderIncome[shop];
                  const updateFI = (patch: Partial<FodderIncome>) => {
                    setAdvanced({
                      ...advanced,
                      fodderIncome: { ...advanced.fodderIncome, [shop]: { ...fi, ...patch } },
                    });
                  };
                  return (
                    <div key={shop} className="flex items-center gap-2 text-sm text-gray-600">
                      <span className="w-10 text-gray-500 shrink-0">{shop}</span>
                      <span className="text-gray-400">每</span>
                      <input type="number" min={1} value={fi.cycleWeeks}
                        onChange={(e) => updateFI({ cycleWeeks: Math.max(1, parseInt(e.target.value) || 1) })}
                        className="w-14 h-8 text-center border border-gray-200 rounded focus:border-amber-500 outline-none" />
                      <span className="text-gray-400">周获取</span>
                      <input type="number" min={0} value={fi.batchCount}
                        onChange={(e) => updateFI({ batchCount: Math.max(0, parseInt(e.target.value) || 0) })}
                        className="w-14 h-8 text-center border border-gray-200 rounded focus:border-amber-500 outline-none" />
                      <span className="text-gray-400">本</span>
                    </div>
                  );
                })}
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-xs text-gray-500 mb-1">紫色书页每周收入</label>
                <input
                  type="number" min={0} value={advanced.weeklyPurpleIncome}
                  onChange={(e) => setAdvanced({ ...advanced, weeklyPurpleIncome: Math.max(0, parseInt(e.target.value) || 0) })}
                  className="w-full text-sm border border-gray-200 rounded px-2 py-1.5 focus:border-purple-400 outline-none"
                />
              </div>
              <div>
                <label className="block text-xs text-gray-500 mb-1">蓝色书页每周收入</label>
                <input
                  type="number" min={0} value={advanced.weeklyBlueIncome}
                  onChange={(e) => setAdvanced({ ...advanced, weeklyBlueIncome: Math.max(0, parseInt(e.target.value) || 0) })}
                  className="w-full text-sm border border-gray-200 rounded px-2 py-1.5 focus:border-blue-400 outline-none"
                />
              </div>
            </div>
          </div>
        </section>

        <button
          onClick={handleGenerate}
          disabled={loading}
          className="w-full py-3 bg-amber-500 hover:bg-amber-600 disabled:bg-gray-300 text-white font-medium rounded-lg transition-colors mb-6 cursor-pointer disabled:cursor-not-allowed"
        >
          {loading ? "规划中..." : "生成规划方案"}
        </button>

        {error && (
          <div className="bg-red-50 border border-red-200 rounded-lg p-4 mb-6">
            <p className="text-sm text-red-700">{error}</p>
          </div>
        )}

        {output && (
          <section>
            <h2 className="text-sm font-medium text-gray-700 mb-3">规划方案</h2>
            <PlanOutput output={output} skills={skills} />
          </section>
        )}
      </div>
    </div>
  );
}
