//! Prompt 模板 — 结算叙事 / 任务拆解 / 每日建议 / 新 AI 场景接入。
//!
//! 模板用 `{变量名}` 占位,由 render() 替换。

/// 根据语气标识返回人格描述字符串。
///
/// - `"coach"`  → 健身教练式，激励正面
/// - `"zen"`    → 禅修伙伴，平静不施压
/// - `"minimal"`→ 简洁数据助手，只陈述事实
/// - `"cat"`    → 猫咪助手，调皮可爱
/// - `"custom"` + custom 非空 → 使用自定义风格描述
/// - 其他（含 `"academic"`）→ 专业学术导师
pub fn tone_description(tone: &str, custom: &str) -> String {
    match tone {
        "coach" => "你是一个充满活力的健身教练式助手，语气激励、正面，多用感叹号和行动词。".to_string(),
        "zen" => "你是一个温和的禅修伙伴，语气平静、不施压，多用包容性表达。".to_string(),
        "minimal" => "你是一个简洁的数据助手，只陈述事实，不加情绪化修饰，回复尽量简短。".to_string(),
        "cat" => "你是一只可爱的猫咪助手，偶尔用喵语，调皮可爱但不失实用。".to_string(),
        "custom" if !custom.is_empty() => format!("请遵循以下风格要求：{custom}"),
        _ => "你是一个专业的学术导师，语气温和但有洞察力，表达结构清晰。".to_string(),
    }
}

