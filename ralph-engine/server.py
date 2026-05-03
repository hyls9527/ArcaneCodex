"""
Ralph MCP Server - 开发纪律强制执行器

通过 MCP 协议暴露 5 个核心 Tool，Agent 无法绕过规则：
1. next_task() - 物理顺序优先：只返回第一个未完成任务
2. verify() - 测试即交付：必须看到 PASS 才允许状态转移
3. update_status() - 状态真实性：先改文件再计算状态（completed 需先通过 verify）
4. get_progress() - 实时进度查询
5. verify_and_complete() - 验证并完成：自动验证后标记完成
"""

import json
import os
import re
import shlex
import subprocess
import threading
from datetime import datetime
from typing import Optional

from mcp.server.fastmcp import FastMCP

mcp = FastMCP("Ralph Protocol")


class TaskFileParser:
    """解析 04-ralph-tasks.md 和 05-test-plan.md 中的任务状态"""

    @staticmethod
    def parse_tasks(file_path: str) -> list[dict]:
        if not os.path.exists(file_path):
            return []
        with open(file_path, "r", encoding="utf-8") as f:
            lines = f.readlines()
        tasks = []
        for i, line in enumerate(lines, 1):
            stripped = line.strip()
            if stripped.startswith("- [x]"):
                tasks.append({"line": i, "text": stripped[6:].strip(), "status": "completed", "raw": line})
            elif stripped.startswith("- [-]"):
                tasks.append({"line": i, "text": stripped[6:].strip(), "status": "blocked", "raw": line})
            elif stripped.startswith("- [ ]"):
                tasks.append({"line": i, "text": stripped[6:].strip(), "status": "pending", "raw": line})
        return tasks

    @staticmethod
    def mark_completed(file_path: str, line_number: int) -> bool:
        with open(file_path, "r", encoding="utf-8") as f:
            lines = f.readlines()
        if line_number < 1 or line_number > len(lines):
            return False
        line = lines[line_number - 1]
        if line.strip().startswith("- [ ]"):
            lines[line_number - 1] = line.replace("- [ ]", "- [x]", 1)
            with open(file_path, "w", encoding="utf-8") as f:
                f.writelines(lines)
            return True
        return False

    @staticmethod
    def count_progress(file_path: str) -> dict:
        tasks = TaskFileParser.parse_tasks(file_path)
        completed = sum(1 for t in tasks if t["status"] == "completed")
        blocked = sum(1 for t in tasks if t["status"] == "blocked")
        pending = sum(1 for t in tasks if t["status"] == "pending")
        total = len(tasks)
        return {"total": total, "completed": completed, "blocked": blocked, "pending": pending}

    @staticmethod
    def find_task_by_line(file_path: str, line_number: int) -> Optional[dict]:
        tasks = TaskFileParser.parse_tasks(file_path)
        for t in tasks:
            if t["line"] == line_number:
                return t
        return None


class FocusLock:
    """单线程专注锁：同一时刻只允许一个任务处于执行状态，持久化到文件"""

    def __init__(self, state_dir: str = ""):
        self._lock = threading.Lock()
        self._current_task: Optional[str] = None
        self._acquired_at: Optional[str] = None
        self._state_file = os.path.join(
            state_dir or os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".ralph"),
            "focus_lock.json",
        )
        self._load()

    def acquire(self, task_id: str) -> tuple[bool, str]:
        self._load()
        with self._lock:
            if self._current_task is not None:
                elapsed = ""
                if self._acquired_at:
                    try:
                        start = datetime.fromisoformat(self._acquired_at)
                        delta = datetime.now() - start
                        elapsed = f" (已执行 {delta.seconds // 60} 分钟)"
                    except Exception:
                        pass
                return False, f"焦点被占用: {self._current_task}{elapsed}。必须先完成当前任务或调用 release_lock 释放。"
            self._current_task = task_id
            self._acquired_at = datetime.now().isoformat()
            self._save()
            return True, f"焦点锁定成功: {task_id}"

    def release(self, task_id: str) -> tuple[bool, str]:
        self._load()
        with self._lock:
            if self._current_task is None:
                return False, "没有任务持有焦点锁"
            if self._current_task != task_id:
                return False, f"焦点锁由 {self._current_task} 持有，{task_id} 无权释放"
            self._current_task = None
            self._acquired_at = None
            self._save()
            return True, f"焦点锁已释放: {task_id}"

    def status(self) -> dict:
        self._load()
        return {
            "locked": self._current_task is not None,
            "current_task": self._current_task,
            "acquired_at": self._acquired_at,
        }

    def _load(self):
        if os.path.exists(self._state_file):
            try:
                with open(self._state_file, "r", encoding="utf-8") as f:
                    data = json.load(f)
                self._current_task = data.get("current_task")
                self._acquired_at = data.get("acquired_at")
            except Exception:
                pass

    def _save(self):
        os.makedirs(os.path.dirname(self._state_file), exist_ok=True)
        data = {
            "current_task": self._current_task,
            "acquired_at": self._acquired_at,
        }
        with open(self._state_file, "w", encoding="utf-8") as f:
            json.dump(data, f, ensure_ascii=False, indent=2)


