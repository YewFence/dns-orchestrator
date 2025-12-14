import { CheckSquare, Loader2, Trash2 } from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import { useShallow } from "zustand/react/shallow"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { Button } from "@/components/ui/button"
import { useAccountStore } from "@/stores"

export function AccountBatchActionBar() {
  const { t } = useTranslation()
  const [showConfirm, setShowConfirm] = useState(false)

  const { selectedAccountIds, accounts, isBatchDeleting } = useAccountStore(
    useShallow((state) => ({
      selectedAccountIds: state.selectedAccountIds,
      accounts: state.accounts,
      isBatchDeleting: state.isBatchDeleting,
    }))
  )

  const selectAllAccounts = useAccountStore((state) => state.selectAllAccounts)
  const clearSelection = useAccountStore((state) => state.clearSelection)
  const batchDeleteAccounts = useAccountStore((state) => state.batchDeleteAccounts)

  const selectedCount = selectedAccountIds.size
  const totalCount = accounts.length
  const isAllSelected = selectedCount === totalCount && totalCount > 0

  if (selectedCount === 0) return null

  const handleDelete = async () => {
    await batchDeleteAccounts()
    setShowConfirm(false)
  }

  return (
    <>
      <div className="fixed inset-x-0 bottom-4 z-50 mx-auto flex w-fit items-center gap-3 rounded-full border bg-background px-4 py-2 shadow-lg">
        <span className="text-muted-foreground text-sm">
          {t("account.selectedCount", { count: selectedCount })}
        </span>
        <Button
          variant="ghost"
          size="sm"
          onClick={isAllSelected ? clearSelection : selectAllAccounts}
        >
          <CheckSquare className="mr-2 h-4 w-4" />
          {isAllSelected ? t("common.deselectAll") : t("common.selectAll")}
        </Button>
        <Button
          variant="destructive"
          size="sm"
          onClick={() => setShowConfirm(true)}
          disabled={isBatchDeleting}
        >
          {isBatchDeleting ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Trash2 className="mr-2 h-4 w-4" />
          )}
          {t("account.batchDelete")}
        </Button>
      </div>

      {/* 批量删除确认 */}
      <AlertDialog open={showConfirm} onOpenChange={setShowConfirm}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("account.batchDeleteConfirm")}</AlertDialogTitle>
            <AlertDialogDescription>
              {t("account.batchDeleteConfirmDesc", { count: selectedCount })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isBatchDeleting}>{t("common.cancel")}</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDelete}
              disabled={isBatchDeleting}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {isBatchDeleting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {t("common.delete")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  )
}
