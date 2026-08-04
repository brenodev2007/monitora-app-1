import { useState } from "react";
import { Radio, WifiOff, PlugZap, Plug, Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { bridge } from "@/lib/bridge";

export function BarraTopo({ conectado, totalAlertasAbertos }: { conectado: boolean; totalAlertasAbertos: number }) {
  const [url, setUrl] = useState("ws://localhost:8080");
  const [carregando, setCarregando] = useState(false);

  async function conectar() {
    setCarregando(true);
    try {
      await bridge.conectarWebSocket(url);
    } catch {
      // a UI já recebe erro via evento de conexão, então só encerramos o estado local
    } finally {
      setCarregando(false);
    }
  }

  async function desconectar() {
    setCarregando(true);
    try {
      await bridge.desconectarWebSocket();
    } finally {
      setCarregando(false);
    }
  }

  return (
    <header className="flex h-16 items-center justify-between border-b border-[var(--color-border)] bg-[var(--color-surface)] px-5">
      <div className="flex items-center gap-2.5">
        <span className="text-sm font-semibold tracking-tight">MONITORA+</span>
        <span className="text-xs text-[var(--color-ink-dim)]">Painel administrativo</span>
      </div>

      <div className="flex flex-1 items-center justify-end gap-3">
        <div className="flex items-center gap-2 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-2)] px-2 py-1.5">
          <PlugZap size={14} className="text-[var(--color-brand)]" />
          <input
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder="wss://seu-servidor/ws"
            className="w-72 rounded-md border border-transparent bg-transparent px-2 py-1 text-xs text-[var(--color-ink)] outline-none placeholder:text-[var(--color-ink-dim)]"
          />
        </div>

        {conectado ? (
          <button
            onClick={desconectar}
            disabled={carregando}
            className="flex items-center gap-1.5 rounded-lg border border-[var(--color-border)] px-3 py-2 text-xs font-medium text-[var(--color-ink)] transition hover:border-[var(--color-brand)]/50"
          >
            {carregando ? <Loader2 size={13} className="animate-spin" /> : <Plug size={13} />}
            Desconectar
          </button>
        ) : (
          <button
            onClick={conectar}
            disabled={carregando}
            className="flex items-center gap-1.5 rounded-lg bg-[var(--color-brand)] px-3 py-2 text-xs font-medium text-[#04211c] transition hover:brightness-110 disabled:opacity-70"
          >
            {carregando ? <Loader2 size={13} className="animate-spin" /> : <Plug size={13} />}
            Conectar
          </button>
        )}

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
          {conectado ? "Conectado em tempo real" : "Desconectado"}
        </div>
      </div>
    </header>
  );
}
