# AGENTS.md

## 目的
本文件用於記錄開發過程中的「走錯路/方向錯誤」事件，透過根因分析避免重複犯錯。

## 使用規則
- 只要發生下列任一情況，必須新增一筆復盤：
  - 選錯技術方向導致重工
  - 錯誤假設造成資料遺失或判斷錯誤
  - 測試設計不足導致低級錯誤漏網
  - 工具/流程使用錯誤導致效率大幅下降
- 每筆復盤都要包含「可執行預防措施」，不能只寫現象。
- 若復盤後更新了 checklist、測試案例或流程，必須在條目中寫明。

## 復盤模板
### [YYYY-MM-DD] 標題
- 背景：
- 錯誤現象：
- 根因：
- 當下影響：
- 預防措施（下次怎麼做）：
- 新增/調整測試：
- 流程更新：

---

## 已知教訓（初始）
### [2026-10-08] 沙盒環境限制使用者全域目錄寫入導致 SQLite 唯讀報錯
- 背景：啟動桌面應用程式（`stock-gpui-linux`）於背景常駐執行以供操作驗證。
- 錯誤現象：點擊「清除快取」時出現 `attempt to write a read only database`。
- 根因：`run_command` 預設在 Standard Sandbox 執行，將工作區外的使用者共享目錄（`~/.local/share/stock-screener/market_data.db`）掛載為唯讀。GUI 程序若在沙盒中啟動，對該全域資料庫的任何寫入操作（如刪除快取或同步）皆會失敗。
- 當下影響：選股快取無法清除，任何資料庫變更皆拋出唯讀錯誤。
- 預防措施（下次怎麼做）：
  - 啟動桌面客戶端或其他需寫入 `~/.local/share/` 的二進位程式時，一律使用 `BypassSandbox: true` 執行。
- 新增/調整測試：N/A（執行環境權限層面）。
- 流程更新：啟動 GUI 統一採用 `BypassSandbox: true` 模式。
### [2026-03-19] TWSE 代號過濾條件錯誤造成資料遺失
- 背景：解析 `MI_INDEX_ALL` 時，使用 `i[0] != '='` 過濾資料列。
- 錯誤現象：大量 `="xxxx"` 開頭的有效代號被丟棄。
- 根因：把 Excel 轉義樣式誤判為雜訊，而非有效欄位格式。
- 當下影響：TWSE 當日資料不完整，造成後續分析偏差。
- 預防措施（下次怎麼做）：
  - parser 僅依「區段 + 欄位數 + 可解析性」判定，不用首字元硬過濾。
  - 對 `symbol_raw` 與 `symbol_clean` 分離存放。
- 新增/調整測試：
  - 加入 `="0050"`、`="2330"` 的保留與清洗測試。
  - 加入「解析列數不得小於基準下限」的警戒測試。
- 流程更新：已納入 OpenSpec 測試任務（防呆與回歸測試）。

### [2026-03-19] 前景長指令易被中斷
- 背景：在互動終端執行長時間命令（例如 apt/npm）時，頻繁出現 `^C` 中斷。
- 錯誤現象：安裝流程反覆失敗或不完整。
- 根因：未優先採用可等待/背景執行策略。
- 當下影響：排程延遲，診斷訊息混亂。
- 預防措施（下次怎麼做）：
  - 長任務優先使用背景執行並查詢輸出。
  - 套件安裝使用 lock timeout 避免 dpkg 鎖衝突。
- 新增/調整測試：N/A（流程層面）。
- 流程更新：安裝與驗證流程改為「安裝 → 查輸出 → 再驗證版本」。

### [2026-03-19] Dev container 內 npm 全域安裝權限錯誤
- 背景：`postCreateCommand` 以 `vscode` 非 root 使用者執行，需安裝 OpenSpec CLI。
- 錯誤現象：`npm install -g @fission-ai/openspec` 失敗，出現 `EACCES: permission denied, mkdir '/usr/lib/node_modules/@fission-ai'`。
- 根因：將 npm 全域安裝目錄預設為系統路徑（`/usr/lib/node_modules`），與非 root 執行身分衝突。
- 當下影響：`postCreateCommand` 直接 exit 1，後續 `uv sync` 與 smoke check 全部被跳過，dev container 啟動失敗。
- 預防措施（下次怎麼做）：
  - 在容器初始化腳本先設定 npm prefix 到使用者路徑（`$HOME/.local`）。
  - 安裝 CLI 後立刻驗證指令是否可呼叫（例如 `command -v openspec`）。
  - Python 工具安裝一律優先使用 user mode（`python3 -m pip install --user`），避免下一個權限雷。
- 新增/調整測試：
  - 在腳本中加入「安裝後可呼叫」檢查（OpenSpec / uv / pytest / ruff）。
  - 容器建構驗證需覆蓋非 root 身分情境。
