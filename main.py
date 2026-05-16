#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
French Exit - 工作电脑安全撤离工具
====================================
入口文件：环境检查 → 启动交互界面

用法:
    python main.py
"""

import sys
import os

# Ensure project root is on path
PROJECT_ROOT = os.path.dirname(os.path.abspath(__file__))
if PROJECT_ROOT not in sys.path:
    sys.path.insert(0, PROJECT_ROOT)

from env_check import ensure_env
from ui.eel_app import start_ui


def main():
    # 环境检查与引导
    if not ensure_env():
        print("\n环境准备完成后，请重新运行本程序。")
        input("按 Enter 键退出...")
        sys.exit(1)

    # 启动 UI
    start_ui()


if __name__ == '__main__':
    main()
