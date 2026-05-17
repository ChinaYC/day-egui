use super::daily::check_cancel;
use super::utils::ElementUtils;
use anyhow::Result;
use headless_chrome::Tab;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, error, info, warn};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum JudgeOutcome {
    Accepted,
    CompilationError,
    RuntimeError,
    WrongAnswer,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    OutputLimitExceeded,
    Pending,
    Unknown,
}

fn escape_for_js_template_literal(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${")
}

fn retry<T, F>(
    _logs: &Arc<Mutex<Vec<String>>>,
    cancel_flag: &Arc<AtomicBool>,
    label: &str,
    attempts: usize,
    mut f: F,
) -> Result<T>
where
    F: FnMut(usize) -> Result<T>,
{
    for i in 1..=attempts {
        check_cancel(cancel_flag)?;
        match f(i) {
            Ok(v) => return Ok(v),
            Err(e) => {
                if i >= attempts {
                    return Err(e);
                }
                warn!("{}失败，准备重试 ({}/{}): {}", label, i, attempts, e);
                std::thread::sleep(Duration::from_millis(600 * i as u64));
            }
        }
    }
    Err(anyhow::anyhow!("{}失败：超过最大重试次数", label))
}

fn read_judge_outcome(tab: &Arc<Tab>) -> Result<(JudgeOutcome, String)> {
    let js_code = r#"
        (function() {
            const text = (document.body && document.body.innerText) ? document.body.innerText : "";
            const hay = text.replace(/\s+/g, " ").trim();
            const pick = (kind, snippet) => JSON.stringify({ kind, snippet: snippet || "" });

            if (hay.includes("编译出错") || hay.includes("Compilation Error")) return pick("CompilationError", hay.slice(0, 2600));
            if (hay.includes("运行错误") || hay.includes("Runtime Error")) return pick("RuntimeError", hay.slice(0, 2600));
            if (hay.includes("解答错误") || hay.includes("Wrong Answer")) return pick("WrongAnswer", hay.slice(0, 2600));
            if (hay.includes("超出时间限制") || hay.includes("Time Limit Exceeded")) return pick("TimeLimitExceeded", hay.slice(0, 2600));
            if (hay.includes("超出内存限制") || hay.includes("Memory Limit Exceeded")) return pick("MemoryLimitExceeded", hay.slice(0, 2600));
            if (hay.includes("超出输出限制") || hay.includes("Output Limit Exceeded")) return pick("OutputLimitExceeded", hay.slice(0, 2600));

            if (hay.includes("通过") || hay.includes("Accepted") || hay.includes("恭喜完成今日打卡任务")) return pick("Accepted", hay.slice(0, 2600));

            if (hay.includes("判题中") || hay.includes("运行中") || hay.includes("Judging") || hay.includes("Running")) return pick("Pending", "");

            return pick("Unknown", hay.slice(0, 2600));
        })();
        "#;
    
    debug!("执行读取判题结果 JS");
    let eval = tab.evaluate(js_code, false)?;

    let json_str = eval
        .value
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| r#"{"kind":"Unknown","snippet":""}"#.to_string());
    
    debug!("判题结果响应: {}", json_str);

    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_default();
    let kind = parsed["kind"].as_str().unwrap_or("Unknown");
    let snippet = parsed["snippet"].as_str().unwrap_or("").to_string();

    let outcome = match kind {
        "Accepted" => JudgeOutcome::Accepted,
        "CompilationError" => JudgeOutcome::CompilationError,
        "RuntimeError" => JudgeOutcome::RuntimeError,
        "WrongAnswer" => JudgeOutcome::WrongAnswer,
        "TimeLimitExceeded" => JudgeOutcome::TimeLimitExceeded,
        "MemoryLimitExceeded" => JudgeOutcome::MemoryLimitExceeded,
        "OutputLimitExceeded" => JudgeOutcome::OutputLimitExceeded,
        "Pending" => JudgeOutcome::Pending,
        _ => JudgeOutcome::Unknown,
    };

    Ok((outcome, snippet))
}

fn wait_for_judge_result(
    tab: &Arc<Tab>,
    _logs: &Arc<Mutex<Vec<String>>>,
    cancel_flag: &Arc<AtomicBool>,
    timeout: Duration,
) -> Result<JudgeOutcome> {
    let start = std::time::Instant::now();
    loop {
        check_cancel(cancel_flag)?;
        if start.elapsed() > timeout {
            return Err(anyhow::anyhow!("等待判题结果超时"));
        }

        let (outcome, _) = read_judge_outcome(tab)?;
        match outcome {
            JudgeOutcome::Pending | JudgeOutcome::Unknown => {
                std::thread::sleep(Duration::from_millis(800));
                continue;
            }
            _ => return Ok(outcome),
        }
    }
}

fn try_switch_language(tab: &Arc<Tab>, lang: &str) -> Result<bool> {
    let lang_safe = escape_for_js_template_literal(lang);
    debug!("执行切换语言 JS, 目标: {}", lang);
    let eval = tab.evaluate(
        &format!(
            r#"
            (async function() {{
                const target = `{}`;
                const norm = (s) => (s || "").replace(/\s+/g, " ").trim();
                const visible = (el) => {{
                    if (!el) return false;
                    const rect = el.getBoundingClientRect();
                    if (rect.width === 0 || rect.height === 0) return false;
                    const style = window.getComputedStyle(el);
                    return style && style.display !== 'none' && style.visibility !== 'hidden' && style.opacity !== '0';
                }};

                // 1. 查找语言切换按钮 (改进版)
                const findLangBtn = () => {{
                    // 使用 reverse() 从后往前找，因为右侧编辑器通常在 DOM 树的较后位置
                    const buttons = Array.from(document.querySelectorAll('button, [role="button"], [role="combobox"]')).reverse();
                    // 优先找明确包含当前语言名称的按钮
                    let btn = buttons.find(b => {{
                        const text = norm(b.innerText);
                        return visible(b) && text.length < 20 && (
                            text.includes("Rust") || text.includes("C++") || text.includes("Java") || 
                            text.includes("Python") || text.includes("Go") || text.includes("JavaScript") ||
                            text.includes("C#") || text.includes("TypeScript") || text.includes("PHP") ||
                            text.includes("Ruby") || text.includes("Swift") || text.includes("Kotlin")
                        );
                    }});
                    
                    if (!btn) {{
                        // 兜底：寻找带有 aria-haspopup 的按钮，或者 id 包含 headlessui-listbox-button
                        btn = buttons.find(b => {{
                            return visible(b) && (
                                b.getAttribute('aria-haspopup') === 'listbox' || 
                                (b.id && b.id.includes('headlessui-listbox-button'))
                            );
                        }});
                    }}
                    return btn;
                }};

                let btn = findLangBtn();
                if (!btn) return JSON.stringify({{ ok: false, reason: "lang-button-not-found" }});

                const currentLang = norm(btn.innerText);
                if (currentLang.includes(target)) return JSON.stringify({{ ok: true, reason: "already-set" }});

                // 2. 点击展开列表 (模拟完整点击事件)
                const fireClick = (el) => {{
                    el.dispatchEvent(new MouseEvent('mousedown', {{ bubbles: true, cancelable: true, view: window }}));
                    el.dispatchEvent(new MouseEvent('mouseup', {{ bubbles: true, cancelable: true, view: window }}));
                    el.click();
                }};

                fireClick(btn);
                
                // 等待列表出现 (增加等待时间)
                await new Promise(r => setTimeout(r, 800));

                // 3. 在整个文档中查找目标语言选项 (更广泛的搜索)
                const findTargetOption = () => {{
                    // 不依赖特定 role，因为 LeetCode 有时会用纯 div/span 布局
                    const candidates = Array.from(document.querySelectorAll('*'))
                        .reverse()
                        .filter(visible)
                        .filter(el => el.children.length <= 1) // 排除大容器，只保留叶子节点或包含单一图标的节点
                        .map(el => ({{ el, text: norm(el.innerText || el.textContent) }}))
                        .filter(({{ text }}) => {{
                            return text === target || 
                                   text === target + " (Beta)" || 
                                   text === target + " (New)";
                        }});
                    
                    // 过滤掉明显的非选项元素（如文章里的代码块标签或大段落）
                    const validCandidates = candidates.filter(({{ el }}) => {{
                        const tag = el.tagName.toLowerCase();
                        if (tag === 'p' || tag === 'h1' || tag === 'h2' || tag === 'h3' || tag === 'code' || tag === 'pre') return false;
                        return true;
                    }});

                    if (validCandidates.length > 0) {{
                        // 优先找带有 cursor: pointer 的，因为选项通常可以点击
                        const pointerCandidates = validCandidates.filter(({{ el }}) => window.getComputedStyle(el).cursor === 'pointer');
                        if (pointerCandidates.length > 0) {{
                            // 如果选中的元素有特定的可点击祖先，优先返回祖先以确保点击有效
                            const opt = pointerCandidates[0].el.closest('[role="option"], [role="menuitem"], li, .ant-select-item') || pointerCandidates[0].el;
                            return opt;
                        }}
                        
                        const opt = validCandidates[0].el.closest('[role="option"], [role="menuitem"], li, .ant-select-item') || validCandidates[0].el;
                        return opt;
                    }}
                    return null;
                }};

                let option = findTargetOption();
                if (!option) {{
                    // 尝试再次点击按钮（有时候第一次点击没反应）
                    fireClick(btn);
                    await new Promise(r => setTimeout(r, 800));
                    option = findTargetOption();
                }}

                if (!option) return JSON.stringify({{ ok: false, reason: "option-not-found", current: currentLang }});

                // 4. 点击选项
                fireClick(option);
                
                // 等待切换完成
                await new Promise(r => setTimeout(r, 1000));
                
                // 5. 再次检查当前语言
                btn = findLangBtn();
                const finalLang = btn ? norm(btn.innerText) : "";
                const ok = finalLang.includes(target);
                return JSON.stringify({{ 
                    ok: ok, 
                    current: finalLang, 
                    reason: ok ? "" : `Final language "${{finalLang}}" does not match target "${{target}}"` 
                }});
            }})();
            "#,
            lang_safe
        ),
        true,
    )?;

    let json_str = eval
        .value
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| r#"{"ok":false,"reason":"eval-failed"}"#.to_string());
    
    debug!("切换语言结果响应: {}", json_str);

    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_default();
    let ok = parsed["ok"].as_bool().unwrap_or(false);
    if !ok {
        warn!("切换语言失败: {}", parsed["reason"].as_str().unwrap_or("unknown"));
    }
    Ok(ok)
}

fn try_set_code(tab: &Arc<Tab>, code: &str) -> Result<(bool, String)> {
    let safe_code = escape_for_js_template_literal(code);
    let eval = tab.evaluate(
        &format!(
            r#"
            (function() {{
                const code = `{}`;
                const expectedLen = code.length;
                let method = "none";

                const editorAvailable = (typeof monaco !== "undefined") && monaco && monaco.editor;
                if (editorAvailable) {{
                    const models = monaco.editor.getModels();
                    if (models && models.length > 0) {{
                        models[0].setValue(code);
                        method = "monaco";
                    }}
                }}

                let textarea = null;
                if (method === "none") {{
                    textarea = document.querySelector("textarea");
                    if (textarea) {{
                        textarea.value = code;
                        textarea.dispatchEvent(new Event("input", {{ bubbles: true }}));
                        method = "textarea";
                    }}
                }}

                return new Promise((resolve) => {{
                    setTimeout(() => {{
                        let current = "";
                        if ((typeof monaco !== "undefined") && monaco && monaco.editor) {{
                            const models = monaco.editor.getModels();
                            if (models && models.length > 0) {{
                                current = models[0].getValue();
                            }}
                        }}
                        if (!current && textarea) current = textarea.value || "";

                        const hasMarker = current.includes("//day编写");
                        const lenOk = current.length >= Math.min(expectedLen, 50) && current.length >= expectedLen * 0.8;
                        const ok = hasMarker && lenOk;

                        resolve(JSON.stringify({{ ok, method, len: current.length, expectedLen, hasMarker }}));
                    }}, 150);
                }});
            }})();
            "#,
            safe_code
        ),
        true,
    )?;

    let json_str = eval
        .value
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| {
            r#"{"ok":false,"method":"none","len":0,"expectedLen":0,"hasMarker":false}"#.to_string()
        });

    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_default();
    let ok = parsed["ok"].as_bool().unwrap_or(false);
    let method = parsed["method"].as_str().unwrap_or("none").to_string();
    Ok((ok, method))
}

