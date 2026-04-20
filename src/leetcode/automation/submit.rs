use anyhow::Result;
use headless_chrome::Tab;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::sync::atomic::AtomicBool;
use super::daily::{add_log, check_cancel};

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
    logs: &Arc<Mutex<Vec<String>>>,
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
                add_log(logs, &format!("{}失败，准备重试 ({}/{}): {}", label, i, attempts, e));
                std::thread::sleep(Duration::from_millis(600 * i as u64));
            }
        }
    }
    Err(anyhow::anyhow!("{}失败：超过最大重试次数", label))
}

fn read_judge_outcome(tab: &Arc<Tab>) -> Result<(JudgeOutcome, String)> {
    let eval = tab.evaluate(
        r#"
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
        "#,
        false,
    )?;

    let json_str = eval
        .value
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| r#"{"kind":"Unknown","snippet":""}"#.to_string());

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
    let eval = tab.evaluate(
        &format!(
            r#"
            (function() {{
                const target = `{}`;
                const langs = new Set(["C++","Java","Python","Python3","Rust","Go","JavaScript","TypeScript","C","C#","Kotlin","Swift"]);

                const norm = (s) => (s || "").replace(/\s+/g, " ").trim();
                const visible = (el) => {{
                    if (!el) return false;
                    if (el.offsetParent === null) return false;
                    const style = window.getComputedStyle(el);
                    if (!style) return true;
                    return style.visibility !== "hidden" && style.display !== "none";
                }};

                const findLangButtonNearSmartMode = (ctx) => {{
                    const buttons = Array.from(ctx.querySelectorAll("button")).filter(visible);
                    if (buttons.length === 0) return null;

                    const smart = buttons.find(b => {{
                        const t = norm(b.innerText);
                        return t.includes("智能模式") || t.toLowerCase().includes("smart");
                    }});

                    if (smart) {{
                        const idx = buttons.indexOf(smart);
                        let best = null;
                        let bestDist = 1e9;
                        for (let i = 0; i < buttons.length; i++) {{
                            const t = norm(buttons[i].innerText);
                            if (!langs.has(t)) continue;
                            const dist = Math.abs(i - idx);
                            if (dist < bestDist) {{
                                bestDist = dist;
                                best = buttons[i];
                            }}
                        }}
                        if (best) return best;
                    }}

                    return buttons.find(b => langs.has(norm(b.innerText))) || null;
                }};

                const editor = document.querySelector(".monaco-editor");
                let scope = editor;
                for (let i = 0; i < 10 && scope && scope.parentElement; i++) scope = scope.parentElement;
                const root = scope || document;

                const btn = findLangButtonNearSmartMode(root) || findLangButtonNearSmartMode(document);
                if (!btn) return JSON.stringify({{ ok: false, current: "", reason: "no-lang-button" }});

                const current = norm(btn.innerText);
                if (current === target) return JSON.stringify({{ ok: true, current, reason: "already" }});

                btn.click();

                const selectTarget = () => {{
                    const popoverSelectors = [
                        '[role="listbox"]',
                        '[role="menu"]',
                        '[role="dialog"]',
                        'div[class*="popover"]',
                        'div[class*="dropdown"]',
                        'div[class*="select"]',
                        'div[class*="Select"]',
                    ].join(',');

                    const popovers = Array.from(document.querySelectorAll(popoverSelectors))
                        .filter(visible)
                        .filter(p => norm(p.textContent).includes(target));

                    const popupRoot = popovers[0] || document;

                    const candidates = Array.from(popupRoot.querySelectorAll('li,button,a,div,span'))
                        .filter(visible)
                        .filter(el => norm(el.textContent) === target);

                    if (candidates.length === 0) return false;

                    const pickClickable = (el) => {{
                        let cur = el;
                        while (cur && cur !== document.body) {{
                            const role = (cur.getAttribute && cur.getAttribute('role')) || '';
                            const tag = (cur.tagName || '').toUpperCase();
                            const style = window.getComputedStyle(cur);
                            const cursor = style ? style.cursor : '';

                            if (role === 'option' || role === 'menuitem') return cur;
                            if (tag === 'LI' || tag === 'BUTTON' || tag === 'A') return cur;
                            if (cursor === 'pointer') return cur;

                            cur = cur.parentElement;
                        }}
                        return el;
                    }};

                    const targetEl = pickClickable(candidates[0]);
                    if (targetEl.scrollIntoView) {{
                        targetEl.scrollIntoView({{ block: 'center', inline: 'center' }});
                    }}

                    const fire = (type) => {{
                        targetEl.dispatchEvent(new MouseEvent(type, {{ bubbles: true, cancelable: true, view: window }}));
                    }};

                    fire('mouseover');
                    fire('mousedown');
                    fire('mouseup');
                    fire('click');
                    return true;
                }};

                const readCurrent = () => {{
                    const btn2 = findLangButtonNearSmartMode(root) || btn;
                    return norm(btn2.innerText);
                }};

                return new Promise((resolve) => {{
                    setTimeout(() => {{
                        const clicked = selectTarget();
                        setTimeout(() => {{
                            const cur2 = readCurrent();
                            resolve(JSON.stringify({{ ok: cur2 === target, current: cur2, clicked }}));
                        }}, 450);
                    }}, 300);
                }});
            }})();
            "#,
            escape_for_js_template_literal(lang)
        ),
        true,
    )?;

    let json_str = eval
        .value
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| r#"{"ok":false,"current":""}"#.to_string());

    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_default();
    Ok(parsed["ok"].as_bool().unwrap_or(false))
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
        .unwrap_or_else(|| r#"{"ok":false,"method":"none","len":0,"expectedLen":0,"hasMarker":false}"#.to_string());

    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_default();
    let ok = parsed["ok"].as_bool().unwrap_or(false);
    let method = parsed["method"].as_str().unwrap_or("none").to_string();
    Ok((ok, method))
}

fn try_click_submit(tab: &Arc<Tab>) -> Result<bool> {
    let eval = tab.evaluate(
        r#"
        (function() {
            const buttons = Array.from(document.querySelectorAll('button'));
            const target = buttons.find(btn => {
                const t = (btn.innerText || '').trim();
                if (!t) return false;
                if (!(t === '提交' || t === 'Submit' || t.includes('提交'))) return false;
                if (btn.disabled) return false;
                const ariaDisabled = btn.getAttribute('aria-disabled');
                if (ariaDisabled === 'true') return false;
                return true;
            });
            if (target) {
                target.click();
                return true;
            }
            return false;
        })();
        "#,
        false,
    )?;

    Ok(eval.value.and_then(|v| v.as_bool()).unwrap_or(false))
}

fn try_claim_points(tab: &Arc<Tab>) -> Result<bool> {
    let eval = tab.evaluate(
        r#"
        (function() {
            const text = document.body ? document.body.innerText : "";
            if (!text.includes("恭喜完成今日打卡任务")) return false;
            const buttons = Array.from(document.querySelectorAll("button"));
            const target = buttons.find(btn => {
                const t = (btn.innerText || "").trim();
                if (!t) return false;
                return (t.includes("领取") && t.includes("积分")) || t.includes("领取奖励");
            });
            if (target) {
                target.click();
                return true;
            }
            return false;
        })();
        "#,
        false,
    )?;
    Ok(eval.value.and_then(|v| v.as_bool()).unwrap_or(false))
}

pub fn submit_code(
    tab: &Arc<Tab>,
    code: &str,
    lang: &str,
    logs: &Arc<Mutex<Vec<String>>>,
    cancel_flag: &Arc<AtomicBool>,
) -> Result<()> {
    add_log(logs, "在题解页面右侧准备填入代码...");

    retry(logs, cancel_flag, "切换语言", 3, |attempt| {
        check_cancel(cancel_flag)?;
        add_log(logs, &format!("正在右侧编辑器切换语言为 {}... (第 {} 次)", lang, attempt));
        let ok = try_switch_language(tab, lang)?;
        if ok {
            Ok(())
        } else {
            Err(anyhow::anyhow!("未成功切换语言到 {}", lang))
        }
    })?;

    let (set_ok, method) = retry(logs, cancel_flag, "写入代码", 3, |attempt| {
        check_cancel(cancel_flag)?;
        add_log(logs, &format!("写入代码到编辑器... (第 {} 次)", attempt));
        let (ok, method) = try_set_code(tab, code)?;
        if ok {
            Ok((true, method))
        } else {
            Err(anyhow::anyhow!("写入后校验失败 (method={})", method))
        }
    })?;
    if set_ok {
        add_log(logs, &format!("✅ 代码写入成功 (method={})", method));
    }

    retry(logs, cancel_flag, "点击提交", 3, |attempt| {
        check_cancel(cancel_flag)?;
        add_log(logs, &format!("点击提交... (第 {} 次)", attempt));
        if try_click_submit(tab)? {
            Ok(())
        } else {
            Err(anyhow::anyhow!("未找到可点击的提交按钮"))
        }
    })?;

    add_log(logs, "⏳ 等待判题结果...");
    let outcome = wait_for_judge_result(tab, logs, cancel_flag, Duration::from_secs(120))?;

    match outcome {
        JudgeOutcome::Accepted => {
            add_log(logs, "✅ 判题通过/打卡成功");
            check_cancel(cancel_flag)?;
            add_log(logs, "尝试领取积分...");
            let claimed = try_claim_points(tab).unwrap_or(false);
            if claimed {
                std::thread::sleep(Duration::from_secs(2));
                add_log(logs, "✅ 已触发领取积分");
            } else {
                add_log(logs, "未检测到可领取积分入口（可能已领取或页面结构变化）");
            }
            add_log(logs, "✅ 打卡流程执行完毕！");
            Ok(())
        }
        other => {
            let (_, snippet) = read_judge_outcome(tab).unwrap_or((JudgeOutcome::Unknown, String::new()));
            Err(anyhow::anyhow!("判题未通过: {:?}\n{}", other, snippet))
        }
    }
}
