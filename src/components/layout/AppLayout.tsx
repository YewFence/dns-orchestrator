import type { ReactNode } from "react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import { Globe, Menu } from "lucide-react"
import { MainContent } from "./MainContent"
import { Sidebar } from "./Sidebar"
import { Sheet, SheetContent, SheetTrigger } from "@/components/ui/sheet"
import { Button } from "@/components/ui/button"

interface AppLayoutProps {
  children?: ReactNode
  onOpenToolbox?: () => void
  onNavigateToMain?: () => void
  onOpenSettings?: () => void
}

export function AppLayout({
  children,
  onOpenToolbox,
  onNavigateToMain,
  onOpenSettings,
}: AppLayoutProps) {
  const { t } = useTranslation()
  const [sidebarOpen, setSidebarOpen] = useState(false)

  return (
    <div className="flex h-screen w-screen flex-col md:flex-row overflow-hidden bg-background md:pb-6">
      {/* 移动端顶部导航 - 仅移动端显示 */}
      <header className="flex items-center gap-2 border-b px-4 py-3 md:hidden">
        <Sheet open={sidebarOpen} onOpenChange={setSidebarOpen}>
          <SheetTrigger asChild>
            <Button variant="ghost" size="icon">
              <Menu className="h-5 w-5" />
            </Button>
          </SheetTrigger>
          <SheetContent side="left" className="w-72 p-0" hideClose>
            <Sidebar
              onOpenToolbox={() => {
                setSidebarOpen(false)
                onOpenToolbox?.()
              }}
              onNavigateToMain={() => {
                setSidebarOpen(false)
                onNavigateToMain?.()
              }}
              onOpenSettings={() => {
                setSidebarOpen(false)
                onOpenSettings?.()
              }}
              onClose={() => setSidebarOpen(false)}
              isMobile
            />
          </SheetContent>
        </Sheet>
        <Globe className="h-5 w-5 text-primary" />
        <h1 className="font-semibold">{t("common.appName")}</h1>
      </header>

      {/* 桌面端侧边栏 - 仅桌面端显示 */}
      <div className="hidden md:contents">
        <Sidebar onOpenToolbox={onOpenToolbox} onNavigateToMain={onNavigateToMain} />
      </div>

      {/* 主内容区 - 始终渲染，不会被卸载 */}
      <div className="flex flex-1 flex-col min-h-0 overflow-hidden">
        {children || <MainContent />}
      </div>
    </div>
  )
}
