import { useState } from "react";
import { Dialog } from "../components/Dialog";
import { StatusView } from "../components/StatusView";

export function States() {
  const [open, setOpen] = useState(false);

  return (
    <section className="page">
      <div className="page-head">
        <span className="page-kicker">E1-07</span>
        <h2>Component states</h2>
        <p>Loading, empty, error, offline and dialog primitives used by later pages.</p>
      </div>
      <div className="state-grid">
        <StatusView kind="loading" title="Loading library" description="Reading the local database…" />
        <StatusView kind="empty" title="No tracks yet" description="Add a folder to start building your library." action={<button>Add folder</button>} />
        <StatusView kind="error" title="Playback failed" description="The output device was removed." action={<button>Retry</button>} />
        <StatusView kind="offline" title="Offline mode" description="Lyrics lookup needs a network connection; playback still works." />
      </div>
      <div className="card">
        <div className="card-head">
          <h3>Dialog</h3>
          <span className="muted">native &lt;dialog&gt; · Escape closes</span>
        </div>
        <button onClick={() => setOpen(true)}>Open dialog</button>
      </div>
      <Dialog open={open} title="Confirm action" onClose={() => setOpen(false)}>
        <p>This is the shared dialog primitive. Content and footer actions are owned by the caller.</p>
        <div className="dialog-actions">
          <button className="ghost" onClick={() => setOpen(false)}>Cancel</button>
          <button onClick={() => setOpen(false)}>Confirm</button>
        </div>
      </Dialog>
    </section>
  );
}
