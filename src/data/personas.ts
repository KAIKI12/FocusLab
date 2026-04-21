/**
 * 人格元数据 · 全量从 prototype/screens/persona-card.html + persona-hatch.html 移植。
 *
 * 数据覆盖:
 *   - 28 条 base(原型 30 减去实现里未采用的 BATS/OWL)
 *   - 16 条 combo(原型 COMBOS 全部)
 *   - 7 天孵化剧情文案
 *
 * 键为人格中文名(与 PersonaView.vue 的 PERSONA_FILES 解析出的 name 一一对应)。
 */

export interface BasePersonaMeta {
  kind: "base";
  code: string;
  emoji: string;
  quote: string;
  hidden: string;
  /** 5 维星级: [时段偏好, DDL模式, 稳定性, 强度, 多线程] */
  dims: [number, number, number, number, number];
}

export interface ComboPersonaMeta {
  kind: "combo";
  emojis: string;
  desc: string;
}

export type PersonaMeta = BasePersonaMeta | ComboPersonaMeta;

export const DIM_LABELS = ["时段偏好", "DDL模式", "稳定性", "强度", "多线程"];
export const DIM_LOW = ["晨型", "提前", "波动", "低频", "单线"];
export const DIM_HIGH = ["夜型", "临阵", "稳态", "高产", "多线"];

/** 7 天孵化文案(对齐 persona-hatch.html DAYS 数组) */
export interface HatchDay {
  day: number;
  emoji: string;
  title: string;
  desc: string;
  /** 进度百分比 0-100 */
  pct: number;
  label: string;
}

export const HATCH_DAYS: HatchDay[] = [
  {
    day: 1,
    emoji: "🥚",
    title: "人格正在孵化中...",
    desc: "你刚刚加入 FocusLab,系统需要观察你的学习习惯。放心,每完成一个番茄钟,你的人格就更清晰一点。",
    pct: 14,
    label: "进度 14% · 1 天数据",
  },
  {
    day: 2,
    emoji: "🥚💫",
    title: "信号采集中",
    desc: "系统正在记录你的专注节奏 —— 你是晨型还是夜型?爆发型还是稳态型?数据正在说话。",
    pct: 28,
    label: "进度 28% · 2 天数据",
  },
  {
    day: 3,
    emoji: "🐣",
    title: "初步信号出现",
    desc: "初步信号显示,你可能是夜型生物。但还不够确定 —— 也许明天你会反转。",
    pct: 42,
    label: "进度 42% · 3 天数据",
  },
  {
    day: 4,
    emoji: "🐣🔍",
    title: "特征识别中",
    desc: "检测到你的 DDL 响应模式:截止前 48 小时完成率骤升。系统开始缩小候选范围...",
    pct: 57,
    label: "进度 57% · DDL 模式初现",
  },
  {
    day: 5,
    emoji: "🐥",
    title: "轮廓正在浮现",
    desc: "数据越来越清晰了...蝙蝠?猎豹?再等等,还有最后两天的数据要看。",
    pct: 71,
    label: "进度 71% · 候选 3 型",
  },
  {
    day: 6,
    emoji: "🐥✨",
    title: "即将揭晓",
    desc: "明天就可以解锁你的专属科研人格了!今天好好表现 —— 你的最终数据还在被记录。",
    pct: 85,
    label: "进度 85% · 明日揭晓",
  },
  {
    day: 7,
    emoji: "🎉",
    title: "解锁!你的科研人格已生成",
    desc: "基于你 7 天的真实数据,你的专属科研人格已经就位。继续使用,未来还会漂移变化。",
    pct: 100,
    label: "进度 100% · 完成",
  },
];

