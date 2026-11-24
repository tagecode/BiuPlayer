# L1-04 搜索增强：艺术家/专辑、拼音与 FTS v2

- 日期：2026-11-24
- 范围：`docs/Mvp.md` L1-04；对应 `F-LIB-09`（搜索）与 `F-LIB-11`（拼音）
- 状态：通过

## 1. 实现

### 1.1 schema v2

`biu-library` 的 `SCHEMA_VERSION` 升到 2，新增一次事务迁移：

- `ALTER TABLE tracks ADD COLUMN search_text TEXT NOT NULL DEFAULT ''`。
- 删除 v1 的 `tracks_fts` 与三个触发器，重建为
  `tracks_fts(title, search_text, path, content='tracks', content_rowid='id', tokenize='unicode61')`。
- 重建 `tracks_ai` / `tracks_ad` / `tracks_au` 三个触发器，并在迁移中 `rebuild` 一次。
- 旧库在迁移前复制为 `library.db.bak`；`schema_version > 2` 仍拒绝降级。

### 1.2 search_text 生成

对 `title`、`artist`、`album` 分别生成：

| 形式 | 示例（晴天） |
|---|---|
| 原文 | `晴天` |
| 全拼（音节空格） | `qing tian` |
| 首字母（空格） | `q t` |
| 首字母连写 | `qt` |
| 全拼连写 | `qingtian` |

最终 `search_text` 用空格连接各段，写入 `tracks.search_text`。英文/数字保持原文，由 `unicode61` 正常分词。

### 1.3 查询

`Database::search(query, limit)`：

- 按空白拆词，每个词只保留字母、数字、`_`、`-`，追加 `*` 前缀，用 `AND` 连接。
- 走 `tracks_fts MATCH`，`ORDER BY bm25(tracks_fts)`。
- 空查询直接返回空数组，避免 FTS 语法错误。

### 1.4 前端

`/library` 增加搜索框：

- 输入后 200ms 防抖调用 `library_search`。
- 有查询时展示搜索结果，否则展示曲目列表。
- 无结果时提示“标题、艺术家、专辑或拼音首字母”。

## 2. 验证

| 项目 | 结果 |
|---|---|
| 迁移测试 | 4/4 通过，fresh 库版本为 2，重复打开不重复迁移 |
| 扫描测试 | 3/3 通过 |
| 拼音/字段搜索 | 8 个查询全部命中：`qingtian`、`qt`、`zhoujielun`、`zjl`、`yehuimei`、`yhm`、`周杰伦`、`晴天` |
| 10,000 曲目 FTS 查询 | 约 1ms（沿用 L1-03 基准） |
| `cargo test --workspace` | 全部通过 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 通过 |
| `cargo fmt --all --check` | 通过 |
| `pnpm build` | 通过（JS 345.59 kB / gzip 109.91 kB） |

## 3. 已知限制

- 未引入 jieba；中文按整段与拼音索引，不做语义分词。
- 未实现 `artist:`、`album:` 字段前缀与 `LIKE` 回退，列为 P1。
- schema v2 迁移只把旧行的 `search_text` 置空；旧曲目需下一次扫描（mtime/size 未变也会因 `search_text` 为空而重读标签）才会获得拼音索引。
- 多音字使用 `pinyin` crate 的默认读音；姓氏/地名多音字可能不准确。
- 尚未在 10 万真实曲目上测搜索延迟。

## 4. 相关提交

- `feat(library): pinyin-aware FTS search with schema v2`
