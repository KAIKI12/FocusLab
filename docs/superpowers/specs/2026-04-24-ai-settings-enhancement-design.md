# AI 设置增强设计

- 日期：2026-04-24
- 状态：已确认，待实现
- 关联页面：[`src/views/SettingsView.vue`](src/views/SettingsView.vue:1)
- 关联后端：[`src-tauri/src/commands/ai_commands.rs`](src-tauri/src/commands/ai_commands.rs:1)、[`src-tauri/src/ai/prompt_templates.rs`](src-tauri/src/ai/prompt_templates.rs:1)、[`src-tauri/src/ai/mod.rs`](src-tauri/src/ai/mod.rs:1)

---

## 1. 背景与现状

当前 FocusLab 的 AI 设置已经具备最基础的连接配置能力：

- 可配置 `provider`
- 可配置 `baseUrl`
- 可配置 `apiKey`
- 可配置 `model`
- 可进行连接测试
- 可查看 AI payload 示例

现状入口位于 [`src/views/SettingsView.vue`](src/views/SettingsView.vue:58) 的 AI 区块，前端调用入口位于 [`src/stores/useAIStore.ts`](src/stores/useAIStore.ts:24)，后端配置与调用集中在 [`src-tauri/src/commands/ai_commands.rs`](src-tauri/src/commands/ai_commands.rs:12) 与 [`src-tauri/src/ai/mod.rs`](src-tauri/src/ai/mod.rs:133)。

但从产品完整度看，目前 AI 设置仍偏“开发者向”，存在以下问题：

1. **只有连接配置，没有使用策略配置**
   - 用户只能填接口参数，无法配置 AI 的交互风格、输出强度、启用范围。

2. **Provider 支持弱感知**
   - 代码层面是 OpenAI-compatible 抽象，但设置页没有把常见 provider 预设做成易选项。

3. **缺少协议格式选择**
   - 当前系统把“供应商名称”和“请求协议”几乎视为同一层概念，这会限制后续支持 Claude 原生格式以及更多自定义网关。

4. **缺少全局 AI 开关**
   - 当前 AI 一旦配置成功，产品内多个入口会默认尝试调用，没有“总开关”可用于临时停用。

5. **缺少连接状态与成本感知**
   - 用户无法知道当前配置是否处于“已连接/待测试/失败”状态，也无法快速理解不同 AI 功能的消耗等级。

6. **设计文档中已有方向但未落地**
   - [`docs/03-AI系统设计.md`](docs/03-AI系统设计.md:140) 已明确提出“AI 语气风格系统”“语气强度滑块”“成本控制”等方向，但现实现尚未承接。

---

## 2. 目标

本次增强的目标不是扩展 AI 能力本身，而是**完善“AI 设置”作为产品控制面板的完整性**，让它同时覆盖：

- 连接配置
- 协议格式配置
- 使用开关
- 交互偏好
- 成本认知
- 连接反馈

最终让 AI 设置从“参数填写区”升级为“AI 能力控制台”。

---

## 3. 设计原则

### 3.1 先增强配置体验，不重做上层 AI 能力组织

现有后端已经通过 [`AIService`](src-tauri/src/ai/mod.rs:136) 抽象 provider，且大部分 AI 命令共用同一调用入口。本次增强应尽量复用现有命令结构，但允许底层从“单一 OpenAI-compatible”升级为“多协议 provider 实现”。

### 3.2 设置项全部落到统一的 settings KV

当前项目已有通用设置读写命令 [`get_setting`](src-tauri/src/commands/settings_commands.rs:9) 与 [`set_setting`](src-tauri/src/commands/settings_commands.rs:16)。AI 新增配置应继续落在 `settings` 表中，避免引入单独配置表。

### 3.3 把 provider 与 api_format 明确拆开

这里是这次设计调整的关键。

- `provider` 表示服务商身份或预设来源
- `api_format` 表示真正决定 HTTP 协议的数据格式

同一个 provider 通常有默认 `api_format`，但对于 `custom` 类型，用户应该能自行指定它是：

