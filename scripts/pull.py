#!/usr/bin/env python3
"""
母库经验拉取脚本（分发模式）
从 vibe-coding-project-sop 母库拉取最新经验到当前项目。

用法:
    python pull.py [--skeleton SKELETON_PATH] [--force]

环境变量:
    SOP_SKELETON_PATH  母库项目根目录路径
"""

import argparse
import os
import sys
from pathlib import Path
from typing import List, Optional, Tuple


KNOWLEDGE_FILES = [
    "lessons-learned.md",
    "troubleshooting.md",
    "decisions.md",
]

AGENTS_FRAGMENT = """## 3.7 母库经验指令（「拉取母库」）

**触发词**：`拉取母库`、`拉取经验`、`更新经验`（去除标点后精确匹配任一）

**防误触**：
- 消息精确匹配上述任一触发词 → 执行母库经验同步流程
- 消息包含触发词但还有其他内容 → 视为正常对话，不触发

**执行流程**：
1. 读取项目中 `<!-- 母库: <路径> -->` 注释，获取母库路径
2. 运行 `python <母库路径>/scripts/pull.py --skeleton <母库路径>`
3. 汇报同步结果
"""


def log(msg: str) -> None:
    print(f"[pull] {msg}")


def error(msg: str) -> None:
    print(f"[pull] ERROR: {msg}", file=sys.stderr)


def find_skeleton_path(args_path: Optional[str]) -> Optional[Path]:
    """定位母库路径"""
    if args_path:
        p = Path(args_path).resolve()
        if p.is_dir() and (p / "scripts" / "pull.py").exists():
            return p
        error(f"母库路径无效: {p}")
        return None

    env_path = os.environ.get("SOP_SKELETON_PATH")
    if env_path:
        p = Path(env_path).resolve()
        if p.is_dir():
            return p

    error("请指定母库路径 --skeleton 或设置 SOP_SKELETON_PATH 环境变量")
    return None


def copy_file(src: Path, dst: Path, force: bool = False) -> str:
    """复制文件，返回操作结果"""
    if dst.exists() and not force:
        return "skipped"

    result = "overwritten" if dst.exists() else "created"
    dst.write_text(src.read_text(encoding="utf-8"), encoding="utf-8")
    return result


def ensure_agents_fragment(target_dir: Path) -> bool:
    """确保 AGENTS.md 中包含母库经验指令"""
    agents_file = target_dir / "AGENTS.md"
    if not agents_file.exists():
        return False

    content = agents_file.read_text(encoding="utf-8")
    if "拉取母库" in content or "拉取经验" in content or "母库经验" in content:
        return True

    # 追加触发词指令
    with open(agents_file, "a", encoding="utf-8") as f:
        f.write("\n\n---\n\n" + AGENTS_FRAGMENT)
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description="从母库拉取经验")
    parser.add_argument("--skeleton", help="母库项目根目录路径")
    parser.add_argument("--force", action="store_true", help="覆盖已有文件")
    args = parser.parse_args()

    skeleton_path = find_skeleton_path(args.skeleton)
    if not skeleton_path:
        return 1

    target_dir = Path.cwd()
    log(f"母库路径: {skeleton_path}")
    log(f"当前项目: {target_dir}")

    results: List[Tuple[str, str]] = []

    for fname in KNOWLEDGE_FILES:
        src = skeleton_path / fname
        dst = target_dir / fname
        if src.exists():
            result = copy_file(src, dst, args.force)
            results.append((fname, result))
        else:
            error(f"母库文件缺失: {src}")

    # 确保 AGENTS.md 有触发词指令
    ensure_agents_fragment(target_dir)

    # 输出报告
    print("\n" + "=" * 40)
    print("拉取结果")
    print("=" * 40)

    created = [f for f, r in results if r == "created"]
    skipped = [f for f, r in results if r == "skipped"]
    overwritten = [f for f, r in results if r == "overwritten"]

    if created:
        print(f"\n[+] 新增 ({len(created)}):")
        for f in created:
            print(f"    {f}")

    if skipped:
        print(f"\n[-] 已存在 ({len(skipped)}):")
        for f in skipped:
            print(f"    {f}")

    if overwritten:
        print(f"\n[~] 已更新 ({len(overwritten)}):")
        for f in overwritten:
            print(f"    {f}")

    print(f"\n[>] 下次使用: 在项目中说「拉取母库」即可拉取最新内容")

    return 0


if __name__ == "__main__":
    sys.exit(main())
