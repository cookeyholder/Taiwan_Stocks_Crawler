use gpui_kit::base::Disableable as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::progress::Progress;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AppContext, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _, Window, div,
    px, rgb,
};

use stock_core::models::GapReport;

use crate::state::CoreState;

fn get_today_str() -> String {
    let tz_offset = chrono::FixedOffset::east_opt(8 * 3600).unwrap();
    chrono::Utc::now()
        .with_timezone(&tz_offset)
        .format("%Y-%m-%d")
        .to_string()
}

pub struct DownloadView {
    pub core_state: CoreState,
    pub start_date_input: Entity<InputState>,
    pub end_date_input: Entity<InputState>,
    pub is_downloading: bool,
    pub is_diagnosing: bool,
    pub progress_ratio: f32,
    pub status_text: String,
    pub db_summary: String,
    pub gap_report: Option<GapReport>,

    // 券商日報表相關狀態
    pub broker_date_input: Entity<InputState>,
    pub is_broker_downloading: bool,
    pub broker_progress_ratio: f32,
    pub broker_status_text: String,
}

impl DownloadView {
    pub fn new(core_state: CoreState, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let today = get_today_str();
        let start_date_input = cx.new(|cx| InputState::new(window, cx).default_value("2026-07-01"));
        let end_date_input = cx.new(|cx| InputState::new(window, cx).default_value(&today));
        let broker_date_input = cx.new(|cx| InputState::new(window, cx).default_value(&today));

        for input_field in [&start_date_input, &end_date_input] {
            cx.subscribe(
                input_field,
                |this: &mut Self, _, event: &gpui_kit::component::input::InputEvent, cx| {
                    if matches!(
                        event,
                        gpui_kit::component::input::InputEvent::PressEnter { .. }
                    ) {
                        this.start_download(cx);
                    }
                },
            )
            .detach();
        }

        cx.subscribe(
            &broker_date_input,
            |this: &mut Self, _, event: &gpui_kit::component::input::InputEvent, cx| {
                if matches!(
                    event,
                    gpui_kit::component::input::InputEvent::PressEnter { .. }
                ) {
                    this.start_broker_download(cx);
                }
            },
        )
        .detach();

        let mut view = Self {
            core_state,
            start_date_input,
            end_date_input,
            is_downloading: false,
            is_diagnosing: false,
            progress_ratio: 0.0,
            status_text: "就緒".into(),
            db_summary: "讀取中...".into(),
            gap_report: None,

            broker_date_input,
            is_broker_downloading: false,
            broker_progress_ratio: 0.0,
            broker_status_text: "就緒（可下載 TWSE 上市普通股券商分點買賣明細）".into(),
        };
        view.refresh_db_summary(cx);
        view
    }

