#![windows_subsystem = "windows"]

use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT, TRUE, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
    MOD_SHIFT, MOD_WIN, VK_DOWN, VK_F1, VK_F10, VK_F11, VK_F12, VK_F2, VK_F3, VK_F4, VK_F5,
    VK_F6, VK_F7, VK_F8, VK_F9, VK_LEFT, VK_RIGHT, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, EnumWindows, GetClassNameW, GetForegroundWindow, GetMessageW,
    GetWindowLongW, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, IsIconic,
    IsWindowVisible, IsZoomed, MoveWindow, PostMessageW, SetForegroundWindow, ShowWindow,
    TranslateMessage, GWL_EXSTYLE, MSG, SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE, WM_CLOSE,
    WM_HOTKEY, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
};

// ==========================================
// 1. THREAD-SAFE LOGGING MECHANISM
// ==========================================

static LOGGER: Mutex<Option<PathBuf>> = Mutex::new(None);

fn init_logger(log_path: PathBuf) {
    if let Ok(mut guard) = LOGGER.lock() {
        *guard = Some(log_path);
    }
}

fn log_msg(msg: &str) {
    // Print to stdout (useful during dev if run from terminal)
    println!("{}", msg);

    // Also write to log file in WiRU directory
    if let Ok(guard) = LOGGER.lock() {
        if let Some(ref path) = *guard {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let _ = writeln!(file, "[{}] {}", timestamp, msg);
            }
        }
    }
}

// ==========================================
// 2. CONFIGURATION TYPES & DEFAULTS
// ==========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeysConfig {
    pub toggle_mode: String,
    pub focus_up: String,
    pub focus_down: String,
    pub focus_left: String,
    pub focus_right: String,
    pub close_window: String,
    pub maximize_window: String,
    pub minimize_window: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub current_mode: String,
    pub hotkeys: HotkeysConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            current_mode: "floating".to_string(),
            hotkeys: HotkeysConfig {
                toggle_mode: "ALT+SHIFT+M".to_string(),
                focus_up: "ALT+SHIFT+UP".to_string(),
                focus_down: "ALT+SHIFT+DOWN".to_string(),
                focus_left: "ALT+SHIFT+LEFT".to_string(),
                focus_right: "ALT+SHIFT+RIGHT".to_string(),
                close_window: "ALT+SHIFT+D".to_string(),
                maximize_window: "ALT+SHIFT+F".to_string(),
                minimize_window: "ALT+SHIFT+I".to_string(),
            },
        }
    }
}

fn get_wiru_dir() -> PathBuf {
    let home = dirs::home_dir()
        .or_else(|| std::env::var("USERPROFILE").ok().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("C:\\Users\\Default"));
    home.join("WiRU")
}

fn load_or_create_config() -> (AppConfig, PathBuf) {
    let wiru_dir = get_wiru_dir();
    let config_dir = wiru_dir.join("config");
    let config_path = config_dir.join("wiruconfig.json");
    let log_path = wiru_dir.join("wiru.log");

    if !config_dir.exists() {
        let _ = fs::create_dir_all(&config_dir);
    }

    init_logger(log_path);

    let config = if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => match serde_json::from_str::<AppConfig>(&content) {
                Ok(cfg) => {
                    log_msg("Successfully loaded wiruconfig.json");
                    cfg
                }
                Err(e) => {
                    log_msg(&format!("Error parsing config JSON: {}. Using defaults.", e));
                    AppConfig::default()
                }
            },
            Err(e) => {
                log_msg(&format!("Error reading config file: {}. Using defaults.", e));
                AppConfig::default()
            }
        }
    } else {
        log_msg("Config file not found. Creating default wiruconfig.json...");
        let default_cfg = AppConfig::default();
        if let Ok(json_str) = serde_json::to_string_pretty(&default_cfg) {
            let _ = fs::write(&config_path, json_str);
        }
        default_cfg
    };

    (config, config_path)
}

