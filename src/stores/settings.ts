import { defineStore } from 'pinia';
import { invoke } from "@tauri-apps/api/core";

type Settings = {
  id: number;
  steam_api_key: string | null;
  poll_interval: number;
  highwatermark: string;
};

export const useSettingsStore = defineStore('settings', {
  state: () => ({
    id: 0,
    steam_api_key: null as string | null,
    poll_interval: 0,
    highwatermark: "0",
    loaded: false,
  } as Settings & { loaded: boolean }),
  actions: {
    async load() {
      const settings = await invoke<Settings>('get_settings_command').catch(() => { });
      if (!settings) return;
      this.id = settings.id;
      this.steam_api_key = settings.steam_api_key;
      this.poll_interval = settings.poll_interval;
      this.highwatermark = settings.highwatermark;
      this.loaded = settings.steam_api_key != null &&
        settings.poll_interval > 0;
    },
    save() {
      return invoke('set_settings_command', {
        settings: {
          id: this.id,
          steam_api_key: this.steam_api_key,
          poll_interval: this.poll_interval,
          highwatermark: this.highwatermark
        } as Settings
      });
    }
  }
})