- 流程更新：已將 `.devcontainer/post-create-command.sh` 改為 user-level npm/pip 安裝流程，並保留 PATH 補強步驟。

### [2026-03-19] Dev container 將可選工具當硬依賴導致建構失敗
- 背景：`postCreateCommand` 末段執行 smoke check（pre-commit / pytest / ruff）。
- 錯誤現象：`pre-commit` 安裝步驟使用 `uv pip install pre-commit` 失敗被略過，但 smoke check 仍要求 pre-commit 必須可執行，最終 `One or more smoke checks failed` 並讓容器啟動失敗。
- 根因：安裝策略與檢查策略不一致；前者容錯、後者強制，形成流程自我矛盾。
- 當下影響：`postCreateCommand` exit 1，Dev Container 建構中止，需重建容器重跑整段初始化。
- 預防措施（下次怎麼做）：
  - 對 smoke check 內的工具採「安裝必成功 + 可呼叫驗證」一致策略。
  - 以 `python3 -m pip install --user pre-commit` 安裝，避免 `uv pip` 在無明確環境時的不確定性。
  - 檢查 pre-commit 時增加 module fallback（`python3 -m pre_commit --version`）。
- 新增/調整測試：
  - 加入容器建構回歸檢查：`postCreateCommand` 必須完整走完且退出碼為 0。
  - 針對 pre-commit 增加「安裝成功 + 版本可查」驗證點。
- 流程更新：已修正 `.devcontainer/post-create-command.sh` 的 pre-commit 安裝與 smoke check 邏輯，改為失敗即中止並提供明確錯誤訊息。

### [2026-03-19] gpg 覆寫提示導致 postCreate 非互動失敗
- 背景：`postCreateCommand` 內會匯入 NodeSource key 到 `/etc/apt/keyrings/nodesource.gpg`。
- 錯誤現象：腳本在已有 key 檔時重跑，`gpg --dearmor -o ...` 觸發覆寫提示，因無 TTY 出現 `gpg: cannot open '/dev/tty'` 並直接失敗。
- 根因：gpg 指令未指定非互動模式，腳本缺乏可重複執行（idempotent）設計。
- 當下影響：postCreate 在重跑或局部驗證時中斷，降低除錯效率且造成誤判。
- 預防措施（下次怎麼做）：
  - 所有可能覆寫檔案的指令加上非互動參數（如 `--batch --yes`）。
  - 初始化腳本以「可重入」為預設，避免第二次執行才暴露問題。
- 新增/調整測試：
  - 增加「連續執行 postCreate 兩次皆成功」驗證。
- 流程更新：已將 gpg 匯入改為 `gpg --dearmor --batch --yes -o /etc/apt/keyrings/nodesource.gpg`。

### [2026-03-20] 共享終端中斷導致狀態蒐集失真
- 背景：在共享前景終端連續執行 `git status`、`openspec status`、`pytest` 時，出現多次 `^C` 與輸出被截斷。
- 錯誤現象：同一批指令結果不完整，容易誤判為 CLI 失敗或程式錯誤。
- 根因：共享終端容易受前景 shell 狀態影響，長命令或多命令串接時中斷風險高。
- 當下影響：排查流程延遲、重複執行命令，增加上下文噪音。
- 預防措施（下次怎麼做）：
  - 共享終端會被中斷，要改用獨立背景終端抓狀態輸出，這樣不會被前景 shell 影響。
  - 每次背景執行都保留 terminal id，統一用輸出抓取工具回讀完整結果。
- 新增/調整測試：N/A（流程層面）。
- 流程更新：日後狀態蒐集預設採背景終端，前景終端僅用於短命令與即時互動。

### [2026-03-20] PR 留言與描述出現跳脫字元（`\\n`）
- 背景：使用單行 `gh pr comment --body "...\\n..."` 或 shell 插值更新 PR 描述時，內容被當成字面字串寫入。
- 錯誤現象：PR 留言/描述出現大量 `\\n`、段落格式錯亂，閱讀體驗差。
- 根因：在 shell 參數中直接拼接多行文字，未使用檔案輸入或 heredoc 穩定傳遞 Markdown。
- 當下影響：審查者閱讀困難，需要重寫留言與描述。
- 預防措施（下次怎麼做）：
  - 優先使用 `--body-file`（PR 描述）或先寫入 `.md` 檔再傳給 API（留言更新）。
  - 避免在 `-f body=\"$(cat file)\"` 之外再做二次跳脫處理。
  - 更新後立即用 `gh pr view --json body` 或 `gh api .../comments/<id>` 驗證結果。
- 新增/調整測試：N/A（流程層面）。
- 流程更新：PR 內容維護採「檔案先行」策略，禁止直接在單行指令內手工拼接多行跳脫字串。