- `openai` 格式
- `claude` 格式

### 3.4 优先做用户可感知收益最高的项

本次先做：

- 全局 AI 开关
- Provider 预设
- 自定义 provider 的协议格式选择
- 模型默认值/候选
- 语气风格
- 语气强度
- 连接状态显示
- 用量提示

不做：

- 多 API Key 管理
- provider 级独立高级参数面板
- 模型能力自动探测
- AI 使用统计报表

---

## 4. 方案对比

### 方案 A：仅补连接配置层

内容：

- 增加更多 provider
- 自动填充默认 base URL
- 增加连接状态显示
- 增加测试结果持久化

优点：

- 改动小
- 技术风险低
- 能快速提升可用性

缺点：

- 仍然没有“AI 如何说话”“AI 是否启用”这类产品级配置
- 仍然无法解决“自定义供应商使用 OpenAI 还是 Claude 格式”的问题
- 与 [`docs/03-AI系统设计.md`](docs/03-AI系统设计.md:140) 的目标不一致

### 方案 B：完整增强设置层 + 协议抽象（推荐）

内容：

- 补连接配置层
- 补自定义 provider 的协议格式选择
- 补全局 AI 开关
- 补语气风格
- 补语气强度
- 补连接状态
- 补用量提示
- 后端支持 `openai` / `claude` 双协议实现

优点：

- 产品完整度最高
- 与现有 AI 设计文档方向一致
- 能覆盖真实使用场景中的中转站、自建网关、第三方代理
- 为后续扩展 provider 提供更清晰的结构边界

缺点：

- 涉及前后端联动
- 需要升级 [`src-tauri/src/ai/mod.rs`](src-tauri/src/ai/mod.rs:1) 的 provider 抽象
- Claude 协议需要单独实现请求与解析逻辑

### 方案 C：前端展示增强，后端不读偏好

内容：

- 前端提供风格、强度、开关、协议格式 UI
- 只做持久化展示，暂不实际接入后端行为

优点：

- 实现快
- UI 演示效果完整

缺点：

- 容易出现“看起来可配，实际上不生效”
- 用户提出的协议格式需求不能真正满足
- 与现有项目中“设置要真正控制行为”的模式不一致

### 结论

选择 **方案 B**。

原因：

- 它在体验、真实可用性、与既有文档一致性之间最平衡。
- 新增复杂度主要集中在 provider 协议层，整体仍是可控范围。

---

## 5. 最终设计

## 5.1 设置页结构调整

调整 [`src/views/SettingsView.vue`](src/views/SettingsView.vue:438) 的 AI 区块，使其分为五个子区域：

### A. AI 总开关

新增一个总开关：

- 标签：`启用 AI 助手`
- 说明：关闭后，任务拆解、每日建议、周度小结、结算寄语等能力全部停止调用 AI
- 数据键：`ai_enabled`

行为规则：

- 关闭时，设置页内 AI 配置区域仍可见，但以“已停用”状态展示
- 关闭后前端 AI 入口可保留按钮，但调用时直接进入降级逻辑或显示“AI 功能已关闭”
- 关闭不清空 API Key 与连接参数，方便之后重新开启

### B. 连接配置区

保留现有字段，但增强交互：

- Provider 下拉：从“通用 provider 类型”升级为“用户熟悉的服务商选项”
- 当 provider=`custom` 时，显示“协议格式”选择器
- Base URL：在切换 provider 或协议格式时自动填默认值
- API Key：本地模型可隐藏或弱化必填提示
- Model：支持默认候选 + 自定义输入
- 测试结果：显示最近一次测试状态

### C. 交互风格区

新增“AI 语气风格”配置，选项对齐 [`docs/03-AI系统设计.md`](docs/03-AI系统设计.md:144)：

- `academic`：学术导师型
- `coach`：健身教练型
- `zen`：禅意陪伴型
- `minimal`：极简数据型
- `cat`：猫咪陪伴型
- `custom`：自定义风格

其中：

- 选中 `custom` 时，展开一个多行输入框用于填写自定义 prompt 描述
- 该描述作为 tone 扩展信息写入后端 prompt 逻辑

