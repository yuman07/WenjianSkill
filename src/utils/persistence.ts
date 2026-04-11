import { load } from "@tauri-apps/plugin-store";
import type { CombatSkillInput, AdvancedSettings } from "../types/planner";

const STORE_FILE = "settings.json";

export interface PersistedState {
  skills: CombatSkillInput[];
  purplePages: number;
  bluePages: number;
  advanced: AdvancedSettings;
}

let storePromise: ReturnType<typeof load> | null = null;

function getStore() {
  if (!storePromise) {
    storePromise = load(STORE_FILE, { autoSave: true });
  }
  return storePromise;
}

export async function saveState(state: PersistedState): Promise<void> {
  const store = await getStore();
  await store.set("appState", state);
}

export async function loadState(): Promise<PersistedState | null> {
  try {
    const store = await getStore();
    const state = await store.get<PersistedState>("appState");
    return state ?? null;
  } catch {
    return null;
  }
}
