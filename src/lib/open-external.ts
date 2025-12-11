/**
 * 打开外部链接工具函数
 * 根据平台使用不同实现
 */

/**
 * 在系统默认浏览器中打开外部链接
 * - Tauri: 使用 @tauri-apps/plugin-opener
 * - Web: 使用 window.open()
 */
export async function openExternal(url: string): Promise<void> {
  if (__PLATFORM__ === "web") {
    window.open(url, "_blank", "noopener,noreferrer")
  } else {
    const { openUrl } = await import("@tauri-apps/plugin-opener")
    await openUrl(url)
  }
}
