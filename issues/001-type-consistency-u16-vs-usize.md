# Issue #001: 型の統一 (u16 vs usize)

## 概要

`cursor.rs` で `u16` と `usize` の型が混在しており、頻繁な型変換が発生している。
一貫性のために型を統一すべき。

## 現状

### Cursor の内部型
```rust
pub struct Cursor {
    x: u16,
    y: u16,
    row_offset: u16,
    col_offset: u16,
}
```

### 外部からの引数
- `buffer_len: usize` (Vec の長さ)
- `max_rows: u16` (ターミナルサイズ)
- `line_len: usize` (文字列の長さ)

### 頻繁な型変換の例

```rust
pub fn move_down(&mut self, max_rows: u16, buffer_len: usize) {
    let current_file_row = self.row_offset + self.y - 1;  // u16
    let last_row = (buffer_len as u16).saturating_sub(1);  // usize → u16
}

pub fn move_to_bottom(&mut self, buffer_len: usize, editor_rows: u16) {
    let last_line = buffer_len.saturating_sub(1) as u16;  // usize → u16
    // ...
}
```

## 問題点

1. **型変換が散在**: `as u16`, `as usize` が多数存在
2. **可読性の低下**: 型変換により意図が不明確になる箇所がある
3. **潜在的なバグ**: 大きな値の切り捨てリスク（現実的には発生しないが）

## 解決案

### Option A: すべて `usize` に統一

**メリット**:
- Rust の慣例に従う（Vec, String の len() は usize を返す）
- 型変換が減る（buffer_len, line_len との統一）
- 将来的な拡張性（65535行以上のファイルにも対応可能）

**デメリット**:
- ターミナルサイズ（termion は u16 を返す）との変換が必要
- メモリ使用量がわずかに増加（64bit 環境で u16 → usize）

**変更箇所**:
- `Cursor` 構造体のフィールド: `u16` → `usize`
- `move_down`, `move_to_bottom` などのシグネチャ

### Option B: すべて `u16` に統一

**メリット**:
- ターミナルサイズと一致
- メモリ効率（usize より小さい）
- 65535行までのファイルで十分（実用上問題なし）

**デメリット**:
- buffer_len, line_len との変換が必要
- Rust の慣例から外れる

**変更箇所**:
- `move_down`, `move_to_bottom` などの引数: `usize` → `u16`
- 呼び出し側で `buffer.len() as u16` が必要

## 推奨

**Option A (usize に統一)** を推奨

**理由**:
1. Rust のエコシステムと一貫性がある
2. 将来の拡張性（大きなファイル、UTF-8 対応時のバイト位置計算など）
3. 型変換の削減（Vec/String との統一）

## 影響範囲

- `src/cursor.rs`: Cursor 構造体、全メソッド
- `src/main.rs`: cursor を使用している箇所
- `src/screen.rs`: cursor の座標を使用している箇所
- テストコード: cursor 関連のテスト

## 実装時の注意点

1. **段階的に移行**: まず Cursor 構造体のフィールドを変更し、コンパイルエラーを解消
2. **テストで検証**: 既存の 45 個のテストが通ることを確認
3. **境界チェック**: usize でもオーバーフローの可能性を考慮（実用上は問題ないが）

## 優先度

**低** - 現時点で動作に問題はなく、Phase 6 以降で対応可能

## 関連

- Phase 7 (UTF-8 対応) で文字位置とバイト位置の変換が必要になる際に、型の統一があると実装しやすい