// ==========================================
// 3. HOTKEY PARSING & REGISTRATION
// ==========================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    ToggleMode = 1,
    FocusUp = 2,
    FocusDown = 3,
    FocusLeft = 4,
    FocusRight = 5,
    CloseWindow = 6,
    MaximizeWindow = 7,
    MinimizeWindow = 8,
}

fn parse_hotkey_string(hotkey_str: &str) -> Option<(HOT_KEY_MODIFIERS, u32)> {
    let parts: Vec<&str> = hotkey_str.split('+').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return None;
    }

    let mut modifiers = MOD_NOREPEAT;
    let mut vk_code: Option<u32> = None;

    for part in parts {
        let upper = part.to_uppercase();
        match upper.as_str() {
            "ALT" => modifiers |= MOD_ALT,
            "CTRL" | "CONTROL" => modifiers |= MOD_CONTROL,
            "SHIFT" => modifiers |= MOD_SHIFT,
            "WIN" | "SUPER" => modifiers |= MOD_WIN,
            "UP" => vk_code = Some(VK_UP.0 as u32),
            "DOWN" => vk_code = Some(VK_DOWN.0 as u32),
            "LEFT" => vk_code = Some(VK_LEFT.0 as u32),
            "RIGHT" => vk_code = Some(VK_RIGHT.0 as u32),
            "F1" => vk_code = Some(VK_F1.0 as u32),
            "F2" => vk_code = Some(VK_F2.0 as u32),
            "F3" => vk_code = Some(VK_F3.0 as u32),
            "F4" => vk_code = Some(VK_F4.0 as u32),
            "F5" => vk_code = Some(VK_F5.0 as u32),
            "F6" => vk_code = Some(VK_F6.0 as u32),
            "F7" => vk_code = Some(VK_F7.0 as u32),
            "F8" => vk_code = Some(VK_F8.0 as u32),
            "F9" => vk_code = Some(VK_F9.0 as u32),
            "F10" => vk_code = Some(VK_F10.0 as u32),
            "F11" => vk_code = Some(VK_F11.0 as u32),
            "F12" => vk_code = Some(VK_F12.0 as u32),
            s if s.len() == 1 => {
                let ch = s.chars().next().unwrap();
                if ch.is_ascii_alphanumeric() {
                    vk_code = Some(ch.to_ascii_uppercase() as u32);
                }
            }
            _ => {}
        }
    }

    vk_code.map(|vk| (modifiers, vk))
}

fn register_all_hotkeys(config: &AppConfig) {
    let hotkeys = [
        (Action::ToggleMode, &config.hotkeys.toggle_mode),
        (Action::FocusUp, &config.hotkeys.focus_up),
        (Action::FocusDown, &config.hotkeys.focus_down),
        (Action::FocusLeft, &config.hotkeys.focus_left),
        (Action::FocusRight, &config.hotkeys.focus_right),
        (Action::CloseWindow, &config.hotkeys.close_window),
        (Action::MaximizeWindow, &config.hotkeys.maximize_window),
        (Action::MinimizeWindow, &config.hotkeys.minimize_window),
    ];

    for (action, key_str) in hotkeys {
        if let Some((mods, vk)) = parse_hotkey_string(key_str) {
            unsafe {
                let res = RegisterHotKey(HWND(std::ptr::null_mut()), action as i32, mods, vk);
                if res.is_ok() {
                    log_msg(&format!(
                        "Registered hotkey for {:?}: '{}'",
                        action, key_str
                    ));
                } else {
                    log_msg(&format!(
                        "Failed to register hotkey for {:?}: '{}'",
                        action, key_str
                    ));
                }
            }
        } else {
            log_msg(&format!(
                "Invalid hotkey string for {:?}: '{}'",
                action, key_str
            ));
        }
    }
}

fn unregister_all_hotkeys() {
    for action in [
        Action::ToggleMode,
        Action::FocusUp,
        Action::FocusDown,
        Action::FocusLeft,
        Action::FocusRight,
        Action::CloseWindow,
        Action::MaximizeWindow,
        Action::MinimizeWindow,
    ] {
        unsafe {
            let _ = UnregisterHotKey(HWND(std::ptr::null_mut()), action as i32);
        }
    }
}

