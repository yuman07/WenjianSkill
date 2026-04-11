import { REALMS, SKILL_LEVELS, classesForRealm, shopsForClass } from "../types/game";
import type { CombatSkillInput } from "../types/planner";

interface Props {
  index: number;
  skill: CombatSkillInput;
  onChange: (skill: CombatSkillInput) => void;
}

export default function SkillCard({ index, skill, onChange }: Props) {
  const availableClasses = classesForRealm(skill.realm);
  const availableShops = shopsForClass(skill.skillClass);

  const handleRealmChange = (realm: typeof skill.realm) => {
    const classes = classesForRealm(realm);
    const cls = classes.includes(skill.skillClass) ? skill.skillClass : classes[0];
    const shops = shopsForClass(cls);
    const shop = shops.includes(skill.shop) ? skill.shop : shops[0];
    onChange({ ...skill, realm, skillClass: cls, shop });
  };

  const handleClassChange = (cls: typeof skill.skillClass) => {
    const shops = shopsForClass(cls);
    const shop = shops.includes(skill.shop) ? skill.shop : shops[0];
    onChange({ ...skill, skillClass: cls, shop });
  };

  return (
    <div className="border border-gray-200 rounded-lg p-4 bg-white shadow-sm">
      <div className="text-sm font-bold text-amber-600 mb-3">神通 #{index + 1}</div>
      <div className="grid grid-cols-2 gap-3">
        <div>
          <label className="block text-xs text-gray-500 mb-1">境界</label>
          <select
            value={skill.realm}
            onChange={(e) => handleRealmChange(e.target.value as typeof skill.realm)}
            className="w-full text-sm border border-gray-200 rounded px-2 py-1.5 bg-white focus:border-amber-500 outline-none"
          >
            {REALMS.map((r) => (
              <option key={r} value={r}>{r}</option>
            ))}
          </select>
        </div>
        <div>
          <label className="block text-xs text-gray-500 mb-1">职业</label>
          <select
            value={skill.skillClass}
            onChange={(e) => handleClassChange(e.target.value as typeof skill.skillClass)}
            className="w-full text-sm border border-gray-200 rounded px-2 py-1.5 bg-white focus:border-amber-500 outline-none"
          >
            {availableClasses.map((c) => (
              <option key={c} value={c}>{c}</option>
            ))}
          </select>
        </div>
        <div>
          <label className="block text-xs text-gray-500 mb-1">商店</label>
          <select
            value={skill.shop}
            onChange={(e) => onChange({ ...skill, shop: e.target.value as typeof skill.shop })}
            disabled={availableShops.length === 1}
            className="w-full text-sm border border-gray-200 rounded px-2 py-1.5 bg-white focus:border-amber-500 outline-none disabled:bg-gray-100 disabled:text-gray-500"
          >
            {availableShops.map((s) => (
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
