#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Eel UI bridge for French Exit.
Exposes Python functions to the JavaScript frontend.
"""

import os
import sys
import threading
from datetime import datetime

import eel

# Add parent directory to path so we can import backend
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from backend import (
    get_drives, scan_paths, assess_risk,
    take_files, backup_and_delete, generate_log,
    format_size, RISK_HIGH, RISK_MED, RISK_LOW,
    ACTION_TAKE, ACTION_DEL, ACTION_IGN,
    HAS_SEND2TRASH,
)

# Global state
_scan_thread = None
_stop_event = threading.Event()
_scan_results = []


def _get_web_dir():
    base = os.path.dirname(os.path.abspath(__file__))
    # PyInstaller support: when frozen, files are extracted to _MEIPASS
    if getattr(sys, 'frozen', False) and hasattr(sys, '_MEIPASS'):
        base = os.path.join(sys._MEIPASS, 'ui')
    return os.path.join(base, 'web')


# ====== Exposed functions ======

@eel.expose
def py_get_drives():
    """Return list of available drives."""
    return get_drives()


@eel.expose
def py_get_env_info():
    """Return environment info for frontend display."""
    return {
        'has_send2trash': HAS_SEND2TRASH,
        'platform': sys.platform,
    }


@eel.expose
def py_start_scan(config):
    """
    Start scanning with given config.
    config: dict with keys:
        - start_date: str "YYYY-MM-DD"
        - drives: list of drive paths
        - custom_paths: list of extra paths
        - exclude_dirs: list of dir names to exclude
        - exempt_paths: list of exempt paths
    """
    global _scan_thread, _stop_event, _scan_results

    # Parse date
    try:
        start_date = datetime.strptime(config['start_date'], '%Y-%m-%d')
        start_ts = start_date.timestamp()
    except (KeyError, ValueError) as e:
        eel.js_on_scan_error(f"日期格式错误: {e}")
        return

    paths = list(config.get('drives', []))
    paths.extend([p for p in config.get('custom_paths', []) if os.path.isdir(p)])

    if not paths:
        eel.js_on_scan_error("没有选择任何扫描路径")
        return

    exclude = set(d.lower().strip() for d in config.get('exclude_dirs', []) if d.strip())
    exempt = [p.strip() for p in config.get('exempt_paths', []) if p.strip()]

    _stop_event.clear()
    _scan_results.clear()

    def progress(scanned, current, found):
        try:
            eel.js_on_scan_progress(scanned, os.path.basename(current), found)
        except Exception:
            pass

    def do_scan():
        try:
            results = scan_paths(
                paths, start_ts,
                exclude_patterns=exclude,
                exempt_paths=exempt,
                progress_callback=progress,
                stop_event=_stop_event
            )
            _scan_results[:] = results
            # Sort by risk then mtime desc
            risk_order = {RISK_HIGH: 0, RISK_MED: 1, RISK_LOW: 2}
            results.sort(key=lambda x: (risk_order.get(x['risk'], 3), -x['mtime_ts']))
            try:
                eel.js_on_scan_done(results)
            except Exception:
                pass
        except Exception as e:
            try:
                eel.js_on_scan_error(str(e))
            except Exception:
                pass

    _scan_thread = threading.Thread(target=do_scan, daemon=True)
    _scan_thread.start()


@eel.expose
def py_stop_scan():
    """Signal the scan thread to stop."""
    _stop_event.set()


@eel.expose
def py_reassess_risk(filepath):
    """Re-assess risk for a single file (useful if frontend wants to refresh)."""
    return assess_risk(filepath)


@eel.expose
def py_execute_actions(actions_config):
    """
    Execute user-selected actions.
    actions_config: dict with keys:
        - take_files: list of file paths
        - delete_files: list of file paths
        - output_dir: str
        - take_mode: 'copy' | 'zip'
        - exempt_paths: list
    Returns: result dict
    """
    take_paths = set(actions_config.get('take_files', []))
    delete_paths = set(actions_config.get('delete_files', []))
    output_dir = actions_config.get('output_dir', os.path.expanduser('~'))
    take_mode = actions_config.get('take_mode', 'copy')
    exempt_paths = actions_config.get('exempt_paths', [])

    take_items = [f for f in _scan_results if f['path'] in take_paths]
    delete_items = [f for f in _scan_results if f['path'] in delete_paths]

    result = {
        'success': True,
        'take_success': 0,
        'take_failed': 0,
        'del_success': 0,
        'del_failed': 0,
        'backup_path': None,
        'log_path': None,
        'output_dir': output_dir,
        'exempt_paths': exempt_paths,
        'errors': []
    }

    take_logs = []
    del_logs = []

    # Take files
    if take_items:
        try:
            s, f, logs = take_files(take_items, output_dir, mode=take_mode)
            result['take_success'] = s
            result['take_failed'] = f
            take_logs = logs
        except Exception as e:
            result['success'] = False
            result['errors'].append(f"带走操作失败: {e}")

    # Delete files
    if delete_items:
        try:
            s, f, backup_path, logs = backup_and_delete(delete_items)
            result['del_success'] = s
            result['del_failed'] = f
            result['backup_path'] = backup_path
            del_logs = logs
        except Exception as e:
            result['success'] = False
            result['errors'].append(f"删除操作失败: {e}")

    # Generate log
    try:
        log_path = generate_log(take_logs, del_logs, result['backup_path'], exempt_paths, output_dir)
        result['log_path'] = log_path
    except Exception as e:
        result['errors'].append(f"日志生成失败: {e}")

    return result


@eel.expose
def py_open_folder(path):
    """Open a folder in system file explorer."""
    if not os.path.isdir(path):
        return False
    if sys.platform == 'win32':
        os.startfile(path)
    elif sys.platform == 'darwin':
        os.system(f'open "{path}"')
    else:
        os.system(f'xdg-open "{path}"')
    return True


# ====== Launch ======

def start_ui():
    web_dir = _get_web_dir()
    eel.init(web_dir)

    start_kwargs = {
        'size': (1280, 800),
        'close_callback': lambda *args: sys.exit(0),
    }

    # Try Chrome/Edge app mode first, fallback to default browser
    for mode in ['chrome', 'edge', 'default']:
        try:
            eel.start('index.html', mode=mode, **start_kwargs)
            break
        except EnvironmentError:
            if mode == 'default':
                raise
            continue


if __name__ == '__main__':
    start_ui()
