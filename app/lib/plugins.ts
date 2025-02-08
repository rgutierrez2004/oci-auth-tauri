import { attachConsole } from '@tauri-apps/plugin-log';

// Initialize plugins
export async function initPlugins() {
    // Initialize logging plugin
    await attachConsole();
}
