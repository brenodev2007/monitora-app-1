import { Database, Server, Globe } from "lucide-react";
import { StatusBadge } from "./StatusBadge";
import { cn, tempoRelativo } from "@/lib/utils";
import type { Servico, TipoServico } from "@/types/monitora";

const icones: Record<TipoServico, React.ElementType> = {
  api: Globe,
  servidor: Server,
  backup: Database,
};

export function ListaServicos({
  servicos,
  selecionadoId,
  onSelecionar,
}: {
  servicos: Servico[];
  selecionadoId: string | null;
  onSelecionar: (id: string) => void;
}) {
  if (servicos.length === 0) {
    return <p className="p-4 text-sm text-[var(--color-ink-dim)]">Sincronizando serviços…</p>;
  }

  return (
    <ul className="flex flex-col gap-1.5 overflow-y-auto p-2 scrollbar-fina">
      {servicos.map((servico) => {
        const Icone = icones[servico.tipo];
        const selecionado = servico.id === selecionadoId;
        return (
          <li key={servico.id}>
            <button
              onClick={() => onSelecionar(servico.id)}
              className={cn(
                "flex w-full flex-col gap-1.5 rounded-xl border px-3 py-2.5 text-left transition",
                selecionado
                  ? "border-[var(--color-brand)]/50 bg-[var(--color-surface-2)]"
                  : "border-transparent hover:bg-[var(--color-surface-2)]",
              )}
            >
              <div className="flex items-center justify-between gap-2">
                <div className="flex items-center gap-2 truncate">
                  <Icone size={14} className="shrink-0 text-[var(--color-ink-dim)]" />
                  <span className="truncate text-sm font-medium">{servico.nome}</span>
                </div>
                <StatusBadge status={servico.status} pulsar />
              </div>
              <div className="flex items-center justify-between text-xs text-[var(--color-ink-dim)]">
                <span className="truncate">{servico.cliente_nome}</span>
                <span className="font-mono-tab shrink-0">{tempoRelativo(servico.ultimo_heartbeat_em)}</span>
              </div>
            </button>
          </li>
        );
      })}
    </ul>
  );
}
