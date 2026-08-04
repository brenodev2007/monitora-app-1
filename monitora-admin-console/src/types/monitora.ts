export type TipoServico = "api" | "servidor" | "backup";
export type StatusServico = "ok" | "atraso" | "falha";
export type CanalAlerta = "email" | "whatsapp" | "discord";

export interface Metricas {
  cpu_pct: number | null;
  ram_pct: number | null;
  tamanho_backup_mb: number | null;
  latencia_ms: number | null;
}

export interface Heartbeat {
  id: string;
  servico_id: string;
  recebido_em: string;
  status: StatusServico;
  metricas: Metricas;
}

export interface Servico {
  id: string;
  usuario_id: string;
  cliente_nome: string;
  nome: string;
  tipo: TipoServico;
  intervalo_esperado_min: number;
  status: StatusServico;
  ultimo_heartbeat_em: string | null;
  historico: Heartbeat[];
}

export interface Alerta {
  id: string;
  servico_id: string;
  servico_nome: string;
  tipo: string;
  canal: CanalAlerta;
  disparado_em: string;
  resolvido: boolean;
}

export interface StatusConexao {
  conectado: boolean;
  ultima_tentativa_em: string;
}
