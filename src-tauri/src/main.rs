#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Ставит безопасные значения окружения WebKit до старта GUI.
///
/// На Linux без GPU-ускорения (например, с драйверами NVIDIA или в виртуалке)
/// WebKitGTK пытается рендерить через DMABUF и выдаёт пустое/белое окно.
/// Отключаем это принудительно, но уважаем явно заданные пользователем значения.
fn apply_webkit_workarounds() {
    for var in [
        "WEBKIT_DISABLE_COMPOSITING_MODE",
        "WEBKIT_DISABLE_DMABUF_RENDERER",
    ] {
        if std::env::var_os(var).is_none() {
            std::env::set_var(var, "1");
        }
    }
}

fn main() {
    apply_webkit_workarounds();
    mono_launcher_lib::run()
}
