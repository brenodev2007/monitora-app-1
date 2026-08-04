import { useState } from "react";
import { ShieldCheck, Loader2 } from "lucide-react";
import { bridge } from "@/lib/bridge";

export function TelaLogin({ onEntrar }: { onEntrar: () => void }) {
  const [email, setEmail] = useState("");
  const [senha, setSenha] = useState("");
  const [carregando, setCarregando] = useState(false);
  const [erro, setErro] = useState<string | null>(null);

  async function entrar(e: React.FormEvent) {
    e.preventDefault();
    setErro(null);
    setCarregando(true);
    try {
      await bridge.autenticar(email, senha);
      onEntrar();
    } catch (falha) {
      setErro(typeof falha === "string" ? falha : "Não foi possível entrar. Confira os dados.");
    } finally {
      setCarregando(false);
    }
  }

  return (
    <div className="flex h-screen w-screen items-center justify-center bg-[var(--color-base)]">
      <form
        onSubmit={entrar}
        className="w-[360px] rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-8 shadow-2xl"
      >
        <div className="mb-6 flex items-center gap-2.5">
          <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-[var(--color-brand)]/15 text-[var(--color-brand)]">
            <ShieldCheck size={18} />
          </div>
          <div>
            <p className="text-sm font-semibold leading-none">MONITORA+</p>
            <p className="text-xs text-[var(--color-ink-dim)]">Painel administrativo</p>
          </div>
        </div>

        <label className="mb-3 block text-xs text-[var(--color-ink-dim)]">
          E-mail
          <input
            autoFocus
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder="admin@contec.com.br"
            className="mt-1 w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-surface-2)] px-3 py-2 text-sm text-[var(--color-ink)] outline-none focus:border-[var(--color-brand)]"
          />
        </label>

        <label className="mb-5 block text-xs text-[var(--color-ink-dim)]">
          Senha
          <input
            type="password"
            value={senha}
            onChange={(e) => setSenha(e.target.value)}
            placeholder="••••••••"
            className="mt-1 w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-surface-2)] px-3 py-2 text-sm text-[var(--color-ink)] outline-none focus:border-[var(--color-brand)]"
          />
        </label>

        {erro && <p className="mb-4 text-xs text-[var(--color-falha)]">{erro}</p>}

        <button
          type="submit"
          disabled={carregando}
          className="flex w-full items-center justify-center gap-2 rounded-lg bg-[var(--color-brand)] py-2.5 text-sm font-medium text-[#04211c] transition hover:brightness-110 disabled:opacity-60"
        >
          {carregando && <Loader2 size={14} className="animate-spin" />}
          Entrar
        </button>

        <p className="mt-4 text-center text-[11px] text-[var(--color-ink-dim)]">
          Acesso restrito ao time Contec
        </p>
      </form>
    </div>
  );
}
