import { createHashRouter, Navigate } from "react-router-dom";
import App from "./App";
import { Shell } from "./Shell";
import { Library } from "./pages/Library";
import { AlbumDetail } from "./pages/AlbumDetail";
import { Albums } from "./pages/Albums";
import { ArtistDetail } from "./pages/ArtistDetail";
import { Artists } from "./pages/Artists";
import { Folders, FolderDetail, Genres, GenreDetail, Years, YearDetail } from "./pages/Browse";
import { NowPlaying } from "./pages/NowPlaying";
import { DesktopLyrics } from "./pages/DesktopLyrics";
import { Settings } from "./pages/Settings";
import { Search } from "./pages/Search";
import { PlaylistDetail } from "./pages/PlaylistDetail";
import { Playlists } from "./pages/Playlists";
import { Placeholder } from "./pages/Placeholder";
import { States } from "./pages/States";
import type { MessageKey } from "./i18n";

function page(pageId: string, titleKey: MessageKey, descKey: MessageKey) {
  return <Placeholder page={pageId} titleKey={titleKey} descKey={descKey} />;
}

export const router = createHashRouter([
  { path: "/desktop-lyrics", element: <DesktopLyrics /> },
  {
    element: <Shell />,
    children: [
      { path: "/", element: <NowPlaying /> },
      { path: "/library", element: <Library /> },
      { path: "/library/albums", element: <Albums /> },
      { path: "/library/albums/:id", element: <AlbumDetail /> },
      { path: "/library/artists", element: <Artists /> },
      { path: "/library/artists/:id", element: <ArtistDetail /> },
      { path: "/library/genres", element: <Genres /> },
      { path: "/library/genres/:name", element: <GenreDetail /> },
      { path: "/library/years", element: <Years /> },
      { path: "/library/years/:year", element: <YearDetail /> },
      { path: "/library/folders", element: <Folders /> },
      { path: "/library/folders/:id", element: <FolderDetail /> },
      { path: "/playlists", element: <Playlists /> },
      { path: "/playlists/:id", element: <PlaylistDetail /> },
      { path: "/search", element: <Search /> },
      { path: "/lyrics/:trackId", element: page("P-09", "pages.lyrics.title", "pages.lyrics.desc") },
      { path: "/equalizer", element: page("P-10", "pages.equalizer.title", "pages.equalizer.desc") },
      { path: "/skins", element: page("P-11", "pages.skins.title", "pages.skins.desc") },
      { path: "/plugins", element: page("P-12", "pages.plugins.title", "pages.plugins.desc") },
      { path: "/settings/:section", element: <Settings /> },
      { path: "/settings", element: <Navigate to="/settings/general" replace /> },
      { path: "/tools", element: page("P-14", "pages.tools.title", "pages.tools.desc") },
      { path: "/dev/states", element: <States /> },
      { path: "/m0", element: <App /> },
    ],
  },
  { path: "*", element: <Navigate to="/" replace /> },
]);
