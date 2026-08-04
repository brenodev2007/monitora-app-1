use crate::estado::Estado;
use crate::models::{
    Alerta, CanalAlerta, Heartbeat, Metricas, Servico, StatusServico, TipoServico,
};
use sqlx::mysql::{MySqlConnectOptions, MySqlPool, MySqlPoolOptions};
use std::{env, str::FromStr, sync::Arc};
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct UsuarioRow {
    id: String,
}

#[derive(sqlx::FromRow)]
struct ServicoRow {
    id: String,
    usuario_id: String,
    cliente_nome: String,
    nome: String,
    tipo: String,
    intervalo_esperado_min: i32,
    status: String,
    ultimo_heartbeat_em: Option<String>,
}

#[derive(sqlx::FromRow)]
struct HeartbeatRow {
    id: String,
    servico_id: String,
    recebido_em: String,
    status: String,
    cpu_pct: Option<f32>,
    ram_pct: Option<f32>,
    tamanho_backup_mb: Option<f32>,
    latencia_ms: Option<i32>,
}

#[derive(sqlx::FromRow)]
struct AlertaRow {
    id: String,
    servico_id: String,
    servico_nome: String,
    tipo: String,
    canal: String,
    disparado_em: String,
    resolvido: bool,
}

pub async fn ensure_ready(estado: &Arc<Estado>) -> Result<(), String> {
    {
        let pool_guard = estado.db_pool.lock().unwrap();
        if pool_guard.is_some() {
            return Ok(());
        }
    }

    let host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("DB_PORT").unwrap_or_else(|_| "3306".to_string());
    let user = env::var("DB_USER").unwrap_or_else(|_| "root".to_string());
    let password = env::var("DB_PASSWORD").unwrap_or_else(|_| "cocinfo018".to_string());
    let database_name = env::var("DB_NAME").unwrap_or_else(|_| "monitora".to_string());
    let base_url = format!("mysql://{user}:{password}@{host}:{port}/");

    let admin_pool = MySqlPoolOptions::new()
        .max_connections(1)
        .connect(&base_url)
        .await
        .map_err(|err| format!("Falha ao conectar ao MySQL para criar o banco: {err}"))?;

    sqlx::query(&format!("CREATE DATABASE IF NOT EXISTS `{database_name}`"))
        .execute(&admin_pool)
        .await
        .map_err(|err| format!("Falha ao criar o banco de dados: {err}"))?;

    let database_url = format!("mysql://{user}:{password}@{host}:{port}/{database_name}");
    let options = MySqlConnectOptions::from_str(&database_url)
        .map_err(|err| format!("URL de conexão inválida: {err}"))?;
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await
        .map_err(|err| format!("Falha ao conectar ao MySQL: {err}"))?;

    {
        let mut pool_guard = estado.db_pool.lock().unwrap();
        if pool_guard.is_none() {
            *pool_guard = Some(pool.clone());
        }
    }

    initialize_schema(&pool).await?;
    seed_data_if_needed(&pool).await?;
    Ok(())
}

pub async fn pool(estado: &Arc<Estado>) -> Result<MySqlPool, String> {
    let guard = estado.db_pool.lock().unwrap();
    guard.clone().ok_or_else(|| "Pool de banco não inicializado".to_string())
}

async fn initialize_schema(pool: &MySqlPool) -> Result<(), String> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS usuarios (
            id VARCHAR(36) PRIMARY KEY,
            nome VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL UNIQUE,
            senha VARCHAR(255) NOT NULL,
            criado_em TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|err| format!("Falha ao criar tabela usuarios: {err}"))?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS servicos (
            id VARCHAR(36) PRIMARY KEY,
            usuario_id VARCHAR(36) NOT NULL,
            cliente_nome VARCHAR(255) NOT NULL,
            nome VARCHAR(255) NOT NULL,
            tipo VARCHAR(50) NOT NULL,
            intervalo_esperado_min INT NOT NULL,
            status VARCHAR(50) NOT NULL,
            ultimo_heartbeat_em VARCHAR(255) NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|err| format!("Falha ao criar tabela servicos: {err}"))?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS heartbeats (
            id VARCHAR(36) PRIMARY KEY,
            servico_id VARCHAR(36) NOT NULL,
            recebido_em VARCHAR(255) NOT NULL,
            status VARCHAR(50) NOT NULL,
            cpu_pct DOUBLE NULL,
            ram_pct DOUBLE NULL,
            tamanho_backup_mb DOUBLE NULL,
            latencia_ms INT NULL,
            INDEX idx_heartbeats_servico_id (servico_id)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|err| format!("Falha ao criar tabela heartbeats: {err}"))?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS alertas (
            id VARCHAR(36) PRIMARY KEY,
            servico_id VARCHAR(36) NOT NULL,
            servico_nome VARCHAR(255) NOT NULL,
            tipo VARCHAR(100) NOT NULL,
            canal VARCHAR(50) NOT NULL,
            disparado_em VARCHAR(255) NOT NULL,
            resolvido BOOLEAN NOT NULL DEFAULT FALSE
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|err| format!("Falha ao criar tabela alertas: {err}"))?;

    Ok(())
}

