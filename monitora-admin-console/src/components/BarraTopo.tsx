import { Radio, WifiOff } from "lucide-react";
import { cn } from "@/lib/utils";

export function BarraTopo({ conectado, totalAlertasAbertos }: { conectado: boolean; totalAlertasAbertos: number }) {
  return (
    <header className="flex h-14 items-center justify-between border-b border-[var(--color-border)] bg-[var(--color-surface)] px-5">
      <div className="flex items-center gap-2.5">
        <span className="text-sm font-semibold tracking-tight">MONITORA+</span>
        <span className="text-xs text-[var(--color-ink-dim)]">Painel administrativo</span>
      </div>

      <div className="flex items-center gap-4">
        {totalAlertasAbertos > 0 && (
          <span className="rounded-full bg-[var(--color-falha)]/15 px-2.5 py-1 text-xs font-medium text-[var(--color-falha)]">
            {totalAlertasAbertos} alerta{totalAlertasAbertos > 1 ? "s" : ""} em aberto
          </span>
        )}
        <div
          className={cn(
            "flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium",
            conectado ? "text-[var(--color-ok)]" : "text-[var(--color-ink-dim)]",
          )}
        >
          {conectado ? <Radio size={13} /> : <WifiOff size={13} />}
          {conectado ? "Conectado em tempo real" : "Conectando…"}
        </div>
      </div>
    </header>
  );
}
