import { cn } from "@/lib/utils";
import type { StatusServico } from "@/types/monitora";

const rotulos: Record<StatusServico, string> = {
  ok: "Operacional",
  atraso: "Atraso",
  falha: "Falha",
};

const estilos: Record<StatusServico, string> = {
  ok: "bg-[color-mix(in_srgb,var(--color-ok)_16%,transparent)] text-[var(--color-ok)] ring-[var(--color-ok)]/30",
  atraso: "bg-[color-mix(in_srgb,var(--color-atraso)_16%,transparent)] text-[var(--color-atraso)] ring-[var(--color-atraso)]/30",
  falha: "bg-[color-mix(in_srgb,var(--color-falha)_16%,transparent)] text-[var(--color-falha)] ring-[var(--color-falha)]/30",
};

export function StatusBadge({ status, pulsar = false }: { status: StatusServico; pulsar?: boolean }) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium ring-1",
        estilos[status],
      )}
    >
      <span
        className={cn(
          "h-1.5 w-1.5 rounded-full",
          status === "ok" && "bg-[var(--color-ok)]",
          status === "atraso" && "bg-[var(--color-atraso)]",
          status === "falha" && "bg-[var(--color-falha)]",
          pulsar && status !== "ok" && "animate-pulse",
        )}
      />
      {rotulos[status]}
    </span>
  );
}
