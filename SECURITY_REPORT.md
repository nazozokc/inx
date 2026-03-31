# ox セキュリティ診断レポート

## 概要

本レポートは ox パッケージマネージャーのセキュリティ問題を重要度別に分類し、各問題の修正方針をまとめたものである。

---

## CRITICAL

### 1. registry URL のバリデーション不足

**場所**: `src/utils/config.ts`

**問題**: `loadConfig()` でロードしたレジストリURLに対してスキーマ検証が行われていない。

**影響**: `http://`、`file://`、`git://` などの不安全プロトコルや悪意のあるURLを指定された場合、git clone 時に任意のコマンドを実行される可能性がある。

**修正方針**:

```ts
function validateRegistryUrl(url: string): boolean {
  try {
    const parsed = new URL(url);
    return parsed.protocol === 'https:' || parsed.protocol === 'ssh:';
  } catch {
    return false;
  }
}
```

`http://` や `file://`、`git://` は弾く。

---

## HIGH

### 2. TOCTOU (Time-of-Check to Time-of-Use)

**場所**: `src/utils/packages.ts:95-137`

**問題**: `installPackage()` で `validateNoSymlinks()` によりチェックを行った後、`fs.cp()` でコピーを行っている間に悪意のあるシンボリックリンクへ書き換わる可能性がある。

**影響**: コピー先に任意のファイルを上書きされる。

**修正方針**: `fs.cp` の `dereference` オプションを `false`（デフォルト）のまま使いつつ、コピー後に目的地のシンボリックリンクを再チェックするか、あるいは `fs.cp` の代わりにシンボリックリンクを追わない独自コピー実装にする。

根本的には「チェックしてからコピー」ではなく「コピーしながらチェック」する構造にするのがベスト。

### 3. uninstallPackage のパストラバーサル

**場所**: `src/utils/packages.ts:139-150`

**問題**: `uninstallPackage()` にパッケージ名のバリデーションがない。

**影響**: `../../../etc/passwd` などのパスで任意のパスを削除される可能性がある。

**修正方針**:

```ts
export async function uninstallPackage(name: string): Promise<void> {
  if (!validatePackageName(name)) {
    throw new Error(`Invalid package name '${name}'`);
  }
  // ...
}
```

---

## MEDIUM

### 4. compareVersions のプレリリース剥ぎ取り

**場所**: `src/commands/upgrade.ts:6-24`

**問題**: `VERSION_TAG_REGEX = /^v?\d+\.\d+\.\d+$/` でフィルタ済みのタグだけが渡るが、この設計が今後変更された場合に脆弱化する可能性がある。

**影響**: プレリリースバージョン（例: `v1.0.0-rc.1`）がバージョン比較に用いられる。

**修正方針**: regex を `^v?\d+\.\d+\.\d+$` のままにしてこれに依存し続けることをコメントで明示しておく：

```ts
// NOTE: Only VERSION_TAG_REGEX-matched tags are passed here.
// Pre-release identifiers (e.g. v1.0.0-rc.1) are excluded upstream.
```

### 5. デバイスファイル検出の欠如

**場所**: `src/utils/packages.ts:22-53`

**問題**: `listInstalledPackages()` でディレクトリ・ファイル以外（デバイスファイル、FIFO、ソケット）を検出していない。

**影響**: 悪意のあるデバイスファイルが読み取り時にシステムを不安定にする。

**修正方針**:

```ts
if (!entry.isDirectory() && !entry.isFile()) {
  return false; // デバイスファイル、FIFO、ソケットを拒否
}
```

---

## LOW

### 6. git タイムアウト未設定

**場所**: `src/utils/registry.ts:5-7`

**問題**: `simple-git` にタイムアウトが設定されていない。

**影響**: git 操作が長時間ブロックされる可能性がある。

**修正方針**:

```ts
function getGit(workingDir?: string): SimpleGit {
  return simpleGit(workingDir, { timeout: { block: 30000 } });
}
```

---

## 推奨される修正順序

1. **最初**: CRITICAL - registry URL 検証
2. **次に**: HIGH - uninstallPackage パストラバーサル
3. **次に**: HIGH - TOCTOU
4. その後: MEDIUM〜LOW の項目

全体として、コアのバリデーション（パッケージ名・タグ名）はちゃんとはいっているし、`symlink` チェックも気を配った形跡がある。

ただし「レジストリ URL そのものを信用しきっている」点が一番やばくて、ここが起点になると他の防御が全部無意味になる。

まず registry URL 検証から直すことをおすすめする。