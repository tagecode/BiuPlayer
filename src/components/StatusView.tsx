import type { ReactNode } from "react";

export type StatusKind = "loading" | "empty" | "error" | "offline";

const icons: Record<StatusKind, string> = {
  loading: "◌",
  empty: "♪",
  error: "!",
  offline: "⌁",
};

type StatusViewProps = {
  kind: StatusKind;
  title: string;
  description?: string;
  action?: ReactNode;
};

export function StatusView({ kind, title, description, action }: StatusViewProps) {
  return (
    <div className={`status-view status-${kind}`} role={kind === "error" ? "alert" : "status"}>
      <span className="status-icon" aria-hidden="true">{icons[kind]}</span>
      <strong>{title}</strong>
      {description && <p className="muted">{description}</p>}
      {action && <div className="status-action">{action}</div>}
    </div>
  );
}
