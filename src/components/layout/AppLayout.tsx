import type { ReactNode } from "react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import { Globe, Menu } from "lucide-react"
import { MainContent } from "./MainContent"
import { Sidebar } from "./Sidebar"
import { Sheet, SheetContent, SheetTrigger } from "@/components/ui/sheet"
import { Button } from "@/components/ui/button"
import { useIsMobile } from "@/hooks/useMediaQuery"

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
  const isMobile = useIsMobile()
  const [sidebarOpen, setSidebarOpen] = useState(false)

  // 移动端布局
  if (isMobile) {
    return (
      <div className="flex h-screen w-screen flex-col overflow-hidden bg-background">
        {/* 移动端顶部导航 */}
        <header className="flex items-center gap-2 border-b px-4 py-3">
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

        {/* 主内容区 */}
        <div className="flex flex-1 flex-col min-h-0 overflow-hidden">
          {children || <MainContent />}
        </div>
      </div>
    )
  }

  // 桌面端保持原有布局
  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background pb-6">
      <Sidebar onOpenToolbox={onOpenToolbox} onNavigateToMain={onNavigateToMain} />
      {children || <MainContent />}
    </div>
  )
}
