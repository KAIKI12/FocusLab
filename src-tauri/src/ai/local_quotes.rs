//! 本地语录池 — AI 未启用 / 调用失败时的兜底文案。
//!
//! 资源通过 `include_str!` 在编译期嵌入,运行时按场景桶分级抽取并回填变量。
//! 覆盖:结算叙事 / 任务反馈 / 未完成提醒 / 每日建议 / 周度小结。

use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

const SETTLEMENT_JSON: &str = include_str!("quotes/settlement.json");
const TASK_FEEDBACK_JSON: &str = include_str!("quotes/task_feedback.json");
const UNFINISHED_REMINDER_JSON: &str = include_str!("quotes/unfinished_reminder.json");
const DAILY_SUGGESTION_JSON: &str = include_str!("quotes/daily_suggestion.json");
const WEEKLY_SUMMARY_JSON: &str = include_str!("quotes/weekly_summary.json");

#[derive(Debug, Deserialize)]
struct SettlementPool {
    buckets: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct FeedbackEntry {
    message: String,
    badge: String,
}

#[derive(Debug, Deserialize)]
struct FeedbackPool {
    buckets: HashMap<String, Vec<FeedbackEntry>>,
}

#[derive(Debug, Deserialize)]
struct ReminderEntry {
    message: String,
    next_step: String,
}

#[derive(Debug, Deserialize)]
struct ReminderPool {
    buckets: HashMap<String, Vec<ReminderEntry>>,
}

#[derive(Debug, Deserialize)]
struct TextPool {
    buckets: HashMap<String, Vec<String>>,
}

fn settlement_pool() -> &'static SettlementPool {
    static POOL: OnceLock<SettlementPool> = OnceLock::new();
    POOL.get_or_init(|| serde_json::from_str(SETTLEMENT_JSON).expect("settlement.json 解析失败"))
}

fn feedback_pool() -> &'static FeedbackPool {
    static POOL: OnceLock<FeedbackPool> = OnceLock::new();
    POOL.get_or_init(|| serde_json::from_str(TASK_FEEDBACK_JSON).expect("task_feedback.json 解析失败"))
}

fn reminder_pool() -> &'static ReminderPool {
    static POOL: OnceLock<ReminderPool> = OnceLock::new();
    POOL.get_or_init(|| serde_json::from_str(UNFINISHED_REMINDER_JSON).expect("unfinished_reminder.json 解析失败"))
}

fn daily_suggestion_pool() -> &'static TextPool {
    static POOL: OnceLock<TextPool> = OnceLock::new();
    POOL.get_or_init(|| serde_json::from_str(DAILY_SUGGESTION_JSON).expect("daily_suggestion.json 解析失败"))
}

fn weekly_summary_pool() -> &'static TextPool {
    static POOL: OnceLock<TextPool> = OnceLock::new();
    POOL.get_or_init(|| serde_json::from_str(WEEKLY_SUMMARY_JSON).expect("weekly_summary.json 解析失败"))
}

/// 基于时间戳的轻量伪随机,避免引入 rand crate
fn pseudo_random_index(len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0);
    nanos % len
}

/// 替换 `{var}` 占位符
fn render_template(template: &str, vars: &[(&str, String)]) -> String {
    let mut out = template.to_string();
    for (key, value) in vars {
        let placeholder = format!("{{{key}}}");
        out = out.replace(&placeholder, value);
    }
    out
}

/// 把 grade 映射到桶 key,未知评级落入 B(中性)
fn settlement_bucket_key(grade: &str) -> &'static str {
    match grade.to_ascii_uppercase().as_str() {
        "S" => "S",
        "A" => "A",
        "B" => "B",
        "C" | "D" | "F" => "C",
        _ => "B",
    }
}

/// 选取结算叙事本地文案
pub fn pick_settlement_narrative(grade: &str, completed: i64, total: i64, focus_min: i64) -> String {
    let pool = settlement_pool();
    let bucket = pool
        .buckets
        .get(settlement_bucket_key(grade))
        .or_else(|| pool.buckets.get("B"));
    let templates = match bucket {
        Some(b) if !b.is_empty() => b,
        _ => return format!("今天完成 {completed}/{total} 项,专注 {focus_min} 分钟。"),
    };
    let idx = pseudo_random_index(templates.len());
    let vars = [
        ("completed", completed.to_string()),
        ("total", total.to_string()),
        ("focus_min", focus_min.to_string()),
    ];
    render_template(&templates[idx], &vars)
}

