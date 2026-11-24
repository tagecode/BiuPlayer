import { useI18n, type MessageKey } from "../i18n";

type PlaceholderProps = {
  page: string;
  titleKey: MessageKey;
  descKey: MessageKey;
};

export function Placeholder({ page, titleKey, descKey }: PlaceholderProps) {
  const { t } = useI18n();
  const todo = ["page.todo.0", "page.todo.1", "page.todo.2"] as const;

  return (
    <section className="page">
      <div className="page-head">
        <span className="page-kicker">{page}</span>
        <h2>{t(titleKey)}</h2>
        <p>{t(descKey)}</p>
      </div>
      <div className="card">
        <div className="card-head">
          <h3>{t("page.todoTitle")}</h3>
          <span className="muted">{t("page.todoHint")}</span>
        </div>
        <ul className="todo-list">
          {todo.map((key) => (
            <li key={key}>{t(key)}</li>
          ))}
        </ul>
      </div>
    </section>
  );
}
