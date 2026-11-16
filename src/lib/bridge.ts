interface TauriInternals {
  invoke(command: string, args?: Record<string, unknown>): Promise<unknown>;
}

declare global {
  interface Window {
    __TAURI_INTERNALS__?: TauriInternals;
  }
}

export const isTauri: boolean = typeof window !== "undefined" && window.__TAURI_INTERNALS__ !== undefined;

export const isMobileDevice: boolean = typeof navigator !== "undefined" && /android|iphone|ipad|ipod/i.test(navigator.userAgent);

export async function call<T>(command: string, args: Record<string, unknown>, fallback: () => T | Promise<T>): Promise<T> {
  if (!isTauri || !window.__TAURI_INTERNALS__) {
    return fallback();
  }
  try {
    return (await window.__TAURI_INTERNALS__.invoke(command, args)) as T;
  } catch {
    return fallback();
  }
}

export type WindowAction = "minimize" | "maximize" | "close";

export function windowAction(action: WindowAction): void {
  void call<void>(`window_${action}`, {}, () => undefined);
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => {
    window.setTimeout(resolve, ms);
  });
}