    pub fn refresh_db_summary(&mut self, cx: &mut Context<Self>) {
        let core = self.core_state.clone();
        cx.spawn(async move |this, cx| {
            let res = core.get_date_range().await;
            let _ = this.update(cx, |view, cx| {
                match res {
                    Ok((Some(start), Some(end), count)) => {
                        view.db_summary = format!(
                            "資料庫狀態：記錄區間 {} ~ {}，共 {} 筆記錄",
                            start, end, count
                        );
                    }
                    Ok((_, _, count)) => {
                        view.db_summary = format!("資料庫狀態：共 {} 筆記錄", count);
                    }
                    Err(e) => {
                        view.db_summary = format!("無法讀取資料庫狀態：{}", e);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn run_download(&mut self, start_date: String, end_date: String, cx: &mut Context<Self>) {
        if self.is_downloading {
            return;
        }
        self.is_downloading = true;
        self.progress_ratio = 0.0;
        self.status_text = format!("開始下載區間 {} ~ {}...", start_date, end_date);
        cx.notify();

        let core = self.core_state.clone();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(usize, usize, String)>();

        cx.spawn(async move |this, cx| {
            let mut dl_task = crate::state::TOKIO_RUNTIME.spawn(async move {
                core.batch_download(&start_date, &end_date, move |done, total, date| {
                    let _ = tx.send((done, total, date));
                })
                .await
            });

            loop {
                tokio::select! {
                    msg = rx.recv() => {
                        if let Some((done, total, date)) = msg {
                            let _ = this.update(cx, |view, cx| {
                                view.progress_ratio = if total > 0 { done as f32 / total as f32 } else { 0.0 };
                                let pct = (view.progress_ratio * 100.0) as u32;
                                view.status_text = format!("正在下載 {}（進度: {}/{}，{}%）", date, done, total, pct);
                                cx.notify();
                            });
                        }
                    }
                    res = &mut dl_task => {
                        let final_res = match res {
                            Ok(inner) => inner,
                            Err(e) => Err(format!("背景任務異常：{}", e)),
                        };
                        let _ = this.update(cx, |view, cx| {
                            view.is_downloading = false;
                            view.progress_ratio = 1.0;
                            match final_res {
                                Ok(msg) => {
                                    view.status_text = msg;
                                    view.refresh_db_summary(cx);
                                }
                                Err(e) => {
                                    view.status_text = format!("下載失敗：{}", e);
                                }
                            }
                            cx.notify();
                        });
                        break;
                    }
                }
            }
        })
        .detach();
    }

    pub fn start_download(&mut self, cx: &mut Context<Self>) {
        if self.is_downloading {
            return;
        }
        let start_date = self.start_date_input.read(cx).value().trim().to_string();
        let end_date = self.end_date_input.read(cx).value().trim().to_string();

        if start_date.is_empty() || end_date.is_empty() {
            self.status_text = "請輸入有效的起始與結束日期".into();
            cx.notify();
            return;
        }

        self.run_download(start_date, end_date, cx);
    }

    pub fn fill_to_today(&mut self, cx: &mut Context<Self>) {
        if self.is_downloading {
            return;
        }
        self.is_downloading = true;
        self.progress_ratio = 0.0;
        self.status_text = "檢查最新資料日期中...".into();
        cx.notify();

        let core = self.core_state.clone();
        cx.spawn(async move |this, cx| {
            let latest_opt = core.get_latest_trade_date().await;
            let today = get_today_str();

            let target = match latest_opt {
                Ok(Some(latest)) => {
                    if latest >= today {
                        Err(format!("資料已是最新（最後記錄日：{}）", latest))
                    } else {
                        let next_day = chrono::NaiveDate::parse_from_str(&latest, "%Y-%m-%d")
                            .map(|d| {
                                (d + chrono::Duration::days(1))
                                    .format("%Y-%m-%d")
                                    .to_string()
                            })
                            .unwrap_or_else(|_| today.clone());
                        Ok((next_day, today))
                    }
                }
                Ok(None) => Err("資料庫尚無資料，請先手動指定起始與結束日期進行下載。".into()),
                Err(e) => Err(format!("查詢最新交易日失敗：{}", e)),
            };

            let _ = this.update(cx, |view, cx| {
                view.is_downloading = false;
                match target {
                    Ok((start_date, end_date)) => {
                        view.run_download(start_date, end_date, cx);
                    }
                    Err(msg) => {
                        view.status_text = msg;
                        cx.notify();
                    }
                }
            });
        })
        .detach();
    }

    pub fn delete_range(&mut self, cx: &mut Context<Self>) {
        if self.is_downloading {
            return;
        }
        let start_date = self.start_date_input.read(cx).value().trim().to_string();
        let end_date = self.end_date_input.read(cx).value().trim().to_string();

        if start_date.is_empty() || end_date.is_empty() {
            self.status_text = "請輸入欲刪除的起始與結束日期".into();
            cx.notify();
            return;
        }

        self.is_downloading = true;
        self.status_text = format!("正在刪除區間 {} ~ {} 的資料...", start_date, end_date);
        cx.notify();

        let core = self.core_state.clone();
        cx.spawn(async move |this, cx| {
            let res = core.delete_date_range(&start_date, &end_date).await;
            let _ = this.update(cx, |view, cx| {
                view.is_downloading = false;
                match res {
                    Ok(msg) => {
                        view.status_text = msg;
                        view.refresh_db_summary(cx);
                    }
                    Err(e) => {
                        view.status_text = format!("刪除失敗：{}", e);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn cancel_download(&mut self, cx: &mut Context<Self>) {
        self.core_state.cancel();
        self.status_text = "已送出取消請求...".into();
        cx.notify();
    }

    pub fn diagnose_gaps(&mut self, cx: &mut Context<Self>) {
        if self.is_downloading || self.is_diagnosing {
            return;
        }
        let start_date = self.start_date_input.read(cx).value().trim().to_string();
        let end_date = self.end_date_input.read(cx).value().trim().to_string();

        if start_date.is_empty() || end_date.is_empty() {
            self.status_text = "請輸入有效的起始與結束日期進行診斷".into();
            cx.notify();
            return;
        }

        self.is_diagnosing = true;
        self.status_text = format!(
            "正在診斷區間 {} ~ {} 的缺漏平日交易日...",
            start_date, end_date
        );
        cx.notify();

        let core = self.core_state.clone();
        cx.spawn(async move |this, cx| {
            let res = core.diagnose_gaps(&start_date, &end_date).await;
            let _ = this.update(cx, |view, cx| {
                view.is_diagnosing = false;
                match res {
                    Ok(report) => {
                        let missing_num = report.missing_dates.len();
                        if missing_num == 0 {
                            view.status_text = format!(
                                "診斷完成：區間內共 {} 個平日均已有資料，無缺漏交易日！",
                                report.total_workdays
                            );
                        } else {
                            view.status_text = format!(
                                "診斷完成：共 {} 個平日中，發現 {} 天缺漏交易日待修補",
                                report.total_workdays, missing_num
                            );
                        }
                        view.gap_report = Some(report);
                    }
                    Err(e) => {
                        view.status_text = format!("診斷失敗：{}", e);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn repair_gaps(&mut self, cx: &mut Context<Self>) {
        if self.is_downloading {
            return;
        }
        let missing_dates = match &self.gap_report {
            Some(report) if !report.missing_dates.is_empty() => report.missing_dates.clone(),
            _ => {
                self.status_text = "目前無缺漏日可修補，請先點擊「診斷缺漏交易日」".into();
                cx.notify();
                return;
            }
        };

        self.is_downloading = true;
        self.progress_ratio = 0.0;
        self.status_text = format!("開始定向修補共 {} 個缺漏交易日...", missing_dates.len());
        cx.notify();

        let core = self.core_state.clone();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(usize, usize, String)>();

        cx.spawn(async move |this, cx| {
            let missing_clone = missing_dates.clone();
            let task_missing = missing_clone.clone();
            let mut repair_task = crate::state::TOKIO_RUNTIME.spawn(async move {
                core.repair_gaps(&task_missing, move |done, total, date| {
                    let _ = tx.send((done, total, date));
                })
                .await
            });

            loop {
                tokio::select! {
                    msg = rx.recv() => {
                        if let Some((done, total, date)) = msg {
                            let _ = this.update(cx, |view, cx| {
                                view.progress_ratio = if total > 0 { done as f32 / total as f32 } else { 0.0 };
                                let pct = (view.progress_ratio * 100.0) as u32;
                                view.status_text = format!("正在修補 {}（進度: {}/{}，{}%）", date, done, total, pct);
                                cx.notify();
                            });
                        }
                    }
                    res = &mut repair_task => {
                        let final_res = match res {
                            Ok(inner) => inner,
                            Err(e) => Err(crate::state::RepairError {
                                message: format!("背景修補任務異常：{}", e),
                                remaining_dates: missing_clone.clone(),
                                inserted_days: 0,
                                total_records: 0,
                            }),
                        };
                        let _ = this.update(cx, |view, cx| {
                            view.is_downloading = false;
                            view.progress_ratio = 1.0;
                            match final_res {
                                Ok(msg) => {
                                    view.status_text = msg;
                                    view.gap_report = None;
                                    view.refresh_db_summary(cx);
                                }
                                Err(err) => {
                                    view.status_text = format!("修補失敗：{}", err.message);
                                    if let Some(ref mut report) = view.gap_report {
                                        if err.remaining_dates.is_empty() {
                                            view.gap_report = None;
                                        } else {
                                            report.missing_dates = err.remaining_dates;
                                            report.recorded_dates += err.inserted_days;
                                        }
                                    }
                                    view.refresh_db_summary(cx);
                                }
                            }
                            cx.notify();
                        });
                        break;
                    }
                }
            }
        })
        .detach();
    }

    pub fn start_broker_download(&mut self, cx: &mut Context<Self>) {
        if self.is_broker_downloading {
            return;
        }
        let target_date = self.broker_date_input.read(cx).value().trim().to_string();
        if target_date.is_empty() {
            self.broker_status_text = "請輸入有效的交易日期（YYYY-MM-DD）".into();
            cx.notify();
            return;
        }

        self.is_broker_downloading = true;
        self.broker_progress_ratio = 0.0;
        self.broker_status_text = format!("開始下載 {} 券商日報表（TWSE 上市普通股）...", target_date);
        cx.notify();

        let core = self.core_state.clone();
        let (tx, mut rx) =
            tokio::sync::mpsc::unbounded_channel::<stock_core::pipeline::broker::BrokerProgress>();

        cx.spawn(async move |this, cx| {
            let mut dl_task = crate::state::TOKIO_RUNTIME.spawn(async move {
                core.fetch_broker_trades(&target_date, None, move |prog| {
                    let _ = tx.send(prog);
                })
                .await
            });

            loop {
                tokio::select! {
                    msg = rx.recv() => {
                        if let Some(prog) = msg {
                            let _ = this.update(cx, |view, cx| {
                                view.broker_progress_ratio = if prog.total_stocks > 0 {
                                    prog.current_index as f32 / prog.total_stocks as f32
                                } else {
                                    0.0
                                };
                                let pct = (view.broker_progress_ratio * 100.0) as u32;
                                view.broker_status_text = format!(
                                    "[{:>3}%] ({}/{}) 個股: {} | 成功: {} | 略過: {} | 失敗: {} | {}",
                                    pct,
                                    prog.current_index,
                                    prog.total_stocks,
                                    prog.current_symbol,
                                    prog.success_count,
                                    prog.skipped_count,
                                    prog.failed_count,
                                    prog.message
                                );
                                cx.notify();
                            });
                        }
                    }
                    res = &mut dl_task => {
                        let final_res = match res {
                            Ok(inner) => inner,
                            Err(e) => Err(format!("背景任務異常：{}", e)),
                        };
                        let _ = this.update(cx, |view, cx| {
                            view.is_broker_downloading = false;
                            view.broker_progress_ratio = 1.0;
                            match final_res {
                                Ok(summary) => {
                                    let status_str = if summary.cancelled {
                                        "使用者手動中斷（已保留進度以利續傳）"
                                    } else {
                                        "全部下載完成"
                                    };
                                    view.broker_status_text = format!(
                                        "執行結束：{}（總標的: {}，成功入庫: {}，斷點略過: {}，失敗: {}）",
                                        status_str,
                                        summary.total_stocks,
                                        summary.success_count,
                                        summary.skipped_count,
                                        summary.failed_count
                                    );
                                }
                                Err(e) => {
                                    view.broker_status_text = format!("下載失敗：{}", e);
                                }
                            }
                            cx.notify();
                        });
                        break;
                    }
                }
            }
        })
        .detach();
    }

    pub fn cancel_broker_download(&mut self, cx: &mut Context<Self>) {
        self.core_state.cancel();
        self.broker_status_text = "已送出取消請求，正在儲存進度以利續傳...".into();
        cx.notify();
    }
}

impl Render for DownloadView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_dl = self.is_downloading;
        let is_diag = self.is_diagnosing;
        let missing_count = self
            .gap_report
            .as_ref()
            .map(|r| r.missing_dates.len())
            .unwrap_or(0);

        div()
            .size_full()
            .flex()
            .flex_col()
            .p_4()
            .gap_4()
            .child(
                div()
                    .p_3()
                    .border_1()
                    .border_color(rgb(0x313244))
                    .rounded_md()
                    .bg(rgb(0x181825))
                    .text_color(rgb(0x89b4fa))
                    .child(self.db_summary.clone()),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(div().text_sm().text_color(rgb(0xa6adc8)).child("起始:"))
                            .child(
                                div()
                                    .w(px(130.))
                                    .child(Input::new(&self.start_date_input).id("dl-start-date")),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(div().text_sm().text_color(rgb(0xa6adc8)).child("結束:"))
                            .child(
                                div()
                                    .w(px(130.))
                                    .child(Input::new(&self.end_date_input).id("dl-end-date")),
                            ),
                    )
                    .child(
                        Button::new("btn-start-download")
                            .label(if is_dl {
                                "下載中..."
                            } else {
                                "下載指定區間"
                            })
                            .primary()
                            .disabled(is_dl)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.start_download(cx);
                            })),
                    )
                    .child(
                        Button::new("btn-sync-today")
                            .label("一鍵補齊至今日")
                            .secondary()
                            .disabled(is_dl)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.fill_to_today(cx);
                            })),
                    )
                    .child(
                        Button::new("btn-diagnose-gaps")
                            .label(if is_diag {
                                "診斷中..."
                            } else {
                                "🔍 診斷缺漏交易日"
                            })
                            .secondary()
                            .disabled(is_dl || is_diag)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.diagnose_gaps(cx);
                            })),
                    )
                    .child(
                        Button::new("btn-repair-gaps")
                            .label(format!("🔧 一鍵修補缺漏 ({})", missing_count))
                            .when(missing_count > 0, |b| b.primary())
                            .when(missing_count == 0, |b| b.secondary())
                            .disabled(is_dl || is_diag || missing_count == 0)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.repair_gaps(cx);
                            })),
                    )
                    .child(
                        Button::new("btn-delete-range")
                            .label("刪除指定區間")
                            .danger()
                            .disabled(is_dl)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.delete_range(cx);
                            })),
                    )
                    .when(is_dl, |this| {
                        this.child(
                            Button::new("btn-cancel-download")
                                .label("取消作業")
                                .danger()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.cancel_download(cx);
                                })),
                        )
                    }),
            )
            .when_some(self.gap_report.clone(), |this, report| {
                if report.missing_dates.is_empty() {
                    this.child(
                        div()
                            .p_2p5()
                            .rounded_md()
                            .border_1()
                            .border_color(rgb(0x16a34a))
                            .bg(rgb(0x0f291e))
                            .text_color(rgb(0x4ade80))
                            .text_sm()
                            .child(format!(
                                "✅ 診斷完成：{} ~ {} 區間共 {} 個平日均已完整入庫，無缺漏交易日！",
                                report.start_date, report.end_date, report.total_workdays
                            )),
                    )
                } else {
                    let preview_dates = if report.missing_dates.len() > 10 {
                        format!(
                            "{} 等共 {} 天",
                            report.missing_dates[..10].join(", "),
                            report.missing_dates.len()
                        )
                    } else {
                        report.missing_dates.join(", ")
                    };
                    this.child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .p_2p5()
                            .rounded_md()
                            .border_1()
                            .border_color(rgb(0xd97706))
                            .bg(rgb(0x2d1a0e))
                            .child(
                                div()
                                    .text_color(rgb(0xfbbf24))
                                    .font_weight(gpui_kit::FontWeight::BOLD)
                                    .text_sm()
                                    .child(format!(
                                        "⚠️ 缺漏警示：{} ~ {} 區間共 {} 個平日中，已入庫 {} 天，缺漏 {} 天待修補",
                                        report.start_date,
                                        report.end_date,
                                        report.total_workdays,
                                        report.recorded_dates,
                                        report.missing_dates.len()
                                    )),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xfcd34d))
                                    .child(format!("缺漏日期清單：{}", preview_dates)),
                            ),
                    )
                }
            })
            .child(
                div()
                    .w_full()
                    .child(Progress::new("download-progress-bar").value(self.progress_ratio)),
            )
            .child(
                div()
                    .text_color(rgb(0xa6adc8))
                    .text_sm()
                    .child(self.status_text.clone()),
            )
            .child(
                div()
                    .p_3()
                    .border_1()
                    .border_color(rgb(0x45475a))
                    .rounded_md()
                    .bg(rgb(0x181825))
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .font_weight(gpui_kit::FontWeight::BOLD)
                                    .text_color(rgb(0xcdd6f4))
                                    .child("📊 集中市場券商買賣日報表（分點成交明細）"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xa6adc8))
                                    .child("範圍：TWSE 上市普通股（自動排除 ETF 與權證）"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(div().text_sm().text_color(rgb(0xa6adc8)).child("交易日:"))
                                    .child(
                                        div()
                                            .w(px(130.))
                                            .child(Input::new(&self.broker_date_input).id("broker-dl-date")),
                                    ),
                            )
                            .child(
                                Button::new("btn-start-broker-download")
                                    .label(if self.is_broker_downloading {
                                        "抓取中..."
                                    } else {
                                        "開始 / 繼續下載"
                                    })
                                    .primary()
                                    .disabled(self.is_broker_downloading)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.start_broker_download(cx);
                                    })),
                            )
                            .when(self.is_broker_downloading, |this| {
                                this.child(
                                    Button::new("btn-cancel-broker-download")
                                        .label("中斷作業")
                                        .danger()
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.cancel_broker_download(cx);
                                        })),
                                )
                            }),
                    )
                    .child(
                        div()
                            .w_full()
                            .child(Progress::new("broker-download-progress-bar").value(self.broker_progress_ratio)),
                    )
                    .child(
                        div()
                            .text_color(rgb(0xa6adc8))
                            .text_xs()
                            .child(self.broker_status_text.clone()),
                    ),
            )
    }
}
