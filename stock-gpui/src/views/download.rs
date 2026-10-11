use gpui_kit::base::Disableable as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::progress::Progress;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AppContext, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _, Window, div,
    px, rgb,
};

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
    pub progress_ratio: f32,
    pub status_text: String,
    pub db_summary: String,
}

impl DownloadView {
    pub fn new(core_state: CoreState, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let today = get_today_str();
        let start_date_input = cx.new(|cx| InputState::new(window, cx).default_value("2026-07-01"));
        let end_date_input = cx.new(|cx| InputState::new(window, cx).default_value(&today));

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

        let mut view = Self {
            core_state,
            start_date_input,
            end_date_input,
            is_downloading: false,
            progress_ratio: 0.0,
            status_text: "就緒".into(),
            db_summary: "讀取中...".into(),
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
        self.status_text = "已送出取消下載請求...".into();
        cx.notify();
    }
}

impl Render for DownloadView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_dl = self.is_downloading;

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
                                .label("取消下載")
                                .danger()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.cancel_download(cx);
                                })),
                        )
                    }),
            )
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
    }
}
