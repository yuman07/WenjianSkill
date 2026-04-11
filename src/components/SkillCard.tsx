import { REALMS, SKILL_LEVELS, classesForRealm, shopsForClass, defaultIncomeForShop } from "../types/game";
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
    const income = defaultIncomeForShop(shop);
    onChange({ ...skill, realm, skillClass: cls, shop, incomeCycleWeeks: income.cycleWeeks, incomeBatchCount: income.batchCount });
  };

  const handleClassChange = (cls: typeof skill.skillClass) => {
    const shops = shopsForClass(cls);
    const shop = shops.includes(skill.shop) ? skill.shop : shops[0];
    const income = defaultIncomeForShop(shop);
    onChange({ ...skill, skillClass: cls, shop, incomeCycleWeeks: income.cycleWeeks, incomeBatchCount: income.batchCount });
  };

  const handleShopChange = (shop: typeof skill.shop) => {
    const income = defaultIncomeForShop(shop);
    onChange({ ...skill, shop, incomeCycleWeeks: income.cycleWeeks, incomeBatchCount: income.batchCount });
  };

  const inputCls = "w-full h-8 text-sm border border-gray-200 rounded px-2 bg-white focus:border-amber-500 outline-none";
  const disabledCls = "w-full h-8 text-sm border border-gray-200 rounded px-2 bg-gray-100 text-gray-500 outline-none";

  return (
    <div className="border border-gray-200 rounded-lg p-4 bg-white shadow-sm">
      <div className="text-sm font-bold text-amber-600 mb-3">神通 #{index + 1}</div>
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
          <select value={skill.currentLevel} onChange={(e) => onChange({ ...skill, currentLevel: e.target.value as typeof skill.currentLevel })} className={inputCls}>
            {SKILL_LEVELS.map((l) => <option key={l} value={l}>{l}</option>)}
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
            {SKILL_LEVELS.map((l) => <option key={l} value={l}>{l}</option>)}
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
