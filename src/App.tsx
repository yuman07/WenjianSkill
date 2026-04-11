import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import SkillCard from "./components/SkillCard";
import AdvancedSettings from "./components/AdvancedSettings";
import PlanOutput from "./components/PlanOutput";
import type { CombatSkillInput, PlannerInput, PlannerOutput, AdvancedSettings as AdvSettings } from "./types/planner";
import { defaultCombatSkill, defaultAdvancedSettings } from "./types/planner";

function createInitialSkills(): CombatSkillInput[] {
  return Array.from({ length: 6 }, () => defaultCombatSkill());
}

export default function App() {
  const [skills, setSkills] = useState<CombatSkillInput[]>(createInitialSkills);
  const [purplePages, setPurplePages] = useState(0);
  const [bluePages, setBluePages] = useState(0);
  const [advanced, setAdvanced] = useState<AdvSettings>(defaultAdvancedSettings);
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
      combatSkills: skills,
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
      <div className="max-w-4xl mx-auto px-4 py-6">
        {/* Header */}
        <h1 className="text-xl font-bold text-gray-800 mb-1">问剑长生 · 神通规划</h1>
        <p className="text-sm text-gray-500 mb-6">选择 6 个战斗神通，设定目标，生成最优升级路径</p>

        {/* 战斗神通输入 */}
        <section className="mb-6">
          <h2 className="text-sm font-medium text-gray-700 mb-3">战斗神通</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {skills.map((skill, i) => (
              <SkillCard
                key={i}
                index={i}
                skill={skill}
                onChange={(s) => updateSkill(i, s)}
              />
            ))}
          </div>
        </section>

        {/* 通用材料 */}
        <section className="mb-6">
          <h2 className="text-sm font-medium text-gray-700 mb-3">通用材料</h2>
          <div className="grid grid-cols-2 gap-4 max-w-md">
            <div className="bg-white border border-gray-200 rounded-lg p-3 shadow-sm">
              <label className="block text-xs text-purple-500 mb-1">紫色书页</label>
              <input
                type="number"
                min={0}
                value={purplePages}
                onChange={(e) => setPurplePages(Math.max(0, parseInt(e.target.value) || 0))}
                className="w-full text-sm border border-gray-200 rounded px-2 py-1.5 focus:border-purple-400 outline-none"
              />
            </div>
            <div className="bg-white border border-gray-200 rounded-lg p-3 shadow-sm">
              <label className="block text-xs text-blue-500 mb-1">蓝色书页</label>
              <input
                type="number"
                min={0}
                value={bluePages}
                onChange={(e) => setBluePages(Math.max(0, parseInt(e.target.value) || 0))}
                className="w-full text-sm border border-gray-200 rounded px-2 py-1.5 focus:border-blue-400 outline-none"
              />
            </div>
          </div>
        </section>

        {/* 高级设置 */}
        <section className="mb-6">
          <AdvancedSettings settings={advanced} onChange={setAdvanced} />
        </section>

        {/* 生成按钮 */}
        <button
          onClick={handleGenerate}
          disabled={loading}
          className="w-full py-3 bg-amber-500 hover:bg-amber-600 disabled:bg-gray-300 text-white font-medium rounded-lg transition-colors mb-6"
        >
          {loading ? "规划中..." : "生成规划方案"}
        </button>

        {/* 错误提示 */}
        {error && (
          <div className="bg-red-50 border border-red-200 rounded-lg p-4 mb-6">
            <p className="text-sm text-red-700">{error}</p>
          </div>
        )}

        {/* 输出 */}
        {output && (
          <section>
            <h2 className="text-sm font-medium text-gray-700 mb-3">规划方案</h2>
            <PlanOutput output={output} skillLabels={skills} />
          </section>
        )}
      </div>
    </div>
  );
}
