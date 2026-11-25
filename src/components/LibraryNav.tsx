import { NavLink } from "react-router-dom";
import { useI18n } from "../i18n";

export function LibraryNav() {
  const { t } = useI18n();
  const tabs = [
    { to: "/library", label: t("nav.library"), end: true },
    { to: "/library/albums", label: t("nav.albums") },
    { to: "/library/artists", label: t("nav.artists") },
    { to: "/library/album-artists", label: t("nav.albumArtists") },
    { to: "/library/genres", label: t("pages.genres.title") },
    { to: "/library/years", label: t("pages.years.title") },
    { to: "/library/folders", label: t("pages.folders.title") },
  ];
  return (
    <nav className="library-nav" aria-label={t("library.browse")}>
      {tabs.map((tab) => (
        <NavLink
          key={tab.to}
          to={tab.to}
          end={tab.end}
          className={({ isActive }) => (isActive ? "library-tab active" : "library-tab")}
        >
          {tab.label}
        </NavLink>
      ))}
    </nav>
  );
}