### D. 语气强度区

新增滑块配置：

- 范围：1~5
- 文案：`少话/简洁 ←→ 多鼓励/详细`
- 数据键：`ai_tone_intensity`

该项不直接写入 prompt 文本修饰，而是由后端读取后映射为：

- `temperature`
- `max_tokens`

这样实现更自然，也更容易统一控制所有 AI 命令。

### E. 用量提示区

新增静态说明卡片，用于解释各 AI 功能的消耗等级，降低用户对 API 消耗的不确定感。

建议展示 3 档：

- Level 1：每日建议 / 日结算寄语 / 简短鼓励，低消耗
- Level 2：任务拆解 / 四象限分类 / 速记整理，中消耗
- Level 3：周度小结 / 长文本分析，高消耗

该分层直接承接 [`docs/03-AI系统设计.md`](docs/03-AI系统设计.md:9)。

---

## 5.2 Provider 与协议格式设计

### 第一层：预设 provider

用于覆盖常见服务商，帮助用户快速填充默认参数。

| ID | 显示名 | 协议格式 | 默认 Base URL | 默认 Model |
|---|---|---|---|---|
| `compatible` | OpenAI 兼容 | `openai` | `https://api.openai.com` | `gpt-4o-mini` |
| `openai` | OpenAI | `openai` | `https://api.openai.com` | `gpt-4o-mini` |
| `deepseek` | DeepSeek | `openai` | `https://api.deepseek.com` | `deepseek-chat` |
| `zhipu` | 智谱 AI | `openai` | `https://open.bigmodel.cn/api/paas/v4` | `glm-4-flash` |
| `qwen` | 通义千问 | `openai` | `https://dashscope.aliyuncs.com/compatible-mode/v1` | `qwen-turbo` |
| `anthropic` | Anthropic Claude | `claude` | `https://api.anthropic.com` | `claude-3-5-sonnet-latest` |
| `ollama` | Ollama 本地 | `openai` | `http://localhost:11434` | `llama3` |

### 第二层：自定义 provider

新增一个 `custom` 选项。用户选择后，不再只配置“是哪家服务商”，而是可以额外指定这个自定义供应商采用哪种 API 协议格式。

新增字段：

- `provider = custom`
- `api_format`：`openai` | `claude`
- `base_url`
- `api_key`
- `model`

这意味着设置页需要同时支持：

1. **预设服务商**：快速填默认值
2. **自定义服务商**：用户自己决定接口地址 + 模型 + 协议格式

这样的设计更贴近真实使用场景，因为很多第三方中转站、自建网关、企业代理并不属于固定品牌，但会兼容某一种协议。

### 技术取舍

后端不再把 provider 仅理解为“服务商名称”，而是拆成两个概念：

- `provider`：服务商身份或预设来源
- `api_format`：真正决定请求协议的格式类型

推荐协议枚举：

- `openai`
- `claude`

原因：

- 当前 [`AIService::configure`](src-tauri/src/ai/mod.rs:147) 实际只内建了 OpenAI-compatible 逻辑，这对大部分国内外兼容服务商够用，但不足以覆盖 Claude 原生协议。
- 用户提出“自定义供应商可选 OpenAI 格式或 Claude 格式”后，系统需要把“品牌/provider”和“协议格式”拆开，否则后续会把多个维度耦合在一起。
- 这样以后即使新增别的服务商，也只是在预设层加选项；真正决定请求实现的是 `api_format`。

换言之：

- `ollama` 仍可默认走 `openai` 格式
- DeepSeek、智谱、通义等继续走 `openai` 格式
- Claude 官方或兼容 Claude 协议的自定义服务走 `claude` 格式
- `custom` provider 是一个协议可选的通用入口，而不是只支持 OpenAI-compatible

---

## 5.3 数据模型设计

新增以下 settings key：

