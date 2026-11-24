import { createHashRouter, Navigate } from "react-router-dom";
import App from "./App";
import { Shell } from "./Shell";
import { Placeholder } from "./pages/Placeholder";

export const router = createHashRouter([
  {
    element: <Shell />,
    children: [
      {
        path: "/",
        element: (
          <Placeholder
            page="P-01"
            title="正在播放"
            description="封面、曲目信息、歌词、可视化与队列的主视图。"
            next={["接入 A1-04 播放状态机", "封面与曲目元数据", "歌词与队列侧栏", "频谱可视化（P2）"]}
          />
        ),
      },
      {
        path: "/library",
        element: (
          <Placeholder
            page="P-02"
            title="曲目"
            description="本地曲库的默认视图，虚拟列表承载 10 万级曲目。"
            next={["D1-01 schema 与查询命令", "L1 曲库列表与排序", "批量选择与右键菜单"]}
          />
        ),
      },
      {
        path: "/library/albums",
        element: <Placeholder page="P-03" title="专辑" description="按专辑聚合，支持封面网格与详情。" next={["专辑网格", "专辑详情与曲目列表"]} />,
      },
      {
        path: "/library/albums/:id",
        element: <Placeholder page="P-04" title="专辑详情" description="单张专辑的曲目、封面与元数据。" next={["专辑查询命令", "播放整张专辑"]} />,
      },
      {
        path: "/library/artists",
        element: <Placeholder page="P-05" title="艺术家" description="按艺术家聚合，支持专辑与曲目下钻。" next={["艺术家索引", "艺术家详情"]} />,
      },
      {
        path: "/library/artists/:id",
        element: <Placeholder page="P-05" title="艺术家详情" description="艺术家的专辑、曲目与简介。" next={["艺术家查询", "全部播放"]} />,
      },
      {
        path: "/library/genres",
        element: <Placeholder page="P-06" title="流派" description="按流派浏览曲库。" next={["流派聚合查询"]} />,
      },
      {
        path: "/playlists",
        element: <Placeholder page="P-07" title="播放列表" description="手动与智能播放列表管理。" next={["播放列表 CRUD", "拖拽排序", "导入导出"]} />,
      },
      {
        path: "/playlists/:id",
        element: <Placeholder page="P-07" title="播放列表详情" description="列表内曲目与批量操作。" next={["列表曲目查询", "重命名与删除"]} />,
      },
      {
        path: "/search",
        element: <Placeholder page="P-08" title="搜索" description="曲目、专辑、艺术家、歌词的统一搜索。" next={["FTS5 查询命令", "搜索建议与高亮"]} />,
      },
      {
        path: "/lyrics/:trackId",
        element: <Placeholder page="P-09" title="歌词编辑" description="逐行打轴、偏移与保存。" next={["LRC 编辑器", "音频时钟与预览"]} />,
      },
      {
        path: "/equalizer",
        element: <Placeholder page="P-10" title="均衡器" description="10 段均衡器与预设。" next={["DSP 图与预设", "实时参数下发"]} />,
      },
      {
        path: "/skins",
        element: <Placeholder page="P-11" title="皮肤" description="皮肤预览、安装与切换。" next={["biuskin 包解析", "主题令牌覆盖"]} />,
      },
      {
        path: "/plugins",
        element: <Placeholder page="P-12" title="插件" description="WASM/原生插件管理、权限与日志。" next={["插件 manifest", "启用/禁用与权限"]} />,
      },
      {
        path: "/settings/:section",
        element: <Placeholder page="P-13" title="设置" description="常规、播放、音频、音效、歌词、皮肤、插件、快捷键、媒体库、高级。" next={["设置分区导航", "settings.json 读写"]} />,
      },
      { path: "/settings", element: <Navigate to="/settings/general" replace /> },
      {
        path: "/tools",
        element: <Placeholder page="P-14" title="音频工具" description="格式转换、抓轨、文件整理与校验（P2）。" next={["工具入口占位"]} />,
      },
      { path: "/m0", element: <App /> },
    ],
  },
  { path: "*", element: <Navigate to="/" replace /> },
]);