class VerificationLog:
    """验证记录：跟踪哪些 task_id 已通过 verify()，防止 update_status 绕过测试"""

    def __init__(self, state_dir: str = ""):
        self._state_file = os.path.join(
            state_dir or os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".ralph"),
            "verification_log.json",
        )
        self._log: dict[str, str] = {}
        self._load()

    def record_pass(self, task_id: str):
        self._log[task_id] = datetime.now().isoformat()
        self._save()

    def is_verified(self, task_id: str) -> bool:
        self._load()
        return task_id in self._log

    def consume(self, task_id: str) -> bool:
        """消费验证记录，返回是否曾通过验证。调用后清除该记录，防止重复使用。"""
        self._load()
        if task_id in self._log:
            del self._log[task_id]
            self._save()
            return True
        return False

    def _load(self):
        if os.path.exists(self._state_file):
            try:
                with open(self._state_file, "r", encoding="utf-8") as f:
                    self._log = json.load(f)
            except Exception:
                self._log = {}

    def _save(self):
        os.makedirs(os.path.dirname(self._state_file), exist_ok=True)
        with open(self._state_file, "w", encoding="utf-8") as f:
            json.dump(self._log, f, ensure_ascii=False, indent=2)


class TestRunner:
    """真实测试执行器：运行测试命令并解析输出"""

    PASS_PATTERNS = [
        re.compile(r"(\d+)\s*passed", re.IGNORECASE),
        re.compile(r"PASS", re.IGNORECASE),
        re.compile(r"✓"),
        re.compile(r"(\d+)\s*tests?\s*(?:passed|succeeded)", re.IGNORECASE),
        re.compile(r"test result:\s*ok", re.IGNORECASE),
        re.compile(r"(\d+)\s*succeeded", re.IGNORECASE),
    ]

    FAIL_PATTERNS = [
        re.compile(r"(\d+)\s*failed", re.IGNORECASE),
        re.compile(r"FAIL", re.IGNORECASE),
        re.compile(r"✗"),
        re.compile(r"test result:\s*FAILED", re.IGNORECASE),
        re.compile(r"ERROR", re.IGNORECASE),
    ]

    ALLOWED_COMMANDS = {"cargo", "npm", "npx", "pnpm", "yarn", "python", "py", "pytest", "vitest"}

    @staticmethod
    def run(command: str, cwd: str, timeout: int = 120) -> dict:
        try:
            is_windows = os.name == "nt"
            parts = shlex.split(command, posix=not is_windows)
            if not parts:
                return {"passed": False, "returncode": -1, "stdout": "", "stderr": "空命令", "command": command, "timed_out": False}

            base = os.path.basename(parts[0]).lower()
            if base not in TestRunner.ALLOWED_COMMANDS and base.replace(".exe", "") not in TestRunner.ALLOWED_COMMANDS:
                return {
                    "passed": False,
                    "returncode": -1,
                    "stdout": "",
                    "stderr": f"不允许的命令: {base}。仅允许: {', '.join(sorted(TestRunner.ALLOWED_COMMANDS))}",
                    "command": command,
                    "timed_out": False,
                }

            result = subprocess.run(
                parts,
                capture_output=True,
                text=True,
                cwd=cwd,
                timeout=timeout,
                shell=is_windows,
            )
            stdout = result.stdout
            stderr = result.stderr
            returncode = result.returncode

            passed = False
            for pattern in TestRunner.PASS_PATTERNS:
                if pattern.search(stdout):
                    passed = True
                    break

            failed = False
            for pattern in TestRunner.FAIL_PATTERNS:
                if pattern.search(stdout) or pattern.search(stderr):
                    failed = True
                    break

            if not passed and not failed:
                passed = returncode == 0

            if failed:
                passed = False

            return {
                "passed": passed,
                "returncode": returncode,
                "stdout": stdout[-2000:] if len(stdout) > 2000 else stdout,
                "stderr": stderr[-1000:] if len(stderr) > 1000 else stderr,
                "command": command,
                "timed_out": False,
            }
        except subprocess.TimeoutExpired:
            return {
                "passed": False,
                "returncode": -1,
                "stdout": "",
                "stderr": f"测试超时 ({timeout}秒)",
                "command": command,
                "timed_out": True,
            }
        except Exception as e:
            return {
                "passed": False,
                "returncode": -1,
                "stdout": "",
                "stderr": str(e),
                "command": command,
                "timed_out": False,
            }