| Key | 默认值 | 说明 |
|---|---|---|
| `ai_enabled` | `1` | 全局 AI 开关 |
| `ai_provider` | `compatible` | 预设 provider 或 `custom` |
| `ai_api_format` | `openai` | 实际请求协议格式：`openai` / `claude` |
| `ai_base_url` | `` | AI 服务地址 |
| `ai_api_key` | `` | API Key |
| `ai_model` | `` | 模型名 |
| `ai_tone` | `academic` | AI 风格 |
| `ai_tone_custom` | `` | 自定义风格说明 |
| `ai_tone_intensity` | `3` | 强度等级 |
| `ai_connection_status` | `` | 最近一次测试结果状态 |
| `ai_connection_checked_at` | `` | 最近一次测试时间戳 |

说明：

- 仍沿用统一 `settings` 表，不引入新的配置表
- 时间字段建议直接存 ISO 字符串，方便前端展示“刚刚/几分钟前/某日某时”

---

## 5.4 前端行为设计

### 初始化加载

在 [`src/views/SettingsView.vue`](src/views/SettingsView.vue:175) 的 `onMounted` 初始化流程中，补充读取：

- `ai_enabled`
- `ai_provider`
- `ai_api_format`
- `ai_base_url`
- `ai_api_key`
- `ai_model`
- `ai_tone`
- `ai_tone_custom`
- `ai_tone_intensity`
- `ai_connection_status`
- `ai_connection_checked_at`

### 保存行为

当前 [`onSaveAI`](src/views/SettingsView.vue:65) 仅保存基础连接字段。增强后改为一次性提交：

- provider
- apiFormat
- baseUrl
- apiKey
- model
- tone
- toneCustom
- intensity
- enabled

### 测试连接行为

当前 [`onTestAI`](src/views/SettingsView.vue:72) 仅返回瞬时字符串。增强后应：

1. 调用测试命令
2. 根据结果写入：
   - `ai_connection_status`
   - `ai_connection_checked_at`
3. 设置页展示“已连接/失败/最近测试时间”

### Provider 切换行为

当用户切换 provider 时：

- 若切到预设 provider，则自动带出该 provider 的默认 `api_format`、Base URL 与 Model
- 若切到 `custom`，则展示额外的“协议格式”选择器，允许用户在 `openai` / `claude` 间切换
- 若当前 Base URL 仍是上一个 provider 的默认值，则自动替换为新 provider 默认值
- 若用户手动改过 Base URL，则不强制覆盖
- Model 同理：仅在用户未手改时自动替换默认值
- 当 `api_format` 改变时，若 Base URL 仍等于协议默认值，可同步切换为该协议推荐默认值

---

## 5.5 Store 设计

[`src/stores/useAIStore.ts`](src/stores/useAIStore.ts:24) 需增强为“配置状态中心”，而不只是调用封装。

建议新增状态：

- `enabled`
- `provider`
- `apiFormat`
- `baseUrl`
- `model`
- `tone`
- `toneCustom`
- `toneIntensity`
- `connectionStatus`
- `connectionCheckedAt`

但本次不必把所有 settings 读取逻辑都下沉到 store。考虑现有项目风格，设置页自己读写 settings 已较多，因此：

- **设置页仍可主导读取与持久化**
- `useAIStore` 只扩展 `configure` 的入参，确保与后端能力对齐

这是最小侵入方案。

---

## 5.6 后端行为设计

### A. `configure_ai` 扩展

当前 [`configure_ai`](src-tauri/src/commands/ai_commands.rs:14) 入参只有：

- `provider`
- `base_url`
- `api_key`
- `model`

增强为可同时接收：

- `enabled`
- `api_format`
- `tone`
- `tone_custom`
- `intensity`

并统一写入 `settings` 表。

其中：

- `provider` 用于表示预设来源或 `custom`
- `api_format` 用于决定底层协议实现

这两个字段不能再混用。

### B. AI 全局开关拦截

所有 AI 命令在真正调用模型前，先读取 `ai_enabled`：

- 为 `0` 时直接返回 `AI 功能已关闭`
- 前端收到后走已有错误提示或降级文案

涉及命令包括：

