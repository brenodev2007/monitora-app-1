use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TipoServico {
    Api,
    Servidor,
    Backup,
}

impl TipoServico {
    pub fn as_db(self) -> &'static str {
        match self {
            Self::Api => "api",
            Self::Servidor => "servidor",
            Self::Backup => "backup",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "servidor" => Self::Servidor,
            "backup" => Self::Backup,
            _ => Self::Api,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StatusServico {
    Ok,
    Atraso,
    Falha,
}

impl StatusServico {
    pub fn as_db(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Atraso => "atraso",
            Self::Falha => "falha",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "atraso" => Self::Atraso,
            "falha" => Self::Falha,
            _ => Self::Ok,
        }
    }
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

impl CanalAlerta {
    pub fn as_db(self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Whatsapp => "whatsapp",
            Self::Discord => "discord",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "whatsapp" => Self::Whatsapp,
            "discord" => Self::Discord,
            _ => Self::Email,
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConexaoSalva {
    pub id: String,
    pub nome: String,
    pub url: String,
}
