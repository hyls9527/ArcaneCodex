"""
Ralph Execution Engine - Intent-Driven Multi-Instance Control

This engine decouples intent from execution, providing:
1. Intent Translation - Natural language to atomic tasks
2. Task Dispatching - Smart allocation based on capability/load
3. Independent Verification - Results validated by separate layer
4. State Machine Feedback - Clear state transitions with verification

Usage:
    from ralph_engine import RalphEngine
    engine = RalphEngine()
    engine.submit_intent("开发一个登录页面，包含前端、后端和测试")
    engine.get_status()
"""
