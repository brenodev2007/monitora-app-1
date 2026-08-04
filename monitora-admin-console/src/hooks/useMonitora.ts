import { useCallback, useEffect, useRef, useState } from "react";
import { bridge } from "@/lib/bridge";
import type { Alerta, Servico } from "@/types/monitora";

/**
 * Cache do último estado conhecido, do lado do renderer.
 * A fonte da verdade continua sendo o processo principal — este hook só
 * espelha o que chega via evento, para a UI nunca ficar "em branco"
 * entre reconexões.
 */
export function useMonitora() {
  const [servicos, setServicos] = useState<Servico[]>([]);
  const [alertas, setAlertas] = useState<Alerta[]>([]);
  const [conectado, setConectado] = useState(false);
  const inicializado = useRef(false);

  useEffect(() => {
    if (inicializado.current) return;
    inicializado.current = true;

    const unlisteners: Promise<() => void>[] = [];

    unlisteners.push(
      bridge.onConexaoStatus((status) => setConectado(status.conectado)),
    );

    unlisteners.push(
      bridge.onServicosSincronizados((lista) => setServicos(lista)),
    );

    unlisteners.push(
      bridge.onHeartbeatRecebido((servicoId, hb) => {
        setServicos((atual) =>
          atual.map((s) =>
            s.id !== servicoId
              ? s
              : {
                  ...s,
                  status: hb.status,
                  ultimo_heartbeat_em: hb.status !== "falha" ? hb.recebido_em : s.ultimo_heartbeat_em,
                  historico: [...s.historico.slice(-39), hb],
                },
          ),
        );
      }),
    );

    unlisteners.push(
      bridge.onAlertaNovo((alerta) => setAlertas((atual) => [alerta, ...atual].slice(0, 100))),
    );

    bridge.listarServicos().then(setServicos).catch(() => {});
    bridge.listarAlertas().then(setAlertas).catch(() => {});
    bridge.statusConexao().then(setConectado).catch(() => {});

    return () => {
      unlisteners.forEach((p) => p.then((fn) => fn()));
    };
  }, []);

  const resolverAlerta = useCallback(async (id: string) => {
    const ok = await bridge.resolverAlerta(id);
    if (ok) {
      setAlertas((atual) => atual.map((a) => (a.id === id ? { ...a, resolvido: true } : a)));
    }
  }, []);

  return { servicos, alertas, conectado, resolverAlerta };
}
