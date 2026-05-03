# Ralph Execution Engine

## 核心架构

```
意图层 (Intent) → 翻译层 (Translator) → 调度层 (Dispatcher) → 执行层 (Executor) → 验证层 (Verifier) → 反馈层 (Feedback)
```

## 状态机

```
IDLE → PLANNING → DISPATCHED → EXECUTING → VERIFYING → COMPLETED/FAILED
```

## 模块

- `core/intent.py` - 意图翻译器
- `core/dispatcher.py` - 任务调度器
- `core/executor.py` - 执行引擎
- `core/verifier.py` - 验证层
- `core/state_machine.py` - 状态机管理
- `core/feedback.py` - 反馈循环
