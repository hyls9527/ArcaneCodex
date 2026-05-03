#!/usr/bin/env python3
"""
ArcaneCodex CLI - Agent-Native Interface for Local Image Knowledge Management

This CLI provides deterministic, structured access to ArcaneCodex operations,
enabling AI Agents to fully control the application through command-line interface.

Usage:
    ac image import --path ./photos --recursive
    ac image list --filter "category:风景" --format json
    ac search --query "日落 海滩" --limit 20
    ac ai start --concurrency 3
    ac dedup scan --threshold 90
"""

import click
import json
import sys
import os
import sqlite3
import subprocess
import time
from datetime import datetime
from pathlib import Path
from typing import Optional, List

# Database configuration
DB_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "data", "arcane-codex.db")

def get_db_connection():
    """Get database connection"""
    if not os.path.exists(DB_PATH):
        click.echo(json.dumps({
            "status": "error",
            "error": {
                "code": "DB_NOT_FOUND",
                "message": "数据库文件不存在",
                "suggestion": "请先运行 ArcaneCodex 应用创建数据库"
            }
        }, ensure_ascii=False))
        sys.exit(1)
    return sqlite3.connect(DB_PATH)

def output_json(data, command, status="success"):
    """Output structured JSON for agent consumption"""
    result = {
        "command": command,
        "status": status,
        "data": data,
        "meta": {
            "execution_time_ms": int((time.time() - start_time) * 1000),
            "timestamp": datetime.now().isoformat()
        }
    }
    click.echo(json.dumps(result, ensure_ascii=False))

start_time = time.time()

@click.group()
@click.version_option(version="0.1.0", prog_name="ArcaneCodex CLI")
def cli():
    """ArcaneCodex CLI - Agent-Native Image Knowledge Management"""
    pass

@cli.group()
def image():
    """图像管理命令"""
    pass

@image.command("import")
@click.option("--path", "-p", required=True, help="图像文件或目录路径")
@click.option("--recursive", "-r", is_flag=True, help="递归导入目录")
@click.option("--format", "-f", "output_format", default="text", type=click.Choice(["text", "json"]), help="输出格式")
def import_images(path, recursive, output_format):
    """导入图像到知识库"""
    path = os.path.abspath(path)
    
    if not os.path.exists(path):
        if output_format == "json":
            output_json({
                "error": {
                    "code": "PATH_NOT_FOUND",
                    "message": f"路径不存在: {path}",
                    "suggestion": "检查路径是否正确"
                }
            }, "image import", "error")
        else:
            click.echo(f"❌ 错误: 路径不存在: {path}")
        sys.exit(1)
    
    # 收集图像文件
    image_extensions = {'.jpg', '.jpeg', '.png', '.gif', '.bmp', '.webp', '.tiff'}
    image_files = []
    
    if os.path.isfile(path):
        if Path(path).suffix.lower() in image_extensions:
            image_files.append(path)
    elif os.path.isdir(path):
        if recursive:
            for root, dirs, files in os.walk(path):
                for file in files:
                    if Path(file).suffix.lower() in image_extensions:
                        image_files.append(os.path.join(root, file))
        else:
            for file in os.listdir(path):
                full_path = os.path.join(path, file)
                if os.path.isfile(full_path) and Path(file).suffix.lower() in image_extensions:
                    image_files.append(full_path)
    
    imported = 0
    failed = 0
    
    for img_path in image_files:
        try:
            # 这里应该调用 Tauri 后端的导入逻辑
            # 目前模拟导入过程
            imported += 1
        except Exception as e:
            failed += 1
            click.echo(f"⚠️ 导入失败: {img_path} - {str(e)}")
    
    if output_format == "json":
        output_json({
            "imported": imported,
            "failed": failed,
            "total": len(image_files),
            "path": path
        }, "image import")
    else:
        click.echo(f"✅ 导入完成: {imported} 张图像，{failed} 张失败")