// ==========================================
// 4. WINDOW FILTERING & SAFETY (GHOST WINDOW PREVENTION)
// ==========================================

const EXCLUDED_CLASSES: &[&str] = &[
    "Progman",
    "WorkerW",
    "Shell_TrayWnd",
    "Windows.UI.Core.CoreWindow",
];

pub fn is_valid_window(hwnd: HWND) -> bool {
    unsafe {
        // 1. Must be visible
        if !IsWindowVisible(hwnd).as_bool() {
            return false;
        }

        // 2. Exclude minimized windows
        if IsIconic(hwnd).as_bool() {
            return false;
        }

        // 3. Must have non-empty window title
        let mut title_buf = [0u16; 512];
        let title_len = GetWindowTextW(hwnd, &mut title_buf);
        if title_len == 0 {
            return false;
        }
        let title = String::from_utf16_lossy(&title_buf[..title_len as usize]);
        if title.trim().is_empty() {
            return false;
        }

        // 4. Must not belong to system window classes
        let mut class_buf = [0u16; 256];
        let class_len = GetClassNameW(hwnd, &mut class_buf);
        if class_len > 0 {
            let class_name = String::from_utf16_lossy(&class_buf[..class_len as usize]);
            if EXCLUDED_CLASSES.iter().any(|&c| c.eq_ignore_ascii_case(&class_name)) {
                return false;
            }
        }

        // 5. Exclude Cloaked / Ghost UWP windows via DWM API
        let mut cloaked: u32 = 0;
        let dwm_res = DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut cloaked as *mut _ as *mut _,
            std::mem::size_of::<u32>() as u32,
        );
        if dwm_res.is_ok() && cloaked != 0 {
            return false;
        }

        // 6. Exclude offscreen coordinates / invalid geometry
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            if rect.left < -10000 || rect.top < -10000 {
                return false;
            }
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            if width <= 0 || height <= 0 {
                return false;
            }
        } else {
            return false;
        }

        // 7. Check window style: exclude ToolWindows unless AppWindow is explicit
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        if (ex_style & WS_EX_TOOLWINDOW.0 != 0) && (ex_style & WS_EX_APPWINDOW.0 == 0) {
            return false;
        }

        true
    }
}

// Struct for collecting windows during EnumWindows
struct EnumData {
    windows: Vec<HWND>,
}

unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = &mut *(lparam.0 as *mut EnumData);
    if is_valid_window(hwnd) {
        data.windows.push(hwnd);
    }
    TRUE
}

pub fn get_valid_windows() -> Vec<HWND> {
    let mut data = EnumData {
        windows: Vec::new(),
    };
    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut data as *mut EnumData as isize),
        );
    }
    data.windows
}

// ==========================================
// 5. FLOATING MODE (EUCLIDEAN DISTANCE FOCUS)
// ==========================================

#[derive(Debug, Clone, Copy)]
pub enum FocusDirection {
    Up,
    Down,
    Left,
    Right,
}

fn get_window_center(hwnd: HWND) -> Option<(i32, i32)> {
    let mut rect = RECT::default();
    unsafe {
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            let cx = (rect.left + rect.right) / 2;
            let cy = (rect.top + rect.bottom) / 2;
            Some((cx, cy))
        } else {
            None
        }
    }
}

fn set_focus_safe(target_hwnd: HWND) {
    unsafe {
        let fg_hwnd = GetForegroundWindow();
        if fg_hwnd == target_hwnd {
            return;
        }
        let fg_thread = GetWindowThreadProcessId(fg_hwnd, None);
        let target_thread = GetWindowThreadProcessId(target_hwnd, None);
        let current_thread = GetCurrentThreadId();

        if fg_thread != 0 && fg_thread != current_thread {
            let _ = AttachThreadInput(current_thread, fg_thread, TRUE);
        }
        if target_thread != 0 && target_thread != current_thread {
            let _ = AttachThreadInput(current_thread, target_thread, TRUE);
        }

        let _ = SetForegroundWindow(target_hwnd);

        if fg_thread != 0 && fg_thread != current_thread {
            let _ = AttachThreadInput(current_thread, fg_thread, false);
        }
        if target_thread != 0 && target_thread != current_thread {
            let _ = AttachThreadInput(current_thread, target_thread, false);
        }
    }
}

