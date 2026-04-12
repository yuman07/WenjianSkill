import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
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
import { saveState, loadState } from "./utils/persistence";

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
  const [showInfo, setShowInfo] = useState(false);
  const infoRef = useRef<HTMLDivElement>(null);
  const initialized = useRef(false);

  // Close info popover when clicking outside
  useEffect(() => {
    if (!showInfo) return;
    const handler = (e: MouseEvent) => {
      if (infoRef.current && !infoRef.current.contains(e.target as Node)) {
        setShowInfo(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [showInfo]);

  // Load persisted state on startup, merging with defaults for forward compatibility
  useEffect(() => {
    loadState().then((saved) => {
      if (saved) {
        const defSkill = defaultCombatSkill();
        setSkills(saved.skills.map((s) => ({ ...defSkill, ...s })));
        setPurplePages(saved.purplePages ?? 0);
        setBluePages(saved.bluePages ?? 0);
        const defAdv = defaultAdvancedSettings();
        setAdvanced({
          ...defAdv,
          ...saved.advanced,
          fodderIncome: { ...defAdv.fodderIncome, ...saved.advanced.fodderIncome },
        });
      }
      initialized.current = true;
    });
  }, []);

  // Auto-save when any input changes (debounced via effect cleanup)
  useEffect(() => {
    if (!initialized.current) return;
    const timer = setTimeout(() => {
      saveState({ skills, purplePages, bluePages, advanced });
    }, 500);
    return () => clearTimeout(timer);
  }, [skills, purplePages, bluePages, advanced]);

  const updateSkill = (idx: number, skill: CombatSkillInput) => {
    const next = [...skills];
    next[idx] = skill;
    setSkills(next);
  };

  // Detect duplicate skills (same realm+class+shop); 百族 skills are exempt
  const duplicateSet = new Set<number>();
  const seen = new Map<string, number>();
  skills.forEach((s, i) => {
    if (s.skillClass === "百族") return; // 百族神通允许重复
    const key = `${s.realm}|${s.skillClass}|${s.shop}`;
    if (seen.has(key)) {
      duplicateSet.add(seen.get(key)!);
      duplicateSet.add(i);
    } else {
      seen.set(key, i);
    }
  });

  const handleGenerate = async () => {
    if (duplicateSet.size > 0) {
      setError("存在重复的神通（相同境界+职业+商店），百族神通除外，请修改后重试");
      return;
    }

    setLoading(true);
    setError(null);
    setOutput(null);

    const input: PlannerInput = {
      combatSkills: skills.map((s) => ({
        realm: s.realm,
        skillClass: s.skillClass,
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
        <div className="flex items-baseline gap-2 mb-1">
          <h1 className="text-xl font-bold text-gray-800">问剑长生 · 神通规划</h1>
          <button onClick={() => open("https://github.com/yuman07/WenjianSkill")}
            className="text-xs text-blue-400 hover:text-blue-600 transition-colors cursor-pointer">v1.0.0</button>
          <div className="relative ml-auto self-center" ref={infoRef}>
            <button
              onClick={() => setShowInfo((v) => !v)}
              className="w-6 h-6 rounded-full border border-gray-300 text-gray-400 hover:border-amber-400 hover:text-amber-500 transition-colors text-xs font-serif font-bold leading-none cursor-pointer flex items-center justify-center"
              title="关于"
            >
              i
            </button>
            {showInfo && (
              <div className="absolute right-0 top-8 w-72 bg-white rounded-lg shadow-lg border border-gray-200 p-4 z-50">
                <p className="text-sm text-gray-700 leading-relaxed">
                  目前该版本可完美适配渡劫前的神通，渡劫的神通目前仍沿用大乘的神通升级消耗不一定准确，之后有准确数据后会再更新。
                </p>
                <hr className="my-3 border-gray-200" />
                <p className="text-sm text-gray-500 text-right">千山暮雪-昆虫子</p>
              </div>
            )}
          </div>
        </div>
        <p className="text-sm text-gray-500 mb-6">选择 6 个战斗神通，设定目标等级，生成最优升级路径</p>

        {/* 战斗神通 */}
        <section className="mb-6">
          <h2 className="text-sm font-medium text-gray-700 mb-3">战斗神通</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
            {skills.map((skill, i) => (
              <SkillCard key={i} index={i} skill={skill} onChange={(s) => updateSkill(i, s)} duplicate={duplicateSet.has(i)} />
            ))}
          </div>
        </section>

        {/* 材料与设置 */}
        <section className="mb-6">
          <h2 className="text-sm font-medium text-gray-700 mb-3">设置与材料</h2>
          <div className="bg-white border border-gray-200 rounded-lg p-4 shadow-sm space-y-4">
            {/* 转换 */}
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-xs text-gray-500 mb-1">每周转换次数</label>
                <input type="number" min={3} max={10} value={advanced.freeConversionsPerWeek}
                  onChange={(e) => setAdvanced({ ...advanced, freeConversionsPerWeek: Math.min(10, Math.max(3, parseInt(e.target.value) || 3)) })}
                  className="w-full h-8 text-sm border border-gray-200 rounded px-2 focus:border-amber-500 outline-none" />
              </div>
              <div>
                <label className="block text-xs text-gray-500 mb-1">转换石个数</label>
                <input type="number" min={0} value={advanced.conversionStones}
                  onChange={(e) => setAdvanced({ ...advanced, conversionStones: Math.max(0, parseInt(e.target.value) || 0) })}
                  className="w-full h-8 text-sm border border-gray-200 rounded px-2 focus:border-amber-500 outline-none" />
              </div>
            </div>

            {/* 狗粮池 */}
            <div>
              <label className="block text-xs text-gray-500 mb-2">狗粮池（非战斗神通的书页）</label>
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
                    <div key={shop} className="grid grid-cols-[2.5rem_1fr_1fr_1fr] gap-2 items-center">
                      <span className="text-xs text-gray-500">{shop}</span>
                      <div className="flex items-center gap-1">
                        <span className="text-xs text-gray-400 shrink-0">现有</span>
                        <input type="number" min={0} step={40} value={fi.initialPages}
                          onChange={(e) => updateFI({ initialPages: Math.max(0, parseInt(e.target.value) || 0) })}
                          className="w-full h-8 text-sm text-center border border-gray-200 rounded focus:border-amber-500 outline-none" />
                        <span className="text-xs text-gray-400 shrink-0">张</span>
                      </div>
                      <div className="flex items-center gap-1">
                        <span className="text-xs text-gray-400 shrink-0">每</span>
                        <input type="number" min={1} value={fi.cycleWeeks}
                          onChange={(e) => updateFI({ cycleWeeks: Math.max(1, parseInt(e.target.value) || 1) })}
                          className="w-full h-8 text-sm text-center border border-gray-200 rounded focus:border-amber-500 outline-none" />
                        <span className="text-xs text-gray-400 shrink-0">周</span>
                      </div>
                      <div className="flex items-center gap-1">
                        <span className="text-xs text-gray-400 shrink-0">获取</span>
                        <input type="number" min={0} value={fi.batchCount}
                          onChange={(e) => updateFI({ batchCount: Math.max(0, parseInt(e.target.value) || 0) })}
                          className="w-full h-8 text-sm text-center border border-gray-200 rounded focus:border-amber-500 outline-none" />
                        <span className="text-xs text-gray-400 shrink-0">本</span>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>

            {/* 紫色书页 */}
            <div className="grid grid-cols-[4rem_1fr_1fr] gap-2 items-center">
              <span className="text-xs font-medium" style={{ color: "#8b5cf6" }}>紫色书页</span>
              <div className="flex items-center gap-1">
                <span className="text-xs shrink-0" style={{ color: "#8b5cf6" }}>现有</span>
                <input type="number" min={0} value={purplePages}
                  onChange={(e) => setPurplePages(Math.max(0, parseInt(e.target.value) || 0))}
                  className="w-full h-8 text-sm text-center border border-gray-200 rounded outline-none" style={{ borderColor: undefined }} onFocus={(e) => e.target.style.borderColor="#8b5cf6"} onBlur={(e) => e.target.style.borderColor=""} />
                <span className="text-xs shrink-0" style={{ color: "#8b5cf6" }}>张</span>
              </div>
              <div className="flex items-center gap-1">
                <span className="text-xs shrink-0" style={{ color: "#8b5cf6" }}>每周获取</span>
                <input type="number" min={0} value={advanced.weeklyPurpleIncome}
                  onChange={(e) => setAdvanced({ ...advanced, weeklyPurpleIncome: Math.max(0, parseInt(e.target.value) || 0) })}
                  className="w-full h-8 text-sm text-center border border-gray-200 rounded outline-none" onFocus={(e) => e.target.style.borderColor="#8b5cf6"} onBlur={(e) => e.target.style.borderColor=""} />
                <span className="text-xs shrink-0" style={{ color: "#8b5cf6" }}>张</span>
              </div>
            </div>

            {/* 蓝色书页 */}
            <div className="grid grid-cols-[4rem_1fr_1fr] gap-2 items-center">
              <span className="text-xs font-medium" style={{ color: "#3b82f6" }}>蓝色书页</span>
              <div className="flex items-center gap-1">
                <span className="text-xs shrink-0" style={{ color: "#3b82f6" }}>现有</span>
                <input type="number" min={0} value={bluePages}
                  onChange={(e) => setBluePages(Math.max(0, parseInt(e.target.value) || 0))}
                  className="w-full h-8 text-sm text-center border border-gray-200 rounded outline-none" onFocus={(e) => e.target.style.borderColor="#3b82f6"} onBlur={(e) => e.target.style.borderColor=""} />
                <span className="text-xs shrink-0" style={{ color: "#3b82f6" }}>张</span>
              </div>
              <div className="flex items-center gap-1">
                <span className="text-xs shrink-0" style={{ color: "#3b82f6" }}>每周获取</span>
                <input type="number" min={0} value={advanced.weeklyBlueIncome}
                  onChange={(e) => setAdvanced({ ...advanced, weeklyBlueIncome: Math.max(0, parseInt(e.target.value) || 0) })}
                  className="w-full h-8 text-sm text-center border border-gray-200 rounded outline-none" onFocus={(e) => e.target.style.borderColor="#3b82f6"} onBlur={(e) => e.target.style.borderColor=""} />
                <span className="text-xs shrink-0" style={{ color: "#3b82f6" }}>张</span>
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
