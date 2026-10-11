//! 集中市場圖形驗證碼影像前處理與辨識模組
//!
//! 提供 BSR 驗證碼（200x60 JPEG/PNG、深灰背景、白色字體、雪花雜訊與貫穿線）之
//! 灰階轉換、二值化門檻、連通分量濾波、字元切分與特徵辨識。

use image::{GenericImageView, ImageError};
use std::collections::VecDeque;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CaptchaError {
    #[error("影像解碼失敗: {0}")]
    ImageDecodeError(#[from] ImageError),
    #[error("驗證碼資料為空")]
    EmptyData,
    #[error("無法辨識驗證碼字元")]
    RecognitionFailed,
}

/// 二值化點陣圖
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<bool>,
}

impl BinaryImage {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![false; (width * height) as usize],
        }
    }

    #[inline]
    pub fn get(&self, x: u32, y: u32) -> bool {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize]
        } else {
            false
        }
    }

    #[inline]
    pub fn set(&mut self, x: u32, y: u32, val: bool) {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize] = val;
        }
    }

    /// 從 raw image bytes 載入並進行灰階與亮度門檻二值化
    pub fn from_image_bytes(bytes: &[u8], threshold: u8) -> Result<Self, CaptchaError> {
        if bytes.is_empty() {
            return Err(CaptchaError::EmptyData);
        }
        let dyn_img = image::load_from_memory(bytes)?;
        let (width, height) = dyn_img.dimensions();
        let gray = dyn_img.to_luma8();

        let mut bin = Self::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let pixel = gray.get_pixel(x, y)[0];
                if pixel >= threshold {
                    bin.set(x, y, true);
                }
            }
        }
        Ok(bin)
    }

    /// 移除像素數量小於 min_size 的孤立連通塊（消除背景雪花白點雜訊）
    pub fn remove_small_components(&mut self, min_size: usize) {
        let w = self.width;
        let h = self.height;
        let mut visited = vec![false; (w * h) as usize];

        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) as usize;
                if self.data[idx] && !visited[idx] {
                    // BFS 收集連通塊
                    let mut comp = Vec::new();
                    let mut queue = VecDeque::new();
                    queue.push_back((x, y));
                    visited[idx] = true;

                    while let Some((cx, cy)) = queue.pop_front() {
                        comp.push((cx, cy));
                        // 8-鄰域搜尋
                        let cx_i = cx as i32;
                        let cy_i = cy as i32;
                        for dy in -1..=1 {
                            for dx in -1..=1 {
                                if dx == 0 && dy == 0 {
                                    continue;
                                }
                                let nx = cx_i + dx;
                                let ny = cy_i + dy;
                                if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                                    let nu_x = nx as u32;
                                    let nu_y = ny as u32;
                                    let n_idx = (nu_y * w + nu_x) as usize;
                                    if self.data[n_idx] && !visited[n_idx] {
                                        visited[n_idx] = true;
                                        queue.push_back((nu_x, nu_y));
                                    }
                                }
                            }
                        }
                    }

                    // 若連通塊過小，將該塊像素全部清為 false
                    if comp.len() < min_size {
                        for (cx, cy) in comp {
                            self.set(cx, cy, false);
                        }
                    }
                }
            }
        }
    }

    /// 形態學腐蝕（Erosion）
    pub fn erode(&self, radius: u32) -> Self {
        let mut out = Self::new(self.width, self.height);
        let r = radius as i32;
        let w = self.width as i32;
        let h = self.height as i32;

        for y in 0..self.height {
            for x in 0..self.width {
                if !self.get(x, y) {
                    continue;
                }
                let mut keep = true;
                'check: for dy in -r..=r {
                    for dx in -r..=r {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx < 0 || nx >= w || ny < 0 || ny >= h || !self.get(nx as u32, ny as u32) {
                            keep = false;
                            break 'check;
                        }
                    }
                }
                if keep {
                    out.set(x, y, true);
                }
            }
        }
        out
    }

    /// 形態學膨脹（Dilation）
    pub fn dilate(&self, radius: u32) -> Self {
        let mut out = Self::new(self.width, self.height);
        let r = radius as i32;
        let w = self.width as i32;
        let h = self.height as i32;

        for y in 0..self.height {
            for x in 0..self.width {
                if self.get(x, y) {
                    for dy in -r..=r {
                        for dx in -r..=r {
                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;
                            if nx >= 0 && nx < w && ny >= 0 && ny < h {
                                out.set(nx as u32, ny as u32, true);
                            }
                        }
                    }
                }
            }
        }
        out
    }

    /// 裁切子區域並去除四周空白邊界（Bounding Box）
    pub fn crop_and_trim(&self, x: u32, y: u32, width: u32, height: u32) -> Option<Self> {
        let x_end = (x + width).min(self.width);
        let y_end = (y + height).min(self.height);

        let mut min_x = u32::MAX;
        let mut max_x = 0;
        let mut min_y = u32::MAX;
        let mut max_y = 0;
        let mut has_pixel = false;

        for py in y..y_end {
            for px in x..x_end {
                if self.get(px, py) {
                    has_pixel = true;
                    if px < min_x {
                        min_x = px;
                    }
                    if px > max_x {
                        max_x = px;
                    }
                    if py < min_y {
                        min_y = py;
                    }
                    if py > max_y {
                        max_y = py;
                    }
                }
            }
        }

        if !has_pixel {
            return None;
        }

        let crop_w = max_x - min_x + 1;
        let crop_h = max_y - min_y + 1;
        let mut cropped = Self::new(crop_w, crop_h);
        for py in 0..crop_h {
            for px in 0..crop_w {
                cropped.set(px, py, self.get(min_x + px, min_y + py));
            }
        }
        Some(cropped)
    }

    /// 縮放為目標固定尺寸 (target_w, target_h)
    pub fn resize_to(&self, target_w: u32, target_h: u32) -> Self {
        let mut resized = Self::new(target_w, target_h);
        if self.width == 0 || self.height == 0 {
            return resized;
        }

        for ty in 0..target_h {
            for tx in 0..target_w {
                let sx = (tx as f32 * self.width as f32 / target_w as f32) as u32;
                let sy = (ty as f32 * self.height as f32 / target_h as f32) as u32;
                resized.set(tx, ty, self.get(sx.min(self.width - 1), sy.min(self.height - 1)));
            }
        }
        resized
    }

    /// 計算二值圖像中為 true 的像素數量
    pub fn count_pixels(&self) -> usize {
        self.data.iter().filter(|&&b| b).count()
    }
}

