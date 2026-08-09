#![cfg_attr(not(target_arch = "wasm32"), windows_subsystem = "windows")]

mod app;

// ---- Нативная сборка (desktop): -------------------------------------------
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([420.0, 820.0])
            .with_min_inner_size([340.0, 500.0])
            .with_title("KPI DSA"),
        ..Default::default()
    };

    eframe::run_native(
        "KPI DSA",
        options,
        Box::new(|cc| Ok(Box::new(app::KpiApp::new(cc)))),
    )
}

// ---- Веб-сборка (wasm32): вызывается браузером автоматически ---------------
// Для bin-крейтов на wasm32 rustc экспортирует `main`, а wasm-bindgen-cli
// (запускаемый Trunk) вызывает его после загрузки модуля.
#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast as _;

    console_error_panic_hook::set_once();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async move {
        let document = web_sys::window()
            .expect("no window")
            .document()
            .expect("no document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("failed to find #the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("#the_canvas_id is not a canvas");

        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(app::KpiApp::new(cc)))),
            )
            .await
            .expect("failed to start eframe");
    });
}
