use gpui_kit::base::Disableable as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::progress::Progress;
use gpui_kit::{
    AppContext, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _, Window,
    div, px, rgb,
};

use crate::state::CoreState;

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
        let start_date_input = cx.new(|cx| InputState::new(window, cx).default_value("2026-03-01"));
        let end_date_input = cx.new(|cx| InputState::new(window, cx).default_value("2026-03-20"));

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

    pub fn start_download(&mut self, cx: &mut Context<Self>) {
        let start_date = self.start_date_input.read(cx).value().trim().to_string();
        let end_date = self.end_date_input.read(cx).value().trim().to_string();

        self.is_downloading = true;
        self.progress_ratio = 0.0;
        self.status_text = "開始下載...".into();
        cx.notify();

        let core = self.core_state.clone();
        cx.spawn(async move |this, cx| {
            let res = core
                .batch_download(&start_date, &end_date, |_done, _total, date| {
                    let _ = date;
                })
                .await;

            let _ = this.update(cx, |view, cx| {
                view.is_downloading = false;
                view.progress_ratio = 1.0;
                match res {
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
        })
        .detach();
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
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .w(px(140.))
                            .child(Input::new(&self.start_date_input).id("dl-start-date")),
                    )
                    .child(
                        div()
                            .w(px(140.))
                            .child(Input::new(&self.end_date_input).id("dl-end-date")),
                    )
                    .child(
                        Button::new("btn-start-download")
                            .label(if is_dl { "下載中..." } else { "下載指定區間" })
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
                                this.start_download(cx);
                            })),
                    ),
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