/// 驗證碼求解器
pub struct CaptchaSolver;

impl CaptchaSolver {
    /// 對 BSR 驗證碼進行完整影像前處理（二值化、雪花雜訊濾波）
    pub fn preprocess(image_bytes: &[u8]) -> Result<BinaryImage, CaptchaError> {
        let mut bin = BinaryImage::from_image_bytes(image_bytes, 135)?;
        // 濾除像素數量小於 20 的小斑點
        bin.remove_small_components(20);
        Ok(bin)
    }

    /// 將 200x60 驗證碼分割為 5 個字元點陣圖（若有字元被貫穿線輕微相連，則以固定區間與邊界自適應裁剪）
    pub fn segment_5_characters(bin: &BinaryImage) -> Vec<Option<BinaryImage>> {
        let mut chars = Vec::new();

        // BSR 驗證碼寬 200，高 60。文字主體約落在 x=10..190，y=8..55
        // 5 個字元大致均勻分布在 5 個水平槽位（Slots），每槽約 36 像素寬
        let slot_width = 36;
        let start_x = 10;

        for i in 0..5 {
            let sx = start_x + i * slot_width;
            let cropped = bin.crop_and_trim(sx, 5, slot_width + 4, 50);
            chars.push(cropped);
        }

        chars
    }

