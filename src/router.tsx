import { createHashRouter, Navigate } from "react-router-dom";
import App from "./App";
import { Shell } from "./Shell";
import { Library } from "./pages/Library";
import { PlaylistDetail } from "./pages/PlaylistDetail";
import { Playlists } from "./pages/Playlists";
import { Placeholder } from "./pages/Placeholder";
import { States } from "./pages/States";
import type { MessageKey } from "./i18n";

function page(pageId: string, titleKey: MessageKey, descKey: MessageKey) {
  return <Placeholder page={pageId} titleKey={titleKey} descKey={descKey} />;
}

export const router = createHashRouter([
  {
    element: <Shell />,
    children: [
      { path: "/", element: page("P-01", "pages.nowPlaying.title", "pages.nowPlaying.desc") },
      { path: "/library", element: <Library /> },
      { path: "/library/albums", element: page("P-03", "pages.albums.title", "pages.albums.desc") },
      { path: "/library/albums/:id", element: page("P-04", "pages.album.title", "pages.album.desc") },
      { path: "/library/artists", element: page("P-05", "pages.artists.title", "pages.artists.desc") },
      { path: "/library/artists/:id", element: page("P-05", "pages.artist.title", "pages.artist.desc") },
      { path: "/library/genres", element: page("P-06", "pages.genres.title", "pages.genres.desc") },
      { path: "/playlists", element: <Playlists /> },
      { path: "/playlists/:id", element: <PlaylistDetail /> },
      { path: "/search", element: page("P-08", "pages.search.title", "pages.search.desc") },
      { path: "/lyrics/:trackId", element: page("P-09", "pages.lyrics.title", "pages.lyrics.desc") },
      { path: "/equalizer", element: page("P-10", "pages.equalizer.title", "pages.equalizer.desc") },
      { path: "/skins", element: page("P-11", "pages.skins.title", "pages.skins.desc") },
      { path: "/plugins", element: page("P-12", "pages.plugins.title", "pages.plugins.desc") },
      { path: "/settings/:section", element: page("P-13", "pages.settings.title", "pages.settings.desc") },
      { path: "/settings", element: <Navigate to="/settings/general" replace /> },
      { path: "/tools", element: page("P-14", "pages.tools.title", "pages.tools.desc") },
      { path: "/dev/states", element: <States /> },
      { path: "/m0", element: <App /> },
    ],
  },
  { path: "*", element: <Navigate to="/" replace /> },
]);