/** name → metadata。name 与 PERSONA_FILES 解析的 name 一一对应。 */
export const PERSONA_META: Record<string, PersonaMeta> = {
  // ---------- Base · 夜型 ----------
  "深夜幽灵": {
    kind: "base",
    code: "GHST",
    emoji: "🌑",
    quote: "0 点之后的你,连番茄钟都不敢打扰你。",
    hidden: "你的导师不知道你几点睡觉,你自己也不知道",
    dims: [5, 4, 1, 5, 1],
  },

  // ---------- Base · 晨型 ----------
  "晨鸣鹤": {
    kind: "base",
    code: "DAWN",
    emoji: "🐓",
    quote: "6 点的图书馆,你已经完成了第一个番茄钟。",
    hidden: "你发的朋友圈时间让室友怀疑人生",
    dims: [1, 2, 4, 3, 2],
  },
  "黎明特攻队": {
    kind: "base",
    code: "DMTK",
    emoji: "🌅",
    quote: "天没亮就开始工作,不是因为勤奋,是因为睡不着。",
    hidden: "焦虑是你的闹钟",
    dims: [1, 3, 3, 4, 1],
  },
  "六点蜜蜂": {
    kind: "base",
    code: "BEE6",
    emoji: "🐝",
    quote: "别人还在闹钟和被窝里搏斗,你已经清空了三封邮件。",
    hidden: "你是导师唯一不用催的学生,虽然导师不知道原因是焦虑",
    dims: [1, 1, 4, 3, 3],
  },

  // ---------- Base · DDL ----------
  "DDL炼金术士": {
    kind: "base",
    code: "DDLX",
    emoji: "🐆",
    quote: "截止日期前三天的你,能量密度超过太阳。",
    hidden: "你不是拖延,你是在「蓄力」,你一直这么告诉自己",
    dims: [3, 5, 1, 5, 3],
  },
  "火山爆发型选手": {
    kind: "base",
    code: "VLCN",
    emoji: "🌋",
    quote: "平静了二十天,最后两天把所有人都干沉默了。",
    hidden: "你的导师以为你在摸鱼,你只是在等待临界点",
    dims: [3, 5, 1, 5, 2],
  },
  "定时炸弹研究员": {
    kind: "base",
    code: "BOMB",
    emoji: "💣",
    quote: "DDL 是你唯一的生产力来源,这一点你心知肚明。",
    hidden: "没有 DDL 的任务对你来说等于不存在",
    dims: [3, 5, 2, 4, 1],
  },

  // ---------- Base · 佛系 ----------
  "稳态水母": {
    kind: "base",
    code: "STBL",
    emoji: "🪼",
    quote: "洋流再乱,你自漂浮。连续 14 天完成率从未低于 70%。",
    hidden: "你是实验室里最不焦虑的人,大家都想知道你的秘密",
    dims: [3, 1, 5, 3, 2],
  },
  "沉浸模式海龟": {
    kind: "base",
    code: "TRTL",
    emoji: "🐢",
    quote: "慢慢来,但你一直在游。别人冲刺,你匀速到终点。",
    hidden: "你进入心流的速度很慢,但出来的时候已经完成了别人三倍的工作",
    dims: [3, 1, 5, 4, 1],
  },
  "冬眠熊": {
    kind: "base",
    code: "HYBR",
    emoji: "🐻",
    quote: "该搁置的搁置,该爆发的爆发。你懂得战略性放弃。",
    hidden: "你的搁置区是一个哲学空间",
    dims: [3, 2, 4, 2, 2],
  },
  "树懒禅师": {
    kind: "base",
    code: "SLTH",
    emoji: "🦥",
    quote: "你不是没在做,你是在等灵感自己来敲门。",
    hidden: "你的搁置区不是垃圾桶,是发酵罐",
    dims: [3, 2, 3, 2, 1],
  },

  // ---------- Base · 猛冲 ----------
  "全天候鲨鱼": {
    kind: "base",
    code: "SHRK",
    emoji: "🦈",
    quote: "鲨鱼不睡觉,你也不停。每个时段都有你的专注记录。",
    hidden: "你的休息是换一个任务继续做",
    dims: [3, 3, 3, 5, 3],
  },
  "蜂巢建筑师": {
    kind: "base",
    code: "HIVE",
    emoji: "🐝",
    quote: "每一格都不浪费。你把碎片时间拼成了整块工作。",
    hidden: "等电梯的时候你在回复导师邮件",
    dims: [3, 2, 4, 4, 3],
  },
  "猎隼专注体": {
    kind: "base",
    code: "FALC",
    emoji: "🦅",
    quote: "锁定目标,其他都是噪音。你的中断次数少得可疑。",
    hidden: "导师叫你你都没听见,这不是缺点,这是天赋",
    dims: [3, 2, 4, 5, 1],
  },
  "满月狼": {
    kind: "base",
    code: "WOLF",
    emoji: "🐺",
    quote: "一旦进入状态,谁也拦不住你,包括你自己。",
    hidden: "你进入心流的频率让心理学教科书都要改版",
    dims: [3, 3, 2, 5, 2],
  },

  // ---------- Base · 崩溃重启 ----------
  "凤凰协议": {
    kind: "base",
    code: "PHNX",
    emoji: "🔥",
    quote: "崩了又怎样,重启就完事了。C 级之后必然是 A 级。",
    hidden: "你的崩溃是有规律的,这反而是一种稳定",
    dims: [3, 4, 1, 4, 2],
  },
  "量子态研究员": {
    kind: "base",
    code: "QNTM",
    emoji: "🌀",
    quote: "你同时存在于所有任务里,又好像哪个都没在做。",
    hidden: "你的工作方式让旁观者焦虑,但你自己很平静",
    dims: [3, 3, 1, 3, 4],
  },
  "学术僵尸": {
    kind: "base",
    code: "ZMBI",
    emoji: "🧟",
    quote: "肉体已经下线,但番茄钟还在转。",
    hidden: "你不是在工作,你是在用工作麻醉自己",
    dims: [4, 3, 2, 5, 1],
  },
  "黑洞吸收者": {
    kind: "base",
    code: "HOLE",
    emoji: "🕳️",
    quote: "任务进来,时间消失,产出玄学。",
    hidden: "你对时间的感知能力已经损坏,请联系导师",
    dims: [3, 4, 1, 4, 3],
  },

  // ---------- Base · 战略 ----------
  "筑巢松鼠": {
    kind: "base",
    code: "NEST",
    emoji: "🐿️",
    quote: "每颗坚果都有它的位置。你的任务清单比别人的论文还整齐。",
    hidden: "你花在规划上的时间,有时候比执行还多",
    dims: [2, 1, 5, 3, 4],
  },
  "里程碑猎人": {
    kind: "base",
    code: "MLST",
    emoji: "🦫",
    quote: "你不是在写论文,你是在打 Boss。一个一个拿下。",
    hidden: "你的人生有章节感,别人活在流水账里",
    dims: [2, 2, 4, 4, 4],
  },
  "章鱼多线程": {
    kind: "base",
    code: "OCTO",
    emoji: "🐙",
    quote: "八条腿同时在干活。多个长线目标并行,你乐在其中。",
    hidden: "你的大脑有多个标签页,从不关闭",
    dims: [3, 3, 3, 4, 5],
  },
  "慢热变形者": {
    kind: "base",
    code: "MORPH",
    emoji: "🦋",
    quote: "前期看不出来,后期让人刮目相看。",
    hidden: "你需要很长的热身期,但爆发的时候没人能追上",
    dims: [3, 3, 3, 4, 2],
  },

  // ---------- Base · 混沌 ----------
  "实验室赌徒": {
    kind: "base",
    code: "GMBL",
    emoji: "🎰",
    quote: "你同时开了五个方向,赌的就是总有一个能跑出来。",
    hidden: "你的导师以为你有方法论,其实你在赌概率",
    dims: [3, 4, 1, 3, 5],
  },
  "俄罗斯套娃": {
    kind: "base",
    code: "NEST2",
    emoji: "🪆",
    quote: "打开一个任务,里面还有三个任务,里面还有三个任务。",
    hidden: "你的任务树比你的论文目录还复杂",
    dims: [3, 3, 2, 3, 5],
  },

  // ---------- Base · 稀有 ----------
  "数据幽灵": {
    kind: "base",
    code: "PERF",
    emoji: "👁️",
    quote: "你的热力图完美到像是 P 的。",
    hidden: "你要么是效率天才,要么是机器人,我们倾向于后者",
    dims: [3, 1, 5, 5, 3],
  },
  "全能异常体": {
    kind: "base",
    code: "OMNI",
    emoji: "🌈",
    quote: "你打破了所有分类规则,我们的算法对你束手无策。",
    hidden: "你是 FocusLab 遇到的最复杂的用户,这是夸你",
    dims: [3, 3, 3, 3, 3],
  },
  "孤独星球": {
    kind: "base",
    code: "SOLO",
    emoji: "🪐",
    quote: "你在自己的轨道上运行,不需要任何人理解。",
    hidden: "你有一套外人看不懂但极其有效的系统",
    dims: [2, 1, 5, 4, 1],
  },

  // ---------- Combo · 16 条组合款 ----------
  "暗夜不死鸟":       { kind: "combo", emojis: "🦇 × 🔥", desc: "深夜崩溃,深夜重生" },
  "截止日期超导体":   { kind: "combo", emojis: "🐆 × 🐙", desc: "三条线同时到期,全部交出" },
  "秩序编织者":       { kind: "combo", emojis: "🐿️ × 🪼", desc: "计划从不崩,人从不崩" },
  "行走的科研机器":   { kind: "combo", emojis: "🧟 × 🦅", desc: "肉体消耗殆尽,专注力满格" },
  "隐形冠军":         { kind: "combo", emojis: "🐻 × 🦋", desc: "前期消失,后期震场" },
  "时间黑洞":         { kind: "combo", emojis: "🕳️ × 🌋", desc: "时间进去,成果出来,过程玄学" },
  "永动机":           { kind: "combo", emojis: "🦇 × 🦈", desc: "白天打工晚上科研,24h 不停转" },
  "薛定谔的作息":     { kind: "combo", emojis: "🐓 × 🌑", desc: "你同时是早鸟和夜猫,量子力学已无法解释" },
  "效率偏执狂":       { kind: "combo", emojis: "🐝 × 🐿️", desc: "连等电梯的 30 秒都在规划" },
  "战略性摆烂":       { kind: "combo", emojis: "🐆 × 🐻", desc: "平时躺平是为了积蓄 DDL 时刻的爆发力" },
  "双面人":           { kind: "combo", emojis: "🐢 × 🐺", desc: "慢的时候特别慢,快的时候特别快,没有中间档" },
  "豪赌不死":         { kind: "combo", emojis: "🎰 × 🔥", desc: "五个方向全炸了,但你从废墟里扒出来一个" },
  "早起但还是赶DDL":  { kind: "combo", emojis: "🐝 × 💣", desc: "起得比谁都早,还是最后一天才交。这就是你" },
  "混沌之子":         { kind: "combo", emojis: "🌀 × 🪆", desc: "你的工作方式是一个分形:越放大越复杂" },
  "休眠火山":         { kind: "combo", emojis: "🦥 × 🌋", desc: "安静到让人以为已经死了,突然喷发一整个月的进度" },
  "机器人嫌疑":       { kind: "combo", emojis: "🦈 × 👁️", desc: "你的数据太完美,我们不得不弹出验证码" },
};

/** 渲染 5 维星级字符串 */
export function stars(v: number): string {
  return "★".repeat(v) + "☆".repeat(5 - v);
}