/// 结算叙事模板
pub fn settlement_prompt(grade: &str, completed: i64, total: i64, focus_min: i64, tone: &str) -> String {
    let tone_instruction = tone_description(tone, "");

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
    tone: &str,
) -> String {
    let tone_instruction = tone_description(tone, "");

    format!(
        "{tone_instruction}\n\n\
        根据以下信息,推荐今天应该优先处理的 3-5 项任务。\n\n\
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
    tone: &str,
) -> String {
    let tone_instruction = tone_description(tone, "");

    format!(
        "{tone_instruction}\n\n\
        请根据本周数据生成简洁的周度小结。\n\n\
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

/// 速记便签 AI 优化模板 — 将不稳定输入梳理为 3 种思路版本
pub fn quick_note_optimization_prompt(raw_text: &str) -> String {
    format!(
        "你是一个专注力管理助手。用户会输入碎片化、跳跃、不完整的想法，可能只有关键词、半句话或未成型判断。\n\
        你的任务不是把它转成任务，而是先理解用户输入意图，再把这段思路梳理为 3 个可保存为灵感的版本。\n\n\
        用户输入:\n{raw_text}\n\n\
        请返回 JSON 格式（不要包含 markdown 代码块标记）:\n\
        {{\n\
          \"candidates\": [\n\
            {{\n\
              \"label\": \"A\",\n\
              \"style\": \"faithful\",\n\
              \"styleName\": \"忠实整理版\",\n\
              \"text\": \"在不改变原意的前提下，把跳跃表达整理成更连贯的思路\"\n\
            }},\n\
            {{\n\
              \"label\": \"B\",\n\
              \"style\": \"question\",\n\
              \"styleName\": \"研究问题版\",\n\
              \"text\": \"把输入中隐含的问题、假设或矛盾提炼成更清晰的研究问题\"\n\
            }},\n\
            {{\n\
              \"label\": \"C\",\n\
              \"style\": \"direction\",\n\
              \"styleName\": \"推进思路版\",\n\
              \"text\": \"整理成一个可继续展开的思考方向，指出下一步值得思考什么\"\n\
            }}\n\
          ]\n\
        }}\n\n\
        要求:\n\
        1. 三个候选都必须围绕用户的思路本身，不要转为待办、任务、执行清单或时间管理建议\n\
        2. 每个 text 应尽可能理解用户输入意图，允许补足逻辑连接，但不要添加用户没有提到的事实\n\
        3. 如果用户输入很短，也要保留不确定性，不要替用户下结论\n\
        4. 每个 text 建议 2-4 句话，优先完整表达，不要为了短而截断\n\
        5. 只返回 JSON，不要其他文字"
    )
}

#[cfg(test)]
mod tests {
    use super::quick_note_optimization_prompt;

    #[test]
    fn quick_note_prompt_focuses_on_thought_versions_not_tasks() {
        let prompt = quick_note_optimization_prompt("脑子里有个实验想法但还没理顺");

        assert!(prompt.contains("专注力管理助手"));
        assert!(prompt.contains("忠实整理版"));
        assert!(prompt.contains("研究问题版"));
        assert!(prompt.contains("推进思路版"));
        assert!(prompt.contains("理解用户输入意图"));
        assert!(!prompt.contains("偏任务导向"));
        assert!(!prompt.contains("待办事项"));
        assert!(!prompt.contains("转为任务"));
    }
}

/// 未完成任务温和提醒模板
pub fn unfinished_reminder_prompt(
    unfinished_tasks: &str,
    completed_summary: &str,
    available_time: &str,
    tone: &str,
    custom: &str,
) -> String {
    let tone_instruction = tone_description(tone, custom);

    format!(
        "{tone_instruction}\n\n\
        你需要根据用户今天的已完成情况与未完成任务,生成一条温和、无压力、可执行的提醒。\n\n\
        严格返回 JSON:\n\
        {{\n\
          \"message\": \"...\",\n\
          \"next_step\": \"...\",\n\
          \"tone\": \"gentle\"\n\
        }}\n\n\
        约束:\n\
        1. message 不超过 60 字,先肯定已完成部分,再自然提到未完成项\n\
        2. 禁止使用“失败、拖延、落后、糟糕”等负面词汇\n\
        3. next_step 必须是一个最小可执行动作,例如“先打开文档补 3 行提纲”\n\
        4. tone 固定为 gentle\n\
        5. 只返回 JSON,不要解释\n\n\
        输入:\n\
        - 未完成任务: {unfinished_tasks}\n\
        - 今日已完成: {completed_summary}\n\
        - 明日可用时间: {available_time}"
    )
}

/// 任务完成正反馈模板
pub fn task_feedback_prompt(
    task_name: &str,
    estimated_minutes: i64,
    actual_minutes: i64,
    quadrant: &str,
    tone: &str,
    custom: &str,
) -> String {
    let tone_instruction = tone_description(tone, custom);

    format!(
        "{tone_instruction}\n\n\
        你是一个温和鼓励型正反馈文案助手,专为用户完成单个任务后生成点赞鼓励文案。\n\n\
        严格只返回 JSON:\n\
        {{\n\
          \"message\": \"...\",\n\
          \"badge\": \"🎯或⭐或💪或✅中选一\",\n\
          \"tone\": \"encouraging\"\n\
        }}\n\n\
        要求:\n\
        1. message 字数不超过 40 字\n\
        2. 禁止使用任何负面词汇,如“终于、居然、竟然、还好、勉强”等\n\
        3. message 必须聚焦于本次任务的具体亮点,结合任务名称、预估时间、实际用时、所属象限生成个性化内容\n\
        4. badge 只能从 🎯 / ⭐ / 💪 / ✅ 中选择一个\n\
        5. tone 固定为 encouraging\n\
        6. 只返回 JSON,不要附加说明\n\n\
        输入:\n\
        - 任务名称: {task_name}\n\
        - 预估时间: {estimated_minutes} 分钟\n\
        - 实际用时: {actual_minutes} 分钟\n\
        - 所属象限: {quadrant}"
    )
}

/// 里程碑拆解建议模板
pub fn milestone_breakdown_prompt(
    goal_name: &str,
    goal_description: &str,
    total_deadline: &str,
    current_progress: &str,
) -> String {
    format!(
        "你是一个里程碑规划助手。用户会提供一个大型长线目标,你需要将其拆解为 3-7 个有序里程碑,帮助用户制定可执行的推进计划。\n\n\
        严格返回如下 JSON,不要输出任何其他内容:\n\
        {{\n\
          \"goal_understanding\": \"（≤40字复述用户目标,确认理解无误）\",\n\
          \"milestones\": [\n\
            {{\n\
              \"name\": \"（里程碑名称）\",\n\
              \"order\": 1,\n\
              \"deadline_hint\": \"（相对时间,如第2周末）\",\n\
              \"priority\": \"high\",\n\
              \"deliverable\": \"（具体可验证的交付物描述）\"\n\
            }}\n\
          ]\n\
        }}\n\n\
        约束:\n\
        - 里程碑数量为 3-7 个\n\
        - goal_understanding 不超过 40 字\n\
        - milestone.name 简洁明确,不超过 15 字\n\
        - deadline_hint 使用相对时间描述,并参考 total_deadline 合理分配\n\
        - priority 只能取值 high / medium / low\n\
        - deliverable 必须具体、可验证\n\
        - milestones 按 order 升序排列\n\
        - 若用户已有进度,从当前进度节点继续规划,不要重复已完成部分\n\
        - 只返回 JSON\n\n\
        输入:\n\
        - 目标名称: {goal_name}\n\
        - 目标描述: {goal_description}\n\
        - 总截止时间: {total_deadline}\n\
        - 当前进度: {current_progress}"
    )
}

/// 四象限自动分类模板
/// 任务预估时长 Prompt（历史数据辅助）
pub fn task_duration_prompt(
    task_name: &str,
    description: &str,
    similar_history: &str,
) -> String {
    format!(
        "你是一个任务时长预估助手。根据任务信息和历史数据，预估完成该任务所需的专注时长。\n\n\
        严格返回如下 JSON，不要输出任何其他内容：\n\
        {{\n\
          \"estimated_minutes\": 60,\n\
          \"confidence\": \"medium\",\n\
          \"reasoning\": \"（≤40字说明预估依据）\",\n\
          \"range\": {{\n\
            \"min\": 45,\n\
            \"max\": 75\n\
          }}\n\
        }}\n\n\
        约束：\n\
        - estimated_minutes 必须是 15 的倍数，范围 15-480\n\
        - confidence 只能取 high / medium / low\n\
        - reasoning 不超过 40 字，用中文\n\
        - range.min 和 range.max 同样是 15 的倍数\n\
        - 若历史数据为空，基于任务名称和描述进行合理估算\n\
        - 只返回 JSON\n\n\
        输入：\n\
        - 任务名称：{task_name}\n\
        - 任务描述：{description}\n\
        - 历史相似任务（任务名 → 实际用时）：\n{similar_history}"
    )
}

/// 里程碑风险预警 Prompt
pub fn milestone_risk_prompt(
    milestone_name: &str,
    goal_name: &str,
    target_date: &str,
    remaining_days: i64,
    done_subtasks: i64,
    total_subtasks: i64,
    recent_activity: &str,
) -> String {
    format!(
        "你是一个里程碑进度风险分析助手。请根据以下里程碑信息，分析当前到期风险并给出具体的补救建议。\n\n\
        严格返回如下 JSON，不要输出任何其他内容：\n\
        {{\n\
          \"risk_level\": \"high | medium | low\",\n\
          \"summary\": \"（≤50字，描述当前风险状况）\",\n\
          \"actions\": [\n\
            \"（建议行动1，≤25字）\",\n\
            \"（建议行动2，≤25字）\"\n\
          ]\n\
        }}\n\n\
        约束：\n\
        - risk_level 只能取 high / medium / low\n\
        - summary 不超过50字，禁止使用「失败」「放弃」「完不成」等负面绝对化词汇\n\
        - summary 应客观描述现状（剩余天数、当前完成率）和主要风险点\n\
        - actions 数组2-4项，每项不超过25字，必须具体可执行\n\
        - 若 total_subtasks 为0（无子任务），risk_level 应为 high，并在 actions 中建议先拆解子任务\n\
        - 只返回 JSON，不输出任何解释或说明\n\n\
        输入：\n\
        - 里程碑名称：{milestone_name}\n\
        - 所属目标：{goal_name}\n\
        - 截止日期：{target_date}\n\
        - 剩余天数：{remaining_days} 天\n\
        - 子任务完成：{done_subtasks} / {total_subtasks} 项\n\
        - 近7天专注时长：{recent_activity}"
    )
}

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

// ---------- 灵感: AI 推荐归属目标 ----------
pub fn suggest_goal_prompt(inspiration_content: &str, goals: &str) -> String {
    format!(
        "你是科研灵感归属助手。判断这条灵感最适合挂到哪个长线目标下,并给出简短理由。\n\n\
        【灵感内容】\n{inspiration_content}\n\n\
        【可选目标列表】(每行: id|name)\n{goals}\n\n\
        要求:\n\
        1. 选出最相关的 1 个目标 id (从列表中);若都不相关回 \"none\"。\n\
        2. 给一句 ≤30 字的理由,说明为什么属于这个目标(或为什么不属于任何目标)。\n\n\
        严格按 JSON 返回,不要其他文字:\n\
        {{\"goalId\": \"<id 或 none>\", \"reason\": \"<理由>\"}}"
    )
}

// ---------- 灵感: AI 起草后续实验 ----------
pub fn draft_followup_prompt(parent_content: &str) -> String {
    format!(
        "你是科研助手。基于以下\"前置灵感\",起草一条\"后续实验\"灵感卡片,用来验证或推进这个想法。\n\n\
        【前置灵感】\n{parent_content}\n\n\
        要求:\n\
        - 用 1-2 句话描述具体可执行的实验步骤或验证方式\n\
        - 开头加上 [后续实验] 前缀\n\
        - 字数控制在 80 字以内\n\
        - 直接返回卡片正文,不要其他解释、不要引号包裹"
    )
}

// ---------- 灵感: AI 纠偏分析 ----------
pub fn correction_analysis_prompt(old_content: &str, new_content: &str) -> String {
    format!(
        "你是科研判断纠偏助手。用户在旧灵感中的判断,被新灵感所质疑/修正。请帮用户复盘。\n\n\
        【旧灵感(可能有误的判断)】\n{old_content}\n\n\
        【新灵感(质疑或修正)】\n{new_content}\n\n\
        请用 JSON 返回纠偏分析,不要其他文字:\n\
        {{\n\
          \"summary\": \"<≤40字: 矛盾/纠偏关键点>\",\n\
          \"oldJudgment\": \"<≤30字: 旧灵感的核心判断>\",\n\
          \"newEvidence\": \"<≤30字: 新灵感提供的证据/视角>\",\n\
          \"suggestion\": \"<≤40字: 建议下一步动作,如做对照实验/补充测量>\"\n\
        }}"
    )
}
