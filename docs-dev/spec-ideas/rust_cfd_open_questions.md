# Rust CFD 未決定の設計課題

議論・検証が必要な設計課題の一覧。決定が固まった項目は個別ファイルに移し、ここから削除する。

| 課題 | 選択肢候補 | 備考 |
|------|----------|------|
| 辞書システム | serde + TOML/YAML / 独自フォーマット | 設定ファイルの表現と読み込み |
| フィールド間の境界条件依存 | evaluate にコンテキスト渡し / ソルバーレベル管理 | 入口BCが他フィールドに依存する場合等 |
| PhysicalBC の行列寄与 | trait メソッド追加 / 別 trait | 陰的離散化での境界条件の行列・ソース項への寄与 |
| MPI なしシリアルビルド | feature flag / SingleWorld | rsmpi への依存を排除したビルドの要否 |

## 決定済み

- [実行時選択メカニズム・ビルドモデル](./rust_cfd_runtime_selection.md)
- [テンソル型システム・FieldValue trait 階層](./rust_cfd_tensor_types.md)
- [メッシュ・フィールド・並列化](./rust_cfd_mesh_field_parallel.md)
- [3層メッシュアーキテクチャ（PrimitiveMesh / PolyMesh / FvMesh）](./mesh_architecture.md)
- [objectRegistry 相当の設計](./rust_cfd_object_registry.md)
