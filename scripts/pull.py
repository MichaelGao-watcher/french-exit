#!/usr/bin/env python3
"""
其他项目从母库一键初始化脚本。

用法：
    python init-from-skeleton.py

功能：
1. 自动创建 config/github-sync.json（预填母库信息）
2. 自动下载/更新 scripts/sync-knowledge.py
3. 自动运行同步，拉取母库经验文件
4. 提示 AGENTS 规则插入方法

依赖：
    pip install requests
"""

import json
import os
import sys
import io
from pathlib import Path

# Windows 终端 UTF-8 支持
if sys.platform == "win32":
    try:
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")
    except Exception:
        pass

try:
    import requests
except ImportError:
    print("[init] 错误: 需要安装 requests 库。运行: pip install requests")
    sys.exit(1)

# ===== 母库配置（硬编码，其他项目无需修改） =====
MOTHER_REPO = "vibe-coding-project-sop"
MOTHER_USERNAME = "MichaelGao1999"
MOTHER_BRANCH = "master"
RAW_BASE = f"https://raw.githubusercontent.com/{MOTHER_USERNAME}/{MOTHER_REPO}/{MOTHER_BRANCH}"
TARGET_FILES = ["decisions.md", "lessons-learned.md", "troubleshooting.md"]


def log(msg: str) -> None:
    print(f"[init] {msg}")


def fetch_raw(filepath: str) -> str | None:
    url = f"{RAW_BASE}/{filepath}"
    try:
        resp = requests.get(url, timeout=30)
        if resp.status_code == 404:
            return None
        resp.raise_for_status()
        return resp.text
    except requests.RequestException as e:
        log(f"拉取失败: {url} -> {e}")
        return None


def ensure_dir(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)


def write_file(path: Path, content: str) -> None:
    ensure_dir(path.parent)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)


def create_config() -> Path:
    config_path = Path("config/github-sync.json")
    if config_path.exists():
        log(f"配置文件已存在: {config_path}")
        with open(config_path, "r", encoding="utf-8") as f:
            cfg = json.load(f)
        if cfg.get("syncFrom"):
            log(f"  syncFrom 已配置为: {cfg['syncFrom']}")
        else:
            log("  syncFrom 为空，将设置为母库...")
            cfg["syncFrom"] = MOTHER_REPO
            write_file(config_path, json.dumps(cfg, indent=2, ensure_ascii=False))
            log("  已更新 syncFrom")
        return config_path

    config = {
        "_comment": "由 init-from-skeleton.py 自动生成。修改后运行 scripts/sync-knowledge.py 同步更新。",
        "username": MOTHER_USERNAME,
        "token": "",
        "includeRepos": [],
        "excludeRepos": [],
        "syncFrom": MOTHER_REPO,
        "_syncFromComment": "只从指定母库拉取经验。如需改为聚合模式（遍历所有仓库），请清空此字段。",
        "targetFiles": TARGET_FILES,
        "branch": "master",
        "backupBeforeMerge": True,
        "mergeStrategy": {
            "lessonsLearned": {"dedupBy": "description", "action": "mergeSources"},
            "troubleshooting": {"dedupBy": "keyword", "action": "keepLongestSolution"},
            "decisions": {"dedupBy": "none", "action": "appendAll"}
        }
    }
    write_file(config_path, json.dumps(config, indent=2, ensure_ascii=False))
    log(f"已创建配置文件: {config_path}")
    return config_path


def download_sync_script() -> Path:
    script_path = Path("scripts/sync-knowledge.py")
    log("正在下载同步脚本...")
    content = fetch_raw("scripts/sync-knowledge.py")
    if content is None:
        log("错误: 无法下载同步脚本，请检查网络连接")
        sys.exit(1)
    write_file(script_path, content)
    log(f"已更新同步脚本: {script_path}")
    return script_path


def run_sync() -> int:
    log("开始从母库同步经验...")
    import subprocess
    result = subprocess.run(
        [sys.executable, "scripts/sync-knowledge.py"],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    if result.returncode != 0:
        print(result.stderr)
    return result.returncode


def print_agents_guide() -> None:
    agents_url = f"{RAW_BASE}/templates/agents-for-others.md"
    print("\n" + "=" * 50)
    print("📋 AGENTS 规则")
    print("=" * 50)
    print(f"其他项目专用指令：{agents_url}")
    print("=" * 50)


def print_summary() -> None:
    print("\n" + "=" * 50)
    print("✅ 拉取完成")
    print("=" * 50)
    print("📁 配置文件：config/github-sync.json")
    print("📄 同步文件：decisions.md, lessons-learned.md, troubleshooting.md")
    print("\n💡 下次更新：python scripts/pull.py")
    print("=" * 50)


def main() -> int:
    log("开始从母库拉取...")
    log(f"母库: {MOTHER_USERNAME}/{MOTHER_REPO} ({MOTHER_BRANCH})")

    create_config()
    download_sync_script()
    sync_rc = run_sync()
    print_agents_guide()
    print_summary()

    return sync_rc


if __name__ == "__main__":
    sys.exit(main())