_state_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".ralph")
_lock = FocusLock(state_dir=_state_dir)
_verify_log = VerificationLog(state_dir=_state_dir)


def _find_project_root() -> str:
    env_root = os.environ.get("RALPH_PROJECT_ROOT", "")
    if env_root and os.path.exists(os.path.join(env_root, "docs", "planning")):
        return os.path.abspath(env_root)
    candidate = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
    if os.path.exists(os.path.join(candidate, "docs", "planning")):
        return os.path.abspath(candidate)
    return os.path.abspath(candidate)


def _get_config_file() -> str:
    return os.path.join(_find_project_root(), "ralph-config.json")


def _load_config() -> dict:
    config_path = _get_config_file()
    defaults = {
        "tasks_file": "",
        "tests_file": "",
        "test_commands": {
            "rust": "cargo test",
            "frontend": "npx vitest run",
        },
        "test_cwd": {
            "rust": "src-tauri",
            "frontend": "frontend",
        },
        "test_timeout": 120,
    }
    if os.path.exists(config_path):
        try:
            with open(config_path, "r", encoding="utf-8") as f:
                user_config = json.load(f)
            defaults.update(user_config)
        except Exception:
            pass
    return defaults


def _get_tasks_file() -> str:
    config = _load_config()
    if config.get("tasks_file"):
        return config["tasks_file"]
    root = _find_project_root()
    planning_dir = os.path.join(root, "docs", "planning")
    if os.path.exists(planning_dir):
        for entry in sorted(os.listdir(planning_dir), reverse=True):
            candidate = os.path.join(planning_dir, entry, "04-ralph-tasks.md")
            if os.path.exists(candidate):
                return candidate
    return os.path.join(root, "04-ralph-tasks.md")


def _get_tests_file() -> str:
    config = _load_config()
    if config.get("tests_file"):
        return config["tests_file"]
    root = _find_project_root()
    planning_dir = os.path.join(root, "docs", "planning")
    if os.path.exists(planning_dir):
        for entry in sorted(os.listdir(planning_dir), reverse=True):
            candidate = os.path.join(planning_dir, entry, "05-test-plan.md")
            if os.path.exists(candidate):
                return candidate
    return os.path.join(root, "05-test-plan.md")


def _resolve_task_id(task_id: str) -> tuple[Optional[str], Optional[int], Optional[str]]:
    """解析 task_id，返回 (file_path, line_number, error)"""
    parts = task_id.split(":")
    if len(parts) != 2:
        return None, None, f"无效的 task_id 格式: {task_id}，应为 'tasks:行号' 或 'tests:行号'"
    source, line_str = parts
    try:
        line_number = int(line_str)
    except ValueError:
        return None, None, f"无效的行号: {line_str}"
    file_path = _get_tests_file() if source == "tests" else _get_tasks_file()
    return file_path, line_number, None


# ─── MCP Tools ────────────────────────────────────────────────


