use std::string::String;

use embedded_graphics::mono_font::ascii::{FONT_10X20, FONT_9X15_BOLD};
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, Rectangle};
use embedded_graphics::text::{Baseline, Text};

use esp_idf_svc::wifi::AccessPointInfo;

use crate::lcd::Screen;
use crate::touch::Touch;

const LIST_X: i32 = 16;
const LIST_Y0: i32 = 80;
const LIST_ROW_H: i32 = 40;
pub const LIST_MAX: usize = 8;

const BTN_Y: i32 = 720;
const BTN_H: i32 = 64;
const BTN_W: i32 = SCREEN_W / 4;
const SCREEN_W: i32 = 480;
const BTN_LABELS: [&str; 4] = ["UP", "DOWN", "OK", "RESCAN"];

const KEY_W: i32 = 48;
const KEY_H: i32 = 52;
const KEY_STEP: i32 = 70;
const KB_Y0: i32 = 408;

const KEY_ROWS: [&str; 4] = ["1234567890", "qwertyuiop", "asdfghjkl", "zxcvbnm"];
const BACKSPACE_KEY: char = '\u{8}';
const ENTER_KEY: char = '\n';
const BACK_KEY: char = '\u{1b}';

const COLOR_BG: Rgb565 = Rgb565::BLACK;
const COLOR_FG: Rgb565 = Rgb565::WHITE;
const COLOR_OK: Rgb565 = Rgb565::GREEN;
const COLOR_ERR: Rgb565 = Rgb565::RED;

fn text_style(fg: Rgb565, bg: Rgb565) -> MonoTextStyleBuilder<'static, Rgb565> {
    MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(fg)
        .background_color(bg)
}

fn draw_text(screen: &mut Screen, x: i32, y: i32, s: &str, fg: Rgb565, bg: Rgb565) {
    let _ = Text::with_baseline(
        s,
        Point::new(x, y),
        text_style(fg, bg).build(),
        Baseline::Top,
    )
    .draw(screen);
}

fn fill(screen: &mut Screen, color: Rgb565) {
    let _ = Rectangle::new(Point::zero(), Size::new(SCREEN_W as u32, 800))
        .into_styled(PrimitiveStyle::with_fill(color))
        .draw(screen);
}

fn fill_rect(screen: &mut Screen, x: i32, y: i32, w: u32, h: u32, color: Rgb565) {
    let _ = Rectangle::new(Point::new(x, y), Size::new(w, h))
        .into_styled(PrimitiveStyle::with_fill(color))
        .draw(screen);
}

fn outline_rect(screen: &mut Screen, x: i32, y: i32, w: u32, h: u32, color: Rgb565) {
    let _ = Rectangle::new(Point::new(x, y), Size::new(w, h))
        .into_styled(PrimitiveStyle::with_stroke(color, 2))
        .draw(screen);
}

const TOUCH_SETTLE_MS: u64 = 30;

const PAINT_EXIT_W: i32 = 96;
const PAINT_EXIT_H: i32 = 48;
const GRID_STEP: i32 = 100;
const DOT_R: u32 = 4;

/// Live raw-coordinate paint test: draw a dot wherever the finger reports,
/// over a 100px grid. Tap EXIT (top-right) to leave.
pub fn paint_test(screen: &mut Screen, display: &crate::lcd::Lcd, touch: &mut Touch) {
    fill(screen, COLOR_BG);
    for gx in (0..=SCREEN_W).step_by(GRID_STEP as usize) {
        fill_rect(screen, gx, 0, 1, 800, Rgb565::new(4, 8, 4));
    }
    for gy in (0..800).step_by(GRID_STEP as usize) {
        fill_rect(screen, 0, gy, SCREEN_W as u32, 1, Rgb565::new(4, 8, 4));
    }
    outline_rect(
        screen,
        SCREEN_W - PAINT_EXIT_W,
        0,
        PAINT_EXIT_W as u32,
        PAINT_EXIT_H as u32,
        COLOR_ERR,
    );
    draw_text(
        screen,
        SCREEN_W - PAINT_EXIT_W + 20,
        14,
        "EXIT",
        COLOR_ERR,
        COLOR_BG,
    );
    let _ = screen.flush(display);
    loop {
        if let Some((x, y)) = touch.read() {
            let (ix, iy) = (x as i32, y as i32);
            if ix >= SCREEN_W - PAINT_EXIT_W && iy < PAINT_EXIT_H {
                log::info!("paint exit");
                return;
            }
            let _ = Circle::new(Point::new(ix - DOT_R as i32, iy - DOT_R as i32), DOT_R * 2)
                .into_styled(PrimitiveStyle::with_fill(COLOR_OK))
                .draw(screen);
            let _ = screen.flush(display);
        }
        std::thread::sleep(std::time::Duration::from_millis(TOUCH_SETTLE_MS));
    }
}