async fn seed_data_if_needed(pool: &MySqlPool) -> Result<(), String> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM usuarios")
        .fetch_one(pool)
        .await
        .map_err(|err| format!("Falha ao validar dados iniciais: {err}"))?;

    if count > 0 {
        return Ok(());
    }

    let usuario_id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO usuarios (id, nome, email, senha) VALUES (?, ?, ?, ?)")
        .bind(&usuario_id)
        .bind("Administrador MONITORA+").bind("admin@contec.com.br")
        .bind("admin123")
        .execute(pool)
        .await
        .map_err(|err| format!("Falha ao inserir usuário inicial: {err}"))?;

    let services = [
        ("Clínica Vitalis", "API pedidos", TipoServico::Api, 5),
        ("Clínica Vitalis", "Backup diário PostgreSQL", TipoServico::Backup, 1440),
        ("Padaria Trigo Dourado", "Servidor PDV", TipoServico::Servidor, 10),
        ("Contec Contábil", "API de notas fiscais", TipoServico::Api, 5),
        ("Contec Contábil", "Backup Prisma/MySQL", TipoServico::Backup, 720),
        ("Auto Peças Rocha", "Servidor loja online", TipoServico::Servidor, 10),
    ];

    for (cliente, nome, tipo, intervalo) in services {
        let servico_id = Uuid::new_v4().to_string();
        let status = StatusServico::Ok.as_db();
        let recebido_em = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO servicos (id, usuario_id, cliente_nome, nome, tipo, intervalo_esperado_min, status, ultimo_heartbeat_em) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&servico_id)
        .bind(&usuario_id)
        .bind(cliente)
        .bind(nome)
        .bind(tipo.as_db())
        .bind(intervalo as i32)
        .bind(status)
        .bind(&recebido_em)
        .execute(pool)
        .await
        .map_err(|err| format!("Falha ao inserir serviço inicial: {err}"))?;

        let hb_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO heartbeats (id, servico_id, recebido_em, status, cpu_pct, ram_pct, tamanho_backup_mb, latencia_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&hb_id)
        .bind(&servico_id)
        .bind(&recebido_em)
        .bind(status)
        .bind(match tipo {
            TipoServico::Servidor => Some(42.0_f32),
            _ => None,
        })
        .bind(match tipo {
            TipoServico::Servidor => Some(64.0_f32),
            _ => None,
        })
        .bind(match tipo {
            TipoServico::Backup => Some(280.0_f32),
            _ => None,
        })
        .bind(match tipo {
            TipoServico::Api => Some(120_i32),
            _ => None,
        })
        .execute(pool)
        .await
        .map_err(|err| format!("Falha ao inserir heartbeat inicial: {err}"))?;
    }

    Ok(())
}

pub async fn listar_servicos(estado: &Arc<Estado>) -> Result<Vec<Servico>, String> {
    ensure_ready(estado).await?;
    let pool = pool(estado).await?;
    let servicos: Vec<ServicoRow> = sqlx::query_as::<_, ServicoRow>(
        "SELECT id, usuario_id, cliente_nome, nome, tipo, intervalo_esperado_min, status, ultimo_heartbeat_em FROM servicos ORDER BY cliente_nome, nome",
    )
    .fetch_all(&pool)
    .await
    .map_err(|err| format!("Falha ao consultar servicos: {err}"))?;

    let mut resultado = Vec::with_capacity(servicos.len());
    for servico in servicos {
        let historico = listar_heartbeats(&pool, &servico.id).await?;
        resultado.push(Servico {
            id: servico.id,
            usuario_id: servico.usuario_id,
            cliente_nome: servico.cliente_nome,
            nome: servico.nome,
            tipo: TipoServico::from_db(&servico.tipo),
            intervalo_esperado_min: servico.intervalo_esperado_min as u32,
            status: StatusServico::from_db(&servico.status),
            ultimo_heartbeat_em: servico.ultimo_heartbeat_em,
            historico,
        });
    }

    Ok(resultado)
}

