use anyhow::Result;
use headless_chrome::Tab;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::sync::atomic::AtomicBool;
use super::daily::{add_log, check_cancel};

pub fn submit_code(tab: &Arc<Tab>, code: &str, lang: &str, logs: &Arc<Mutex<Vec<String>>>, cancel_flag: &Arc<AtomicBool>) -> Result<()> {
    add_log(logs, "在题解页面右侧准备填入代码...");
    
    check_cancel(cancel_flag)?;
    add_log(logs, &format!("正在右侧编辑器切换语言为 {}...", lang));
    tab.evaluate(
        &format!(
            r#"
            (function() {{
                const targetLang = '{}';
                // 查找右侧编辑器上方的语言选择器下拉框，常见特征：具有包含语言名称的按钮或选择器
                const selectors = document.querySelectorAll('button, div[class*="select"], div[class*="popover"]');
                for (let sel of selectors) {{
                    // 确保这不是左侧的题解标签，左侧的题解标签没有下拉框通常
                    if (sel.innerText && (sel.innerText.includes('C++') || sel.innerText.includes('Java') || sel.innerText.includes('Python') || sel.innerText.includes('Rust')) && !sel.className.includes('TabBarItem')) {{
                        // 如果已经是目标语言，直接返回
                        if (sel.innerText.trim() === targetLang) {{
                            return true;
                        }}
                        
                        sel.click();
                        setTimeout(() => {{
                            const options = document.querySelectorAll('div[role="option"], li, div[class*="item"]');
                            for (let opt of options) {{
                                if (opt.innerText && opt.innerText.trim() === targetLang) {{
                                    opt.click();
                                    break;
                                }}
                            }}
                        }}, 500);
                        return true;
                    }}
                }}
                return false;
            }})();
            "#,
            lang
        ),
        false
    )?;
    std::thread::sleep(Duration::from_secs(2));

    check_cancel(cancel_flag)?;
    add_log(logs, "写入代码到编辑器...");
    let safe_code = code.replace('\\', "\\\\").replace('`', "\\`");
    tab.evaluate(
        &format!(
            r#"
            (function() {{
                const code = `{}`;
                // 方法 1: 使用 Monaco 编辑器 API
                if (typeof monaco !== 'undefined' && monaco.editor) {{
                    const models = monaco.editor.getModels();
                    if (models.length > 0) {{
                        models[0].setValue(code);
                        return 'Monaco code set';
                    }}
                }}
                
                // 方法 2: 使用 textarea 作为后备
                const textarea = document.querySelector('textarea');
                if (textarea) {{
                    textarea.value = code;
                    textarea.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    return 'Textarea code set';
                }}
                
                return 'Editor not found';
            }})();
            "#,
            safe_code
        ),
        false
    ).ok();

    check_cancel(cancel_flag)?;
    add_log(logs, "点击提交...");
    tab.evaluate(
        r#"
        (function() {
            const buttons = document.querySelectorAll('button');
            for (let btn of buttons) {
                if (btn.innerText && btn.innerText.trim() === '提交') {
                    btn.click();
                    return true;
                }
            }
            return false;
        })();
        "#,
        false
    )?;

    add_log(logs, "⏳ 等待判题结果...");
    std::thread::sleep(Duration::from_secs(6));

    check_cancel(cancel_flag)?;
    add_log(logs, "尝试领取积分...");
    tab.evaluate(
        r#"
        (function() {
            const text = document.body.innerText;
            if (text.includes("恭喜完成今日打卡任务")) {
                const buttons = document.querySelectorAll('button');
                for (let btn of buttons) {
                    if (btn.innerText && ((btn.innerText.includes('领取') && btn.innerText.includes('积分')) || btn.innerText.includes('领取奖励'))) {
                        btn.click();
                        return true;
                    }
                }
            }
            return false;
        })();
        "#,
        false
    ).ok();

    std::thread::sleep(Duration::from_secs(2));
    add_log(logs, "✅ 打卡流程执行完毕！");

    Ok(())
}
