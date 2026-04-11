import { SHOPS, SKILL_LEVELS } from "../types/game";
import type { CombatSkillInput } from "../types/planner";

interface Props {
  index: number;
  skill: CombatSkillInput;
  onChange: (skill: CombatSkillInput) => void;
}

export default function SkillCard({ index, skill, onChange }: Props) {
  return (
    <div className="border border-gray-200 rounded-lg p-4 bg-white shadow-sm">
      <div className="flex items-center gap-2 mb-3">
        <span className="text-sm font-bold text-amber-600">#{index + 1}</span>
        <input
          type="text"
          placeholder="备注名称（可选）"
          value={skill.label}
          onChange={(e) => onChange({ ...skill, label: e.target.value })}
          className="flex-1 text-sm border-b border-gray-200 focus:border-amber-500 outline-none py-1 bg-transparent"
        />
      </div>
      <div className="grid grid-cols-2 gap-3">
        <div>
          <label className="block text-xs text-gray-500 mb-1">商店</label>
          <select
            value={skill.shop}
            onChange={(e) => onChange({ ...skill, shop: e.target.value as typeof skill.shop })}
            className="w-full text-sm border border-gray-200 rounded px-2 py-1.5 bg-white focus:border-amber-500 outline-none"
          >
            {SHOPS.map((s) => (
              <option key={s} value={s}>{s}</option>
            ))}
          </select>
        </div>
        <div>
          <label className="block text-xs text-gray-500 mb-1">当前等级</label>
          <select
            value={skill.currentLevel}
            onChange={(e) => onChange({ ...skill, currentLevel: e.target.value as typeof skill.currentLevel })}
            className="w-full text-sm border border-gray-200 rounded px-2 py-1.5 bg-white focus:border-amber-500 outline-none"
          >
            {SKILL_LEVELS.map((l) => (
              <option key={l} value={l}>{l}</option>
            ))}
          </select>
        </div>
        <div>
          <label className="block text-xs text-gray-500 mb-1">剩余本体书页</label>
          <input
            type="number"
            min={0}
            step={40}
            value={skill.remainingPages}
            onChange={(e) => onChange({ ...skill, remainingPages: Math.max(0, parseInt(e.target.value) || 0) })}
            className="w-full text-sm border border-gray-200 rounded px-2 py-1.5 focus:border-amber-500 outline-none"
          />
        </div>
        <div>
          <label className="block text-xs text-gray-500 mb-1">目标等级</label>
          <select
            value={skill.targetLevel}
            onChange={(e) => onChange({ ...skill, targetLevel: e.target.value as typeof skill.targetLevel })}
            className="w-full text-sm border border-gray-200 rounded px-2 py-1.5 bg-white focus:border-amber-500 outline-none"
          >
            {SKILL_LEVELS.map((l) => (
              <option key={l} value={l}>{l}</option>
            ))}
          </select>
        </div>
      </div>
    </div>
  );
}
