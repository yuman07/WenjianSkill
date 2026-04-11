import { useState } from "react";
import { SHOPS } from "../types/game";
import type { AdvancedSettings as AdvancedSettingsType } from "../types/planner";
import type { Shop } from "../types/game";

interface Props {
  settings: AdvancedSettingsType;
  onChange: (settings: AdvancedSettingsType) => void;
}

export default function AdvancedSettings({ settings, onChange }: Props) {
  const [open, setOpen] = useState(false);

  const updatePool = (shop: Shop, val: number) => {
    onChange({
      ...settings,
      nonCombatPools: { ...settings.nonCombatPools, [shop]: Math.max(0, val) },
    });
  };

  const updateIncome = (shop: Shop, val: number) => {
    onChange({
      ...settings,
      weeklyShopIncome: { ...settings.weeklyShopIncome, [shop]: Math.max(0, val) },
    });
  };

  return (
    <div className="border border-gray-200 rounded-lg bg-white shadow-sm">
      <button
        onClick={() => setOpen(!open)}
        className="w-full flex items-center justify-between px-4 py-3 text-sm font-medium text-gray-700 hover:bg-gray-50 transition-colors"
      >
        <span>高级设置</span>
        <span className={`transition-transform ${open ? "rotate-180" : ""}`}>▼</span>
      </button>
      {open && (
        <div className="px-4 pb-4 space-y-4 border-t border-gray-100">
          {/* 转换石 */}
          <div className="pt-3">
            <label className="block text-xs text-gray-500 mb-1">转换石数量</label>
            <input
              type="number"
              min={0}
              value={settings.conversionStones}
              onChange={(e) => onChange({ ...settings, conversionStones: Math.max(0, parseInt(e.target.value) || 0) })}
              className="w-32 text-sm border border-gray-200 rounded px-2 py-1.5 focus:border-amber-500 outline-none"
            />
          </div>

          {/* 非战斗书页池 */}
          <div>
            <label className="block text-xs text-gray-500 mb-2">非战斗书页池（按商店）</label>
            <div className="grid grid-cols-5 gap-2">
              {SHOPS.map((shop) => (
                <div key={shop}>
                  <label className="block text-xs text-gray-400 mb-1">{shop}</label>
                  <input
                    type="number"
                    min={0}
                    step={40}
                    value={settings.nonCombatPools[shop]}
                    onChange={(e) => updatePool(shop, parseInt(e.target.value) || 0)}
                    className="w-full text-sm border border-gray-200 rounded px-2 py-1.5 focus:border-amber-500 outline-none"
                  />
                </div>
              ))}
            </div>
          </div>

          {/* 每周商店收入 */}
          <div>
            <label className="block text-xs text-gray-500 mb-2">每周商店获取次数</label>
            <div className="grid grid-cols-5 gap-2">
              {SHOPS.map((shop) => (
                <div key={shop}>
                  <label className="block text-xs text-gray-400 mb-1">{shop}</label>
                  <input
                    type="number"
                    min={0}
                    value={settings.weeklyShopIncome[shop]}
                    onChange={(e) => updateIncome(shop, parseInt(e.target.value) || 0)}
                    className="w-full text-sm border border-gray-200 rounded px-2 py-1.5 focus:border-amber-500 outline-none"
                  />
                </div>
              ))}
            </div>
          </div>

          {/* 每周紫色/蓝色收入 */}
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-xs text-gray-500 mb-1">紫色书页每周收入</label>
              <input
                type="number"
                min={0}
                value={settings.weeklyPurpleIncome}
                onChange={(e) => onChange({ ...settings, weeklyPurpleIncome: Math.max(0, parseInt(e.target.value) || 0) })}
                className="w-full text-sm border border-gray-200 rounded px-2 py-1.5 focus:border-amber-500 outline-none"
              />
            </div>
            <div>
              <label className="block text-xs text-gray-500 mb-1">蓝色书页每周收入</label>
              <input
                type="number"
                min={0}
                value={settings.weeklyBlueIncome}
                onChange={(e) => onChange({ ...settings, weeklyBlueIncome: Math.max(0, parseInt(e.target.value) || 0) })}
                className="w-full text-sm border border-gray-200 rounded px-2 py-1.5 focus:border-amber-500 outline-none"
              />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
