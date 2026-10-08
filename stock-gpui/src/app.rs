use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::theme::{Theme, ThemeMode};
use gpui_kit::{
    AppContext, Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled as _, Window, div, px, rgb,
};

use crate::state::CoreState;
use crate::views::{AnalysisView, BacktestView, DownloadView, ScreenerView, SecuritiesView};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NavTab {
    #[default]
    Analysis,
    Screener,
    Backtest,
    Download,
    Securities,
}

impl NavTab {
    pub fn label(&self) -> &'static str {
        match self {
            NavTab::Analysis => "個股分析",
            NavTab::Screener => "布林選股",
            NavTab::Backtest => "回測系統",
            NavTab::Download => "下載管理",
            NavTab::Securities => "證券清單",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FontSizeLevel {
    Normal,
    #[default]
    Medium,
    Large,
    XLarge,
}

impl FontSizeLevel {
    pub fn size_px(&self) -> gpui_kit::Pixels {
        match self {
            FontSizeLevel::Normal => px(14.0),
            FontSizeLevel::Medium => px(16.0),
            FontSizeLevel::Large => px(18.0),
            FontSizeLevel::XLarge => px(20.0),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            FontSizeLevel::Normal => "字體: 標準 (14px)",
            FontSizeLevel::Medium => "字體: 偏大 (16px)",
            FontSizeLevel::Large => "字體: 大 (18px)",
            FontSizeLevel::XLarge => "字體: 特大 (20px)",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            FontSizeLevel::Normal => FontSizeLevel::Medium,
            FontSizeLevel::Medium => FontSizeLevel::Large,
            FontSizeLevel::Large => FontSizeLevel::XLarge,
            FontSizeLevel::XLarge => FontSizeLevel::Normal,
        }
    }
}

pub struct AppView {
    pub core_state: CoreState,
    pub active_tab: NavTab,
    pub is_dark: bool,
    pub font_size: FontSizeLevel,
    pub analysis_view: Entity<AnalysisView>,
    pub screener_view: Entity<ScreenerView>,
    pub backtest_view: Entity<BacktestView>,
    pub download_view: Entity<DownloadView>,
    pub securities_view: Entity<SecuritiesView>,
}

impl AppView {
    pub fn new(core_state: CoreState, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let analysis_view = cx.new(|cx| AnalysisView::new(core_state.clone(), window, cx));
        let screener_view = cx.new(|cx| ScreenerView::new(core_state.clone(), window, cx));
        let backtest_view = cx.new(|cx| BacktestView::new(core_state.clone(), window, cx));
        let download_view = cx.new(|cx| DownloadView::new(core_state.clone(), window, cx));
        let securities_view = cx.new(|cx| SecuritiesView::new(core_state.clone(), window, cx));

        Theme::change(ThemeMode::Light, None, cx);

        Self {
            core_state,
            active_tab: NavTab::Analysis,
            is_dark: false,
            font_size: FontSizeLevel::Medium,
            analysis_view,
            screener_view,
            backtest_view,
            download_view,
            securities_view,
        }
    }

    pub fn set_tab(&mut self, tab: NavTab, cx: &mut Context<Self>) {
        self.active_tab = tab;
        cx.notify();
    }

    pub fn toggle_theme(&mut self, cx: &mut Context<Self>) {
        self.is_dark = !self.is_dark;
        let mode = if self.is_dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };
        Theme::change(mode, None, cx);
        cx.notify();
    }

    pub fn cycle_font_size(&mut self, cx: &mut Context<Self>) {
        self.font_size = self.font_size.next();
        cx.notify();
    }

    pub fn shutdown(&self) {
        self.core_state.shutdown();
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_dark = self.is_dark;
        let active = self.active_tab;

        // 頂部導覽列
        let nav_tabs = [
            NavTab::Analysis,
            NavTab::Screener,
            NavTab::Backtest,
            NavTab::Download,
            NavTab::Securities,
        ];

        let tab_buttons = nav_tabs.into_iter().map(|tab| {
            let is_selected = active == tab;
            let bg = if is_selected {
                if is_dark {
                    rgb(0x3b82f6)
                } else {
                    rgb(0x2563eb)
                }
            } else {
                rgb(0x000000).opacity(0.0)
            };
            let text_color = if is_selected {
                rgb(0xffffff)
            } else if is_dark {
                rgb(0xa6adc8)
            } else {
                rgb(0x334155)
            };

            div()
                .id(format!("tab-btn-{:?}", tab))
                .px_3()
                .py_1()
                .rounded_md()
                .bg(bg)
                .text_color(text_color)
                .font_weight(if is_selected {
                    gpui_kit::FontWeight::BOLD
                } else {
                    gpui_kit::FontWeight::MEDIUM
                })
                .cursor_pointer()
                .child(tab.label())
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_tab(tab, cx);
                }))
        });

        let header = div()
            .h(px(52.))
            .px_4()
            .border_b_1()
            .border_color(if is_dark {
                rgb(0x313244)
            } else {
                rgb(0xe2e8f0)
            })
            .bg(if is_dark {
                rgb(0x181825)
            } else {
                rgb(0xf8fafc)
            })
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .id("app-title")
                            .test_support()
                            .font_weight(gpui_kit::FontWeight::BOLD)
                            .text_color(if is_dark {
                                rgb(0x89b4fa)
                            } else {
                                rgb(0x1e3a8a)
                            })
                            .child("台股布林通道擠壓選股系統"),
                    )
                    .child(div().flex().gap_2().children(tab_buttons)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("btn-font-size-toggle")
                            .label(self.font_size.label())
                            .secondary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.cycle_font_size(cx);
                            })),
                    )
                    .child(
                        Button::new("btn-theme-toggle")
                            .label(if is_dark {
                                "切換淺色模式"
                            } else {
                                "切換深色模式"
                            })
                            .secondary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_theme(cx);
                            })),
                    ),
            );

        // 主畫面內容切換
        let content = match self.active_tab {
            NavTab::Analysis => div().size_full().child(self.analysis_view.clone()),
            NavTab::Screener => div().size_full().child(self.screener_view.clone()),
            NavTab::Backtest => div().size_full().child(self.backtest_view.clone()),
            NavTab::Download => div().size_full().child(self.download_view.clone()),
            NavTab::Securities => div().size_full().child(self.securities_view.clone()),
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .text_size(self.font_size.size_px())
            .bg(if is_dark {
                rgb(0x1e1e2e)
            } else {
                rgb(0xffffff)
            })
            .child(header)
            .child(div().flex_1().size_full().child(content))
    }
}