- [`ai_decompose_task`](src-tauri/src/commands/ai_commands.rs:67)
- [`ai_settlement_narrative`](src-tauri/src/commands/ai_commands.rs:100)
- [`ai_daily_suggestions`](src-tauri/src/commands/ai_commands.rs:135)
- [`ai_classify_quadrant`](src-tauri/src/commands/ai_commands.rs:197)
- [`ai_optimize_quick_note`](src-tauri/src/commands/ai_commands.rs:240)
- [`ai_weekly_summary`](src-tauri/src/commands/ai_commands.rs:260)
- `test_ai_connection` 是否受开关限制：**不受限制**，因为用户在关闭状态下也可能需要先测试配置

### C. 语气风格接入

当前仅少数 prompt 接收 `tone` 文本，且来源不统一。增强后建议统一策略：

- 后端增加读取 `ai_tone` / `ai_tone_custom`
- prompt 模板接收统一 tone 描述
- `custom` 时拼入用户自定义的风格说明

首批接入建议覆盖：

- 日结算寄语
- 每日建议
- 周度小结

任务拆解、四象限分类这类偏结构化输出场景，可暂不受 tone 影响，避免影响可解析性。

### D. 强度滑块接入

所有适合自然语言输出的 AI 命令，在调用 [`CompletionOptions`](src-tauri/src/ai/mod.rs:26) 前，读取 `ai_tone_intensity`，做统一映射：

| 强度 | temperature | max_tokens |
|---|---:|---:|
| 1 | 0.30 | 100 |
| 2 | 0.45 | 180 |
| 3 | 0.60 | 280 |
| 4 | 0.75 | 380 |
| 5 | 0.90 | 500 |

接入范围：

- 日结算寄语
- 每日建议
- 周度小结
- 速记整理

不建议接入：

- 任务拆解
- 四象限分类

原因：

- 这两类更偏结构化输出，稳定性应优先于“语气变化”。

---

## 5.7 协议抽象设计

[`src-tauri/src/ai/mod.rs`](src-tauri/src/ai/mod.rs:1) 当前只有 `CompatibleProvider`，本次设计调整后应升级为“双协议抽象”：

- `OpenAICompatibleProvider`
- `ClaudeProvider`

它们都实现统一的 [`AIProvider`](src-tauri/src/ai/mod.rs:34) trait。

### OpenAI 格式

维持现状：

- 请求路径：`/v1/chat/completions`
- 核心字段：`model`、`messages`、`temperature`、`max_tokens`
- 适用：OpenAI、DeepSeek、智谱、通义、Ollama、各类 OpenAI-compatible 网关

### Claude 格式

新增 Claude 原生协议实现，典型设计为：

- 请求路径：`/v1/messages`
- Header 包含 `x-api-key` 与 `anthropic-version`
- Body 结构与 OpenAI 格式不同
- 返回结果需要单独解析文本 `content`

### 配置路由规则

[`AIService::configure`](src-tauri/src/ai/mod.rs:147) 后续不应只根据 `provider_type` 分支，而应根据 `api_format` 分支：

- `api_format = openai` → 创建 `OpenAICompatibleProvider`
- `api_format = claude` → 创建 `ClaudeProvider`

这样：

- 预设 provider 只是帮用户填默认值
- 真正决定 HTTP 协议的是 `api_format`

这是本次设计调整里最关键的结构变化。

---

## 5.8 Prompt 模板设计

[`src-tauri/src/ai/prompt_templates.rs`](src-tauri/src/ai/prompt_templates.rs:1) 增加一个统一 helper，用于把 tone ID 映射成 prompt 文本。

示例思路：

- `academic` → “请使用理性、专业、结构清晰的表达方式。”
- `coach` → “请使用积极、有推动感、鼓励式表达方式。”
- `zen` → “请使用温和、包容、不施压的表达方式。”
- `minimal` → “请尽量简洁，以事实和建议为主，避免情绪化表达。”
- `cat` → “请使用轻松、可爱、减压的表达方式，但不要过度幼稚。”
- `custom` → 使用 `ai_tone_custom`