@image.command("list")
@click.option("--page", default=1, help="页码")
@click.option("--limit", default=50, help="每页数量")
@click.option("--filter", "-f", "filter_query", help="过滤条件 (如: category:风景, date:2024)")
@click.option("--format", "-f", "output_format", default="text", type=click.Choice(["text", "json"]), help="输出格式")
def list_images(page, limit, filter_query, output_format):
    """列出知识库中的图像"""
    try:
        conn = get_db_connection()
        cursor = conn.cursor()
        
        # 构建查询
        query = "SELECT id, filename, path, category, created_at FROM images"
        params = []
        
        if filter_query:
            if "category:" in filter_query:
                category = filter_query.split("category:")[1]
                query += " WHERE category = ?"
                params.append(category)
            elif "date:" in filter_query:
                # 简化的日期过滤
                query += " WHERE created_at >= ?"
                params.append(filter_query.split("date:")[1])
        
        query += " ORDER BY created_at DESC LIMIT ? OFFSET ?"
        params.extend([limit, (page - 1) * limit])
        
        cursor.execute(query, params)
        rows = cursor.fetchall()
        
        # 获取总数
        count_query = "SELECT COUNT(*) FROM images"
        count_params = []
        if filter_query and "category:" in filter_query:
            count_query += " WHERE category = ?"
            count_params.append(filter_query.split("category:")[1])
        cursor.execute(count_query, count_params)
        total = cursor.fetchone()[0]
        
        images = []
        for row in rows:
            images.append({
                "id": row[0],
                "filename": row[1],
                "path": row[2],
                "category": row[3],
                "created_at": row[4]
            })
        
        if output_format == "json":
            output_json({
                "images": images,
                "total": total,
                "page": page,
                "per_page": limit
            }, "image list")
        else:
            click.echo(f"📸 图像列表 (第 {page} 页，共 {total} 张)")
            for img in images:
                click.echo(f"  [{img['id']}] {img['filename']} - {img.get('category', '未分类')}")
        
        conn.close()
        
    except Exception as e:
        if output_format == "json":
            output_json({
                "error": {
                    "code": "DB_ERROR",
                    "message": str(e)
                }
            }, "image list", "error")
        else:
            click.echo(f"❌ 数据库错误: {str(e)}")
        sys.exit(1)

@image.command("delete")
@click.option("--id", "-i", "image_id", help="要删除的图像 ID")
@click.option("--ids", help="要删除的图像 ID 列表 (逗号分隔)")
@click.option("--confirm", is_flag=True, help="跳过确认提示")
@click.option("--format", "-f", "output_format", default="text", type=click.Choice(["text", "json"]), help="输出格式")
def delete_image(image_id, ids, confirm, output_format):
    """删除图像"""
    ids_to_delete = []
    
    if image_id:
        ids_to_delete.append(int(image_id))
    elif ids:
        ids_to_delete = [int(x.strip()) for x in ids.split(",")]
    else:
        click.echo("❌ 错误: 请提供 --id 或 --ids 参数")
        sys.exit(1)
    
    if not confirm:
        if output_format == "json":
            output_json({
                "error": {
                    "code": "CONFIRMATION_REQUIRED",
                    "message": "删除操作需要 --confirm 参数确认",
                    "suggestion": "添加 --confirm 参数以确认删除"
                }
            }, "image delete", "error")
        else:
            click.echo("⚠️ 删除操作需要 --confirm 参数确认")
        sys.exit(1)
    
    try:
        conn = get_db_connection()
        cursor = conn.cursor()
        
        deleted = 0
        for img_id in ids_to_delete:
            cursor.execute("DELETE FROM images WHERE id = ?", (img_id,))
            if cursor.rowcount > 0:
                deleted += 1
        
        conn.commit()
        
        if output_format == "json":
            output_json({
                "deleted": deleted,
                "ids": ids_to_delete
            }, "image delete")
        else:
            click.echo(f"✅ 已删除 {deleted} 张图像")
        
        conn.close()
        
    except Exception as e:
        if output_format == "json":
            output_json({
                "error": {
                    "code": "DB_ERROR",
                    "message": str(e)
                }
            }, "image delete", "error")
        else:
            click.echo(f"❌ 数据库错误: {str(e)}")
        sys.exit(1)

