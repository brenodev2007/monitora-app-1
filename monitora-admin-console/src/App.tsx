import { useMemo, useState } from "react";
import { TelaLogin } from "@/components/TelaLogin";
import { BarraTopo } from "@/components/BarraTopo";
import { ListaServicos } from "@/components/ListaServicos";
import { PainelServico } from "@/components/PainelServico";
import { PainelAlertas } from "@/components/PainelAlertas";
import { useMonitora } from "@/hooks/useMonitora";

export default function App() {
  const [autenticado, setAutenticado] = useState(false);
  const [servicoSelecionadoId, setServicoSelecionadoId] = useState<string | null>(null);
  const { servicos, alertas, conectado, resolverAlerta } = useMonitora();

  const servicoSelecionado = useMemo(
    () => servicos.find((s) => s.id === servicoSelecionadoId) ?? servicos[0] ?? null,
    [servicos, servicoSelecionadoId],
  );

  const alertasAbertos = alertas.filter((a) => !a.resolvido).length;

  if (!autenticado) {
    return <TelaLogin onEntrar={() => setAutenticado(true)} />;
  }

  return (
    <div className="flex h-screen flex-col bg-[var(--color-base)] text-[var(--color-ink)]">
      <BarraTopo conectado={conectado} totalAlertasAbertos={alertasAbertos} />
      <div className="flex flex-1 overflow-hidden">
        <nav className="w-72 border-r border-[var(--color-border)] bg-[var(--color-surface)]">
          <ListaServicos
            servicos={servicos}
            selecionadoId={servicoSelecionado?.id ?? null}
            onSelecionar={setServicoSelecionadoId}
          />
        </nav>
        <PainelServico servico={servicoSelecionado} />
        <PainelAlertas alertas={alertas} onResolver={resolverAlerta} />
      </div>
    </div>
  );
}
