#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
File Cleaner - Scan and selectively delete files.
A simple GUI tool to scan directories, filter by size/type, and delete selected files.
"""

import os
import sys
import threading
import tkinter as tk
from tkinter import ttk, filedialog, messagebox
from datetime import datetime
from pathlib import Path

try:
    from send2trash import send2trash
    HAS_SEND2TRASH = True
except ImportError:
    HAS_SEND2TRASH = False


def format_size(size_bytes):
    """Convert bytes to human-readable string."""
    if size_bytes < 1024:
        return f"{size_bytes} B"
    elif size_bytes < 1024 ** 2:
        return f"{size_bytes / 1024:.2f} KB"
    elif size_bytes < 1024 ** 3:
        return f"{size_bytes / (1024 ** 2):.2f} MB"
    else:
        return f"{size_bytes / (1024 ** 3):.2f} GB"


def scan_directory(root_path, min_size=0, extensions=None, progress_callback=None):
    """Recursively scan directory and return file list."""
    results = []
    total = 0
    try:
        for dirpath, dirnames, filenames in os.walk(root_path):
            for filename in filenames:
                filepath = os.path.join(dirpath, filename)
                try:
                    stat = os.stat(filepath)
                    size = stat.st_size
                    if size < min_size:
                        continue
                    if extensions:
                        ext = os.path.splitext(filename)[1].lower()
                        if ext not in extensions:
                            continue
                    mtime = datetime.fromtimestamp(stat.st_mtime).strftime('%Y-%m-%d %H:%M')
                    results.append({
                        'path': filepath,
                        'name': filename,
                        'size': size,
                        'size_str': format_size(size),
                        'mtime': mtime
                    })
                except (OSError, PermissionError):
                    continue
                total += 1
                if progress_callback and total % 500 == 0:
                    progress_callback(total, filepath)
    except PermissionError:
        pass
    return results


class FileCleanerApp:
    def __init__(self, root):
        self.root = root
        self.root.title("File Cleaner - 文件清理工具")
        self.root.geometry("900x600")
        self.root.minsize(700, 450)

        self.files_data = []
        self.checked_items = set()

        self._build_ui()

    def _build_ui(self):
        # === Top Control Panel ===
        control_frame = ttk.Frame(self.root, padding=10)
        control_frame.pack(fill=tk.X)

        ttk.Label(control_frame, text="扫描目录:").grid(row=0, column=0, sticky=tk.W, padx=5)
        self.path_var = tk.StringVar(value=os.path.expanduser("~"))
        ttk.Entry(control_frame, textvariable=self.path_var, width=50).grid(row=0, column=1, sticky=tk.EW, padx=5)
        ttk.Button(control_frame, text="浏览...", command=self._browse).grid(row=0, column=2, padx=5)

        ttk.Label(control_frame, text="最小大小:").grid(row=1, column=0, sticky=tk.W, padx=5, pady=5)
        self.min_size_var = tk.StringVar(value="1")
        size_combo = ttk.Combobox(control_frame, textvariable=self.min_size_var, width=10, values=["0", "1", "10", "100", "500", "1024"])
        size_combo.grid(row=1, column=1, sticky=tk.W, padx=5, pady=5)
        ttk.Label(control_frame, text="MB").grid(row=1, column=1, sticky=tk.W, padx=(80, 0), pady=5)

        ttk.Label(control_frame, text="文件类型:").grid(row=1, column=2, sticky=tk.W, padx=5, pady=5)
        self.ext_var = tk.StringVar()
        ttk.Entry(control_frame, textvariable=self.ext_var, width=20).grid(row=1, column=3, sticky=tk.W, padx=5, pady=5)
        ttk.Label(control_frame, text="例: .mp4 .zip (留空=全部)", foreground="gray").grid(row=1, column=4, sticky=tk.W, padx=5, pady=5)

        ttk.Button(control_frame, text="开始扫描", command=self._start_scan).grid(row=0, column=3, rowspan=2, padx=10, pady=5)

        control_frame.columnconfigure(1, weight=1)

        # === Progress Bar ===
        self.progress_var = tk.DoubleVar(value=0)
        self.progress_bar = ttk.Progressbar(self.root, variable=self.progress_var, maximum=100, mode='indeterminate')
        self.progress_bar.pack(fill=tk.X, padx=10, pady=(0, 5))
        self.status_var = tk.StringVar(value="就绪")
        ttk.Label(self.root, textvariable=self.status_var).pack(anchor=tk.W, padx=10)

        # === Treeview ===
        tree_frame = ttk.Frame(self.root)
        tree_frame.pack(fill=tk.BOTH, expand=True, padx=10, pady=5)

        columns = ('select', 'name', 'size', 'mtime', 'path')
        self.tree = ttk.Treeview(tree_frame, columns=columns, show='headings', selectmode='browse')
        self.tree.heading('select', text='选择')
        self.tree.heading('name', text='文件名')
        self.tree.heading('size', text='大小')
        self.tree.heading('mtime', text='修改时间')
        self.tree.heading('path', text='路径')
        self.tree.column('select', width=50, anchor=tk.CENTER)
        self.tree.column('name', width=150)
        self.tree.column('size', width=80, anchor=tk.E)
        self.tree.column('mtime', width=120)
        self.tree.column('path', width=400)

        vsb = ttk.Scrollbar(tree_frame, orient=tk.VERTICAL, command=self.tree.yview)
        hsb = ttk.Scrollbar(tree_frame, orient=tk.HORIZONTAL, command=self.tree.xview)
        self.tree.configure(yscrollcommand=vsb.set, xscrollcommand=hsb.set)

        self.tree.grid(row=0, column=0, sticky='nsew')
        vsb.grid(row=0, column=1, sticky='ns')
        hsb.grid(row=1, column=0, sticky='ew')
        tree_frame.grid_rowconfigure(0, weight=1)
        tree_frame.grid_columnconfigure(0, weight=1)

        self.tree.bind('<ButtonRelease-1>', self._on_tree_click)

        # === Bottom Actions ===
        btn_frame = ttk.Frame(self.root, padding=10)
        btn_frame.pack(fill=tk.X)

        ttk.Button(btn_frame, text="全选", command=self._select_all).pack(side=tk.LEFT, padx=5)
        ttk.Button(btn_frame, text="取消全选", command=self._deselect_all).pack(side=tk.LEFT, padx=5)
        ttk.Button(btn_frame, text="删除选中", command=self._delete_selected).pack(side=tk.RIGHT, padx=5)

        # Recycle bin warning
        if not HAS_SEND2TRASH:
            warn = ttk.Label(btn_frame, text="⚠️ 未安装 send2trash，删除将直接永久删除！", foreground="red")
            warn.pack(side=tk.RIGHT, padx=10)

    def _browse(self):
        path = filedialog.askdirectory()
        if path:
            self.path_var.set(path)

    def _start_scan(self):
        path = self.path_var.get().strip()
        if not path or not os.path.isdir(path):
            messagebox.showerror("错误", "请选择有效的扫描目录")
            return

        try:
            min_mb = float(self.min_size_var.get() or 0)
        except ValueError:
            min_mb = 0

        ext_text = self.ext_var.get().strip().lower()
        extensions = None
        if ext_text:
            extensions = [e.strip() if e.strip().startswith('.') else '.' + e.strip() for e in ext_text.split()]

        # Clear previous
        for item in self.tree.get_children():
            self.tree.delete(item)
        self.files_data.clear()
        self.checked_items.clear()

        self.status_var.set("正在扫描...")
        self.progress_bar.start()

        def progress(count, current):
            self.status_var.set(f"已扫描 {count} 个文件... {os.path.basename(current)}")
            self.root.update_idletasks()

        def do_scan():
            results = scan_directory(path, min_size=int(min_mb * 1024 * 1024), extensions=extensions, progress_callback=progress)
            self.root.after(0, lambda: self._scan_done(results))

        threading.Thread(target=do_scan, daemon=True).start()

    def _scan_done(self, results):
        self.progress_bar.stop()
        self.files_data = results
        for idx, f in enumerate(results):
            tag = 'checked' if idx in self.checked_items else ''
            self.tree.insert('', tk.END, iid=str(idx), values=('☐', f['name'], f['size_str'], f['mtime'], f['path']), tags=(tag,))
        self.tree.tag_configure('checked', background='#e6f7ff')
        self.status_var.set(f"扫描完成，共 {len(results)} 个文件")

    def _on_tree_click(self, event):
        region = self.tree.identify_region(event.x, event.y)
        if region != 'cell':
            return
        col = self.tree.identify_column(event.x)
        if col != '#1':  # not the 'select' column
            return
        row = self.tree.identify_row(event.y)
        if not row:
            return
        idx = int(row)
        if idx in self.checked_items:
            self.checked_items.discard(idx)
            self.tree.item(row, values=('☐', *self.tree.item(row, 'values')[1:]))
            self.tree.item(row, tags=('',))
        else:
            self.checked_items.add(idx)
            self.tree.item(row, values=('☑', *self.tree.item(row, 'values')[1:]))
            self.tree.item(row, tags=('checked',))

    def _select_all(self):
        for idx in range(len(self.files_data)):
            self.checked_items.add(idx)
            self.tree.item(str(idx), values=('☑', *self.tree.item(str(idx), 'values')[1:]), tags=('checked',))

    def _deselect_all(self):
        self.checked_items.clear()
        for idx in range(len(self.files_data)):
            self.tree.item(str(idx), values=('☐', *self.tree.item(str(idx), 'values')[1:]), tags=('',))

    def _delete_selected(self):
        if not self.checked_items:
            messagebox.showwarning("提示", "请先勾选要删除的文件")
            return

        files_to_delete = [self.files_data[i]['path'] for i in self.checked_items]
        total_size = sum(self.files_data[i]['size'] for i in self.checked_items)

        msg = f"确定要删除选中的 {len(files_to_delete)} 个文件吗？\n总计: {format_size(total_size)}\n\n"
        if HAS_SEND2TRASH:
            msg += "文件将被移到回收站。"
        else:
            msg += "⚠️ 将永久删除，无法恢复！"

        if not messagebox.askyesno("确认删除", msg):
            return

        success = 0
        failed = 0
        for path in files_to_delete:
            try:
                if HAS_SEND2TRASH:
                    send2trash(path)
                else:
                    os.remove(path)
                success += 1
            except Exception as e:
                failed += 1
                print(f"删除失败: {path} - {e}")

        messagebox.showinfo("完成", f"删除完成\n成功: {success}\n失败: {failed}")
        self._start_scan()  # rescan


def main():
    root = tk.Tk()
    app = FileCleanerApp(root)
    root.mainloop()


if __name__ == '__main__':
    main()