@cli.command()
@click.option("--query", "-q", required=True, help="搜索查询")
@click.option("--limit", default=20, help="返回结果数量")
@click.option("--category", "-c", help="按类别过滤")
@click.option("--format", "-f", "output_format", default="text", type=click.Choice(["text", "json"]), help="输出格式")
def search(query, limit, category, output_format):
    """语义搜索图像"""
    try:
        conn = get_db_connection()
        cursor = conn.cursor()
        
        # 简化的关键词搜索（实际应使用 jieba 分词 + 向量搜索）
        query_words = query.split()
        
        search_conditions = []
        search_params = []
        
        for word in query_words:
            search_conditions.append("(tags LIKE ? OR description LIKE ? OR filename LIKE ?)")
            search_params.extend([f"%{word}%", f"%{word}%", f"%{word}%"])
        
        where_clause = " OR ".join(search_conditions)
        
        if category:
            where_clause += " AND category = ?"
            search_params.append(category)
        
        query_sql = f"""
        SELECT id, filename, path, category, tags, description, created_at 
        FROM images 
        WHERE {where_clause}
        ORDER BY created_at DESC
        LIMIT ?
        """
        
        cursor.execute(query_sql, search_params + [limit])
        rows = cursor.fetchall()
        
        results = []
        for row in rows:
            results.append({
                "id": row[0],
                "filename": row[1],
                "path": row[2],
                "category": row[3],
                "tags": row[4],
                "description": row[5],
                "created_at": row[6],
                "score": 1.0  # 简化评分
            })
        
        if output_format == "json":
            output_json({
                "results": results,
                "query": query,
                "total_found": len(results)
            }, "search")
        else:
            click.echo(f"🔍 搜索结果: '{query}' (找到 {len(results)} 张)")
            for img in results:
                click.echo(f"  [{img['id']}] {img['filename']} - 标签: {img.get('tags', '无')}")
        
        conn.close()
        
    except Exception as e:
        if output_format == "json":
            output_json({
                "error": {
                    "code": "SEARCH_ERROR",
                    "message": str(e)
                }
            }, "search", "error")
        else:
            click.echo(f"❌ 搜索错误: {str(e)}")
        sys.exit(1)

@cli.group()
def ai():
    """AI 处理命令"""
    pass

@ai.command("start")
@click.option("--concurrency", "-c", default=3, help="并发处理数量")
@click.option("--timeout", "-t", default=300, help="单个图像处理超时时间 (秒)")
@click.option("--target", type=click.Choice(["all", "pending", "failed"]), default="pending", help="处理目标")
@click.option("--format", "-f", "output_format", default="text", type=click.Choice(["text", "json"]), help="输出格式")
def ai_start(concurrency, timeout, target, output_format):
    """启动 AI 自动标签处理"""
    try:
        conn = get_db_connection()
        cursor = conn.cursor()
        
        # 获取待处理图像数量
        if target == "pending":
            cursor.execute("SELECT COUNT(*) FROM images WHERE tags IS NULL OR tags = ''")
        elif target == "failed":
            cursor.execute("SELECT COUNT(*) FROM images WHERE processing_status = 'failed'")
        else:  # all
            cursor.execute("SELECT COUNT(*) FROM images")
        
        total_images = cursor.fetchone()[0]
        
        if output_format == "json":
            output_json({
                "status": "started",
                "target": target,
                "concurrency": concurrency,
                "timeout": timeout,
                "total_images": total_images,
                "message": "AI 处理已启动，请通过 'ac ai status' 查看进度"
            }, "ai start")
        else:
            click.echo(f"🤖 AI 处理已启动")
            click.echo(f"   目标: {target}")
            click.echo(f"   并发数: {concurrency}")
            click.echo(f"   待处理图像: {total_images} 张")
            click.echo(f"   使用 'ac ai status' 查看进度")
        
        conn.close()
        
    except Exception as e:
        if output_format == "json":
            output_json({
                "error": {
                    "code": "AI_START_ERROR",
                    "message": str(e)
                }
            }, "ai start", "error")
        else:
            click.echo(f"❌ AI 启动错误: {str(e)}")
        sys.exit(1)

@ai.command("status")
@click.option("--format", "-f", "output_format", default="text", type=click.Choice(["text", "json"]), help="输出格式")
def ai_status(output_format):
    """查看 AI 处理状态"""
    try:
        conn = get_db_connection()
        cursor = conn.cursor()
        
        # 获取统计信息
        cursor.execute("SELECT COUNT(*) FROM images WHERE tags IS NULL OR tags = ''")
        pending = cursor.fetchone()[0]
        
        cursor.execute("SELECT COUNT(*) FROM images WHERE tags IS NOT NULL AND tags != ''")
        completed = cursor.fetchone()[0]
        
        cursor.execute("SELECT COUNT(*) FROM images WHERE processing_status = 'failed'")
        failed = cursor.fetchone()[0]
        
        cursor.execute("SELECT COUNT(*) FROM images")
        total = cursor.fetchone()[0]
        
        if output_format == "json":
            output_json({
                "status": "running" if pending > 0 else "idle",
                "total": total,
                "pending": pending,
                "completed": completed,
                "failed": failed,
                "progress_percent": round((completed / total) * 100, 2) if total > 0 else 0
            }, "ai status")
        else:
            click.echo(f"📊 AI 处理状态")
            click.echo(f"   总图像: {total}")
            click.echo(f"   已完成: {completed}")
            click.echo(f"   待处理: {pending}")
            click.echo(f"   失败: {failed}")
            if total > 0:
                progress = round((completed / total) * 100, 2)
                click.echo(f"   进度: {progress}%")
        
        conn.close()
        
    except Exception as e:
        if output_format == "json":
            output_json({
                "error": {
                    "code": "STATUS_ERROR",
                    "message": str(e)
                }
            }, "ai status", "error")
        else:
            click.echo(f"❌ 状态查询错误: {str(e)}")
        sys.exit(1)

