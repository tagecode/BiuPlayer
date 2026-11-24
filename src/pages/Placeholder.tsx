type PlaceholderProps = {
  page: string;
  title: string;
  description: string;
  next: string[];
};

export function Placeholder({ page, title, description, next }: PlaceholderProps) {
  return (
    <section className="page">
      <div className="page-head">
        <span className="page-kicker">{page}</span>
        <h2>{title}</h2>
        <p>{description}</p>
      </div>
      <div className="card">
        <div className="card-head">
          <h3>本页待实现</h3>
          <span className="muted">E1-05 占位 · 后续任务填充</span>
        </div>
        <ul className="todo-list">
          {next.map((item) => (
            <li key={item}>{item}</li>
          ))}
        </ul>
      </div>
    </section>
  );
}
