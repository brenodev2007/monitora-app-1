import { useEffect, useState, type FormEvent } from "react";
import { Radio, WifiOff, PlugZap, Plug, Loader2, Plus, Pencil, Trash2, X } from "lucide-react";
import { cn } from "@/lib/utils";
import { bridge, type ConexaoSalva } from "@/lib/bridge";

const URL_PADRAO = "ws://localhost:8080";

export function BarraTopo({ conectado, totalAlertasAbertos }: { conectado: boolean; totalAlertasAbertos: number }) {
  const [url, setUrl] = useState(URL_PADRAO);
  const [carregando, setCarregando] = useState(false);
  const [conexoes, setConexoes] = useState<ConexaoSalva[]>([]);
  const [modalAberto, setModalAberto] = useState(false);
  const [modalConfirmacaoAberto, setModalConfirmacaoAberto] = useState(false);
  const [modoModal, setModoModal] = useState<"criar" | "editar">("criar");
  const [formNome, setFormNome] = useState("");
  const [formUrl, setFormUrl] = useState("");
  const [conexaoEditandoId, setConexaoEditandoId] = useState<string | null>(null);
  const [conexaoParaExcluir, setConexaoParaExcluir] = useState<ConexaoSalva | null>(null);

  useEffect(() => {
    async function carregarConexoes() {
      try {
        const lista = await bridge.listarConexoes();
        if (lista.length > 0) {
          setConexoes(lista);
          setUrl(lista[0].url);
        }
      } catch {
        setConexoes([]);
      }
    }

    carregarConexoes();
  }, []);

  async function conectar() {
    setCarregando(true);
    try {
      await bridge.conectarWebSocket(url);
    } finally {
      setCarregando(false);
    }
  }

  async function desconectar() {
    setCarregando(true);
    try {
      await bridge.desconectarWebSocket();
    } finally {
      setCarregando(false);
    }
  }

  function abrirModalCriar() {
    setModoModal("criar");
    setConexaoEditandoId(null);
    setFormNome("");
    setFormUrl(url);
    setModalAberto(true);
  }

  function abrirModalEditar(conexao: ConexaoSalva) {
    setModoModal("editar");
    setConexaoEditandoId(conexao.id);
    setFormNome(conexao.nome);
    setFormUrl(conexao.url);
    setModalAberto(true);
  }

  async function salvarConexao(e: FormEvent) {
    e.preventDefault();
    const nome = formNome.trim();
    const endpoint = formUrl.trim();

    if (!nome || !endpoint) return;

    try {
      let lista: ConexaoSalva[];
      if (modoModal === "editar" && conexaoEditandoId) {
        lista = await bridge.atualizarConexao(conexaoEditandoId, nome, endpoint);
      } else {
        lista = await bridge.salvarConexao(nome, endpoint);
      }

      setConexoes(lista);
      if (modoModal !== "editar") {
        setUrl(endpoint);
      }
    } finally {
      setModalAberto(false);
      setFormNome("");
      setFormUrl("");
      setConexaoEditandoId(null);
    }
  }

  function usarConexao(conexao: ConexaoSalva) {
    setUrl(conexao.url);
    setModalAberto(false);
  }

  async function confirmarExclusao(conexao: ConexaoSalva) {
    setConexaoParaExcluir(conexao);
    setModalConfirmacaoAberto(true);
  }

  async function excluirConexao() {
    if (!conexaoParaExcluir) return;

    const lista = await bridge.deletarConexao(conexaoParaExcluir.id);
    setConexoes(lista);

    if (url === conexaoParaExcluir.url) {
      const restante = lista[0];
      if (restante) {
        setUrl(restante.url);
      } else {
        setUrl(URL_PADRAO);
      }
    }

    setConexaoParaExcluir(null);
    setModalConfirmacaoAberto(false);
  }

  return (
    <>
      <header className="flex h-16 items-center justify-between border-b border-[var(--color-border)] bg-[var(--color-surface)] px-5">
        <div className="flex items-center gap-2.5">
          <span className="text-sm font-semibold tracking-tight">MONITORA+</span>
          <span className="text-xs text-[var(--color-ink-dim)]">Painel administrativo</span>
        </div>

        <div className="flex flex-1 items-center justify-end gap-3">
          <div className="flex items-center gap-2 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-2)] px-2 py-1.5">
            <PlugZap size={14} className="text-[var(--color-brand)]" />
            <input
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder="wss://seu-servidor/ws"
              className="w-72 rounded-md border border-transparent bg-transparent px-2 py-1 text-xs text-[var(--color-ink)] outline-none placeholder:text-[var(--color-ink-dim)]"
            />
          </div>

          <button
            onClick={abrirModalCriar}
            className="flex items-center gap-1.5 rounded-lg border border-[var(--color-border)] px-3 py-2 text-xs font-medium text-[var(--color-ink)] transition hover:border-[var(--color-brand)]/50"
          >
            <Plus size={13} />
            Nova conexão
          </button>

          {conectado ? (
            <button
              onClick={desconectar}
              disabled={carregando}
              className="flex items-center gap-1.5 rounded-lg border border-[var(--color-border)] px-3 py-2 text-xs font-medium text-[var(--color-ink)] transition hover:border-[var(--color-brand)]/50"
            >
              {carregando ? <Loader2 size={13} className="animate-spin" /> : <Plug size={13} />}
              Desconectar
            </button>
          ) : (
            <button
              onClick={conectar}
              disabled={carregando}
              className="flex items-center gap-1.5 rounded-lg bg-[var(--color-brand)] px-3 py-2 text-xs font-medium text-[#04211c] transition hover:brightness-110 disabled:opacity-70"
            >
              {carregando ? <Loader2 size={13} className="animate-spin" /> : <Plug size={13} />}
              Conectar
            </button>
          )}

          {totalAlertasAbertos > 0 && (
            <span className="rounded-full bg-[var(--color-falha)]/15 px-2.5 py-1 text-xs font-medium text-[var(--color-falha)]">
              {totalAlertasAbertos} alerta{totalAlertasAbertos > 1 ? "s" : ""} em aberto
            </span>
          )}
          <div
            className={cn(
              "flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium",
              conectado ? "text-[var(--color-ok)]" : "text-[var(--color-ink-dim)]",
            )}
          >
            {conectado ? <Radio size={13} /> : <WifiOff size={13} />}
            {conectado ? "Conectado em tempo real" : "Desconectado"}
          </div>
        </div>
      </header>

      {modalAberto && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 px-4">
          <div className="w-full max-w-lg rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-2xl">
            <div className="mb-4 flex items-center justify-between">
              <div>
                <p className="text-sm font-semibold">{modoModal === "criar" ? "Nova conexão" : "Editar conexão"}</p>
                <p className="text-xs text-[var(--color-ink-dim)]">Salve um endpoint de WebSocket para usar depois</p>
              </div>
              <button onClick={() => setModalAberto(false)} className="rounded-lg p-1.5 text-[var(--color-ink-dim)] hover:bg-[var(--color-surface-2)]">
                <X size={16} />
              </button>
            </div>

            <form onSubmit={salvarConexao} className="space-y-3">
              <label className="block text-xs text-[var(--color-ink-dim)]">
                Nome da conexão
                <input
                  value={formNome}
                  onChange={(e) => setFormNome(e.target.value)}
                  className="mt-1 w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-surface-2)] px-3 py-2 text-sm text-[var(--color-ink)] outline-none focus:border-[var(--color-brand)]"
                  placeholder="Servidor produção"
                />
              </label>

              <label className="block text-xs text-[var(--color-ink-dim)]">
                URL do WebSocket
                <input
                  value={formUrl}
                  onChange={(e) => setFormUrl(e.target.value)}
                  className="mt-1 w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-surface-2)] px-3 py-2 text-sm text-[var(--color-ink)] outline-none focus:border-[var(--color-brand)]"
                  placeholder="wss://seu-servidor/ws"
                />
              </label>

              <div className="flex items-center justify-between gap-2 pt-2">
                <button
                  type="button"
                  onClick={() => setModalAberto(false)}
                  className="rounded-lg border border-[var(--color-border)] px-3 py-2 text-sm text-[var(--color-ink-dim)]"
                >
                  Cancelar
                </button>
                <button
                  type="submit"
                  className="rounded-lg bg-[var(--color-brand)] px-3 py-2 text-sm font-medium text-[#04211c]"
                >
                  Salvar conexão
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {modalConfirmacaoAberto && conexaoParaExcluir && (
        <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black/70 px-4">
          <div className="w-full max-w-md rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-2xl">
            <p className="text-sm font-semibold">Excluir conexão?</p>
            <p className="mt-2 text-sm text-[var(--color-ink-dim)]">
              Tem certeza que deseja remover <span className="font-medium text-[var(--color-ink)]">{conexaoParaExcluir.nome}</span>?
            </p>
            <div className="mt-5 flex justify-end gap-2">
              <button
                onClick={() => {
                  setConexaoParaExcluir(null);
                  setModalConfirmacaoAberto(false);
                }}
                className="rounded-lg border border-[var(--color-border)] px-3 py-2 text-sm text-[var(--color-ink-dim)]"
              >
                Cancelar
              </button>
              <button
                onClick={excluirConexao}
                className="rounded-lg bg-[var(--color-falha)] px-3 py-2 text-sm font-medium text-white"
              >
                Excluir
              </button>
            </div>
          </div>
        </div>
      )}

      <div className="border-b border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-3">
        <div className="mb-2 flex items-center justify-between">
          <p className="text-xs font-medium uppercase tracking-wide text-[var(--color-ink-dim]">Conexões salvas</p>
          <button onClick={abrirModalCriar} className="text-xs text-[var(--color-brand)]">+ adicionar</button>
        </div>

        {conexoes.length === 0 ? (
          <p className="text-xs text-[var(--color-ink-dim)]">Nenhuma conexão salva ainda.</p>
        ) : (
          <ul className="space-y-2">
            {conexoes.map((conexao) => (
              <li key={conexao.id} className="flex items-center justify-between rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-2)] px-3 py-2">
                <button onClick={() => usarConexao(conexao)} className="flex-1 text-left">
                  <p className="text-sm font-medium">{conexao.nome}</p>
                  <p className="text-[11px] text-[var(--color-ink-dim)]">{conexao.url}</p>
                </button>
                <div className="flex items-center gap-1.5">
                  <button onClick={() => abrirModalEditar(conexao)} className="rounded-lg p-1.5 text-[var(--color-ink-dim)] hover:bg-[var(--color-surface)]">
                    <Pencil size={13} />
                  </button>
                  <button onClick={() => confirmarExclusao(conexao)} className="rounded-lg p-1.5 text-[var(--color-falha)] hover:bg-[var(--color-surface)]">
                    <Trash2 size={13} />
                  </button>
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>
    </>
  );
}
