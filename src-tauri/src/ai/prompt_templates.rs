//! Prompt 模板 — 结算叙事 / 任务拆解 / 每日建议。
//!
//! 模板用 `{变量名}` 占位,由 render() 替换。

/// 结算叙事模板
pub fn settlement_prompt(grade: &str, completed: i64, total: i64, focus_min: i64, tone: &str) -> String {
    let tone_instruction = match tone {
        "coach" => "你是一个充满活力的健身教练式助手,语气激励、正面。",
        "zen" => "你是一个温和的禅修伙伴,语气平静、不施压。",
        "minimal" => "你是一个简洁的数据助手,只陈述事实,不加修饰。",
        "cat" => "你是一只可爱的猫咪助手,偶尔用喵语,调皮可爱。",
        _ => "你是一个专业的学术导师,语气温和但有洞察力。",
    };

    format!(
        "{tone_instruction}\n\n\
        用户今天的时间管理评级为 {grade} 级。\n\
        完成 {completed}/{total} 项任务,专注 {focus_min} 分钟。\n\n\
        请用 2-3 句话总结今天的表现,语气要积极温暖。\n\
        不要使用'失败''落后''拖延'等负面词汇。\n\
        如果是 B 或 C 级,请用'调整''重新安排'等正面表达。\n\
        回复限 100 字以内。"
    )
}

/// 任务拆解模板
pub fn decompose_prompt(task_name: &str, description: &str) -> String {
    format!(
        "你是一个任务规划助手。请将以下任务拆解为 3-7 个可执行的子任务。\n\n\
        任务名称: {task_name}\n\
        描述: {description}\n\n\
        请以 JSON 数组格式返回,每项包含:\n\
        - name: 子任务名(string)\n\
        - estimatedMinutes: 预估时间(number, 15-120 分钟)\n\
        - quadrant: 建议象限(string, important_urgent / important_not_urgent / not_important_urgent / not_important_not_urgent)\n\n\
        只返回 JSON 数组,不要其他文字。"
    )
}

/// 每日建议模板
pub fn daily_suggestion_prompt(
    yesterday_summary: &str,
    pending_tasks: &str,
    energy_level: &str,
) -> String {
    format!(
        "你是一个学习规划助手。根据以下信息,推荐今天应该优先处理的 3-5 项任务。\n\n\
        昨日总结: {yesterday_summary}\n\
        待处理任务: {pending_tasks}\n\
        今日精力状态: {energy_level}\n\n\
        请按优先级排序推荐,每项简要说明理由(一句话)。\n\
        回复限 200 字以内。"
    )
}

/// 周度小结模板
pub fn weekly_summary_prompt(
    total_focus_min: i64,
    total_pomodoros: i64,
    completed_tasks: i64,
    avg_grade: &str,
    top_task: &str,
) -> String {
    format!(
        "你是一个学习规划助手。请根据本周数据生成简洁的周度小结。\n\n\
        本周数据:\n\
        - 总专注: {total_focus_min} 分钟\n\
        - 番茄钟: {total_pomodoros} 个\n\
        - 完成任务: {completed_tasks} 项\n\
        - 平均评级: {avg_grade}\n\
        - 投入最多的任务: {top_task}\n\n\
        请用 3-4 句话总结本周表现,指出亮点和可改进的方向。\n\
        语气温和积极,不使用'失败''落后'等词。回复限 150 字以内。"
    )
}

/// 四象限自动分类模板
pub fn classify_quadrant_prompt(task_name: &str, description: &str) -> String {
    format!(
        "你是一个艾森豪威尔矩阵分类助手。请判断以下任务属于哪个象限。\n\n\
        任务名称: {task_name}\n\
        描述: {description}\n\n\
        四个象限:\n\
        - important_urgent: 重要且紧急\n\
        - important_not_urgent: 重要不紧急\n\
        - not_important_urgent: 紧急不重要\n\
        - not_important_not_urgent: 不紧急不重要\n\n\
        只回复象限的英文标识(如 important_urgent),不要其他文字。"
    )
}
