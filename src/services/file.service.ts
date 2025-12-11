/**
 * 文件服务
 * 封装平台差异的文件操作
 */

export interface FileFilter {
  name: string
  extensions: string[]
}

export interface SaveFileOptions {
  defaultFilename: string
  filters?: FileFilter[]
}

export interface OpenFileOptions {
  filters?: FileFilter[]
}

export interface OpenFileResult {
  content: string
  filename: string
}

/**
 * 保存文件
 * - Tauri: 弹出保存对话框 + 写入文件
 * - Web: 创建 Blob 触发下载
 *
 * @returns 是否成功保存（用户取消返回 false）
 */
export async function saveFile(content: string, options: SaveFileOptions): Promise<boolean> {
  if (__PLATFORM__ === "web") {
    return saveFileWeb(content, options)
  }
  return saveFileTauri(content, options)
}

/**
 * 打开文件
 * - Tauri: 弹出文件选择对话框 + 读取文件
 * - Web: 创建 input[type=file] + FileReader
 *
 * @returns 文件内容和文件名，用户取消返回 null
 */
export async function openFile(options?: OpenFileOptions): Promise<OpenFileResult | null> {
  if (__PLATFORM__ === "web") {
    return openFileWeb(options)
  }
  return openFileTauri(options)
}

// ============ Tauri 实现 ============

async function saveFileTauri(content: string, options: SaveFileOptions): Promise<boolean> {
  const { save } = await import("@tauri-apps/plugin-dialog")
  const { writeTextFile } = await import("@tauri-apps/plugin-fs")

  const filePath = await save({
    defaultPath: options.defaultFilename,
    filters: options.filters,
  })

  if (!filePath) return false

  await writeTextFile(filePath, content)
  return true
}

async function openFileTauri(options?: OpenFileOptions): Promise<OpenFileResult | null> {
  const { open } = await import("@tauri-apps/plugin-dialog")
  const { readTextFile } = await import("@tauri-apps/plugin-fs")

  const filePath = await open({
    multiple: false,
    filters: options?.filters,
  })

  if (!filePath) return null

  const content = await readTextFile(filePath as string)
  const filename = (filePath as string).split("/").pop() || "file"

  return { content, filename }
}

// ============ Web 实现 ============

function saveFileWeb(content: string, options: SaveFileOptions): Promise<boolean> {
  return new Promise((resolve) => {
    const blob = new Blob([content], { type: "application/json" })
    const url = URL.createObjectURL(blob)

    const a = document.createElement("a")
    a.href = url
    a.download = options.defaultFilename
    a.style.display = "none"

    document.body.appendChild(a)
    a.click()

    // 清理
    setTimeout(() => {
      document.body.removeChild(a)
      URL.revokeObjectURL(url)
    }, 100)

    // Web 端无法知道用户是否真的保存了，假设成功
    resolve(true)
  })
}

function openFileWeb(options?: OpenFileOptions): Promise<OpenFileResult | null> {
  return new Promise((resolve) => {
    const input = document.createElement("input")
    input.type = "file"
    input.style.display = "none"

    // 设置文件类型过滤
    if (options?.filters?.length) {
      const extensions = options.filters.flatMap((f) => f.extensions.map((ext) => `.${ext}`))
      input.accept = extensions.join(",")
    }

    input.onchange = () => {
      const file = input.files?.[0]
      if (!file) {
        resolve(null)
        return
      }

      const reader = new FileReader()
      reader.onload = () => {
        resolve({
          content: reader.result as string,
          filename: file.name,
        })
      }
      reader.onerror = () => {
        resolve(null)
      }
      reader.readAsText(file)
    }

    // 用户取消
    input.oncancel = () => {
      resolve(null)
    }

    document.body.appendChild(input)
    input.click()

    // 清理（延迟以确保事件触发）
    setTimeout(() => {
      document.body.removeChild(input)
    }, 60000) // 1 分钟后清理，确保用户有足够时间选择
  })
}
