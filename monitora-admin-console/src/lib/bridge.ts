import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Alerta, Heartbeat, Servico, StatusConexao } from "@/types/monitora";

/**
 * Ponto único de contato com o processo principal (Rust).
 * A UI nunca fala diretamente com a "conexão" — ela chama comandos e
 * escuta eventos, exatamente como descrito na seção 6.4 da especificação
 * (WebSocket mantido no main process, repassado por IPC).
 */
export const bridge = {
  autenticar: (email: string, senha: string) =>
    invoke<boolean>("autenticar", { email, senha }),

  listarServicos: () => invoke<Servico[]>("listar_servicos"),

  listarAlertas: () => invoke<Alerta[]>("listar_alertas"),

  resolverAlerta: (id: string) => invoke<boolean>("resolver_alerta", { id }),

  statusConexao: () => invoke<StatusConexao>("status_conexao"),

  onConexaoStatus(cb: (status: StatusConexao) => void): Promise<UnlistenFn> {
    return listen<StatusConexao>("conexao:status", (e) => cb(e.payload));
  },

  onServicosSincronizados(cb: (servicos: Servico[]) => void): Promise<UnlistenFn> {
    return listen<Servico[]>("servicos:sincronizados", (e) => cb(e.payload));
  },

  onHeartbeatRecebido(cb: (servicoId: string, hb: Heartbeat) => void): Promise<UnlistenFn> {
    return listen<[string, Heartbeat]>("heartbeat:recebido", (e) =>
      cb(e.payload[0], e.payload[1]),
    );
  },

  onAlertaNovo(cb: (alerta: Alerta) => void): Promise<UnlistenFn> {
    return listen<Alerta>("alerta:novo", (e) => cb(e.payload));
  },
};
