#[cfg(all(not(target_arch = "wasm32"), target_os = "linux"))]
pub fn send_system_notification(title: &str, body: &str) {
    let _ = notify_rust::Notification::new()
        .summary(title)
        .body(body)
        .show();
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "linux")))]
pub fn send_system_notification(_title: &str, _body: &str) {}

#[cfg(target_arch = "wasm32")]
pub fn send_system_notification(_title: &str, _body: &str) {}

