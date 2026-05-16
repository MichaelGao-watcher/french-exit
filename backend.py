#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
French Exit - Core Backend
==========================
Pure Python logic for scanning, risk assessment, backup, and file operations.
Zero UI dependencies.
"""

import os
import sys
import shutil
import threading
import zipfile
from datetime import datetime
from pathlib import Path

try:
    from send2trash import send2trash
    HAS_SEND2TRASH = True
except ImportError:
    HAS_SEND2TRASH = False

# ====== Constants ======
RISK_HIGH = "高"
RISK_MED = "中"
RISK_LOW = "低"

ACTION_TAKE = "take"
ACTION_DEL = "delete"
ACTION_IGN = "ignore"

# Default system directories to skip
DEFAULT_SKIP_DIRS = {
    'windows', 'program files', 'program files (x86)', 'programdata',
    '$recycle.bin', 'system volume information', 'recovery',
    'intel', 'nvidia', 'amd', 'drivers', 'syswow64', 'system32',
    'microsoft office', 'windowsapps', 'application data',
}

DEFAULT_SKIP_NAMES = {
    'node_modules', '__pycache__', '.git', '.svn', '.hg',
    'venv', '.venv', 'env', '.tox', '.pytest_cache',
}


def format_size(size_bytes: int) -> str:
    """Convert bytes to human-readable string."""
    if size_bytes < 1024:
        return f"{size_bytes} B"
    elif size_bytes < 1024 ** 2:
        return f"{size_bytes / 1024:.2f} KB"
    elif size_bytes < 1024 ** 3:
        return f"{size_bytes / (1024 ** 2):.2f} MB"
    else:
        return f"{size_bytes / (1024 ** 3):.2f} GB"


def assess_risk(filepath: str):
    """
    Assess risk level of a single file.
    Returns: (risk_level, reason_string)
    """
    path_lower = filepath.lower()
    name_lower = os.path.basename(filepath).lower()
    ext = os.path.splitext(name_lower)[1]

    reasons = []
    score = 0.0

    # High-sensitivity paths
    high_paths = [
        'pictures', 'photos', 'images', 'camera', 'screenshot', '截屏', '截图',
        'music', 'videos', 'movies', 'movie', 'video',
        'desktop', 'downloads', 'documents',
        'wechat', 'weixin', 'qq', 'telegram', 'dingtalk', '飞书', 'lark',
        'personal', 'private', '我的', '个人', '私密', '隐私',
    ]
    # High-sensitivity extensions
    high_exts = {
        '.jpg', '.jpeg', '.png', '.gif', '.bmp', '.raw', '.heic', '.tiff', '.webp',
        '.mp4', '.mov', '.avi', '.mkv', '.flv', '.wmv',
        '.mp3', '.wav', '.flac', '.aac', '.ogg', '.m4a',
        '.pdf', '.doc', '.docx', '.xls', '.xlsx', '.ppt', '.pptx',
        '.txt', '.md', '.epub', '.mobi',
    }
    # High-sensitivity filename keywords
    high_names = [
        'resume', 'cv', '简历', 'curriculum',
        'idcard', '身份证', 'identity', 'id_card',
        'passport', '护照',
        'bank', '银行', 'creditcard', '信用卡',
        'salary', '工资', 'payslip', '薪资', '收入证明',
        'offer', '合同', 'contract', '劳动',
        'private', 'personal', 'secret', '机密',
        'love', 'letter', '日记', 'diary', 'chat', '聊天', 'record', '记录',
    ]
    # Code extensions
    code_exts = {'.py', '.js', '.ts', '.java', '.go', '.rs', '.cpp', '.c', '.h', '.rb', '.php', '.swift', '.kt'}

    for hp in high_paths:
        if hp in path_lower:
            score += 2.0
            reasons.append(f"个人目录({hp})")
            break

    if ext in high_exts:
        score += 1.0
        reasons.append("个人文件类型")

    for hn in high_names:
        if hn in name_lower:
            score += 3.0
            reasons.append("敏感文件名")
            break

    if ext in code_exts:
        if any(k in path_lower for k in ['github', 'gitee', 'gitlab', '个人', 'personal', 'my-', 'my_', 'side-project']):
            score += 2.0
            reasons.append("疑似个人项目")
        else:
            score += 0.5
            reasons.append("代码文件")

    # Installers / large downloads: low value, reduce score
    if ext in {'.exe', '.msi', '.dmg', '.pkg', '.deb', '.rpm', '.zip', '.rar', '.7z', '.iso', '.torrent'}:
        if 'download' in path_lower or '安装包' in path_lower:
            score -= 1.0
            reasons.append("可重下安装包")

    # Cache / temp / logs: very low risk
    if any(k in path_lower for k in ['cache', 'temp', 'tmp', 'log', '日志', '缓存', 'crash']):
        score = min(score, 0.5)
        reasons.append("缓存/日志")

    # Empty files
    try:
        if os.path.getsize(filepath) == 0:
            score = min(score, 0.5)
            reasons.append("空文件")
    except OSError:
        pass

    if score >= 3.0:
        return RISK_HIGH, "; ".join(reasons) if reasons else "高敏感度"
    elif score >= 1.0:
        return RISK_MED, "; ".join(reasons) if reasons else "中敏感度"
    else:
        return RISK_LOW, "; ".join(reasons) if reasons else "低风险"


def should_skip_dir(dirname: str, extra_excludes: set = None) -> bool:
    """Check if a directory should be skipped during scanning."""
    dlower = dirname.lower()
    if dlower in DEFAULT_SKIP_DIRS:
        return True
    if dlower.startswith('.'):
        return True
    if dlower in DEFAULT_SKIP_NAMES:
        return True
    if extra_excludes:
        for pat in extra_excludes:
            if pat.lower() in dlower or dlower in pat.lower():
                return True
    return False


def is_exempt(filepath: str, exempt_paths: list) -> bool:
    """Check if a file path is inside any exempt path."""
    if not exempt_paths:
        return False
    fp_lower = filepath.lower()
    for ep in exempt_paths:
        ep_lower = ep.lower().rstrip(os.sep)
        if fp_lower.startswith(ep_lower + os.sep) or fp_lower == ep_lower:
            return True
    return False


def get_drives():
    """
    Return list of available drives/root paths.
    Windows: ['C:\\', 'D:\\', ...]
    macOS/Linux: ['/']
    """
    if os.name == 'nt':
        import string
        import ctypes
        drives = []
        for letter in string.ascii_uppercase:
            drive = f"{letter}:\\"
            try:
                dtype = ctypes.windll.kernel32.GetDriveTypeW(drive)
                if dtype in (2, 3, 4):  # removable, fixed, remote
                    drives.append(drive)
            except Exception:
                pass
        return drives if drives else [os.path.expanduser("~")]
    else:
        return ['/']


def scan_paths(
    paths,
    start_timestamp: float,
    exclude_patterns: set = None,
    exempt_paths: list = None,
    progress_callback=None,
    stop_event: threading.Event = None
):
    """
    Recursively scan paths for files modified after start_timestamp.
    Returns list of file dicts.
    """
    results = []
    scanned = 0

    for root_path in paths:
        if not os.path.isdir(root_path):
            continue
        try:
            for dirpath, dirnames, filenames in os.walk(root_path):
                if stop_event and stop_event.is_set():
                    return results

                # Filter out system/hidden dirs
                dirnames[:] = [d for d in dirnames if not should_skip_dir(d, exclude_patterns)]

                for filename in filenames:
                    filepath = os.path.join(dirpath, filename)

                    if is_exempt(filepath, exempt_paths):
                        continue

                    try:
                        stat = os.stat(filepath)
                        mtime = stat.st_mtime
                        if mtime < start_timestamp:
                            continue

                        size = stat.st_size
                        mtime_str = datetime.fromtimestamp(mtime).strftime('%Y-%m-%d %H:%M')
                        risk, reason = assess_risk(filepath)

                        results.append({
                            'path': filepath,
                            'name': filename,
                            'size': size,
                            'size_str': format_size(size),
                            'mtime': mtime_str,
                            'mtime_ts': mtime,
                            'risk': risk,
                            'reason': reason,
                            'action': ACTION_IGN,
                        })
                    except (OSError, PermissionError):
                        continue

                    scanned += 1
                    if progress_callback and scanned % 300 == 0:
                        progress_callback(scanned, filepath, len(results))
        except (PermissionError, OSError):
            continue

    return results


def build_dest_path(source_path: str, out_dir: str) -> str:
    """Build destination path preserving drive letter structure."""
    drive, rest = os.path.splitdrive(source_path)
    drive_clean = drive.replace(':', '').upper() if drive else ''
    safe_path = rest.lstrip('\\/').replace('/', os.sep)
    if drive_clean:
        return os.path.normpath(os.path.join(out_dir, drive_clean, safe_path))
    return os.path.normpath(os.path.join(out_dir, safe_path))


def take_files(file_list, dest_dir: str, mode: str = 'copy'):
    """
    Copy or zip files to dest_dir.
    mode: 'copy' | 'zip'
    Returns: (success_count, fail_count, log_lines)
    """
    os.makedirs(dest_dir, exist_ok=True)
    success = 0
    failed = 0
    log_lines = []

    if mode == 'zip':
        zip_name = f"french_exit_take_{datetime.now().strftime('%Y%m%d_%H%M%S')}.zip"
        zip_path = os.path.join(dest_dir, zip_name)
        try:
            with zipfile.ZipFile(zip_path, 'w', zipfile.ZIP_DEFLATED) as zf:
                for f in file_list:
                    try:
                        arcname = os.path.relpath(f['path'], os.path.dirname(f['path']))
                        # Use full path inside zip to avoid collisions
                        drive, rest = os.path.splitdrive(f['path'])
                        drive_clean = drive.replace(':', '').upper() if drive else 'ROOT'
                        safe_arc = os.path.join(drive_clean, rest.lstrip('\\/'))
                        zf.write(f['path'], safe_arc)
                        success += 1
                        log_lines.append(f"[TAKE_ZIP] {f['path']} -> {zip_path}")
                    except Exception as e:
                        failed += 1
                        log_lines.append(f"[TAKE_ZIP_FAIL] {f['path']} - {e}")
        except Exception as e:
            for f in file_list:
                failed += 1
                log_lines.append(f"[TAKE_ZIP_FAIL] {f['path']} - {e}")
    else:
        for f in file_list:
            try:
                dest = build_dest_path(f['path'], dest_dir)
                os.makedirs(os.path.dirname(dest), exist_ok=True)
                shutil.copy2(f['path'], dest)
                success += 1
                log_lines.append(f"[TAKE] {f['path']} -> {dest}")
            except Exception as e:
                failed += 1
                log_lines.append(f"[TAKE_FAIL] {f['path']} - {e}")

    return success, failed, log_lines


def backup_and_delete(file_list, backup_dir: str = None):
    """
    1. Zip backup all files to backup_dir.
    2. Move files to trash (or delete permanently if send2trash unavailable).
    Returns: (success_del, fail_del, backup_path, log_lines)
    """
    if backup_dir is None:
        backup_dir = os.path.join(os.path.expanduser("~"), "AppData", "Local", "Temp")
        if not os.path.isdir(backup_dir):
            backup_dir = os.path.expanduser("~")

    timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
    backup_name = f"french_exit_backup_{timestamp}.zip"
    backup_path = os.path.join(backup_dir, backup_name)

    log_lines = []
    success_del = 0
    fail_del = 0

    # Create backup zip
    try:
        with zipfile.ZipFile(backup_path, 'w', zipfile.ZIP_DEFLATED) as zf:
            for f in file_list:
                try:
                    drive, rest = os.path.splitdrive(f['path'])
                    drive_clean = drive.replace(':', '').upper() if drive else 'ROOT'
                    arcname = os.path.join(drive_clean, rest.lstrip('\\/'))
                    zf.write(f['path'], arcname)
                except Exception as e:
                    log_lines.append(f"[BACKUP_FAIL] {f['path']} - {e}")
    except Exception as e:
        log_lines.append(f"[BACKUP_ZIP_FAIL] {backup_path} - {e}")
        backup_path = None

    # Delete files
    for f in file_list:
        try:
            if HAS_SEND2TRASH:
                send2trash(f['path'])
            else:
                if os.path.isdir(f['path']):
                    shutil.rmtree(f['path'])
                else:
                    os.remove(f['path'])
            success_del += 1
            log_lines.append(f"[DEL] {f['path']}")
        except Exception as e:
            fail_del += 1
            log_lines.append(f"[DEL_FAIL] {f['path']} - {e}")

    return success_del, fail_del, backup_path, log_lines


def generate_log(take_logs, del_logs, backup_path, exempt_paths, output_dir):
    """Generate operation log file."""
    os.makedirs(output_dir, exist_ok=True)
    log_path = os.path.join(output_dir, "french_exit_log.txt")
    lines = [
        "French Exit 操作日志",
        f"时间: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}",
        f"输出目录: {output_dir}",
        "",
    ]
    if exempt_paths:
        lines.append("豁免路径:")
        for ep in exempt_paths:
            lines.append(f"  - {ep}")
        lines.append("")

    if backup_path:
        lines.append(f"删除前备份: {backup_path}")
        lines.append("")

    if take_logs:
        lines.append("=== 带走记录 ===")
        for log in take_logs:
            lines.append(log)
        lines.append("")

    if del_logs:
        lines.append("=== 删除记录 ===")
        for log in del_logs:
            lines.append(log)
        lines.append("")

    lines.append("French Exit 已完成。除豁免路径外，所有本地文件均已完成剔除或转移。")

    with open(log_path, 'w', encoding='utf-8') as f:
        f.write("\n".join(lines))

    return log_path