@mcp.tool()
def next_task() -> dict:
    """获取下一个应该执行的任务。严格按 04-ralph-tasks.md 的物理顺序，只返回第一个 [ ] 未完成任务。已完成的 [x] 和被阻塞的 [-] 会被跳过。"""
    tasks_file = _get_tasks_file()
    tasks = TaskFileParser.parse_tasks(tasks_file)

    for task in tasks:
        if task["status"] == "pending":
            return {
                "found": True,
                "task": task,
                "source": os.path.basename(tasks_file),
                "progress": TaskFileParser.count_progress(tasks_file),
                "lock": _lock.status(),
            }

    return {
        "found": False,
        "task": None,
        "message": "所有任务已完成或被阻塞",
        "progress": TaskFileParser.count_progress(tasks_file),
        "lock": _lock.status(),
    }


@mcp.tool()
def verify(task_id: str, test_type: str = "auto") -> dict:
    """验证任务是否真正完成。运行真实测试命令，只有看到 PASS 才允许标记完成。这是'测试即交付'铁律的强制执行点。验证通过后会在内部记录，后续 update_status(completed) 才会放行。

    Args:
        task_id: 任务标识，格式为 'tasks:行号' 或 'tests:行号'
        test_type: 测试类型，可选 'rust'/'frontend'/'auto'。auto 会根据任务文本自动判断
    """
    config = _load_config()
    project_root = _find_project_root()

    file_path, line_number, error = _resolve_task_id(task_id)
    if error:
        return {"passed": False, "error": error}

    target = TaskFileParser.find_task_by_line(file_path, line_number)
    if target is None:
        return {"passed": False, "error": f"行 {line_number} 不是有效的任务行"}

    if target["status"] == "completed":
        return {"passed": True, "message": "任务已完成", "task": target}

    if target["status"] == "blocked":
        return {"passed": False, "error": f"任务被阻塞: {target['text']}", "task": target}

    if test_type == "auto":
        task_text_lower = target["text"].lower()
        if "rust" in task_text_lower or "cargo" in task_text_lower or "数据库" in task_text_lower or "后端" in task_text_lower:
            test_type = "rust"
        elif "前端" in task_text_lower or "frontend" in task_text_lower or "组件" in task_text_lower or "react" in task_text_lower:
            test_type = "frontend"
        else:
            test_type = "rust"

    test_command = config["test_commands"].get(test_type, config["test_commands"].get("rust"))
    if not test_command:
        return {"passed": False, "error": f"未配置测试命令: {test_type}"}

    test_cwd = config.get("test_cwd", {}).get(test_type, "")
    cwd = os.path.join(project_root, test_cwd) if test_cwd else project_root

    result = TestRunner.run(test_command, cwd=cwd, timeout=config.get("test_timeout", 120))

    if result["passed"]:
        _verify_log.record_pass(task_id)
        return {
            "passed": True,
            "task": target,
            "test_output": result["stdout"][-500:],
            "test_command": result["command"],
            "message": "验证通过，现在可以调用 update_status 标记完成",
        }
    else:
        return {
            "passed": False,
            "task": target,
            "test_output": result["stdout"][-500:],
            "test_error": result["stderr"][-500:],
            "test_command": result["command"],
            "timed_out": result["timed_out"],
        }


