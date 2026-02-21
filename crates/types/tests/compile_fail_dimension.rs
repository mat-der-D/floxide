// NOTE: trybuild は .stderr ファイルとコンパイラ出力を完全一致で比較する。
// Rust や typenum のバージョンアップでエラーメッセージが変わると、
// 「コンパイルが失敗すること」自体は正しくてもこのテストが壊れる。
//
// 対処: Rust バージョンアップ後は以下を実行して .stderr ファイルを再生成すること。
//   TRYBUILD=overwrite cargo test -p dugong-types
//
// .stderr ファイルは tests/compile_fail/*.stderr に配置されている。

#[test]
fn compile_fail_dimension_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/add_different_dims.rs");
    t.compile_fail("tests/compile_fail/sub_different_dims.rs");
}
