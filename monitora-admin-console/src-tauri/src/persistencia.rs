use crate::models::ConexaoSalva;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

pub fn caminho_arquivo(app: &tauri::AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("conexoes.json")
}

pub fn carregar_conexoes(app: &tauri::AppHandle) -> Vec<ConexaoSalva> {
    let caminho = caminho_arquivo(app);
    log::info!("[PERSIST] Carregando conexões de: {}", caminho.display());

    if !caminho.exists() {
        log::info!("[PERSIST] Arquivo não encontrado — retornando conexão padrão (localhost:8080).");
        return vec![ConexaoSalva {
            id: "conn-local".to_string(),
            nome: "Local".to_string(),
            url: "ws://localhost:8080".to_string(),
        }];
    }

    match fs::read_to_string(&caminho) {
        Ok(texto) => match serde_json::from_str::<Vec<ConexaoSalva>>(&texto) {
            Ok(lista) => {
                log::info!("[PERSIST] {} conexão(ões) carregada(s) do disco.", lista.len());
                lista
            }
            Err(err) => {
                log::error!("[PERSIST] Erro ao parsear conexoes.json: {} — usando padrão.", err);
                vec![ConexaoSalva {
                    id: "conn-local".to_string(),
                    nome: "Local".to_string(),
                    url: "ws://localhost:8080".to_string(),
                }]
            }
        },
        Err(err) => {
            log::error!("[PERSIST] Erro ao ler conexoes.json: {} — usando padrão.", err);
            vec![ConexaoSalva {
                id: "conn-local".to_string(),
                nome: "Local".to_string(),
                url: "ws://localhost:8080".to_string(),
            }]
        }
    }
}

pub fn salvar_conexoes(app: &tauri::AppHandle, conexoes: &[ConexaoSalva]) -> Result<(), String> {
    let caminho = caminho_arquivo(app);
    log::info!("[PERSIST] Salvando {} conexão(ões) em: {}", conexoes.len(), caminho.display());

    if let Some(pai) = caminho.parent() {
        if let Err(err) = fs::create_dir_all(pai) {
            log::warn!("[PERSIST] Não foi possível criar diretório pai '{}': {}", pai.display(), err);
        }
    }

    let texto = serde_json::to_string_pretty(conexoes).map_err(|err| {
        let msg = err.to_string();
        log::error!("[PERSIST] Erro ao serializar conexões: {}", msg);
        msg
    })?;

    fs::write(&caminho, texto).map_err(|err| {
        let msg = err.to_string();
        log::error!("[PERSIST] Erro ao escrever arquivo '{}': {}", caminho.display(), msg);
        msg
    })?;

    log::info!("[PERSIST] Conexões salvas com sucesso.");
    Ok(())
}
