/** Line-level lyric timing helpers (kept pure so they can be unit tested). */

export type TimedLine = { timeMs?: number | null };

/**
 * Index of the line that should be highlighted at `positionMs + offsetMs`.
 * Returns -1 before the first timestamped line.
 */
export function activeLineIndex(lines: TimedLine[], positionMs: number, offsetMs = 0): number {
  const at = positionMs + offsetMs;
  let active = -1;
  for (let index = 0; index < lines.length; index += 1) {
    if ((lines[index].timeMs ?? 0) <= at) {
      active = index;
    } else {
      break;
    }
  }
  return active;
}

export type PositionAnchor = {
  positionMs: number;
  playing: boolean;
  atMs: number;
};

/**
 * Extrapolate the last player snapshot (emitted at 4 Hz) to `nowMs` so lyric
 * highlighting does not lag up to 250 ms behind the audio.
 */
export function interpolatePosition(
  anchor: PositionAnchor,
  nowMs: number,
  durationMs: number,
): number {
  const elapsed = anchor.playing ? Math.max(0, nowMs - anchor.atMs) : 0;
  const next = anchor.positionMs + elapsed;
  if (durationMs > 0) {
    return Math.max(0, Math.min(next, durationMs));
  }
  return Math.max(0, next);
}