pub fn wait_touch(touch: &mut Touch) -> (u16, u16) {
    while touch.read().is_some() {
        std::thread::sleep(std::time::Duration::from_millis(TOUCH_SETTLE_MS));
    }
    loop {
        if let Some(p) = touch.read() {
            return p;
        }
        std::thread::sleep(std::time::Duration::from_millis(TOUCH_SETTLE_MS));
    }
}

pub fn scan_screen(screen: &mut Screen) {
    fill(screen, COLOR_BG);
    draw_text(screen, LIST_X, 24, "SCANNING WIFI...", COLOR_FG, COLOR_BG);
}

pub fn status(screen: &mut Screen, title: &str, detail: &str, color: Rgb565) {
    fill(screen, COLOR_BG);
    draw_text(screen, LIST_X, 24, title, color, COLOR_BG);
    draw_text(screen, LIST_X, 60, detail, COLOR_FG, COLOR_BG);
}

#[derive(Clone, Copy, PartialEq)]
pub enum ApAction {
    Up,
    Down,
    Ok,
    Rescan,
    Row(usize),
}

pub fn ap_list(screen: &mut Screen, aps: &[AccessPointInfo], sel: usize) {
    fill(screen, COLOR_BG);
    draw_text(screen, LIST_X, 24, "SELECT NETWORK:", COLOR_FG, COLOR_BG);
    for (i, ap) in aps.iter().take(LIST_MAX).enumerate() {
        let y = LIST_Y0 + i as i32 * LIST_ROW_H;
        let selected = i == sel;
        let bg = if selected { COLOR_OK } else { COLOR_BG };
        let fg = if selected { COLOR_BG } else { COLOR_FG };
        fill_rect(screen, 0, y, SCREEN_W as u32, LIST_ROW_H as u32, bg);
        draw_text(screen, LIST_X, y + 8, &ap.ssid, fg, bg);
    }
    for (i, label) in BTN_LABELS.iter().enumerate() {
        let x = i as i32 * BTN_W;
        outline_rect(
            screen,
            x + 2,
            BTN_Y + 2,
            BTN_W as u32 - 4,
            BTN_H as u32 - 4,
            COLOR_FG,
        );
        let style = text_style(COLOR_FG, COLOR_BG).font(&FONT_9X15_BOLD).build();
        let _ = Text::with_baseline(
            label,
            Point::new(x + 12, BTN_Y + BTN_H / 2),
            style,
            Baseline::Middle,
        )
        .draw(screen);
    }
}

/// Blocks until one of the bottom buttons is tapped. Draws a dot at each
/// decoded touch position for calibration.
pub fn pick_ap(screen: &mut Screen, display: &crate::lcd::Lcd, touch: &mut Touch) -> ApAction {
    loop {
        let (x, y) = wait_touch(touch);
        log::info!("touch {x} {y}");
        fill_rect(screen, x as i32, y as i32, 6, 6, COLOR_ERR);
        let _ = screen.flush(display);
        let (x, y) = (x as i32, y as i32);
        if y < BTN_Y {
            if y >= LIST_Y0 {
                let row = ((y - LIST_Y0) / LIST_ROW_H) as usize;
                if row < LIST_MAX {
                    return ApAction::Row(row);
                }
            }
            continue;
        }
        return match x / BTN_W {
            0 => ApAction::Up,
            1 => ApAction::Down,
            2 => ApAction::Ok,
            _ => ApAction::Rescan,
        };
    }
}

