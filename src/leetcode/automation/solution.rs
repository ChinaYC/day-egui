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
    wait_for_element_with_text(tab, "div, span, a", "题解", Duration::from_secs(10), cancel_flag)
        .map_err(|e| anyhow::anyhow!("题解区加载超时: {}", e))?;

    check_cancel(cancel_flag)?;
    add_log(logs, "正在选择官方或热门题解...");
    tab.evaluate(
        r#"
        (function() {
            const norm = (s) => (s || "").replace(/\s+/g, " ").trim();
            const isValidSolutionLink = (a) => {
                if (!a) return false;
                const href = a.getAttribute("href") || "";
                if (!href) return false;
                if (!href.includes("/solutions/")) return false;
                if (href.endsWith("/solutions/")) return false;
                return true;
            };

            const all = Array.from(document.querySelectorAll('a[href*="/solutions/"]')).filter(isValidSolutionLink);

            const pickByText = (needle) => {
                for (const a of all) {
                    const t = norm(a.innerText);
                    if (!t) continue;
                    if (t.toLowerCase().includes(needle.toLowerCase())) return a;
                    const card = a.closest("article, div") || a.parentElement;
                    const cardText = norm(card ? card.innerText : "");
                    if (cardText.toLowerCase().includes(needle.toLowerCase())) return a;
                }
                return null;
            };

            const rust = pickByText("Rust");
            if (rust) {
                rust.click();
                return true;
            }

            const official = pickByText("官方");
            if (official) {
                official.click();
                return true;
            }

            if (all.length > 0) {
                all[0].click();
                return true;
            }

            return false;
        })();
        "#,
        false
    )?;
    std::thread::sleep(Duration::from_secs(2));

    check_cancel(cancel_flag)?;
    // 等待题解代码加载
    wait_for_element_with_text(tab, "pre, code, .monaco-editor, div", "代码", Duration::from_secs(10), cancel_flag)
        .map_err(|e| anyhow::anyhow!("题解代码区加载超时: {}", e))?;
    add_log(logs, "✅ 题解页面加载完成，开始选择 Rust 或 C++ 语言...");

    check_cancel(cancel_flag)?;
    
    // 明确点击题解中的语言标签（优先 Rust，其次 C++）
    let detected_lang_eval = tab.evaluate(
        r#"
        (function() {
            const order = ['Rust', 'C++'];

            const isVisible = (el) => {
                if (!el) return false;
                if (el.offsetParent === null) return false;
                const style = window.getComputedStyle(el);
                if (!style) return true;
                return style.visibility !== 'hidden' && style.display !== 'none';
            };

            const hasLangFeatures = (lang) => {
                const blocks = Array.from(document.querySelectorAll('pre, code, .view-lines'))
                    .filter(b => isVisible(b));
                const text = blocks.map(b => (b.innerText || '')).join('\n');
                if (lang === 'Rust') {
                    return text.includes('impl Solution')
                        || text.includes('pub fn')
                        || text.includes('struct Solution')
                        || text.includes('use std')
                        || text.includes('Vec<')
                        || text.includes('HashMap')
                        || text.includes('i32');
                }
                if (lang === 'C++') {
                    return text.includes('class Solution') || text.includes('vector<') || text.includes('public:');
                }
                return false;
            };

            const findLangTab = (lang) => {
                const candidates = Array.from(document.querySelectorAll('[role="tab"], button, div, span, li'))
                    .filter(el => !el.closest('.monaco-editor'))
                    .filter(el => (el.innerText || '').trim() === lang)
                    .filter(isVisible);
                return candidates[0] || null;
            };

            const clickTab = (lang) => {
                const tab = findLangTab(lang);
                if (!tab) return false;
                tab.click();
                return true;
            };

            return new Promise((resolve) => {
                const tryNext = (idx) => {
                    if (idx >= order.length) {
                        resolve('Unknown');
                        return;
                    }
                    const lang = order[idx];
                    const clicked = clickTab(lang);
                    setTimeout(() => {
                        if (clicked && hasLangFeatures(lang)) {
                            resolve(lang);
                        } else {
                            tryNext(idx + 1);
                        }
                    }, 350);
                };
                tryNext(0);
            });
        })();
        "#,
        true
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
                if (block.closest && block.closest('.monaco-editor')) continue;
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
                for (let block of codeBlocks) {
                    if (block.closest && block.closest('.monaco-editor')) continue;
                    const t = block.innerText.trim();
                    if (t) return JSON.stringify({code: t, lang: 'Unknown'});
                }
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
    let extracted_code = normalize_code(parsed["code"].as_str().unwrap_or(""));
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

fn normalize_code(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'
            | '\u{2001}'
            | '\u{2002}'
            | '\u{2003}'
            | '\u{2004}'
            | '\u{2005}'
            | '\u{2006}'
            | '\u{2007}'
            | '\u{2008}'
            | '\u{2009}'
            | '\u{200A}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}' => out.push(' '),
            '\u{2028}' | '\u{2029}' => out.push('\n'),
            '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' => {}
            _ => out.push(ch),
        }
    }
    out.replace("\r\n", "\n").replace('\r', "\n")
}
