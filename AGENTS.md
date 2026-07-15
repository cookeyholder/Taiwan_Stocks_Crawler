
## 語言規範

所有輸出、PR message、commit message、程式碼註解都必須使用**臺灣繁體中文（zh-TW）**。

禁止用詞：反饋、其他非正規內部行話。避免中國慣用語（如默認、配置、資源庫、用戶、視圖、代碼、視頻、網絡、運行、優化）。

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:

- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them — don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:

- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it — don't delete it.

When your changes create orphans:

- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:

- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:

```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.

Always us zh-tw to write commit message, pr message and comments. And talk with me in zh-tw.

修改程式碼以後，先在本地端進行編譯和測試，確保沒有語法錯誤或測試失敗，再提交 PR。PR 描述應該清晰說明改動內容和目的，並且與相關的 issue 或任務關聯起來。

---

## 復盤記錄

### [2026-06-09] Tkinter 關閉時 pending after 定時器導致 Tcl 異常
- 背景：在 Tkinter GUI 中，使用 `after()` 定時輪詢任務佇列。當用戶關閉視窗時，會執行銷毀流程。
- 錯誤現象：關閉視窗後，控制台拋出 `invalid command name "xxx_process_queue" while executing "xxx_process_queue" ("after" script)` 的錯誤警告。
- 根因：錯誤地假設 `after_cancel` 一定能完全撤銷所有 pending 定時器。實際上，如果 timer 的時間已到並已移入 Tcl event queue 中排隊，`after_cancel` 無法將其撤銷。此時若立即調用 `root.destroy()` 銷毀 Tcl interpreter，當 event loop 隨後執行該 callback 時，就會因為找不到已被銷毀的 command 而報錯。
- 當下影響：雖然程式可正常結束，但控制台會殘留 `invalid command name` 的 Traceback 警告，影響使用者體驗與日誌清潔度。
- 預防措施（下次怎麼做）：
  - 對於 Tkinter 程式，退出時改用 `root.quit()` 結束 `mainloop()`，並在 `mainloop()` 退出後才在主程式進入點呼叫 `root.destroy()` 銷毀視窗與釋放 interpreter。這樣能確保 event loop 結束後才銷毀 Tcl 環境，徹底杜絕 pending timer 在銷毀後執行的 Race Condition。
- 新增/調整測試：N/A (GUI 環境測試在無 GUI 的 CI 中會跳過)。
- 流程更新：已修正 `gui.py` 的關閉邏輯與 `main()` 入口點。

## 相關模組與檔案

| 模組                           | 路徑                                       | 說明                                   |
| ------------------------------ | ------------------------------------------ | -------------------------------------- |
| `backtest::Strategy`           | `stock-core/src/backtest/mod.rs`           | 策略 trait：evaluate, id, min_history  |
| `backtest::BacktestEngine`     | `stock-core/src/backtest/engine.rs`        | 回測引擎：run_date, run_range          |
| `backtest::bollinger`          | `stock-core/src/backtest/bollinger.rs`     | 布林擠壓策略實作                       |
| `backtest::*` 型別             | `stock-core/src/models.rs`                 | SignalRecord, BacktestOutput, Signal   |
| `db::MarketRepository` 輔助查詢 | `stock-core/src/db/mod.rs`                | get_batch_ohlcv, get_active_symbols 等 |
| `pipeline::Pipeline::screen`   | `stock-core/src/pipeline/mod.rs`           | 已改用 BacktestEngine                   |
| CLI `backtest` 子命令          | `stock-cli/src/main.rs`                    | backtest --date / --start / --end      |

## Agent skills

### Issue tracker

Issues are tracked on GitHub Issues via the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

The five canonical triage roles use default labels. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context — one `CONTEXT.md` + `docs/adr/` at the repo root. See `docs/agents/domain.md`.

