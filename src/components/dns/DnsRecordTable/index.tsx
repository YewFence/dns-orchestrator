import { Loader2 } from "lucide-react"
import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import { useTranslation } from "react-i18next"
import { useDebouncedCallback } from "use-debounce"
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
import { TIMING } from "@/constants"
import { useIsMobile } from "@/hooks/useMediaQuery"
import { useDnsStore } from "@/stores"
import type { DnsRecord } from "@/types"
import { DnsBatchActionBar } from "../DnsBatchActionBar"
import { DnsRecordForm } from "../DnsRecordForm"
import { DnsTableToolbar } from "../DnsTableToolbar"
import { useDnsTableSort } from "../useDnsTableSort"
import { DesktopTable } from "./DesktopTable"
import { MobileCardList } from "./MobileCardList"
import type { DnsRecordTableProps } from "./types"

export function DnsRecordTable({ accountId, domainId, supportsProxy }: DnsRecordTableProps) {
  const { t } = useTranslation()
  const isMobile = useIsMobile()

  // 使用 useShallow 优化 store 订阅粒度
  const {
    records,
    isLoading,
    isLoadingMore,
    isDeleting,
    hasMore,
    totalCount,
    currentDomainId,
    keyword,
    recordType,
    selectedRecordIds,
    isSelectMode,
    isBatchDeleting,
  } = useDnsStore(
    useShallow((state) => ({
      records: state.records,
      isLoading: state.isLoading,
      isLoadingMore: state.isLoadingMore,
      isDeleting: state.isDeleting,
      hasMore: state.hasMore,
      totalCount: state.totalCount,
      currentDomainId: state.currentDomainId,
      keyword: state.keyword,
      recordType: state.recordType,
      selectedRecordIds: state.selectedRecordIds,
      isSelectMode: state.isSelectMode,
      isBatchDeleting: state.isBatchDeleting,
    }))
  )

  // actions 单独获取（函数引用稳定，不需要 shallow）
  const setKeyword = useDnsStore((state) => state.setKeyword)
  const setRecordType = useDnsStore((state) => state.setRecordType)
  const fetchRecords = useDnsStore((state) => state.fetchRecords)
  const fetchMoreRecords = useDnsStore((state) => state.fetchMoreRecords)
  const deleteRecord = useDnsStore((state) => state.deleteRecord)
  const toggleSelectMode = useDnsStore((state) => state.toggleSelectMode)
  const toggleRecordSelection = useDnsStore((state) => state.toggleRecordSelection)
  const selectAllRecords = useDnsStore((state) => state.selectAllRecords)
  const clearSelection = useDnsStore((state) => state.clearSelection)
  const batchDeleteRecords = useDnsStore((state) => state.batchDeleteRecords)

  const [showAddForm, setShowAddForm] = useState(false)
  const [editingRecord, setEditingRecord] = useState<DnsRecord | null>(null)
  const [deletingRecord, setDeletingRecord] = useState<DnsRecord | null>(null)
  const [showBatchDeleteConfirm, setShowBatchDeleteConfirm] = useState(false)
  const sentinelRef = useRef<HTMLElement | null>(null)
  const scrollContainerRef = useRef<HTMLDivElement>(null)

  // 使用排序 hook
  const { sortField, sortDirection, sortedRecords, handleSort } = useDnsTableSort(records)

  // 统一的 ref callback
  const setSentinelRef = useCallback((node: HTMLElement | null) => {
    sentinelRef.current = node
  }, [])

  // 防抖搜索
  const debouncedSearch = useDebouncedCallback((searchKeyword: string) => {
    fetchRecords(accountId, domainId, searchKeyword, recordType)
  }, TIMING.DEBOUNCE_DELAY)

  // 处理搜索输入变化
  const handleSearchChange = useCallback(
    (value: string) => {
      setKeyword(value)
      debouncedSearch(value)
    },
    [setKeyword, debouncedSearch]
  )

  // 处理类型选择变化
  const handleTypeChange = useCallback(
    (type: string) => {
      const newType = recordType === type ? "" : type
      setRecordType(newType)
      fetchRecords(accountId, domainId, keyword, newType)
    },
    [recordType, setRecordType, fetchRecords, accountId, domainId, keyword]
  )

  // 清除所有筛选
  const clearFilters = useCallback(() => {
    setKeyword("")
    setRecordType("")
    fetchRecords(accountId, domainId, "", "")
  }, [setKeyword, setRecordType, fetchRecords, accountId, domainId])

  useEffect(() => {
    fetchRecords(accountId, domainId)
  }, [accountId, domainId, fetchRecords])

  // 无限滚动 IntersectionObserver
  const handleObserver = useCallback(
    (entries: IntersectionObserverEntry[]) => {
      const [entry] = entries
      if (entry.isIntersecting && hasMore && !isLoadingMore) {
        fetchMoreRecords(accountId, domainId)
      }
    },
    [hasMore, isLoadingMore, fetchMoreRecords, accountId, domainId]
  )

  useEffect(() => {
    const sentinel = sentinelRef.current
    const scrollContainer = scrollContainerRef.current
    if (!(sentinel && scrollContainer)) return

    const observer = new IntersectionObserver(handleObserver, {
      root: scrollContainer,
      rootMargin: "100px",
    })
    observer.observe(sentinel)

    return () => observer.disconnect()
  }, [handleObserver])

  const hasActiveFilters = useMemo(() => !!(keyword || recordType), [keyword, recordType])

  const handleDelete = useCallback((record: DnsRecord) => setDeletingRecord(record), [])
  const handleEdit = useCallback((record: DnsRecord) => {
    setEditingRecord(record)
    setShowAddForm(true)
  }, [])
  const handleFormClose = useCallback(() => {
    setShowAddForm(false)
    setEditingRecord(null)
  }, [])

  const handleRefresh = useCallback(() => {
    fetchRecords(accountId, domainId, keyword, recordType)
  }, [fetchRecords, accountId, domainId, keyword, recordType])

  const confirmDelete = async () => {
    if (!deletingRecord) return
    await deleteRecord(accountId, deletingRecord.id, domainId)
    setDeletingRecord(null)
  }

  // 只有域名切换时才显示全屏 loading
  const isInitialLoading = isLoading && currentDomainId !== domainId

  if (isInitialLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      {/* Toolbar */}
      <DnsTableToolbar
        totalCount={totalCount}
        isLoading={isLoading}
        keyword={keyword}
        recordType={recordType}
        hasRecords={records.length > 0}
        isSelectMode={isSelectMode}
        onSearchChange={handleSearchChange}
        onTypeChange={handleTypeChange}
        onClearFilters={clearFilters}
        onRefresh={handleRefresh}
        onToggleSelectMode={toggleSelectMode}
        onAdd={() => setShowAddForm(true)}
      />

      {/* Table / Card List */}
      <div ref={scrollContainerRef} className="min-h-0 flex-1 overflow-auto">
        {isMobile ? (
          <MobileCardList
            records={sortedRecords}
            isLoading={isLoading}
            isLoadingMore={isLoadingMore}
            isDeleting={isDeleting}
            isSelectMode={isSelectMode}
            selectedRecordIds={selectedRecordIds}
            hasActiveFilters={hasActiveFilters}
            supportsProxy={supportsProxy}
            onEdit={handleEdit}
            onDelete={handleDelete}
            onToggleSelect={toggleRecordSelection}
            onSelectAll={selectAllRecords}
            onClearSelection={clearSelection}
            setSentinelRef={setSentinelRef}
          />
        ) : (
          <DesktopTable
            records={sortedRecords}
            isLoading={isLoading}
            isLoadingMore={isLoadingMore}
            isDeleting={isDeleting}
            isSelectMode={isSelectMode}
            selectedRecordIds={selectedRecordIds}
            hasActiveFilters={hasActiveFilters}
            supportsProxy={supportsProxy}
            sortField={sortField}
            sortDirection={sortDirection}
            onSort={handleSort}
            onEdit={handleEdit}
            onDelete={handleDelete}
            onToggleSelect={toggleRecordSelection}
            onSelectAll={selectAllRecords}
            onClearSelection={clearSelection}
            setSentinelRef={setSentinelRef}
          />
        )}
      </div>

      {/* Add/Edit Form Dialog */}
      {showAddForm && (
        <DnsRecordForm
          accountId={accountId}
          domainId={domainId}
          record={editingRecord}
          onClose={handleFormClose}
          supportsProxy={supportsProxy}
        />
      )}

      {/* Delete Confirmation Dialog */}
      <AlertDialog
        open={!!deletingRecord}
        onOpenChange={(open) => !open && setDeletingRecord(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("dns.deleteConfirm")}</AlertDialogTitle>
            <AlertDialogDescription>
              {t("dns.deleteConfirmDesc", { name: deletingRecord?.name })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isDeleting}>{t("common.cancel")}</AlertDialogCancel>
            <AlertDialogAction
              onClick={confirmDelete}
              disabled={isDeleting}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {isDeleting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {t("common.delete")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Batch Delete Confirmation Dialog */}
      <AlertDialog open={showBatchDeleteConfirm} onOpenChange={setShowBatchDeleteConfirm}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("dns.batchDeleteConfirm")}</AlertDialogTitle>
            <AlertDialogDescription>
              {t("dns.batchDeleteConfirmDesc", { count: selectedRecordIds.size })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isBatchDeleting}>{t("common.cancel")}</AlertDialogCancel>
            <AlertDialogAction
              onClick={async () => {
                setShowBatchDeleteConfirm(false)
                await batchDeleteRecords(accountId, domainId)
              }}
              disabled={isBatchDeleting}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {isBatchDeleting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {t("common.delete")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Batch Action Bar */}
      {isSelectMode && (
        <DnsBatchActionBar
          selectedCount={selectedRecordIds.size}
          isDeleting={isBatchDeleting}
          onClearSelection={clearSelection}
          onDelete={() => setShowBatchDeleteConfirm(true)}
        />
      )}
    </div>
  )
}
