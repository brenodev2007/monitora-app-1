use crate::estado::Estado;
use crate::models::{Alerta, CanalAlerta, Heartbeat, Metricas, Servico, StatusServico, TipoServico};
use chrono::Utc;
use rand::Rng;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;
use uuid::Uuid;

fn agora_iso() -> String {
    Utc::now().to_rfc3339()
}

fn servicos_seed() -> Vec<Servico> {
    let seed = [
        ("Clínica Vitalis", "API pedidos", TipoServico::Api, 5),
        ("Clínica Vitalis", "Backup diário PostgreSQL", TipoServico::Backup, 1440),
        ("Padaria Trigo Dourado", "Servidor PDV", TipoServico::Servidor, 10),
        ("Contec Contábil", "API de notas fiscais", TipoServico::Api, 5),
        ("Contec Contábil", "Backup Prisma/MySQL", TipoServico::Backup, 720),
        ("Auto Peças Rocha", "Servidor loja online", TipoServico::Servidor, 10),
    ];

    seed.into_iter()
        .map(|(cliente, nome, tipo, intervalo)| Servico {
            id: Uuid::new_v4().to_string(),
            usuario_id: Uuid::new_v4().to_string(),
            cliente_nome: cliente.to_string(),
            nome: nome.to_string(),
            tipo,
            intervalo_esperado_min: intervalo,
            status: StatusServico::Ok,
            ultimo_heartbeat_em: Some(agora_iso()),
            historico: Vec::new(),
        })
        .collect()
}

/// Simula, no processo principal, o papel descrito em 6.1/6.3/6.4 da
/// especificação: agente enviando heartbeat + worker verificando atraso.
/// Troque esta função por uma conexão WebSocket real para
/// ws://<host-da-api>/admin quando o backend estiver disponível.
pub fn iniciar(app: AppHandle, estado: Arc<Estado>) {
    {
        let mut servicos = estado.servicos.lock().unwrap();
        *servicos = servicos_seed();
    }

    tauri::async_runtime::spawn(async move {
        // "Handshake" inicial de conexão
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
        {
            let mut conexao = estado.conectado.lock().unwrap();
            conexao.conectado = true;
            conexao.ultima_tentativa_em = agora_iso();
        }
        let status = estado.conectado.lock().unwrap().clone();
        let _ = app.emit("conexao:status", status);
        let snapshot = estado.servicos.lock().unwrap().clone();
        let _ = app.emit("servicos:sincronizados", snapshot);

        loop {
            tokio::time::sleep(std::time::Duration::from_secs(4)).await;

            let mut rng = rand::thread_rng();
            let mut novos_alertas: Vec<Alerta> = Vec::new();
            let mut heartbeats_emitidos: Vec<(String, Heartbeat)> = Vec::new();

            {
                let mut servicos = estado.servicos.lock().unwrap();
                for servico in servicos.iter_mut() {
                    let rolagem: f32 = rng.gen();

                    // ~78% recebe heartbeat normal, ~14% atraso, ~8% falha reportada
                    let novo_status = if rolagem < 0.78 {
                        StatusServico::Ok
                    } else if rolagem < 0.92 {
                        StatusServico::Atraso
                    } else {
                        StatusServico::Falha
                    };

                    let estava_saudavel = matches!(servico.status, StatusServico::Ok);
                    servico.status = novo_status;

                    if !matches!(novo_status, StatusServico::Falha) {
                        servico.ultimo_heartbeat_em = Some(agora_iso());
                    }

                    let heartbeat = Heartbeat {
                        id: Uuid::new_v4().to_string(),
                        servico_id: servico.id.clone(),
                        recebido_em: agora_iso(),
                        status: novo_status,
                        metricas: Metricas {
                            cpu_pct: matches!(servico.tipo, TipoServico::Servidor)
                                .then(|| rng.gen_range(15.0..85.0)),
                            ram_pct: matches!(servico.tipo, TipoServico::Servidor)
                                .then(|| rng.gen_range(30.0..90.0)),
                            tamanho_backup_mb: matches!(servico.tipo, TipoServico::Backup)
                                .then(|| rng.gen_range(120.0..900.0)),
                            latencia_ms: matches!(servico.tipo, TipoServico::Api)
                                .then(|| rng.gen_range(40..320)),
                        },
                    };
                    servico.historico.push(heartbeat.clone());
                    if servico.historico.len() > 40 {
                        servico.historico.remove(0);
                    }
                    heartbeats_emitidos.push((servico.id.clone(), heartbeat));

                    if estava_saudavel && !matches!(novo_status, StatusServico::Ok) {
                        let canal = match rng.gen_range(0..3) {
                            0 => CanalAlerta::Email,
                            1 => CanalAlerta::Whatsapp,
                            _ => CanalAlerta::Discord,
                        };
                        novos_alertas.push(Alerta {
                            id: Uuid::new_v4().to_string(),
                            servico_id: servico.id.clone(),
                            servico_nome: format!("{} — {}", servico.cliente_nome, servico.nome),
                            tipo: if matches!(novo_status, StatusServico::Falha) {
                                "falha_reportada".to_string()
                            } else {
                                "heartbeat_ausente".to_string()
                            },
                            canal,
                            disparado_em: agora_iso(),
                            resolvido: false,
                        });
                    }
                }
            }

            for (servico_id, heartbeat) in heartbeats_emitidos {
                let _ = app.emit("heartbeat:recebido", (servico_id, heartbeat));
            }

            for alerta in novos_alertas {
                {
                    let mut alertas = estado.alertas.lock().unwrap();
                    alertas.insert(0, alerta.clone());
                    if alertas.len() > 100 {
                        alertas.truncate(100);
                    }
                }
                let _ = app.emit("alerta:novo", &alerta);
                let _ = app
                    .notification()
                    .builder()
                    .title("MONITORA+ — novo alerta")
                    .body(format!("{} está fora do esperado", alerta.servico_nome))
                    .show();
            }
        }
    });
}