@mcp.tool()
def update_status(task_id: str, status: str, reason: str = "") -> dict:
    """更新任务状态。必须先修改 04/05 文件，然后重新扫描计算真实进度。标记 completed 前必须先通过 verify() 验证，否则会被拒绝。

    Args:
        task_id: 任务标识，格式为 'tasks:行号' 或 'tests:行号'
        status: 目标状态，'completed'（需先 verify 通过）或 'blocked'
        reason: 状态变更原因
    """
    file_path, line_number, error = _resolve_task_id(task_id)
    if error:
        return {"success": False, "error": error}

    if status == "completed":
        if not _verify_log.consume(task_id):
            return {
                "success": False,
                "error": f"任务 {task_id} 尚未通过 verify() 验证。必须先调用 verify(task_id) 且测试通过后，才能标记完成。",
                "hint": "调用 verify(task_id) 运行测试验证",
            }
        marked = TaskFileParser.mark_completed(file_path, line_number)
        if not marked:
            return {"success": False, "error": "无法标记完成：行不存在或已经是其他状态"}
    elif status == "blocked":
        if not os.path.exists(file_path):
            return {"success": False, "error": f"文件不存在: {file_path}"}
        with open(file_path, "r", encoding="utf-8") as f:
            lines = f.readlines()
        if line_number < 1 or line_number > len(lines):
            return {"success": False, "error": f"行号越界: {line_number}"}
        line = lines[line_number - 1]
        if line.strip().startswith("- [ ]"):
            lines[line_number - 1] = line.replace("- [ ]", "- [-]", 1)
            with open(file_path, "w", encoding="utf-8") as f:
                f.writelines(lines)
        else:
            return {"success": False, "error": "只能将 [ ] 标记为 [-] blocked"}
    else:
        return {"success": False, "error": f"不支持的状态: {status}，仅允许 'completed' 或 'blocked'"}

    tasks_progress = TaskFileParser.count_progress(_get_tasks_file())
    tests_progress = TaskFileParser.count_progress(_get_tests_file())

    return {
        "success": True,
        "task_id": task_id,
        "new_status": status,
        "reason": reason,
        "tasks_progress": tasks_progress,
        "tests_progress": tests_progress,
    }



@mcp.tool()
def get_progress() -> dict:
    """获取项目整体进度。从 04-ralph-tasks.md 和 05-test-plan.md 实时扫描计算，不使用缓存。"""
    tasks_file = _get_tasks_file()
    tests_file = _get_tests_file()

    tasks_progress = TaskFileParser.count_progress(tasks_file)
    tests_progress = TaskFileParser.count_progress(tests_file)

    return {
        "tasks": tasks_progress,
        "tests": tests_progress,
        "tasks_file": tasks_file,
        "tests_file": tests_file,
    }


@mcp.resource("ralph://state")
def get_state() -> str:
    """获取 Ralph 当前完整状态视图（只读，从文件实时计算）"""
    tasks_file = _get_tasks_file()
    tests_file = _get_tests_file()

    tasks_progress = TaskFileParser.count_progress(tasks_file)
    tests_progress = TaskFileParser.count_progress(tests_file)

    lines = [
        "# RALPH STATE (Auto-generated, read-only)",
        "",
        f"Generated: {datetime.now().isoformat()}",
        "",
        "## Tasks Progress",
        f"- Total: {tasks_progress['total']}",
        f"- Completed: {tasks_progress['completed']}",
        f"- Blocked: {tasks_progress['blocked']}",
        f"- Pending: {tasks_progress['pending']}",
        f"- Completion: {tasks_progress['completed'] / max(tasks_progress['total'], 1) * 100:.1f}%",
        "",
        "## Tests Progress",
        f"- Total: {tests_progress['total']}",
        f"- Completed: {tests_progress['completed']}",
        f"- Blocked: {tests_progress['blocked']}",
        f"- Pending: {tests_progress['pending']}",
        f"- Completion: {tests_progress['completed'] / max(tests_progress['total'], 1) * 100:.1f}%",
        "",
        "## Multi-Agent Coordination",
        "- Single-thread lock: REMOVED (支持多 Agent 并发协作)",
    ]
    return "\n".join(lines)


@mcp.prompt()
def ralph_workflow(task_description: str) -> str:
    """生成 Ralph 工作流提示词，引导 Agent 按铁律执行任务

    Args:
        task_description: 要执行的任务描述
    """
    return f"""你正在 Ralph Protocol 治理下工作。必须严格遵守以下流程：

1. 调用 next_task() 获取当前任务
2. 执行任务：{task_description}
3. 调用 verify(task_id) 验证任务完成
4. 如果验证失败，修复问题后重新验证
5. 如果验证通过，调用 update_status(task_id, "completed") 更新状态
6. 回到步骤 1

铁律提醒：
- 禁止跳步：必须按物理顺序执行
- 禁止跳过测试：必须先 verify() 通过，才能 update_status("completed")
- 禁止凭记忆更新状态：必须通过 MCP Tool 修改
- update_status("completed") 如果没有先 verify() 会被拒绝"""


if __name__ == "__main__":
    mcp.run()
