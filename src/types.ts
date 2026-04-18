/**
 * 前端类型定义 — 与后端 Rust 的 serde 序列化结果对齐。
 */

export interface Task {
  id: string;
  name: string;
  description: string | null;
  quadrant: string;
  status: string;
  created_at: string;
  updated_at: string;
  completed_at: string | null;
}

export interface CreateTaskInput {
  name: string;
  quadrant?: string;
}
