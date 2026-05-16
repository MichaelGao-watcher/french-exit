#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Environment check and user guidance for French Exit.
"""

import sys
import subprocess
import os

# Windows: set console to UTF-8 so Chinese prints correctly
if sys.platform == 'win32':
    try:
        import ctypes
        ctypes.windll.kernel32.SetConsoleOutputCP(65001)
    except Exception:
        pass

MIN_PYTHON_VERSION = (3, 8)


def check_python():
    """Check if Python version meets minimum requirement."""
    current = sys.version_info[:2]
    if current < MIN_PYTHON_VERSION:
        return False, f"需要 Python {MIN_PYTHON_VERSION[0]}.{MIN_PYTHON_VERSION[1]}+，当前为 {current[0]}.{current[1]}"
    return True, f"Python {current[0]}.{current[1]} [OK]"


def check_module(module_name):
    """Check if a Python module can be imported."""
    try:
        __import__(module_name)
        return True
    except ImportError:
        return False


def check_browser():
    """Check if a suitable browser is available for Eel."""
    # Eel can fallback to system default browser, but Chrome/Edge is best
    browsers = ['chrome', 'msedge', 'firefox']
    found = []
    for b in browsers:
        if sys.platform == 'win32':
            # Try to find via where command
            try:
                result = subprocess.run(['where', b], capture_output=True, text=True)
                if result.returncode == 0:
                    found.append(b)
            except Exception:
                pass
        else:
            try:
                result = subprocess.run(['which', b], capture_output=True, text=True)
                if result.returncode == 0:
                    found.append(b)
            except Exception:
                pass
    return found


def run_env_check():
    """
    Run full environment check and return a report dict.
    If critical issues found, print guidance to console.
    """
    report = {
        'ok': True,
        'python_ok': False,
        'python_msg': '',
        'eel_ok': False,
        'send2trash_ok': False,
        'browsers': [],
        'fix_commands': [],
        'messages': []
    }

    # Python version
    ok, msg = check_python()
    report['python_ok'] = ok
    report['python_msg'] = msg
    report['messages'].append(msg)
    if not ok:
        report['ok'] = False
        report['messages'].append("请前往 https://www.python.org/downloads/ 安装最新版 Python。")
        return report

    # Required modules
    if check_module('eel'):
        report['eel_ok'] = True
        report['messages'].append("eel 已安装 [OK]")
    else:
        report['ok'] = False
        report['messages'].append("eel 未安装 [MISSING]")
        report['fix_commands'].append("pip install eel")

    if check_module('send2trash'):
        report['send2trash_ok'] = True
        report['messages'].append("send2trash 已安装 [OK]")
    else:
        report['messages'].append("send2trash 未安装 [WARN]（删除将无法进入回收站）")
        report['fix_commands'].append("pip install send2trash")

    # Browser check
    browsers = check_browser()
    report['browsers'] = browsers
    if browsers:
        report['messages'].append(f"检测到浏览器: {', '.join(browsers)} [OK]")
    else:
        report['messages'].append("未检测到 Chrome/Edge/Firefox，将尝试使用系统默认浏览器。")

    return report


def safe_print(text):
    """Print with encoding fallback for Windows terminals."""
    try:
        print(text)
    except UnicodeEncodeError:
        print(text.encode('gbk', 'ignore').decode('gbk'))


def print_guidance(report):
    """Print environment guidance to console."""
    safe_print("=" * 50)
    safe_print("French Exit 环境检查")
    safe_print("=" * 50)
    for msg in report['messages']:
        safe_print(f"  {msg}")

    if report['fix_commands']:
        safe_print("")
        safe_print("请运行以下命令安装缺失依赖：")
        safe_print("-" * 50)
        for cmd in report['fix_commands']:
            safe_print(f"  {cmd}")
        safe_print("-" * 50)
        safe_print("")
        safe_print("安装完成后，重新运行本程序。")
        return False

    safe_print("")
    safe_print("环境检查通过，正在启动 French Exit...")
    return True


def ensure_env():
    """Convenience function: check and print guidance. Returns True if ready."""
    report = run_env_check()
    return print_guidance(report)


if __name__ == '__main__':
    ensure_env()
