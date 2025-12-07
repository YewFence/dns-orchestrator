import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { ArrowLeft, FileText, Globe, Lock, MapPin, Wrench } from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import { DnsLookup } from "./DnsLookup"
import { IpLookup } from "./IpLookup"
import { SslCheck } from "./SslCheck"
import { WhoisLookup } from "./WhoisLookup"

interface ToolboxPageProps {
  onBack: () => void
}

export function ToolboxPage({ onBack }: ToolboxPageProps) {
  const { t } = useTranslation()
  const [activeTab, setActiveTab] = useState("dns")

  return (
    <div className="flex flex-1 flex-col min-h-0 overflow-hidden">
      {/* Header */}
      <div className="flex items-center gap-3 border-b bg-background px-4 py-3 sm:gap-4 sm:px-6 sm:py-4">
        <Button variant="ghost" size="icon" onClick={onBack}>
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <div className="flex items-center gap-2">
          <Wrench className="h-5 w-5 text-primary" />
          <h2 className="font-semibold text-xl">{t("toolbox.title")}</h2>
        </div>
      </div>

      {/* Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="flex flex-1 flex-col min-h-0">
        <div className="overflow-x-auto border-b px-4 sm:px-6">
          <TabsList className="h-auto flex-nowrap gap-1 bg-transparent p-0">
            <TabsTrigger
              value="dns"
              className="gap-1.5 rounded-none border-b-2 border-transparent px-3 py-2.5 data-[state=active]:border-primary data-[state=active]:bg-transparent data-[state=active]:shadow-none"
            >
              <Globe className="h-4 w-4" />
              DNS
            </TabsTrigger>
            <TabsTrigger
              value="whois"
              className="gap-1.5 rounded-none border-b-2 border-transparent px-3 py-2.5 data-[state=active]:border-primary data-[state=active]:bg-transparent data-[state=active]:shadow-none"
            >
              <FileText className="h-4 w-4" />
              WHOIS
            </TabsTrigger>
            <TabsTrigger
              value="ssl"
              className="gap-1.5 rounded-none border-b-2 border-transparent px-3 py-2.5 data-[state=active]:border-primary data-[state=active]:bg-transparent data-[state=active]:shadow-none"
            >
              <Lock className="h-4 w-4" />
              SSL
            </TabsTrigger>
            <TabsTrigger
              value="ip"
              className="gap-1.5 rounded-none border-b-2 border-transparent px-3 py-2.5 data-[state=active]:border-primary data-[state=active]:bg-transparent data-[state=active]:shadow-none"
            >
              <MapPin className="h-4 w-4" />
              IP
            </TabsTrigger>
          </TabsList>
        </div>

        <ScrollArea className="flex-1 min-h-0">
          <div className="mx-auto max-w-4xl p-4 sm:p-6">
            <TabsContent value="dns" className="mt-0">
              <DnsLookup />
            </TabsContent>
            <TabsContent value="whois" className="mt-0">
              <WhoisLookup />
            </TabsContent>
            <TabsContent value="ssl" className="mt-0">
              <SslCheck />
            </TabsContent>
            <TabsContent value="ip" className="mt-0">
              <IpLookup />
            </TabsContent>
          </div>
        </ScrollArea>
      </Tabs>
    </div>
  )
}