pub fn handle_floating_focus(dir: FocusDirection) {
    unsafe {
        let curr_fg = GetForegroundWindow();
        let (curr_cx, curr_cy) = match get_window_center(curr_fg) {
            Some(c) => c,
            None => {
                log_msg("No valid active foreground window found for directional focus.");
                return;
            }
        };

        let windows = get_valid_windows();
        let mut best_target: Option<(HWND, f64)> = None;

        for hwnd in windows {
            if hwnd == curr_fg {
                continue;
            }

            if let Some((cx, cy)) = get_window_center(hwnd) {
                let is_in_direction = match dir {
                    FocusDirection::Up => cy < curr_cy,
                    FocusDirection::Down => cy > curr_cy,
                    FocusDirection::Left => cx < curr_cx,
                    FocusDirection::Right => cx > curr_cx,
                };

                if is_in_direction {
                    let dx = (cx - curr_cx) as f64;
                    let dy = (cy - curr_cy) as f64;
                    let dist = (dx * dx + dy * dy).sqrt();

                    match best_target {
                        None => best_target = Some((hwnd, dist)),
                        Some((_, min_dist)) => {
                            if dist < min_dist {
                                best_target = Some((hwnd, dist));
                            }
                        }
                    }
                }
            }
        }

        if let Some((target, dist)) = best_target {
            log_msg(&format!(
                "Floating mode: Focusing window {:?} (Euclidean distance: {:.2})",
                target, dist
            ));
            set_focus_safe(target);
        } else {
            log_msg(&format!(
                "Floating mode: No candidate window found in direction {:?}",
                dir
            ));
        }
    }
}

// ==========================================
// 6. TILING MODE (AUTOMATIC VERTICAL COLUMNS)
// ==========================================

pub fn apply_tiling() {
    unsafe {
        let fg_hwnd = GetForegroundWindow();
        let hmonitor = MonitorFromWindow(fg_hwnd, MONITOR_DEFAULTTONEAREST);

        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };

        if !GetMonitorInfoW(hmonitor, &mut mi).as_bool() {
            log_msg("Failed to get monitor info for tiling.");
            return;
        }

        let work_area = mi.rcWork;
        let work_x = work_area.left;
        let work_y = work_area.top;
        let work_w = work_area.right - work_area.left;
        let work_h = work_area.bottom - work_area.top;

        let all_windows = get_valid_windows();
        // Filter windows to those on current monitor
        let monitor_windows: Vec<HWND> = all_windows
            .into_iter()
            .filter(|&hwnd| MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) == hmonitor)
            .collect();

        let count = monitor_windows.len();
        if count == 0 {
            log_msg("Tiling mode: No valid windows to tile on active monitor.");
            return;
        }

        log_msg(&format!(
            "Tiling mode: Tiling {} windows on monitor bounds ({}, {}, {}, {})",
            count, work_x, work_y, work_w, work_h
        ));

        let col_width = work_w / count as i32;

        for (i, &hwnd) in monitor_windows.iter().enumerate() {
            let x = work_x + (i as i32 * col_width);
            let y = work_y;
            let width = if i == count - 1 {
                work_w - (i as i32 * col_width)
            } else {
                col_width
            };
            let height = work_h;

            let _ = MoveWindow(hwnd, x, y, width, height, TRUE);
        }
    }
}

// ==========================================
// 7. NEW WINDOW CONTROL ACTIONS
// ==========================================

