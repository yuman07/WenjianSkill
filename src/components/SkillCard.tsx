import { REALMS, SKILL_LEVELS, classesForRealm, shopsForClass, defaultIncomeForShop, availableLevels } from "../types/game";
import type { Realm, SkillClass, SkillLevel } from "../types/game";
import type { CombatSkillInput } from "../types/planner";

interface Props {
  index: number;
  skill: CombatSkillInput;
  onChange: (skill: CombatSkillInput) => void;
  duplicate?: boolean;
}

export default function SkillCard({ index, skill, onChange, duplicate }: Props) {
  const availableClasses = classesForRealm(skill.realm);
  const availableShops = shopsForClass(skill.skillClass);
  const levels = availableLevels(skill.realm, skill.skillClass);
  const currentIdx = SKILL_LEVELS.indexOf(skill.currentLevel);
  const targetLevels = levels.filter((l) => SKILL_LEVELS.indexOf(l) >= currentIdx);

  /** Clamp a level to the available levels for a given realm+class */
  const clampLevel = (level: SkillLevel, realm: Realm, cls: SkillClass): SkillLevel => {
    const lvls = availableLevels(realm, cls);
    return lvls.includes(level) ? level : lvls[lvls.length - 1];
  };

  /** Ensure targetLevel >= currentLevel after clamping */
  const ensureTargetGeCurrent = (cur: SkillLevel, tgt: SkillLevel, realm: Realm, cls: SkillClass): SkillLevel => {
    const ci = SKILL_LEVELS.indexOf(cur);
    const ti = SKILL_LEVELS.indexOf(tgt);
    if (ti >= ci) return tgt;
    // Target fell below current after clamp — push it up
    const lvls = availableLevels(realm, cls);
    return lvls.find((l) => SKILL_LEVELS.indexOf(l) >= ci) ?? cur;
  };

  const handleRealmChange = (realm: typeof skill.realm) => {
    const classes = classesForRealm(realm);
    const cls = classes.includes(skill.skillClass) ? skill.skillClass : classes[0];
    const shops = shopsForClass(cls);
    const shop = shops.includes(skill.shop) ? skill.shop : shops[0];
    const income = defaultIncomeForShop(shop);
    const currentLevel = clampLevel(skill.currentLevel, realm, cls);
    const targetLevel = ensureTargetGeCurrent(currentLevel, clampLevel(skill.targetLevel, realm, cls), realm, cls);
    onChange({ ...skill, realm, skillClass: cls, shop, currentLevel, targetLevel, incomeCycleWeeks: income.cycleWeeks, incomeBatchCount: income.batchCount });
  };

  const handleClassChange = (cls: typeof skill.skillClass) => {
    const shops = shopsForClass(cls);
    const shop = shops.includes(skill.shop) ? skill.shop : shops[0];
    const income = defaultIncomeForShop(shop);
    const currentLevel = clampLevel(skill.currentLevel, skill.realm, cls);
    const targetLevel = ensureTargetGeCurrent(currentLevel, clampLevel(skill.targetLevel, skill.realm, cls), skill.realm, cls);
    onChange({ ...skill, skillClass: cls, shop, currentLevel, targetLevel, incomeCycleWeeks: income.cycleWeeks, incomeBatchCount: income.batchCount });
  };

  const handleShopChange = (shop: typeof skill.shop) => {
    const income = defaultIncomeForShop(shop);
    onChange({ ...skill, shop, incomeCycleWeeks: income.cycleWeeks, incomeBatchCount: income.batchCount });
  };

  const inputCls = "w-full h-8 text-sm border border-gray-200 rounded px-2 bg-white focus:border-amber-500 outline-none";
  const disabledCls = "w-full h-8 text-sm border border-gray-200 rounded px-2 bg-gray-100 text-gray-500 outline-none";

  return (
    <div className={`border rounded-lg p-4 bg-white shadow-sm ${duplicate ? "border-red-400" : "border-gray-200"}`}>
      <div className="flex items-center gap-2 mb-3">
        <span className="text-sm font-bold text-amber-600">神通 #{index + 1}</span>
        {duplicate && <span className="text-xs text-red-500">重复神通</span>}
      </div>
      <div className="grid grid-cols-2 gap-x-3 gap-y-2">
        <div>
          <label className="block text-xs text-gray-500 mb-1">境界</label>
          <select value={skill.realm} onChange={(e) => handleRealmChange(e.target.value as typeof skill.realm)} className={inputCls}>
            {REALMS.map((r) => <option key={r} value={r}>{r}</option>)}
          </select>
        </div>
        <div>
          <label className="block text-xs text-gray-500 mb-1">职业</label>
          <select value={skill.skillClass} onChange={(e) => handleClassChange(e.target.value as typeof skill.skillClass)} className={inputCls}>
            {availableClasses.map((c) => <option key={c} value={c}>{c}</option>)}
          </select>
        </div>
        <div>
          <label className="block text-xs text-gray-500 mb-1">商店</label>
          <select value={skill.shop} onChange={(e) => handleShopChange(e.target.value as typeof skill.shop)}
            disabled={availableShops.length === 1} className={availableShops.length === 1 ? disabledCls : inputCls}>
            {availableShops.map((s) => <option key={s} value={s}>{s}</option>)}
          </select>
        </div>
        <div>
          <label className="block text-xs text-gray-500 mb-1">当前等级</label>
          <select value={skill.currentLevel} onChange={(e) => {
            const cur = e.target.value as SkillLevel;
            const ci = SKILL_LEVELS.indexOf(cur);
            const ti = SKILL_LEVELS.indexOf(skill.targetLevel);
            const targetLevel = ti >= ci ? skill.targetLevel : cur;
            onChange({ ...skill, currentLevel: cur, targetLevel });
          }} className={inputCls}>
            {levels.map((l) => <option key={l} value={l}>{l}</option>)}
          </select>
        </div>
        <div>
          <label className="block text-xs text-gray-500 mb-1">剩余书页</label>
          <input type="number" min={0} step={40} value={skill.remainingPages}
            onChange={(e) => onChange({ ...skill, remainingPages: Math.max(0, parseInt(e.target.value) || 0) })}
            className={inputCls} />
        </div>
        <div>
          <label className="block text-xs text-gray-500 mb-1">目标等级</label>
          <select value={skill.targetLevel} onChange={(e) => onChange({ ...skill, targetLevel: e.target.value as typeof skill.targetLevel })} className={inputCls}>
            {targetLevels.map((l) => <option key={l} value={l}>{l}</option>)}
          </select>
        </div>
      </div>
      <div className="mt-2 pt-2 border-t border-gray-100">
        <label className="block text-xs text-gray-500 mb-1">本体兑换</label>
        <div className="grid grid-cols-2 gap-x-3">
          <div className="flex items-center gap-1">
            <span className="text-xs text-gray-400 shrink-0">每</span>
            <input type="number" min={1} value={skill.incomeCycleWeeks}
              onChange={(e) => onChange({ ...skill, incomeCycleWeeks: Math.max(1, parseInt(e.target.value) || 1) })}
              className={inputCls} />
            <span className="text-xs text-gray-400 shrink-0">周</span>
          </div>
          <div className="flex items-center gap-1">
            <span className="text-xs text-gray-400 shrink-0">获取</span>
            <input type="number" min={0} value={skill.incomeBatchCount}
              onChange={(e) => onChange({ ...skill, incomeBatchCount: Math.max(0, parseInt(e.target.value) || 0) })}
              className={inputCls} />
            <span className="text-xs text-gray-400 shrink-0">本</span>
          </div>
        </div>
      </div>
    </div>
  );
}