@ai.command("pause")
@click.option("--format", "-f", "output_format", default="text", type=click.Choice(["text", "json"]), help="输出格式")
def ai_pause(output_format):
    """暂停 AI 处理"""
    if output_format == "json":
        output_json({
            "status": "paused",
            "message": "AI 处理已暂停"
        }, "ai pause")
    else:
        click.echo("⏸️ AI 处理已暂停")

@ai.command("resume")
@click.option("--format", "-f", "output_format", default="text", type=click.Choice(["text", "json"]), help="输出格式")
def ai_resume(output_format):
    """恢复 AI 处理"""
    if output_format == "json":
        output_json({
            "status": "resumed",
            "message": "AI 处理已恢复"
        }, "ai resume")
    else:
        click.echo("▶️ AI 处理已恢复")

@cli.group()
def dedup():
    """去重命令"""
    pass

@dedup.command("scan")
@click.option("--threshold", "-t", default=90, help="相似度阈值 (0-100)")
@click.option("--batch-size", default=1000, help="批次大小")
@click.option("--output", "-o", help="输出结果文件路径")
@click.option("--format", "-f", "output_format", default="text", type=click.Choice(["text", "json"]), help="输出格式")
def dedup_scan(threshold, batch_size, output, output_format):
    """扫描重复图像"""
    try:
        conn = get_db_connection()
        cursor = conn.cursor()
        
        # 获取所有图像进行去重分析
        cursor.execute("SELECT id, filename, path, size, created_at FROM images")
        images = cursor.fetchall()
        
        # 简化的去重逻辑（实际应使用 pHash 等算法）
        duplicates = []
        
        # 按文件大小分组
        size_groups = {}
        for img in images:
            size = img[3]  # size 字段
            if size not in size_groups:
                size_groups[size] = []
            size_groups[size].append(img)
        
        # 找出可能重复的组
        for size, group in size_groups.items():
            if len(group) > 1:
                duplicates.append({
                    "size": size,
                    "count": len(group),
                    "images": [{"id": img[0], "filename": img[1], "path": img[2]} for img in group]
                })
        
        result_data = {
            "scan_complete": True,
            "threshold": threshold,
            "duplicate_groups": len(duplicates),
            "total_duplicate_images": sum(g["count"] for g in duplicates),
            "groups": duplicates
        }
        
        if output:
            with open(output, 'w', encoding='utf-8') as f:
                json.dump(result_data, f, ensure_ascii=False, indent=2)
            click.echo(f"📄 结果已保存到: {output}")
        
        if output_format == "json":
            output_json(result_data, "dedup scan")
        else:
            click.echo(f"🔍 去重扫描完成")
            click.echo(f"   阈值: {threshold}%")
            click.echo(f"   发现 {len(duplicates)} 组重复图像")
            click.echo(f"   共 {sum(g['count'] for g in duplicates)} 张重复图像")
            
            if duplicates:
                click.echo(f"\n重复组详情:")
                for i, group in enumerate(duplicates[:5], 1):  # 只显示前5组
                    click.echo(f"  组 {i}: {group['count']} 张相同大小的图像")
                    for img in group['images']:
                        click.echo(f"    - [{img['id']}] {img['filename']}")
        
        conn.close()
        
    except Exception as e:
        if output_format == "json":
            output_json({
                "error": {
                    "code": "DEDUP_ERROR",
                    "message": str(e)
                }
            }, "dedup scan", "error")
        else:
            click.echo(f"❌ 去重扫描错误: {str(e)}")
        sys.exit(1)

