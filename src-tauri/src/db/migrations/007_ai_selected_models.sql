-- 007: 为 chat profile 添加 selected_models 列 (JSON 数组,用户在设置中勾选的模型列表)
-- SQLite ALTER TABLE 对已有数据的 NOT NULL 新列要求 DEFAULT 是常量;
-- '[]' 是字符串字面量常量,但部分 SQLite 实现仍会拒绝。
-- 这里分两步:先加可空列,再 UPDATE 填空值,最终业务层兜底空数组。
ALTER TABLE ai_chat_profiles ADD COLUMN selected_models TEXT DEFAULT '[]';
UPDATE ai_chat_profiles SET selected_models = '[]' WHERE selected_models IS NULL;
