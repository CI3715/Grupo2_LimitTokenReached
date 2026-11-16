/** Programa intentos sin solapar operaciones. El llamador controla la sesión. */
export function startSyncSchedule({ sync, onError, target = window, intervalMs = 300_000 }: {
  sync: () => Promise<void>;
  onError: (error: unknown) => void;
  target?: Window;
  intervalMs?: number;
}) {
  let active = true;
  let running = false;
  async function run() {
    if (!active || running) return;
    running = true;
    try { await sync(); } catch (error) { if (active) onError(error); } finally { running = false; }
  }
  const timer = target.setInterval(() => { void run(); }, intervalMs);
  const onOnline = () => { void run(); };
  target.addEventListener("online", onOnline);
  void run();
  return { run, stop() { active = false; target.clearInterval(timer); target.removeEventListener("online", onOnline); } };
}
