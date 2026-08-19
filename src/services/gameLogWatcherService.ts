import { commands, type LogLocationSnapshot } from '@/platform/tauri/bindings';

export async function getCurrentLogLocation(): Promise<LogLocationSnapshot | null> {
    return commands.logWatcherGetCurrentLocation();
}
