#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
use font_kit::family_name::FamilyName;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
use font_kit::properties::Properties;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
use font_kit::source::SystemSource;

pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 1. 尝试从系统中加载字体 (仅限非移动端/非 Web)
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    let system_font_bytes = {
        let source = SystemSource::new();
        let font_names = vec![
            "PingFang SC",      // Mac 默认黑体
            "STHeiti",          // Mac 老黑体
            "Hiragino Sans GB", // Mac 另一种黑体
            "Microsoft YaHei",  // Windows 默认微软雅黑
            "SimHei",           // Windows 黑体
            "Noto Sans CJK SC", // Linux 常见中文字体
            "WenQuanYi Micro Hei",
        ];

        let mut data = None;
        for name in font_names {
            if let Ok(handle) =
                source.select_best_match(&[FamilyName::Title(name.to_string())], &Properties::new())
            {
                match handle {
                    font_kit::handle::Handle::Path { path, .. } => {
                        if let Ok(bytes) = std::fs::read(&path) {
                            data = Some(bytes);
                            break;
                        }
                    }
                    font_kit::handle::Handle::Memory { bytes, .. } => {
                        data = Some((*bytes).clone());
                        break;
                    }
                }
            }
        }
        data
    };

    // 2. 加载内嵌的 WenKai 字体作为所有平台的兜底
    let wankai_bytes = include_bytes!("../assets/WenKai.ttf").to_vec();

    // 优先使用系统字体，如果系统字体加载失败，则使用内嵌字体
    let final_bytes = {
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            system_font_bytes.unwrap_or(wankai_bytes)
        }
        #[cfg(any(target_os = "android", target_arch = "wasm32"))]
        {
            wankai_bytes
        }
    };

    fonts.font_data.insert(
        "custom_chinese".to_owned(),
        std::sync::Arc::new(egui::FontData::from_owned(final_bytes)),
    );

    // 插入到比例字体家族首位
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "custom_chinese".to_owned());

    // 插入到等宽字体家族
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("custom_chinese".to_owned());

    ctx.set_fonts(fonts);
}
