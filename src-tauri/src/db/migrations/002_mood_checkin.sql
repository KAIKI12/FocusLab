-- 002_mood_checkin · 心情打卡接入日结算
-- settlements 增加晚间情绪(evening_mood) + 早晨意图(morning_intent) 两档 1-5,允许 NULL。

ALTER TABLE settlements ADD COLUMN evening_mood   INTEGER;
ALTER TABLE settlements ADD COLUMN morning_intent INTEGER;
