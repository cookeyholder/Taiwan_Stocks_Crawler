use gpui_kit::ScrollDelta;
use stock_core::models::OhlcvRecord;

/// K 線圖檢視視窗控制器（支援縮放、水平平移、拖曳）
#[derive(Clone, Debug)]
pub struct ChartViewport {
    /// 當前顯示的 K 棒數量（預設 40 根，最小 15 根）
    pub visible_count: usize,
    /// 往歷史方向平移的 K 棒數量（0 表示顯示最新資料）
    pub offset: usize,
    /// 滑鼠拖曳起始 X 座標
    pub drag_start_x: Option<f32>,
}

impl Default for ChartViewport {
    fn default() -> Self {
        Self {
            visible_count: 40,
            offset: 0,
            drag_start_x: None,
        }
    }
}

impl ChartViewport {
    pub fn zoom_in(&mut self) {
        if self.visible_count > 15 {
            self.visible_count = self.visible_count.saturating_sub(5).max(15);
        }
    }

    pub fn zoom_out(&mut self, total: usize) {
        if self.visible_count < total {
            self.visible_count = (self.visible_count + 5).min(total);
        }
    }

    pub fn pan_left(&mut self, total: usize) {
        let max_offset = total.saturating_sub(self.visible_count);
        self.offset = (self.offset + 5).min(max_offset);
    }

    pub fn pan_right(&mut self) {
        self.offset = self.offset.saturating_sub(5);
    }

    pub fn reset(&mut self) {
        self.visible_count = 40;
        self.offset = 0;
        self.drag_start_x = None;
    }

    /// 計算可見資料切片索引範圍 `(start, end)`
    pub fn slice_range(&self, total: usize) -> (usize, usize) {
        if total == 0 {
            return (0, 0);
        }
        let vis = self.visible_count.min(total).max(1);
        let max_offset = total.saturating_sub(vis);
        let off = self.offset.min(max_offset);
        let end = total - off;
        let start = end.saturating_sub(vis);
        (start, end)
    }

    /// 處理滑鼠滾輪縮放與平移
    pub fn handle_scroll_wheel(&mut self, delta: &ScrollDelta, total: usize) -> bool {
        match delta {
            ScrollDelta::Lines(pt) => {
                if pt.y.abs() > 0.05 {
                    if pt.y > 0.0 {
                        self.zoom_in();
                    } else {
                        self.zoom_out(total);
                    }
                    true
                } else if pt.x.abs() > 0.05 {
                    if pt.x < 0.0 {
                        self.pan_left(total);
                    } else {
                        self.pan_right();
                    }
                    true
                } else {
                    false
                }
            }
            ScrollDelta::Pixels(pt) => {
                let py = pt.y.as_f32();
                let px_val = pt.x.as_f32();
                if py.abs() > 0.5 {
                    if py > 0.0 {
                        self.zoom_in();
                    } else {
                        self.zoom_out(total);
                    }
                    true
                } else if px_val.abs() > 0.5 {
                    if px_val < 0.0 {
                        self.pan_left(total);
                    } else {
                        self.pan_right();
                    }
                    true
                } else {
                    false
                }
            }
        }
    }

    /// 處理滑鼠水平拖曳平移
    pub fn handle_mouse_move(&mut self, current_x: f32, total: usize) -> bool {
        if let Some(start_x) = self.drag_start_x {
            let diff = current_x - start_x;
            if diff.abs() >= 12.0 {
                if diff > 0.0 {
                    self.pan_left(total);
                } else {
                    self.pan_right();
                }
                self.drag_start_x = Some(current_x);
                return true;
            }
        }
        false
    }
}

/// 計算可見 K 棒的縱軸價格刻度
pub fn compute_price_scales(slice: &[OhlcvRecord]) -> Option<[f64; 5]> {
    if slice.is_empty() {
        return None;
    }
    let mut max_p = f64::MIN;
    let mut min_p = f64::MAX;
    for r in slice {
        if let Some(h) = r.high_price
            && h > max_p
        {
            max_p = h;
        }
        if let Some(l) = r.low_price
            && l < min_p
        {
            min_p = l;
        }
    }
    if max_p <= min_p || max_p <= 0.0 {
        return None;
    }
    let range = max_p - min_p;
    Some([
        max_p,
        min_p + 0.75 * range,
        min_p + 0.50 * range,
        min_p + 0.25 * range,
        min_p,
    ])
}

/// 計算可見 K 棒的縱軸成交量刻度
pub fn compute_volume_scales(slice: &[OhlcvRecord]) -> Option<(i64, i64, i64)> {
    if slice.is_empty() {
        return None;
    }
    let max_v = slice
        .iter()
        .filter_map(|r| r.volume_shares)
        .max()
        .unwrap_or(0);
    if max_v <= 0 {
        return None;
    }
    Some((max_v, max_v / 2, 0))
}