fn draw_key(screen: &mut Screen, x: i32, y: i32, w_keys: i32, label: &str) {
    let style = text_style(COLOR_FG, COLOR_BG).font(&FONT_9X15_BOLD).build();
    outline_rect(
        screen,
        x + 2,
        y + 2,
        (w_keys * KEY_W) as u32 - 4,
        KEY_H as u32 - 4,
        COLOR_FG,
    );
    let tw = label.chars().count() as i32 * 9;
    let _ = Text::with_baseline(
        label,
        Point::new(x + (w_keys * KEY_W - tw) / 2, y + KEY_H / 2),
        style,
        Baseline::Middle,
    )
    .draw(screen);
}

fn draw_char_key(screen: &mut Screen, x: i32, y: i32, ch: char) {
    outline_rect(
        screen,
        x + 2,
        y + 2,
        KEY_W as u32 - 4,
        KEY_H as u32 - 4,
        COLOR_FG,
    );
    let style = text_style(COLOR_FG, COLOR_BG).font(&FONT_9X15_BOLD).build();
    let _ = Text::with_baseline(
        &ch.to_string(),
        Point::new(x + KEY_W / 2, y + KEY_H / 2),
        style,
        Baseline::Middle,
    )
    .draw(screen);
}

fn hit_key(x: i32, y: i32) -> Option<char> {
    if y < KB_Y0 || x >= KEY_W * 10 {
        return None;
    }
    let row = ((y - KB_Y0) / KEY_STEP) as usize;
    let col = (x / KEY_W) as usize;
    match row {
        0..=3 => KEY_ROWS[row].chars().nth(col).or(Some(BACKSPACE_KEY)),
        4 => match col {
            0..=1 => Some(' '),
            2 => Some('.'),
            3 => Some('-'),
            4 => Some('_'),
            5 => Some(BACK_KEY),
            _ => Some(ENTER_KEY),
        },
        _ => None,
    }
}

fn pass_screen(screen: &mut Screen, ssid: &str, pass: &str) {
    fill(screen, COLOR_BG);
    draw_text(screen, LIST_X, 16, ssid, COLOR_OK, COLOR_BG);
    let shown: String = "*".repeat(pass.chars().count());
    draw_text(screen, LIST_X, 48, &shown, COLOR_FG, COLOR_BG);

    for (r, keys) in KEY_ROWS.iter().enumerate() {
        let y = KB_Y0 + r as i32 * KEY_STEP;
        for (c, ch) in keys.chars().enumerate() {
            draw_char_key(screen, c as i32 * KEY_W, y, ch);
        }
    }
    let y = KB_Y0 + 3 * KEY_STEP;
    draw_key(screen, 7 * KEY_W, y, 3, "DEL");
    let y = KB_Y0 + 4 * KEY_STEP;
    draw_char_key(screen, 0, y, ' ');
    draw_char_key(screen, KEY_W, y, ' ');
    draw_char_key(screen, 2 * KEY_W, y, '.');
    draw_char_key(screen, 3 * KEY_W, y, '-');
    draw_char_key(screen, 4 * KEY_W, y, '_');
    draw_key(screen, 5 * KEY_W, y, 1, "BACK");
    draw_key(screen, 6 * KEY_W, y, 4, "ENTER");
}

/// Keyboard entry. None = user pressed BACK (return to selection).
pub fn type_pass(
    screen: &mut Screen,
    display: &crate::lcd::Lcd,
    ssid: &str,
    touch: &mut Touch,
) -> Option<String> {
    let mut pass = String::new();
    loop {
        pass_screen(screen, ssid, &pass);
        let _ = screen.flush(display);
        let (x, y) = wait_touch(touch);
        match hit_key(x as i32, y as i32) {
            Some(BACKSPACE_KEY) => {
                pass.pop();
            }
            Some(ENTER_KEY) if !pass.is_empty() => return Some(pass),
            Some(BACK_KEY) => return None,
            Some(ch @ (' ' | '.' | '-' | '_')) => {
                if pass.len() < 63 {
                    pass.push(ch);
                }
            }
            Some(ch) if ch.is_ascii_alphanumeric() && pass.len() < 63 => {
                pass.push(ch);
            }
            _ => {}
        }
    }
}