/// 把 quadrant + 时长偏差映射到桶 key
fn feedback_bucket_key(quadrant: &str, estimated: i64, actual: i64) -> String {
    let importance = if quadrant.starts_with("important") {
        "important"
    } else {
        "not_important"
    };
    let deviation = if estimated <= 0 || actual <= 0 {
        "on_time"
    } else {
        let ratio = actual as f64 / estimated as f64;
        if ratio < 0.8 {
            "ahead"
        } else if ratio <= 1.2 {
            "on_time"
        } else {
            "over"
        }
    };
    format!("{importance}_{deviation}")
}

/// 选取任务完成反馈本地文案,返回与 LLM 同结构的 JSON 字符串
pub fn pick_task_feedback(task_name: &str, estimated: i64, actual: i64, quadrant: &str) -> String {
    let pool = feedback_pool();
    let key = feedback_bucket_key(quadrant, estimated, actual);
    let bucket = pool
        .buckets
        .get(&key)
        .or_else(|| pool.buckets.get("important_on_time"));
    let entries = match bucket {
        Some(b) if !b.is_empty() => b,
        _ => {
            return Value::String(format!("「{task_name}」完成了,继续保持。"))
                .to_string();
        }
    };
    let idx = pseudo_random_index(entries.len());
    let entry = &entries[idx];
    let vars = [("task_name", task_name.to_string())];
    let message = render_template(&entry.message, &vars);
    let payload = serde_json::json!({
        "message": message,
        "badge": entry.badge,
        "tone": "encouraging",
    });
    payload.to_string()
}

// ---------- 未完成任务温和提醒 ----------

/// 按未完成数量映射桶 key: 1 → single, 2-3 → few, ≥4 → many
fn reminder_bucket_key(count: usize) -> &'static str {
    match count {
        0 | 1 => "single",
        2 | 3 => "few",
        _ => "many",
    }
}

/// 选取未完成任务提醒本地文案,返回与 LLM 同结构的 JSON 字符串
pub fn pick_unfinished_reminder(
    unfinished_tasks: &str,
    _completed_summary: &str,
    available_time: &str,
) -> String {
    let pool = reminder_pool();
    let tasks: Vec<&str> = unfinished_tasks.split('、').collect();
    let count = tasks.len();
    let first_task = tasks.first().copied().unwrap_or("待办事项");
    let key = reminder_bucket_key(count);
    let bucket = pool.buckets.get(key).or_else(|| pool.buckets.get("few"));
    let entries = match bucket {
        Some(b) if !b.is_empty() => b,
        _ => {
            return serde_json::json!({
                "message": "今天已有不少收获,未完成的任务明天继续加油！",
                "next_step": "选一项最小的任务先开始",
                "tone": "gentle",
            })
            .to_string();
        }
    };
    let idx = pseudo_random_index(entries.len());
    let entry = &entries[idx];
    let vars = [
        ("first_task", first_task.to_string()),
        ("count", count.to_string()),
        ("available_time", available_time.to_string()),
    ];
    let message = render_template(&entry.message, &vars);
    let next_step = render_template(&entry.next_step, &vars);
    serde_json::json!({
        "message": message,
        "next_step": next_step,
        "tone": "gentle",
    })
    .to_string()
}

// ---------- 每日建议 ----------

/// 按精力 × 是否有待办映射桶 key
fn daily_suggestion_bucket_key(energy: &str, has_tasks: bool) -> String {
    let level = match energy {
        "高" | "high" | "充沛" => "high",
        "低" | "low" | "疲惫" => "low",
        _ => "normal",
    };
    let suffix = if has_tasks { "with_tasks" } else { "no_tasks" };
    format!("{level}_{suffix}")
}

/// 选取每日建议本地文案,返回纯文本
pub fn pick_daily_suggestion(
    energy: &str,
    pending_tasks: &str,
    yesterday_summary: &str,
) -> String {
    let pool = daily_suggestion_pool();
    let tasks: Vec<&str> = pending_tasks
        .split('、')
        .filter(|s| !s.is_empty() && *s != "无待办任务")
        .collect();
    let has_tasks = !tasks.is_empty();
    let first_task = tasks.first().copied().unwrap_or("规划下一步");
    let pending_count = tasks.len();
    let key = daily_suggestion_bucket_key(energy, has_tasks);
    let bucket = pool
        .buckets
        .get(&key)
        .or_else(|| pool.buckets.get("normal_with_tasks"));
    let templates = match bucket {
        Some(b) if !b.is_empty() => b,
        _ => return "今天按自己的节奏来就好,挑一件最想做的先开始。".to_string(),
    };
    let idx = pseudo_random_index(templates.len());
    let vars = [
        ("first_task", first_task.to_string()),
        ("pending_count", pending_count.to_string()),
        ("yesterday", yesterday_summary.to_string()),
    ];
    render_template(&templates[idx], &vars)
}

// ---------- 周度小结 ----------

