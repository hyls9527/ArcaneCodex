"""
Ralph Protocol MCP Server 单元测试

测试覆盖:
1. TaskFileParser - 任务文件解析与状态修改
2. FocusLock - 单线程专注锁（持久化）
3. VerificationLog - 验证记录（防绕过）
4. TestRunner - 测试执行器（命令白名单）
5. MCP Tools - next_task / verify / update_status / acquire_lock / release_lock / get_progress
"""

import json
import os
import sys
import tempfile
import unittest

RALPH_ENGINE_DIR = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, RALPH_ENGINE_DIR)

from server import TaskFileParser, FocusLock, VerificationLog, TestRunner
from server import next_task, verify, update_status, acquire_lock, release_lock, get_progress


class TestTaskFileParser(unittest.TestCase):

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.tasks_file = os.path.join(self.temp_dir, "04-ralph-tasks.md")
        with open(self.tasks_file, "w", encoding="utf-8") as f:
            f.write("# Tasks\n\n")
            f.write("## Phase 1\n")
            f.write("- [x] 已完成任务 A\n")
            f.write("- [ ] 待执行任务 B\n")
            f.write("- [-] 被阻塞任务 C\n")
            f.write("- [ ] 待执行任务 D\n")

    def test_parse_tasks_counts(self):
        tasks = TaskFileParser.parse_tasks(self.tasks_file)
        self.assertEqual(len(tasks), 4)

    def test_parse_tasks_statuses(self):
        tasks = TaskFileParser.parse_tasks(self.tasks_file)
        self.assertEqual(tasks[0]["status"], "completed")
        self.assertEqual(tasks[1]["status"], "pending")
        self.assertEqual(tasks[2]["status"], "blocked")
        self.assertEqual(tasks[3]["status"], "pending")

    def test_parse_tasks_line_numbers(self):
        tasks = TaskFileParser.parse_tasks(self.tasks_file)
        self.assertEqual(tasks[0]["line"], 4)
        self.assertEqual(tasks[1]["line"], 5)
        self.assertEqual(tasks[2]["line"], 6)
        self.assertEqual(tasks[3]["line"], 7)

    def test_mark_completed(self):
        result = TaskFileParser.mark_completed(self.tasks_file, 5)
        self.assertTrue(result)
        tasks = TaskFileParser.parse_tasks(self.tasks_file)
        self.assertEqual(tasks[1]["status"], "completed")

    def test_mark_completed_already_done(self):
        result = TaskFileParser.mark_completed(self.tasks_file, 4)
        self.assertFalse(result)

    def test_mark_completed_invalid_line(self):
        result = TaskFileParser.mark_completed(self.tasks_file, 999)
        self.assertFalse(result)

    def test_count_progress(self):
        progress = TaskFileParser.count_progress(self.tasks_file)
        self.assertEqual(progress["total"], 4)
        self.assertEqual(progress["completed"], 1)
        self.assertEqual(progress["blocked"], 1)
        self.assertEqual(progress["pending"], 2)

    def test_count_progress_empty_file(self):
        empty_file = os.path.join(self.temp_dir, "empty.md")
        with open(empty_file, "w") as f:
            f.write("# No tasks\n")
        progress = TaskFileParser.count_progress(empty_file)
        self.assertEqual(progress["total"], 0)

    def test_count_progress_nonexistent_file(self):
        progress = TaskFileParser.count_progress("/nonexistent/file.md")
        self.assertEqual(progress["total"], 0)

    def test_first_pending_is_next(self):
        tasks = TaskFileParser.parse_tasks(self.tasks_file)
        first_pending = next(t for t in tasks if t["status"] == "pending")
        self.assertEqual(first_pending["text"], "待执行任务 B")

    def test_mark_completed_updates_progress(self):
        TaskFileParser.mark_completed(self.tasks_file, 5)
        progress = TaskFileParser.count_progress(self.tasks_file)
        self.assertEqual(progress["completed"], 2)
        self.assertEqual(progress["pending"], 1)

    def test_find_task_by_line(self):
        task = TaskFileParser.find_task_by_line(self.tasks_file, 5)
        self.assertIsNotNone(task)
        self.assertEqual(task["text"], "待执行任务 B")

    def test_find_task_by_line_not_found(self):
        task = TaskFileParser.find_task_by_line(self.tasks_file, 1)
        self.assertIsNone(task)


