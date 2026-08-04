use crate::models::{Alerta, Servico};
use std::sync::Mutex;

/// Estado central mantido no processo principal (Rust), nunca exposto
/// diretamente ao renderer. A UI só recebe cópias via commands/eventos.
/// Isso evita duplicação de conexão/estado entre processos — o mesmo
/// problema do bug do socket-bridge.ts no ti-chamados.
pub struct Estado {
    pub servicos: Mutex<Vec<Servico>>,
    pub alertas: Mutex<Vec<Alerta>>,
    pub conectado: Mutex<bool>,
    pub autenticado: Mutex<bool>,
}

impl Default for Estado {
    fn default() -> Self {
        Self {
            servicos: Mutex::new(Vec::new()),
            alertas: Mutex::new(Vec::new()),
            conectado: Mutex::new(false),
            autenticado: Mutex::new(false),
        }
    }
}
