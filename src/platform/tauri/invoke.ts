import type { InvokeArgs } from '@tauri-apps/api/core';

import { PlatformUnavailableError } from './errors';

type InvokeFn = typeof import('@tauri-apps/api/core').invoke;

let invokeFn: InvokeFn | null = null;

async function loadInvoke(): Promise<InvokeFn> {
    if (invokeFn) {
        return invokeFn;
    }

    try {
        const core = await import('@tauri-apps/api/core');
        invokeFn = core.invoke;
        return invokeFn;
    } catch {
        throw new PlatformUnavailableError('Unable to load Tauri invoke API');
    }
}

export async function invokeTauri<TReturn = unknown>(
    command: string,
    args?: InvokeArgs
): Promise<TReturn> {
    const invoke = await loadInvoke();
    return invoke<TReturn>(command, args);
}