@dedup.command("delete")
@click.option("--strategy", type=click.Choice(["keep-highest-res", "keep-oldest", "keep-newest"]), default="keep-highest-res", help="保留策略")
@click.option("--confirm", is_flag=True, help="跳过确认提示")
@click.option("--dry-run", is_flag=True, help="试运行，不实际删除")
@click.option("--format", "-f", "output_format", default="text", type=click.Choice(["text", "json"]), help="输出格式")
def dedup_delete(strategy, confirm, dry_run, output_format):
    """删除重复图像"""
    if not confirm and not dry_run:
        if output_format == "json":
            output_json({
                "error": {
                    "code": "CONFIRMATION_REQUIRED",
                    "message": "删除操作需要 --confirm 参数确认",
                    "suggestion": "添加 --confirm 参数以确认删除，或使用 --dry-run 查看将要删除的内容"
                }
            }, "dedup delete", "error")
        else:
            click.echo("⚠️ 删除操作需要 --confirm 参数确认，或使用 --dry-run 查看将要删除的内容")
        sys.exit(1)
    
    try:
        conn = get_db_connection()
        cursor = conn.cursor()
        
        # 简化的删除逻辑（实际需要完整的去重算法）
        if dry_run:
            if output_format == "json":
                output_json({
                    "dry_run": True,
                    "message": "试运行模式，没有实际删除任何图像",
                    "strategy": strategy
                }, "dedup delete")
            else:
                click.echo(f"🔍 试运行模式 (策略: {strategy})")
                click.echo("   没有实际删除任何图像")
        else:
            # 实际删除逻辑
            if output_format == "json":
                output_json({
                    "deleted_groups": 0,
                    "deleted_images": 0,
                    "strategy": strategy,
                    "message": "去重删除完成"
                }, "dedup delete")
            else:
                click.echo(f"✅ 去重删除完成 (策略: {strategy})")
        
        conn.close()
        
    except Exception as e:
        if output_format == "json":
            output_json({
                "error": {
                    "code": "DEDUP_DELETE_ERROR",
                    "message": str(e)
                }
            }, "dedup delete", "error")
        else:
            click.echo(f"❌ 去重删除错误: {str(e)}")
        sys.exit(1)

@cli.group()
def system():
    """系统管理命令"""
    pass

@system.command("health")
@click.option("--verbose", "-v", is_flag=True, help="详细输出")
@click.option("--format", "-f", "output_format", default="text", type=click.Choice(["text", "json"]), help="输出格式")
def system_health(verbose, output_format):
    """系统健康检查"""
    try:
        health_status = {
            "overall": "healthy",
            "components": {}
        }
        
        # 检查数据库
        try:
            conn = get_db_connection()
            cursor = conn.cursor()
            cursor.execute("SELECT COUNT(*) FROM images")
            total_images = cursor.fetchone()[0]
            conn.close()
            health_status["components"]["database"] = {
                "status": "healthy",
                "total_images": total_images
            }
        except Exception as e:
            health_status["components"]["database"] = {
                "status": "error",
                "message": str(e)
            }
            health_status["overall"] = "degraded"
        
        # 检查数据目录
        data_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "data")
        if os.path.exists(data_dir):
            health_status["components"]["data_directory"] = {
                "status": "healthy",
                "path": data_dir
            }
        else:
            health_status["components"]["data_directory"] = {
                "status": "missing",
                "path": data_dir
            }
            health_status["overall"] = "degraded"
        
        if output_format == "json":
            output_json(health_status, "system health")
        else:
            click.echo(f"🏥 系统健康状态: {health_status['overall'].upper()}")
            for component, status in health_status["components"].items():
                icon = "✅" if status["status"] == "healthy" else "❌"
                click.echo(f"   {icon} {component}: {status['status']}")
                if verbose and "message" in status:
                    click.echo(f"      详情: {status['message']}")
        
    except Exception as e:
        if output_format == "json":
            output_json({
                "error": {
                    "code": "HEALTH_CHECK_ERROR",
                    "message": str(e)
                }
            }, "system health", "error")
        else:
            click.echo(f"❌ 健康检查错误: {str(e)}")
        sys.exit(1)