pub fn handle_close_window(current_mode: &str) {
    unsafe {
        let fg_hwnd = GetForegroundWindow();
        if is_valid_window(fg_hwnd) {
            log_msg(&format!("Closing window: {:?}", fg_hwnd));
            let _ = PostMessageW(fg_hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));

            if current_mode == "tiling" {
                // Allow a small delay for window closure before re-tiling remaining windows
                std::thread::sleep(std::time::Duration::from_millis(50));
                apply_tiling();
            }
        } else {
            log_msg("No valid foreground window to close.");
        }
    }
}

pub fn handle_minimize_window(current_mode: &str) {
    unsafe {
        let fg_hwnd = GetForegroundWindow();
        if is_valid_window(fg_hwnd) {
            log_msg(&format!("Minimizing window: {:?}", fg_hwnd));
            let _ = ShowWindow(fg_hwnd, SW_MINIMIZE);

            if current_mode == "tiling" {
                std::thread::sleep(std::time::Duration::from_millis(30));
                apply_tiling();
            }
        } else {
            log_msg("No valid foreground window to minimize.");
        }
    }
}

pub fn handle_maximize_window(current_mode: &str) {
    unsafe {
        let fg_hwnd = GetForegroundWindow();
        if is_valid_window(fg_hwnd) {
            let is_maximized = IsZoomed(fg_hwnd).as_bool();
            if is_maximized {
                log_msg(&format!("Restoring window from full screen: {:?}", fg_hwnd));
                let _ = ShowWindow(fg_hwnd, SW_RESTORE);

                // Critical Rule: If in Tiling mode, restored window must exit full screen and immediately re-tile
                if current_mode == "tiling" {
                    std::thread::sleep(std::time::Duration::from_millis(30));
                    apply_tiling();
                }
            } else {
                log_msg(&format!("Maximizing window to full screen: {:?}", fg_hwnd));
                let _ = ShowWindow(fg_hwnd, SW_MAXIMIZE);
            }
        } else {
            log_msg("No valid foreground window to toggle maximize/restore.");
        }
    }
}

// ==========================================
// 8. MAIN ENTRY POINT & MESSAGE LOOP
// ==========================================

fn main() {
    let (config, config_path) = load_or_create_config();
    log_msg(&format!(
        "WiRU Window Manager started! Config path: {:?}",
        config_path
    ));

    let mut current_mode = config.current_mode.to_lowercase();
    log_msg(&format!("Initial Window Manager Mode: {}", current_mode));

    register_all_hotkeys(&config);

    // If initial mode is tiling, tile immediately on startup
    if current_mode == "tiling" {
        apply_tiling();
    }

    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0).as_bool() {
            if msg.message == WM_HOTKEY {
                let action_id = msg.wParam.0 as i32;
                match action_id {
                    1 => {
                        // Toggle mode
                        if current_mode == "tiling" {
                            current_mode = "floating".to_string();
                            log_msg("Switched mode to FLOATING");
                        } else {
                            current_mode = "tiling".to_string();
                            log_msg("Switched mode to TILING");
                            apply_tiling();
                        }
                    }
                    2 => {
                        // Focus Up
                        if current_mode == "floating" {
                            handle_floating_focus(FocusDirection::Up);
                        }
                    }
                    3 => {
                        // Focus Down
                        if current_mode == "floating" {
                            handle_floating_focus(FocusDirection::Down);
                        }
                    }
                    4 => {
                        // Focus Left
                        if current_mode == "floating" {
                            handle_floating_focus(FocusDirection::Left);
                        }
                    }
                    5 => {
                        // Focus Right
                        if current_mode == "floating" {
                            handle_floating_focus(FocusDirection::Right);
                        }
                    }
                    6 => {
                        // Close Window
                        handle_close_window(&current_mode);
                    }
                    7 => {
                        // Maximize / Restore Window
                        handle_maximize_window(&current_mode);
                    }
                    8 => {
                        // Minimize Window
                        handle_minimize_window(&current_mode);
                    }
                    _ => {}
                }
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    unregister_all_hotkeys();
    log_msg("WiRU Window Manager exited cleanly.");
}
