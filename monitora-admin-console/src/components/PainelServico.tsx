import { useMemo } from "react";
import { Area, AreaChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { StatusBadge } from "./StatusBadge";
import { formatarHora, tempoRelativo } from "@/lib/utils";
import type { Servico } from "@/types/monitora";

const rotulosTipo: Record<Servico["tipo"], string> = {
  api: "API",
  servidor: "Servidor",
  backup: "Rotina de backup",
};

function metricaPrincipal(servico: Servico) {
  switch (servico.tipo) {
    case "api":
      return { chave: "latencia_ms" as const, rotulo: "Latência (ms)" };
    case "servidor":
      return { chave: "cpu_pct" as const, rotulo: "CPU (%)" };
    case "backup":
      return { chave: "tamanho_backup_mb" as const, rotulo: "Tamanho do backup (MB)" };
  }
}

export function PainelServico({ servico }: { servico: Servico | null }) {
  const metrica = servico ? metricaPrincipal(servico) : null;

  const dados = useMemo(() => {
    if (!servico || !metrica) return [];
    return servico.historico.map((hb) => ({
      hora: formatarHora(hb.recebido_em),
      valor: hb.metricas[metrica.chave] ?? null,
      status: hb.status,
    }));
  }, [servico, metrica]);

  if (!servico) {
    return (
      <div className="flex h-full flex-1 items-center justify-center text-sm text-[var(--color-ink-dim)]">
        Selecione um serviço para ver o histórico
      </div>
    );
  }

  return (
    <div className="flex flex-1 flex-col gap-6 overflow-y-auto p-6 scrollbar-fina">
      <div className="flex items-start justify-between">
        <div>
          <p className="text-xs uppercase tracking-wide text-[var(--color-ink-dim)]">
            {servico.cliente_nome} · {rotulosTipo[servico.tipo]}
          </p>
          <h2 className="mt-1 text-lg font-semibold">{servico.nome}</h2>
        </div>
        <StatusBadge status={servico.status} />
      </div>

      <div className="grid grid-cols-3 gap-3">
        <CartaoMetrica rotulo="Último heartbeat" valor={tempoRelativo(servico.ultimo_heartbeat_em)} />
        <CartaoMetrica rotulo="Intervalo esperado" valor={`${servico.intervalo_esperado_min} min`} />
        <CartaoMetrica rotulo="Registros no histórico" valor={String(servico.historico.length)} />
      </div>

      <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4">
        <p className="mb-3 text-xs font-medium text-[var(--color-ink-dim)]">{metrica?.rotulo}</p>
        <div className="h-56">
          {dados.length > 1 ? (
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={dados} margin={{ top: 4, right: 8, left: -20, bottom: 0 }}>
                <defs>
                  <linearGradient id="gradienteMetrica" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="var(--color-brand)" stopOpacity={0.35} />
                    <stop offset="100%" stopColor="var(--color-brand)" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <XAxis dataKey="hora" stroke="var(--color-ink-dim)" fontSize={11} tickLine={false} axisLine={false} />
                <YAxis stroke="var(--color-ink-dim)" fontSize={11} tickLine={false} axisLine={false} width={40} />
                <Tooltip
                  contentStyle={{
                    background: "var(--color-surface-2)",
                    border: "1px solid var(--color-border)",
                    borderRadius: 8,
                    fontSize: 12,
                  }}
                />
                <Area
                  type="monotone"
                  dataKey="valor"
                  stroke="var(--color-brand)"
                  fill="url(#gradienteMetrica)"
                  strokeWidth={2}
                  connectNulls
                />
              </AreaChart>
            </ResponsiveContainer>
          ) : (
            <div className="flex h-full items-center justify-center text-xs text-[var(--color-ink-dim)]">
              Aguardando heartbeats suficientes para o gráfico…
            </div>
          )}
        </div>
      </div>

      <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)]">
        <p className="border-b border-[var(--color-border)] px-4 py-3 text-xs font-medium text-[var(--color-ink-dim)]">
          Últimos heartbeats
        </p>
        <ul className="max-h-52 overflow-y-auto scrollbar-fina">
          {[...servico.historico]
            .slice(-10)
            .reverse()
            .map((hb) => (
              <li
                key={hb.id}
                className="flex items-center justify-between border-b border-[var(--color-border)] px-4 py-2 text-xs last:border-0"
              >
                <span className="font-mono-tab text-[var(--color-ink-dim)]">{formatarHora(hb.recebido_em)}</span>
                <StatusBadge status={hb.status} />
              </li>
            ))}
        </ul>
      </div>
    </div>
  );
}

function CartaoMetrica({ rotulo, valor }: { rotulo: string; valor: string }) {
  return (
    <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-3">
      <p className="text-[11px] text-[var(--color-ink-dim)]">{rotulo}</p>
      <p className="mt-1 font-mono-tab text-sm font-medium">{valor}</p>
    </div>
  );
}
