use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TipoServico {
    Api,
    Servidor,
    Backup,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StatusServico {
    Ok,
    Atraso,
    Falha,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metricas {
    pub cpu_pct: Option<f32>,
    pub ram_pct: Option<f32>,
    pub tamanho_backup_mb: Option<f32>,
    pub latencia_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub id: String,
    pub servico_id: String,
    pub recebido_em: String, // ISO 8601
    pub status: StatusServico,
    pub metricas: Metricas,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Servico {
    pub id: String,
    pub usuario_id: String,
    pub cliente_nome: String,
    pub nome: String,
    pub tipo: TipoServico,
    pub intervalo_esperado_min: u32,
    pub status: StatusServico,
    pub ultimo_heartbeat_em: Option<String>,
    pub historico: Vec<Heartbeat>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CanalAlerta {
    Email,
    Whatsapp,
    Discord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alerta {
    pub id: String,
    pub servico_id: String,
    pub servico_nome: String,
    pub tipo: String,
    pub canal: CanalAlerta,
    pub disparado_em: String,
    pub resolvido: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusConexao {
    pub conectado: bool,
    pub ultima_tentativa_em: String,
}