fn try_click_submit(tab: &Arc<Tab>) -> Result<bool> {
    ElementUtils::click_by_text(tab, "button", "提交")
}

fn try_claim_points(tab: &Arc<Tab>) -> Result<bool> {
    let has_claim = ElementUtils::eval_bool(tab, "document.body.innerText.includes('恭喜完成今日打卡任务')")?;
    if !has_claim {
        return Ok(false);
    }
    ElementUtils::click_by_text(tab, "button", "领取")
}

pub fn submit_code(
    tab: &Arc<Tab>,
    code: &str,
    lang: &str,
    logs: &Arc<Mutex<Vec<String>>>,
    cancel_flag: &Arc<AtomicBool>,
) -> Result<()> {
    info!(user = true, "在题解页面右侧准备填入代码...");

    retry(logs, cancel_flag, "切换语言", 3, |attempt| {
        check_cancel(cancel_flag)?;
        info!(user = true, "正在右侧编辑器切换语言为 {}... (第 {} 次)", lang, attempt);
        let ok = try_switch_language(tab, lang)?;
        if ok {
            Ok(())
        } else {
            Err(anyhow::anyhow!("未成功切换语言到 {}", lang))
        }
    })?;

    let (set_ok, method) = retry(logs, cancel_flag, "写入代码", 3, |attempt| {
        check_cancel(cancel_flag)?;
        info!(user = true, "写入代码到编辑器... (第 {} 次)", attempt);
        let (ok, method) = try_set_code(tab, code)?;
        if ok {
            Ok((true, method))
        } else {
            Err(anyhow::anyhow!("写入后校验失败 (method={})", method))
        }
    })?;
    if set_ok {
        info!(user = true, "✅ 代码写入成功 (method={})", method);
    }

    retry(logs, cancel_flag, "点击提交", 3, |attempt| {
        check_cancel(cancel_flag)?;
        info!(user = true, "点击提交... (第 {} 次)", attempt);
        if try_click_submit(tab)? {
            Ok(())
        } else {
            Err(anyhow::anyhow!("未找到可点击的提交按钮"))
        }
    })?;

    info!(user = true, "⏳ 等待判题结果...");
    let outcome = wait_for_judge_result(tab, logs, cancel_flag, Duration::from_secs(120))?;

    match outcome {
        JudgeOutcome::Accepted => {
            info!(user = true, "✅ 判题通过/打卡成功");
            check_cancel(cancel_flag)?;
            info!(user = true, "尝试领取积分...");
            let claimed = try_claim_points(tab).unwrap_or(false);
            if claimed {
                std::thread::sleep(Duration::from_secs(2));
                info!(user = true, "✅ 已触发领取积分");
            } else {
                info!(user = true, "未检测到可领取积分入口（可能已领取或页面结构变化）");
            }
            info!(user = true, "✅ 打卡流程执行完毕！");
            Ok(())
        }
        other => {
            let (_, snippet) =
                read_judge_outcome(tab).unwrap_or((JudgeOutcome::Unknown, String::new()));
            let err_msg = format!("判题未通过: {:?}\n{}", other, snippet);
            error!(user = true, "{}", err_msg);
            Err(anyhow::anyhow!(err_msg))
        }
    }
}
