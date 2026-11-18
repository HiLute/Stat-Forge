// В релизной сборке под Windows не открывать консольное окно рядом с GUI.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod rules;
mod tabs;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    select_renderer()?;
    let ui = AppWindow::new()?;
    // Идентификатор приложения для Wayland/X11: по нему композитор
    // показывает имя окна и применяет правила окон. Задаётся до показа окна.
    slint::set_xdg_app_id("stat-forge")?;
    // Единственный источник версии — Cargo.toml: он же попадает в ресурсы
    // .exe на Windows (build.rs) и в Info.plist на macOS (сборка релиза).
    ui.set_app_version(env!("CARGO_PKG_VERSION").into());

    tabs::random::install(&ui);
    tabs::standard::install(&ui);
    tabs::point_buy::install(&ui);
    tabs::experiments::install(&ui);

    ui.run()
}

/// На Linux рисуем процессором. OpenGL-рендер на Wayland с драйвером NVIDIA
/// блокирует главный поток в `eglSwapBuffers`, пока композитор не покажет
/// предыдущий кадр; скрытому окну он кадр не показывает — приложение перестаёт
/// отвечать, и Hyprland выводит «Приложение не отвечает». Программному
/// рендеру ждать нечего, а интерфейсу такого размера его скорости хватает.
/// Переменная `SLINT_BACKEND`, если задана, по-прежнему имеет приоритет.
fn select_renderer() -> Result<(), slint::PlatformError> {
    if cfg!(all(unix, not(target_os = "macos"))) && std::env::var_os("SLINT_BACKEND").is_none() {
        slint::BackendSelector::new()
            .backend_name("winit".into())
            .renderer_name("software".into())
            .select()?;
    }
    Ok(())
}
