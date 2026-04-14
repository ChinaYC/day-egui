use anyhow::Result;
use headless_chrome::Tab;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::sync::atomic::AtomicBool;
use super::daily::{add_log, check_cancel};
use super::browser::wait_for_element_with_text;

pub fn extract_solution_code(tab: &Arc<Tab>, problem_url: &str, logs: &Arc<Mutex<Vec<String>>>, cancel_flag: &Arc<AtomicBool>) -> Result<(String, String)> {
    // 构造题解 URL: 把链接最后的 / 去掉（如果有），然后加上 /solutions/
    let base_url = problem_url.trim_end_matches('/');
    // 但很多时候 URL 带有参数，比如 ?envType=daily-question
    let base_url_no_query = base_url.split('?').next().unwrap_or(base_url);
    let solution_url = format!("{}/solutions/", base_url_no_query);
    
    add_log(logs, "正在进入题解区...");
    tab.navigate_to(&solution_url).map_err(|e| anyhow::anyhow!(e))?;
    std::thread::sleep(Duration::from_secs(5));

    check_cancel(cancel_flag)?;
    // 智能等待题解区加载
    wait_for_element_with_text(tab, "div, span, a", "题解", Duration::from_secs(10), cancel_flag).ok();

    check_cancel(cancel_flag)?;
    add_log(logs, "正在选择官方或热门题解...");
    tab.evaluate(
        r#"
        (function() {
            // 优先点击包含“官方”字样的题解，或者第一篇题解
            const solutions = document.querySelectorAll('a[href*="/solutions/"]');
            for (let s of solutions) {
                if (s.innerText.includes('官方')) {
                    s.click();
                    return true;
                }
            }
            if (solutions.length > 0) {
                solutions[0].click();
                return true;
            }
            return false;
        })();
        "#,
        false
    )?;

    check_cancel(cancel_flag)?;
    // 等待题解代码加载
    wait_for_element_with_text(tab, "pre, code, .monaco-editor, div", "代码", Duration::from_secs(10), cancel_flag).ok();
    add_log(logs, "✅ 题解页面加载完成，开始选择 Rust 或 C++ 语言...");

    check_cancel(cancel_flag)?;
    
    // 明确点击题解中的语言标签（优先 Rust，其次 C++）
    let detected_lang_eval = tab.evaluate(
        r#"
        (function() {
            // 题解区的语言标签通常具有 TabBarItem_item 或者 tab 相关的 class
            const tabs = document.querySelectorAll('div[class*="TabBarItem_item"], div[class*="tab"], span[class*="tab"], div.cursor-pointer');
            
            // 优先寻找 Rust
            for (let t of tabs) {
                if (t.innerText && t.innerText.trim() === 'Rust' && !t.closest('.monaco-editor')) {
                    t.click();
                    return 'Rust';
                }
            }
            
            // 其次寻找 C++
            for (let t of tabs) {
                if (t.innerText && t.innerText.trim() === 'C++' && !t.closest('.monaco-editor')) {
                    t.click();
                    return 'C++';
                }
            }
            
            // 兜底方案：如果下拉框，先点击下拉框再选
            const dropdowns = document.querySelectorAll('div[class*="language"], button[class*="language"]');
            if (dropdowns.length > 0) {
                dropdowns[0].click();
                setTimeout(() => {
                    const options = document.querySelectorAll('li, div[role="option"], div[class*="item"]');
                    for (let opt of options) {
                        if (opt.innerText && opt.innerText.trim() === 'Rust') {
                            opt.click();
                            return 'Rust';
                        }
                    }
                    for (let opt of options) {
                        if (opt.innerText && opt.innerText.trim() === 'C++') {
                            opt.click();
                            return 'C++';
                        }
                    }
                }, 500);
            }
            return 'Unknown';
        })();
        "#,
        false
    ).map_err(|e| anyhow::anyhow!(e))?;
    
    let mut selected_lang = detected_lang_eval.value
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "Unknown".to_string());
        
    std::thread::sleep(Duration::from_secs(2));

    add_log(logs, &format!("提取 {} 题解代码...", selected_lang));
    let code_eval = tab.evaluate(
        r#"
        (function() {
            const codeBlocks = document.querySelectorAll(
                'pre[class*="language-"], div[class*="code-block"] pre, code, .view-lines'
            );
            // 优先匹配代码特征
            for (let block of codeBlocks) {
                const text = block.innerText.trim();
                // 匹配 Rust 特征
                if (text.includes('impl Solution') || text.includes('pub fn') || text.includes('struct Solution')) {
                    return JSON.stringify({code: text, lang: 'Rust'});
                }
                // 匹配 C++ 特征
                if (text.includes('class Solution') || text.includes('vector<') || text.includes('public:')) {
                    return JSON.stringify({code: text, lang: 'C++'});
                }
            }
            // 兜底返回第一个包含代码的块
            if (codeBlocks.length > 0) {
                return JSON.stringify({code: codeBlocks[0].innerText.trim(), lang: 'Unknown'});
            }
            return JSON.stringify({code: "", lang: "Unknown"});
        })();
        "#,
        false
    ).map_err(|e| anyhow::anyhow!(e))?;

    let json_str = code_eval.value
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| r#"{"code":"","lang":"Unknown"}"#.to_string());
        
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_default();
    let extracted_code = parsed["code"].as_str().unwrap_or("").to_string();
    let code_lang = parsed["lang"].as_str().unwrap_or("Unknown").to_string();

    if extracted_code.is_empty() {
        add_log(logs, "❌ 未提取到代码，请检查题解页面是否正常");
        return Err(anyhow::anyhow!("未提取到代码 (Failed to extract code)"));
    }

    // 如果前端识别到具体语言，用特征语言覆盖
    if code_lang != "Unknown" {
        selected_lang = code_lang;
    }
    // 如果没有识别出来，默认当做 C++（最常见）
    if selected_lang == "Unknown" {
        selected_lang = "C++".to_string();
    }

    let code_with_comment = format!("//day编写 ({})\n{}", selected_lang, extracted_code);
    add_log(logs, &format!("✅ 代码提取成功 ({} 语言, {} 字符)", selected_lang, code_with_comment.len()));

    Ok((code_with_comment, selected_lang))
}