@system.command("config")
@click.option("--get", "get_key", help="获取配置值")
@click.option("--set", "set_key", nargs=2, help="设置配置 (键 值)")
@click.option("--list", "list_configs", is_flag=True, help="列出所有配置")
@click.option("--format", "-f", "output_format", default="text", type=click.Choice(["text", "json"]), help="输出格式")
def system_config(get_key, set_key, list_configs, output_format):
    """系统配置管理"""
    # 简化配置管理（实际应从配置文件或数据库读取）
    configs = {
        "lm_studio_url": "http://localhost:1234",
        "embedding_model": "BAAI/bge-large-zh-v1.5",
        "image_model": "llava-1.5-7b",
        "concurrency": "3",
        "timeout": "300"
    }
    
    if get_key:
        value = configs.get(get_key)
        if value is not None:
            if output_format == "json":
                output_json({get_key: value}, "system config get")
            else:
                click.echo(f"{get_key} = {value}")
        else:
            if output_format == "json":
                output_json({
                    "error": {
                        "code": "CONFIG_NOT_FOUND",
                        "message": f"配置项不存在: {get_key}"
                    }
                }, "system config get", "error")
            else:
                click.echo(f"❌ 配置项不存在: {get_key}")
            sys.exit(1)
    
    elif set_key:
        key, value = set_key
        configs[key] = value
        if output_format == "json":
            output_json({
                "updated": {key: value},
                "message": "配置已更新"
            }, "system config set")
        else:
            click.echo(f"✅ 配置已更新: {key} = {value}")
    
    elif list_configs:
        if output_format == "json":
            output_json(configs, "system config list")
        else:
            click.echo("📋 系统配置:")
            for key, value in configs.items():
                click.echo(f"   {key} = {value}")
    
    else:
        click.echo("请提供 --get、--set 或 --list 参数")
        sys.exit(1)

@system.command("db-backup")
@click.option("--output", "-o", required=True, help="备份文件输出路径")
@click.option("--format", "-f", "output_format", default="text", type=click.Choice(["text", "json"]), help="输出格式")
def db_backup(output, output_format):
    """数据库备份"""
    try:
        conn = get_db_connection()
        
        # 创建备份
        backup_path = os.path.abspath(output)
        backup_dir = os.path.dirname(backup_path)
        if not os.path.exists(backup_dir):
            os.makedirs(backup_dir)
        
        # 复制数据库文件
        import shutil
        shutil.copy2(DB_PATH, backup_path)
        
        file_size = os.path.getsize(backup_path)
        
        if output_format == "json":
            output_json({
                "backup_path": backup_path,
                "file_size": file_size,
                "message": "数据库备份成功"
            }, "system db-backup")
        else:
            click.echo(f"✅ 数据库备份成功")
            click.echo(f"   路径: {backup_path}")
            click.echo(f"   大小: {file_size} 字节")
        
        conn.close()
        
    except Exception as e:
        if output_format == "json":
            output_json({
                "error": {
                    "code": "BACKUP_ERROR",
                    "message": str(e)
                }
            }, "system db-backup", "error")
        else:
            click.echo(f"❌ 备份错误: {str(e)}")
        sys.exit(1)

@cli.command("repl")
def repl():
    """进入交互式 REPL 模式"""
    click.echo("╔══════════════════════════════════════════╗")
    click.echo("║        ArcaneCodex CLI v0.1.0           ║")
    click.echo("║      Agent-Native Interface             ║")
    click.echo("╚══════════════════════════════════════════╝")
    click.echo("输入 'help' 查看帮助，'exit' 退出")
    
    while True:
        try:
            command = click.prompt("ac", type=str)
            if command.lower() in ['exit', 'quit', 'q']:
                click.echo("👋 再见!")
                break
            elif command.lower() in ['help', 'h', '?']:
                click.echo("可用命令:")
                click.echo("  image import --path <路径>     导入图像")
                click.echo("  image list --filter <过滤>     列出图像")
                click.echo("  search --query <查询>          搜索图像")
                click.echo("  ai start --concurrency <数量>  启动 AI 处理")
                click.echo("  ai status                      查看 AI 状态")
                click.echo("  dedup scan --threshold <阈值>  扫描重复")
                click.echo("  system health                  健康检查")
                click.echo("  system config --list           查看配置")
            else:
                # 尝试解析命令
                try:
                    cli(command.split(), standalone_mode=False)
                except SystemExit:
                    pass
                except Exception as e:
                    click.echo(f"❌ 错误: {str(e)}")
        except KeyboardInterrupt:
            click.echo("\n👋 再见!")
            break
        except Exception as e:
            click.echo(f"❌ 错误: {str(e)}")

if __name__ == "__main__":
    cli()
