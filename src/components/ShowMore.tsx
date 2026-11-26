import { useEffect, useState } from "react";
import { useI18n } from "../i18n";

/**
 * Client-side pagination for catalog lists.
 *
 * Album/artist/genre/year views can hold tens of thousands of rows; rendering
 * them all at once stalls the window. The first page stays small and the rest
 * load on demand (Mvp LIB-04 "分页或虚拟").
 */
export function useLoadMore(total: number, step = 120) {
  const [visible, setVisible] = useState(step);
  useEffect(() => {
    setVisible(step);
  }, [total, step]);
  return {
    visible: Math.min(visible, total),
    hasMore: visible < total,
    loadMore: () => setVisible((value) => value + step),
  };
}

export function ShowMore({ hasMore, onClick }: { hasMore: boolean; onClick: () => void }) {
  const { t } = useI18n();
  if (!hasMore) return null;
  return (
    <div className="load-more-row">
      <button type="button" className="ghost" onClick={onClick}>
        {t("common.loadMore")}
      </button>
    </div>
  );
}
