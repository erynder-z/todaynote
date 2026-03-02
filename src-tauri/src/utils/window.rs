use crate::models::config::AppConfig;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, RunEvent, WindowEvent};

pub fn setup_main_window(app_handle: &AppHandle) -> Result<(), String> {
    let config = AppConfig::load();

    if config.remember_window_size {
        if let Some(true) = config.window_maximized {
            let handle_for_call = app_handle.clone();
            let handle_for_closure = handle_for_call.clone();

            let _ = handle_for_call.run_on_main_thread(move || {
                if let Some(window) = handle_for_closure.get_webview_window("main") {
                    if let Err(e) = window.maximize() {
                        eprintln!("Failed to maximize window: {e}");
                    }
                }
            });

            return Ok(());
        }
    }

    let window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    if config.remember_window_size {
        if let (Some(w), Some(h), Some(x), Some(y)) = (
            config.window_width,
            config.window_height,
            config.window_x,
            config.window_y,
        ) {
            let _ = window.set_size(PhysicalSize::new(w as u32, h as u32));
            let _ = window.set_position(PhysicalPosition::new(x as i32, y as i32));
            return Ok(());
        }
    }

    if let Some(monitor) = window.current_monitor().map_err(|e| e.to_string())? {
        let monitor_size = monitor.size();
        let mut new_height = (monitor_size.height as f64 * 0.8) as u32;
        let mut new_width = (new_height as f64 * 4.0 / 3.0) as u32;

        if new_width > monitor_size.width {
            new_width = (monitor_size.width as f64 * 0.8) as u32;
            new_height = (new_width as f64 * 3.0 / 4.0) as u32;
        }

        let _ = window.set_size(PhysicalSize::new(new_width, new_height));
        let _ = window.set_position(PhysicalPosition::new(50, 50));
    }

    Ok(())
}

pub fn handle_window_event(app_handle: &AppHandle, event: &RunEvent) {
    if let RunEvent::WindowEvent {
        label,
        event: WindowEvent::Moved(_) | WindowEvent::Resized(_),
        ..
    } = event
    {
        if label == "main" {
            if let Some(window) = app_handle.get_webview_window("main") {
                let mut config = AppConfig::load();
                if config.remember_window_size {
                    if let Ok(size) = window.inner_size() {
                        config.window_width = Some(size.width as f64);
                        config.window_height = Some(size.height as f64);
                    }
                    if let Ok(pos) = window.outer_position() {
                        config.window_x = Some(pos.x as f64);
                        config.window_y = Some(pos.y as f64);
                    }

                    if let Ok(is_maximized) = window.is_maximized() {
                        config.window_maximized = Some(is_maximized);
                    }
                    config.save();
                }
            }
        }
    }
}