class TestFocusLock(unittest.TestCase):

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.lock = FocusLock(state_dir=self.temp_dir)

    def test_acquire_success(self):
        success, msg = self.lock.acquire("task-1")
        self.assertTrue(success)
        self.assertIn("task-1", msg)

    def test_acquire_blocks_second(self):
        self.lock.acquire("task-1")
        success, msg = self.lock.acquire("task-2")
        self.assertFalse(success)
        self.assertIn("task-1", msg)

    def test_release_success(self):
        self.lock.acquire("task-1")
        success, msg = self.lock.release("task-1")
        self.assertTrue(success)

    def test_release_wrong_task(self):
        self.lock.acquire("task-1")
        success, msg = self.lock.release("task-2")
        self.assertFalse(success)

    def test_release_when_no_lock(self):
        success, msg = self.lock.release("task-1")
        self.assertFalse(success)

    def test_acquire_after_release(self):
        self.lock.acquire("task-1")
        self.lock.release("task-1")
        success, msg = self.lock.acquire("task-2")
        self.assertTrue(success)

    def test_status_locked(self):
        self.lock.acquire("task-1")
        status = self.lock.status()
        self.assertTrue(status["locked"])
        self.assertEqual(status["current_task"], "task-1")

    def test_status_unlocked(self):
        status = self.lock.status()
        self.assertFalse(status["locked"])
        self.assertIsNone(status["current_task"])

    def test_persistence(self):
        lock2 = FocusLock(state_dir=self.temp_dir)
        self.lock.acquire("task-persist")
        status2 = lock2.status()
        self.assertTrue(status2["locked"])
        self.assertEqual(status2["current_task"], "task-persist")

    def test_persistence_after_release(self):
        self.lock.acquire("task-persist")
        self.lock.release("task-persist")
        lock2 = FocusLock(state_dir=self.temp_dir)
        status2 = lock2.status()
        self.assertFalse(status2["locked"])

    def test_full_lifecycle(self):
        s1, _ = self.lock.acquire("task-A")
        self.assertTrue(s1)
        s2, _ = self.lock.acquire("task-B")
        self.assertFalse(s2)
        s3, _ = self.lock.release("task-A")
        self.assertTrue(s3)
        s4, _ = self.lock.acquire("task-B")
        self.assertTrue(s4)
        s5, _ = self.lock.release("task-B")
        self.assertTrue(s5)


class TestVerificationLog(unittest.TestCase):

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.log = VerificationLog(state_dir=self.temp_dir)

    def test_record_and_check(self):
        self.log.record_pass("tasks:5")
        self.assertTrue(self.log.is_verified("tasks:5"))

    def test_not_verified(self):
        self.assertFalse(self.log.is_verified("tasks:99"))

    def test_consume_success(self):
        self.log.record_pass("tasks:5")
        result = self.log.consume("tasks:5")
        self.assertTrue(result)
        self.assertFalse(self.log.is_verified("tasks:5"))

    def test_consume_twice_fails(self):
        self.log.record_pass("tasks:5")
        self.log.consume("tasks:5")
        result = self.log.consume("tasks:5")
        self.assertFalse(result)

    def test_consume_without_record(self):
        result = self.log.consume("tasks:99")
        self.assertFalse(result)

    def test_persistence(self):
        self.log.record_pass("tasks:7")
        log2 = VerificationLog(state_dir=self.temp_dir)
        self.assertTrue(log2.is_verified("tasks:7"))


class TestTestRunner(unittest.TestCase):

    def test_pass_pattern_echo(self):
        result = TestRunner.run("cargo --version", cwd=".")
        self.assertTrue(result["passed"])

    def test_pass_pattern_ok(self):
        result = TestRunner.run("cargo --version", cwd=".")
        self.assertTrue(result["passed"])

    def test_fail_pattern(self):
        result = TestRunner.run("exit 1", cwd=".")
        self.assertFalse(result["passed"])

    def test_exit_code_zero(self):
        result = TestRunner.run("cargo --version", cwd=".")
        self.assertTrue(result["passed"])

    def test_exit_code_nonzero(self):
        result = TestRunner.run("exit 1", cwd=".")
        self.assertFalse(result["passed"])

    def test_disallowed_command(self):
        result = TestRunner.run("rm -rf /tmp/test", cwd=".")
        self.assertFalse(result["passed"])
        self.assertIn("不允许的命令", result["stderr"])

    def test_empty_command(self):
        result = TestRunner.run("", cwd=".")
        self.assertFalse(result["passed"])

    def test_allowed_command_cargo(self):
        result = TestRunner.run("cargo --version", cwd=".")
        self.assertTrue(result["passed"])

    def test_allowed_command_npx(self):
        result = TestRunner.run("npx --version", cwd=".")
        self.assertTrue(result["passed"])


