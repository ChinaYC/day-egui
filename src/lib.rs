#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub mod leetcode;
pub mod theme;
pub mod todo;

pub use app::TemplateApp;

// ---------------------------------------------------------------------------
// Android entry point
// ---------------------------------------------------------------------------
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: eframe::android_activity::AndroidApp) {
    use std::env;
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let mut options = eframe::NativeOptions::default();
    options.android_app = Some(app);

    let _ = eframe::run_native(
        "EfficiencyTool",
        options,
        Box::new(|cc| Ok(Box::new(TemplateApp::new(cc)))),
    );
}
