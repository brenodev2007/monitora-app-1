import { Mail, MessageCircle, Check } from "lucide-react";
import { cn, tempoRelativo } from "@/lib/utils";
import type { Alerta, CanalAlerta } from "@/types/monitora";

const iconesCanal: Record<CanalAlerta, React.ElementType> = {
  email: Mail,
  whatsapp: MessageCircle,
  discord: MessageCircle,
};

const rotulosCanal: Record<CanalAlerta, string> = {
  email: "E-mail",
  whatsapp: "WhatsApp",
  discord: "Discord",
};

export function PainelAlertas({
  alertas,
  onResolver,
}: {
  alertas: Alerta[];
  onResolver: (id: string) => void;
}) {
  return (
    <aside className="flex w-80 flex-col border-l border-[var(--color-border)] bg-[var(--color-surface)]">
      <div className="border-b border-[var(--color-border)] px-4 py-3">
        <p className="text-sm font-semibold">Alertas</p>
        <p className="text-xs text-[var(--color-ink-dim)]">e-mail · WhatsApp · Discord</p>
      </div>

      {alertas.length === 0 ? (
        <p className="p-4 text-xs text-[var(--color-ink-dim)]">Nenhum alerta disparado ainda.</p>
      ) : (
        <ul className="flex-1 overflow-y-auto scrollbar-fina">
          {alertas.map((alerta) => {
            const Icone = iconesCanal[alerta.canal];
            return (
              <li
                key={alerta.id}
                className={cn(
                  "border-b border-[var(--color-border)] px-4 py-3",
                  alerta.resolvido && "opacity-50",
                )}
              >
                <div className="mb-1 flex items-center justify-between">
                  <span className="flex items-center gap-1.5 text-[11px] text-[var(--color-ink-dim)]">
                    <Icone size={12} />
                    {rotulosCanal[alerta.canal]}
                  </span>
                  <span className="font-mono-tab text-[11px] text-[var(--color-ink-dim)]">
                    {tempoRelativo(alerta.disparado_em)}
                  </span>
                </div>
                <p className="text-sm font-medium leading-snug">{alerta.servico_nome}</p>
                <p className="text-xs text-[var(--color-ink-dim)]">
                  {alerta.tipo === "falha_reportada" ? "Falha reportada pelo agente" : "Heartbeat ausente"}
                </p>
                {!alerta.resolvido && (
                  <button
                    onClick={() => onResolver(alerta.id)}
                    className="mt-2 flex items-center gap-1 rounded-md border border-[var(--color-border)] px-2 py-1 text-[11px] text-[var(--color-ink-dim)] transition hover:border-[var(--color-brand)]/50 hover:text-[var(--color-ink)]"
                  >
                    <Check size={11} /> Marcar como tratado
                  </button>
                )}
              </li>
            );
          })}
        </ul>
      )}
    </aside>
  );
}