该 helper 被自然语言场景复用，避免每个 prompt 独立维护风格映射。

---

## 5.9 错误处理与降级

### 场景 1：AI 关闭

- 后端返回明确错误：`AI 功能已关闭`
- 前端将其转化为温和提示，不作为异常崩溃处理

### 场景 2：provider 配置不完整

- 缺失 API Key 且不是 Ollama：保存允许，但测试连接失败
- 前端显示“待补全配置”而非“连接失败”

### 场景 3：Claude 配置不完整

- 若 `api_format=claude` 但缺失 Claude 所需 header 前置条件或 base URL 不正确，测试连接应返回协议明确错误
- 前端应提示“当前配置采用 Claude 格式，请检查 URL / Key / 模型名是否匹配该协议”

### 场景 4：测试失败

- 保留最近失败状态与时间
- 不清空已有配置

### 场景 5：自定义 tone 为空

- 若 `ai_tone=custom` 且内容为空，保存时提示补全
- 或自动回退到 `academic`

推荐：**前端阻止保存**，避免生成无意义配置。

---

## 5.10 测试设计

### 前端测试

重点验证：

1. provider 切换时默认 `api_format` / Base URL / Model 是否正确更新
2. 选择 `custom` 时是否出现协议格式选择器
3. Ollama 时 API Key 字段是否弱化显示
4. tone 选择与自定义输入展示逻辑是否正确
5. 强度滑块显示值与存储值是否一致
6. 连接状态展示是否正确反映最近测试结果

### 后端测试

重点验证：

1. `configure_ai` 能正确写入新增 settings
2. `ai_enabled=0` 时 AI 命令是否被正确拦截
3. `api_format=openai` 与 `api_format=claude` 是否正确路由到不同 provider
4. Claude 响应内容能否稳定解析为文本
5. 强度等级映射是否正确
6. tone helper 是否能覆盖预设与 custom 两类输入
7. 结构化输出命令未被 tone/intensity 误影响

---

## 6. 影响范围与副作用分析

### 影响文件

- [`src/views/SettingsView.vue`](src/views/SettingsView.vue:1)
- [`src/stores/useAIStore.ts`](src/stores/useAIStore.ts:1)
- [`src-tauri/src/commands/ai_commands.rs`](src-tauri/src/commands/ai_commands.rs:1)
- [`src-tauri/src/ai/mod.rs`](src-tauri/src/ai/mod.rs:1)
- [`src-tauri/src/ai/prompt_templates.rs`](src-tauri/src/ai/prompt_templates.rs:1)
- 可能补充：[`docs/03-AI系统设计.md`](docs/03-AI系统设计.md:1) 后续同步更新

### 潜在副作用

1. **协议实现复杂度上升**
   - 从单一 OpenAI-compatible 扩展为 `openai + claude` 双协议后，后端 provider 抽象和响应解析复杂度会提高。

2. **AI 输出风格变化**
   - 引入 tone/intensity 后，自然语言场景输出会更可变，需防止和现有 UI 长度不匹配。

3. **设置页复杂度上升**
   - AI 区块会从“简单 4 字段表单”变成“多分组控制面板”，需通过分区和说明降低认知负担。

4. **旧用户兼容性**
   - 旧用户没有新增 settings key 时，必须有稳定默认值。

5. **结构化输出稳定性风险**
   - 如果 tone/intensity 不慎影响 JSON 输出场景，可能导致解析失败，因此本设计明确限制影响范围。

---

## 7. 实施结论

本设计确认采用 **完整增强方案（方案 B）**：

- 前端补全 AI 设置页为完整控制台
- 后端在保持现有命令结构的前提下，扩展为按 `api_format` 选择底层协议实现
- 所有新增项统一落到 `settings` 表
- 自定义供应商支持选择 `openai` 或 `claude` 协议格式
- 语气强度通过 `temperature + max_tokens` 映射实现
- 结构化 AI 能力保持稳定优先，不强行套用语气变化

该方案能在不重做上层 AI 能力组织的前提下，显著提升设置页的产品完成度、用户理解度与后续扩展性。