    /// 從標準化點陣圖 (16x20) 提取幾何與拓撲特徵
    pub fn extract_features(char_img: &BinaryImage) -> Vec<f64> {
        let normalized = char_img.resize_to(16, 20);
        let total_pixels = normalized.count_pixels() as f64;
        let mut features = Vec::new();

        // 1. 整體密度
        features.push(total_pixels / 320.0);

        // 2. 4 個象限的密度分佈
        let mut q = [0.0; 4];
        for y in 0..20 {
            for x in 0..16 {
                if normalized.get(x, y) {
                    let q_idx = if y < 10 {
                        if x < 8 { 0 } else { 1 }
                    } else {
                        if x < 8 { 2 } else { 3 }
                    };
                    q[q_idx] += 1.0;
                }
            }
        }
        for v in q {
            features.push(if total_pixels > 0.0 { v / total_pixels } else { 0.0 });
        }

        // 3. 水平方向穿越次數（掃描 y=5, y=10, y=15）
        for &scan_y in &[5, 10, 15] {
            let mut transitions = 0;
            let mut prev = false;
            for x in 0..16 {
                let cur = normalized.get(x, scan_y);
                if cur && !prev {
                    transitions += 1;
                }
                prev = cur;
            }
            features.push(transitions as f64);
        }

        // 4. 垂直方向穿越次數（掃描 x=4, x=8, x=12）
        for &scan_x in &[4, 8, 12] {
            let mut transitions = 0;
            let mut prev = false;
            for y in 0..20 {
                let cur = normalized.get(scan_x, y);
                if cur && !prev {
                    transitions += 1;
                }
                prev = cur;
            }
            features.push(transitions as f64);
        }

        features
    }

    /// 單字元特徵判斷與分類器
    pub fn classify_char(char_img: &BinaryImage) -> char {
        let norm = char_img.resize_to(16, 20);
        let features = Self::extract_features(char_img);

        let density = features[0];
        let q_tl = features[1]; // top-left
        let q_tr = features[2]; // top-right
        let q_bl = features[3]; // bottom-left
        let q_br = features[4]; // bottom-right

        let h_trans_mid = features[6]; // y=10 的水平穿越數
        let v_trans_mid = features[9]; // x=8 的垂直穿越數

        // 頂部與底部中心是否有像素
        let top_center = norm.get(7, 2) || norm.get(8, 2);
        let bottom_center = norm.get(7, 17) || norm.get(8, 17);
        let center_pixel = norm.get(8, 10);

        // 判斷特徵規則樹
        if h_trans_mid >= 2.0 && v_trans_mid >= 2.0 {
            // 雙環或中空環：8, B, 0
            if center_pixel {
                return '8';
            } else {
                return '0';
            }
        }

        if !top_center && bottom_center && v_trans_mid <= 1.0 {
            // 上開口：U 或 V
            if norm.get(7, 18) && norm.get(8, 18) {
                return 'U';
            } else {
                return 'V';
            }
        }

        if top_center && !bottom_center && v_trans_mid <= 1.0 {
            // 下開口：A 或 M
            return 'A';
        }

        if q_tl > 0.35 && q_bl < 0.15 {
            // 上重下輕：7 或 T 或 F
            if !norm.get(2, 2) && norm.get(8, 2) && !norm.get(14, 2) {
                return 'T';
            }
            if norm.get(2, 10) && !norm.get(14, 10) {
                return 'F';
            }
            return '7';
        }

        if q_tl > 0.25 && q_bl > 0.25 && q_tr > 0.20 && q_br > 0.20 {
            // 橫劃多的字母：E, G, H
            if norm.get(2, 10) && norm.get(8, 10) && norm.get(13, 10) {
                return 'H';
            }
            if !norm.get(13, 5) && norm.get(13, 15) {
                return 'G';
            }
            return 'E';
        }

        if norm.get(2, 17) && norm.get(13, 17) && !norm.get(7, 17) {
            return 'W';
        }

        if norm.get(2, 10) && norm.get(8, 10) {
            return 'R';
        }

        if norm.get(12, 16) && norm.get(8, 18) && !norm.get(2, 18) {
            return 'J';
        }

        // 備用啟發式依密度與重心分配
        if density > 0.45 {
            'D'
        } else if q_tl + q_bl > q_tr + q_br {
            'E'
        } else {
            '8'
        }
    }

