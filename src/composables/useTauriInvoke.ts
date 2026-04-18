/**
 * useTauriInvoke · 薄包装 Tauri 2 的 invoke,提供类型化调用与集中错误处理。
 *
 * 现阶段错误用 console.error 打出;后续可替换为 toast composable。
 */

import { invoke, type InvokeArgs } from "@tauri-apps/api/core";

export async function invokeCmd<T>(
  cmd: string,
  args?: InvokeArgs,
): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (err) {
    console.error(`[invoke:${cmd}]`, err);
    throw err;
  }
}
