# ai-risajuu-rs
Rust製のAIりさじゅう

# version
## v1.x
### v1.1.0
- 履歴リセット機能
### v1.0.0
- サーバー毎の記憶

## v0.x
### v0.5.1
- システム指示改善
### v0.5.0
- chat.rsモジュール分離
### v0.4.2
- load_text関数分離 (main.rs)
- ストリーミング応答仮対応
### v0.4.1
- system instruction対応
- history対応
### v0.4.0
- チャネル実装
### v0.3.1
- エラーハンドリング
- モジュール分割
### v0.3.0
- pure rust化（PyO3 & genai API → gemini-rs）
### v0.2.0
- rustfmt.toml設定
- PyO3導入（Gemini対応）
### v0.1.0
- serenity試験運用

# todo
[ ] メンション検出
[ ] thinking
[ ] Geminiタイムアウト設定・応答終了理由検査
[ ] リアクション付与機能
[ ] カスタムステータス
[ ] 画像認識・ドキュメント読み込み
[ ] メタデータ読み取り（ユーザー名・チャンネル名・投稿日時）
[ ] 関数呼び出し
[ ] README.md読み込み