async fn listar_heartbeats(pool: &MySqlPool, servico_id: &str) -> Result<Vec<Heartbeat>, String> {
    let rows: Vec<HeartbeatRow> = sqlx::query_as::<_, HeartbeatRow>(
        "SELECT id, servico_id, recebido_em, status, cpu_pct, ram_pct, tamanho_backup_mb, latencia_ms FROM heartbeats WHERE servico_id = ? ORDER BY recebido_em DESC LIMIT 40",
    )
    .bind(servico_id)
    .fetch_all(pool)
    .await
    .map_err(|err| format!("Falha ao consultar heartbeats: {err}"))?;

    Ok(rows
        .into_iter()
        .map(|row| Heartbeat {
            id: row.id,
            servico_id: row.servico_id,
            recebido_em: row.recebido_em,
            status: StatusServico::from_db(&row.status),
            metricas: Metricas {
                cpu_pct: row.cpu_pct,
                ram_pct: row.ram_pct,
                tamanho_backup_mb: row.tamanho_backup_mb,
                latencia_ms: row.latencia_ms.map(|v| v as u32),
            },
        })
        .collect())
}

pub async fn listar_alertas(estado: &Arc<Estado>) -> Result<Vec<Alerta>, String> {
    ensure_ready(estado).await?;
    let pool = pool(estado).await?;
    let rows: Vec<AlertaRow> = sqlx::query_as::<_, AlertaRow>(
        "SELECT id, servico_id, servico_nome, tipo, canal, disparado_em, resolvido FROM alertas ORDER BY disparado_em DESC LIMIT 100",
    )
    .fetch_all(&pool)
    .await
    .map_err(|err| format!("Falha ao consultar alertas: {err}"))?;

    Ok(rows
        .into_iter()
        .map(|row| Alerta {
            id: row.id,
            servico_id: row.servico_id,
            servico_nome: row.servico_nome,
            tipo: row.tipo,
            canal: CanalAlerta::from_db(&row.canal),
            disparado_em: row.disparado_em,
            resolvido: row.resolvido,
        })
        .collect())
}

pub async fn autenticar(estado: &Arc<Estado>, email: String, senha: String) -> Result<bool, String> {
    ensure_ready(estado).await?;
    let pool = pool(estado).await?;
    let email = email.trim();
    let senha = senha.trim();

    if email.is_empty() || senha.is_empty() {
        return Err("Informe e-mail e senha.".into());
    }

    let row: Option<UsuarioRow> = sqlx::query_as::<_, UsuarioRow>(
        "SELECT id FROM usuarios WHERE email = ? AND senha = ? LIMIT 1",
    )
    .bind(email)
    .bind(senha)
    .fetch_optional(&pool)
    .await
    .map_err(|err| format!("Falha ao validar credenciais: {err}"))?;

    let autenticado = row.is_some();
    *estado.autenticado.lock().unwrap() = autenticado;
    Ok(autenticado)
}

pub async fn resolver_alerta(estado: &Arc<Estado>, id: String) -> Result<bool, String> {
    ensure_ready(estado).await?;
    let pool = pool(estado).await?;
    let result = sqlx::query("UPDATE alertas SET resolvido = TRUE WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|err| format!("Falha ao resolver alerta: {err}"))?;

    Ok(result.rows_affected() > 0)
}

pub async fn salvar_heartbeat(estado: &Arc<Estado>, heartbeat: Heartbeat) -> Result<(), String> {
    ensure_ready(estado).await?;
    let pool = pool(estado).await?;

    sqlx::query(
        "INSERT INTO heartbeats (id, servico_id, recebido_em, status, cpu_pct, ram_pct, tamanho_backup_mb, latencia_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&heartbeat.id)
    .bind(&heartbeat.servico_id)
    .bind(&heartbeat.recebido_em)
    .bind(heartbeat.status.as_db())
    .bind(heartbeat.metricas.cpu_pct)
    .bind(heartbeat.metricas.ram_pct)
    .bind(heartbeat.metricas.tamanho_backup_mb)
    .bind(heartbeat.metricas.latencia_ms.map(|v| v as i32))
    .execute(&pool)
    .await
    .map_err(|err| format!("Falha ao salvar heartbeat no banco: {err}"))?;

    sqlx::query("UPDATE servicos SET status = ?, ultimo_heartbeat_em = ? WHERE id = ?")
        .bind(heartbeat.status.as_db())
        .bind(&heartbeat.recebido_em)
        .bind(&heartbeat.servico_id)
        .execute(&pool)
        .await
        .map_err(|err| format!("Falha ao atualizar status do serviço: {err}"))?;

    Ok(())
}

pub async fn salvar_alerta(estado: &Arc<Estado>, alerta: Alerta) -> Result<(), String> {
    ensure_ready(estado).await?;
    let pool = pool(estado).await?;

    sqlx::query(
        "INSERT INTO alertas (id, servico_id, servico_nome, tipo, canal, disparado_em, resolvido) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&alerta.id)
    .bind(&alerta.servico_id)
    .bind(&alerta.servico_nome)
    .bind(&alerta.tipo)
    .bind(alerta.canal.as_db())
    .bind(&alerta.disparado_em)
    .bind(alerta.resolvido)
    .execute(&pool)
    .await
    .map_err(|err| format!("Falha ao salvar alerta no banco: {err}"))?;

    Ok(())
}
