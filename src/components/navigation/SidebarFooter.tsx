import { Button } from "@/components/ui/button"
import { Settings, Wrench } from "lucide-react"
import { useTranslation } from "react-i18next"

interface SidebarFooterProps {
  onOpenToolbox?: () => void
  onOpenSettings?: () => void
}

export function SidebarFooter({ onOpenToolbox, onOpenSettings }: SidebarFooterProps) {
  const { t } = useTranslation()

  return (
    <div className="border-t p-2 space-y-1">
      <Button variant="ghost" className="w-full justify-start gap-3 h-10" onClick={onOpenToolbox}>
        <Wrench className="h-4 w-4" />
        <span>{t("toolbox.title")}</span>
      </Button>
      <Button variant="ghost" className="w-full justify-start gap-3 h-10" onClick={onOpenSettings}>
        <Settings className="h-4 w-4" />
        <span>{t("settings.title")}</span>
      </Button>
    </div>
  )
}
