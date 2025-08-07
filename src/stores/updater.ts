import { defineStore } from 'pinia';

// import { check as checkUpdater, Update } from '@tauri-apps/plugin-updater';

type Updater = {
  // update: Update | null;
  version: string | null;
};

export const useUpdaterStore = defineStore('updater', {
  state: () => ({
    // update: null,
    version: null,
  } as Updater),
  getters: {
    // isUpdateAvailable(state): boolean {
    //   return state.update !== null;
    // },
  },
  actions: {
    // async check() {
    //   const update = await checkUpdater()
    //   this.update = update;
    //   return update;
    // },
    // install() {
    //   if (!this.update) return;
    //   return this.update.downloadAndInstall();
    // },
    // download() {
    //   if (!this.update) return;
    //   return this.update.download();
    // },
  }
})