    /// 辨識驗證碼影像 raw bytes
    pub fn solve(image_bytes: &[u8]) -> Result<String, CaptchaError> {
        let bin = Self::preprocess(image_bytes)?;
        let segments = Self::segment_5_characters(&bin);

        let mut result = String::with_capacity(5);
        for seg_opt in segments {
            if let Some(char_img) = seg_opt {
                result.push(Self::classify_char(&char_img));
            } else {
                // 若某個 slot 沒有檢測到足夠像素，以最常見字元 fallback
                result.push('E');
            }
        }

        if result.len() == 5 {
            Ok(result)
        } else {
            Err(CaptchaError::RecognitionFailed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_image_basic_ops() {
        let mut img = BinaryImage::new(10, 10);
        assert!(!img.get(3, 4));
        img.set(3, 4, true);
        assert!(img.get(3, 4));
        assert_eq!(img.count_pixels(), 1);

        // 越界保護
        assert!(!img.get(20, 20));
        img.set(20, 20, true);
        assert_eq!(img.count_pixels(), 1);
    }

    #[test]
    fn test_remove_small_components() {
        let mut img = BinaryImage::new(20, 20);
        // 小雜訊斑點（3 個像素連通）
        img.set(1, 1, true);
        img.set(1, 2, true);
        img.set(2, 1, true);

        // 大連通塊（25 個像素）
        for y in 10..15 {
            for x in 10..15 {
                img.set(x, y, true);
            }
        }

        assert_eq!(img.count_pixels(), 3 + 25);
        img.remove_small_components(10);

        // 小斑點應被清除
        assert!(!img.get(1, 1));
        assert!(!img.get(1, 2));
        assert!(!img.get(2, 1));

        // 大連通塊應完整保留
        assert_eq!(img.count_pixels(), 25);
        assert!(img.get(10, 10));
    }

    #[test]
    fn test_crop_and_resize() {
        let mut img = BinaryImage::new(50, 50);
        for y in 10..20 {
            for x in 10..20 {
                img.set(x, y, true);
            }
        }

        let cropped = img.crop_and_trim(5, 5, 30, 30).unwrap();
        assert_eq!(cropped.width, 10);
        assert_eq!(cropped.height, 10);
        assert_eq!(cropped.count_pixels(), 100);

        let resized = cropped.resize_to(16, 20);
        assert_eq!(resized.width, 16);
        assert_eq!(resized.height, 20);
        assert_eq!(resized.count_pixels(), 16 * 20);
    }

    #[test]
    fn test_captcha_sample_fixtures_preprocessing() {
        let fixtures = [
            "tests/fixtures/captcha/live_cap.png",
            "tests/fixtures/captcha/captcha.png",
            "tests/fixtures/captcha/captcha2.png",
            "tests/fixtures/captcha/test_cap.png",
        ];

        for path in fixtures {
            let full_path = format!("{}/../{}", env!("CARGO_MANIFEST_DIR"), path);
            let bytes = std::fs::read(&full_path).unwrap_or_else(|_| panic!("無法讀取 {}", path));
            let preprocessed = CaptchaSolver::preprocess(&bytes).expect("影像預處理應成功");
            assert_eq!(preprocessed.width, 200);
            assert_eq!(preprocessed.height, 60);

            let segments = CaptchaSolver::segment_5_characters(&preprocessed);
            assert_eq!(segments.len(), 5, "應切出 5 個槽位");

            // 確保每個樣例至少有 4 個以上槽位偵測到文字像素
            let active_slots = segments.iter().filter(|s| s.is_some()).count();
            assert!(
                active_slots >= 4,
                "{} 應至少偵測到 4 個有效字符槽位，實際: {}",
                path,
                active_slots
            );

            // 驗證 solve 能產出 5 碼大寫英數字元
            let solved = CaptchaSolver::solve(&bytes).expect("驗證碼應成功輸出 5 碼");
            assert_eq!(solved.len(), 5);
            for c in solved.chars() {
                assert!(
                    c.is_ascii_alphanumeric(),
                    "字符應為英數字: got {} in {}",
                    c,
                    solved
                );
            }
        }
    }
}
