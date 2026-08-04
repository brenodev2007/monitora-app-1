use crate::models::ConexaoSalva;
use std::fs;
use std::path::PathBuf;
use tauri::path::BaseDirectory;
use tauri::Manager;

pub fn caminho_arquivo(app: &tauri::AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."))
        .join("conexoes.json")
}

pub fn carregar_conexoes(app: &tauri::AppHandle) -> Vec<ConexaoSalva> {
    let caminho = caminho_arquivo(app);
    if !caminho.exists() {
        return vec![ConexaoSalva {
            id: "conn-local".to_string(),
            nome: "Local".to_string(),
            url: "ws://localhost:8080".to_string(),
        }];
    }

    match fs::read_to_string(&caminho) {
        Ok(texto) => serde_json::from_str::<Vec<ConexaoSalva>>(&texto).unwrap_or_default(),
        Err(_) => vec![ConexaoSalva {
            id: "conn-local".to_string(),
            nome: "Local".to_string(),
            url: "ws://localhost:8080".to_string(),
        }],
    }
}

pub fn salvar_conexoes(app: &tauri::AppHandle, conexoes: &[ConexaoSalva]) -> Result<(), String> {
    let caminho = caminho_arquivo(app);
    if let Some(pai) = caminho.parent() {
        let _ = fs::create_dir_all(pai);
    }

    let texto = serde_json::to_string_pretty(conexoes).map_err(|err| err.to_string())?;
    fs::write(caminho, texto).map_err(|err| err.to_string())
}