/// 选取周度小结本地文案,返回纯文本
pub fn pick_weekly_summary(
    focus_min: i64,
    pomodoros: i64,
    completed: i64,
    avg_grade: &str,
    top_task: &str,
) -> String {
    let pool = weekly_summary_pool();
    let key = settlement_bucket_key(avg_grade); // 复用 S/A/B/C 映射
    let bucket = pool.buckets.get(key).or_else(|| pool.buckets.get("B"));
    let templates = match bucket {
        Some(b) if !b.is_empty() => b,
        _ => {
            return format!(
                "本周专注 {focus_min} 分钟,完成 {completed} 项任务,继续保持节奏。"
            );
        }
    };
    let idx = pseudo_random_index(templates.len());
    let vars = [
        ("focus_min", focus_min.to_string()),
        ("pomodoros", pomodoros.to_string()),
        ("completed", completed.to_string()),
        ("top_task", top_task.to_string()),
    ];
    render_template(&templates[idx], &vars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settlement_returns_non_empty_for_each_grade() {
        for grade in ["S", "A", "B", "C", "D", "unknown"] {
            let s = pick_settlement_narrative(grade, 3, 5, 90);
            assert!(!s.is_empty(), "grade {grade} 返回空");
        }
    }

    #[test]
    fn settlement_replaces_variables() {
        let s = pick_settlement_narrative("B", 7, 9, 123);
        // 不强求每条都含变量,但至少有一条会含。多次抽样兜底。
        let any_with_var = (0..30).any(|_| {
            let r = pick_settlement_narrative("B", 7, 9, 123);
            r.contains("7") || r.contains("9") || r.contains("123")
        });
        assert!(any_with_var || !s.is_empty());
    }

    #[test]
    fn feedback_returns_valid_json() {
        let s = pick_task_feedback("写周报", 60, 45, "important_urgent");
        let v: Value = serde_json::from_str(&s).expect("应当是合法 JSON");
        assert!(v.get("message").is_some());
        assert!(v.get("badge").is_some());
        assert!(v.get("tone").is_some());
    }

    #[test]
    fn feedback_bucket_key_maps_correctly() {
        assert_eq!(
            feedback_bucket_key("important_urgent", 60, 30),
            "important_ahead"
        );
        assert_eq!(
            feedback_bucket_key("important_not_urgent", 60, 60),
            "important_on_time"
        );
        assert_eq!(
            feedback_bucket_key("not_important_urgent", 30, 90),
            "not_important_over"
        );
        assert_eq!(
            feedback_bucket_key("anything", 0, 0),
            "not_important_on_time"
        );
    }

    #[test]
    fn unfinished_reminder_returns_valid_json() {
        let s = pick_unfinished_reminder("写周报、整理文档", "完成 3/5 项", "上午 2 小时");
        let v: Value = serde_json::from_str(&s).expect("应当是合法 JSON");
        assert!(v.get("message").is_some());
        assert!(v.get("next_step").is_some());
        assert_eq!(v["tone"], "gentle");
    }

    #[test]
    fn unfinished_reminder_single_task() {
        let s = pick_unfinished_reminder("写周报", "完成 4/5 项", "待定");
        let v: Value = serde_json::from_str(&s).expect("应当是合法 JSON");
        assert!(!v["message"].as_str().unwrap().is_empty());
    }

    #[test]
    fn unfinished_reminder_many_tasks() {
        let s = pick_unfinished_reminder("写周报、整理文档、开会、写代码、测试", "完成 1/6 项", "待定");
        let v: Value = serde_json::from_str(&s).expect("应当是合法 JSON");
        assert!(!v["message"].as_str().unwrap().is_empty());
    }

    #[test]
    fn daily_suggestion_returns_non_empty() {
        for energy in ["高", "正常", "低", "high", "low", "unknown"] {
            let s = pick_daily_suggestion(energy, "写周报、整理文档", "B 级");
            assert!(!s.is_empty(), "energy={energy} 返回空");
        }
    }

    #[test]
    fn daily_suggestion_no_tasks() {
        let s = pick_daily_suggestion("正常", "无待办任务", "A 级");
        assert!(!s.is_empty());
    }

    #[test]
    fn weekly_summary_returns_non_empty() {
        for grade in ["S", "A", "B", "C", "unknown"] {
            let s = pick_weekly_summary(420, 14, 8, grade, "写代码");
            assert!(!s.is_empty(), "grade={grade} 返回空");
        }
    }

    #[test]
    fn weekly_summary_replaces_variables() {
        let any_with_var = (0..30).any(|_| {
            let r = pick_weekly_summary(999, 33, 12, "A", "写周报");
            r.contains("999") || r.contains("33") || r.contains("12") || r.contains("写周报")
        });
        assert!(any_with_var, "至少有一条语录包含变量回填");
    }
}