class TestMCPToolsIntegration(unittest.TestCase):

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.tasks_file = os.path.join(self.temp_dir, "04-ralph-tasks.md")
        self.tests_file = os.path.join(self.temp_dir, "05-test-plan.md")
        self.state_dir = os.path.join(self.temp_dir, ".ralph")
        os.makedirs(self.state_dir, exist_ok=True)
        with open(self.tasks_file, "w", encoding="utf-8") as f:
            f.write("# Tasks\n\n")
            f.write("- [x] 已完成\n")
            f.write("- [ ] 待执行A\n")
            f.write("- [ ] 待执行B\n")
        with open(self.tests_file, "w", encoding="utf-8") as f:
            f.write("# Tests\n\n")
            f.write("- [ ] TC-001 测试用例\n")
            f.write("- [x] TC-002 已通过\n")

    def _patch_paths(self):
        import server
        self._orig_tf = server._get_tasks_file
        self._orig_tstf = server._get_tests_file
        self._orig_lock = server._lock
        self._orig_vlog = server._verify_log
        server._get_tasks_file = lambda: self.tasks_file
        server._get_tests_file = lambda: self.tests_file
        server._lock = FocusLock(state_dir=self.state_dir)
        server._verify_log = VerificationLog(state_dir=self.state_dir)

    def _restore_paths(self):
        import server
        server._get_tasks_file = self._orig_tf
        server._get_tests_file = self._orig_tstf
        server._lock = self._orig_lock
        server._verify_log = self._orig_vlog

    def test_next_task_returns_first_pending(self):
        self._patch_paths()
        try:
            result = next_task()
            self.assertTrue(result["found"])
            self.assertEqual(result["task"]["text"], "待执行A")
        finally:
            self._restore_paths()

    def test_next_task_all_done(self):
        all_done_file = os.path.join(self.temp_dir, "all_done.md")
        with open(all_done_file, "w", encoding="utf-8") as f:
            f.write("- [x] Done\n")
        import server
        orig = server._get_tasks_file
        server._get_tasks_file = lambda: all_done_file
        try:
            result = next_task()
            self.assertFalse(result["found"])
        finally:
            server._get_tasks_file = orig

    def test_update_status_completed_without_verify_rejected(self):
        self._patch_paths()
        try:
            result = update_status("tasks:5", "completed")
            self.assertFalse(result["success"])
            self.assertIn("verify()", result["error"])
        finally:
            self._restore_paths()

    def test_update_status_completed_after_verify(self):
        self._patch_paths()
        import server
        try:
            server._verify_log.record_pass("tasks:5")
            result = update_status("tasks:5", "completed")
            self.assertTrue(result["success"])
            tasks = TaskFileParser.parse_tasks(self.tasks_file)
            pending = [t for t in tasks if t["status"] == "pending"]
            self.assertEqual(len(pending), 1)
        finally:
            self._restore_paths()

    def test_verify_record_consumed_on_update(self):
        self._patch_paths()
        import server
        try:
            server._verify_log.record_pass("tasks:5")
            update_status("tasks:5", "completed")
            result = update_status("tasks:5", "completed")
            self.assertFalse(result["success"])
        finally:
            self._restore_paths()

    def test_update_status_blocked(self):
        blocked_file = os.path.join(self.temp_dir, "blocked_tasks.md")
        with open(blocked_file, "w", encoding="utf-8") as f:
            f.write("# Tasks\n\n")
            f.write("- [ ] 待执行A\n")
            f.write("- [ ] 待执行B\n")
        self._patch_paths()
        import server
        server._get_tasks_file = lambda: blocked_file
        try:
            result = update_status("tasks:4", "blocked", "依赖缺失")
            self.assertTrue(result["success"])
            tasks = TaskFileParser.parse_tasks(blocked_file)
            blocked = [t for t in tasks if t["status"] == "blocked"]
            self.assertEqual(len(blocked), 1)
        finally:
            self._restore_paths()

    def test_update_status_invalid_format(self):
        result = update_status("invalid-id", "completed")
        self.assertFalse(result["success"])

    def test_update_status_invalid_status(self):
        result = update_status("tasks:5", "skipped")
        self.assertFalse(result["success"])

    def test_get_progress(self):
        self._patch_paths()
        try:
            result = get_progress()
            self.assertEqual(result["tasks"]["total"], 3)
            self.assertEqual(result["tasks"]["completed"], 1)
            self.assertEqual(result["tests"]["total"], 2)
            self.assertEqual(result["tests"]["completed"], 1)
        finally:
            self._restore_paths()

    def test_acquire_and_release_lock(self):
        self._patch_paths()
        try:
            r1 = acquire_lock("tasks:5")
            self.assertTrue(r1["acquired"])
            r2 = acquire_lock("tasks:6")
            self.assertFalse(r2["acquired"])
            r3 = release_lock("tasks:5")
            self.assertTrue(r3["released"])
            r4 = acquire_lock("tasks:6")
            self.assertTrue(r4["acquired"])
            release_lock("tasks:6")
        finally:
            self._restore_paths()


def run_tests():
    loader = unittest.TestLoader()
    suite = unittest.TestSuite()

    suite.addTests(loader.loadTestsFromTestCase(TestTaskFileParser))
    suite.addTests(loader.loadTestsFromTestCase(TestFocusLock))
    suite.addTests(loader.loadTestsFromTestCase(TestVerificationLog))
    suite.addTests(loader.loadTestsFromTestCase(TestTestRunner))
    suite.addTests(loader.loadTestsFromTestCase(TestMCPToolsIntegration))

    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)

    print("\n" + "=" * 70)
    print("Ralph Protocol MCP Server 测试报告")
    print("=" * 70)
    print(f"总测试数: {result.testsRun}")
    print(f"通过: {result.testsRun - len(result.failures) - len(result.errors)}")
    print(f"失败: {len(result.failures)}")
    print(f"错误: {len(result.errors)}")
    print("=" * 70)

    return result


if __name__ == "__main__":
    run_tests()
