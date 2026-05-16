// French Exit - Frontend Logic (Wizard Style)

let allResults = [];
let visibleResults = [];
let selectedPaths = new Set();
let sortCol = 'risk';
let sortAsc = true;
let lastBackupPath = null;
let lastOutputDir = null;

let customPaths = [];
let exemptPaths = [];
let extraExcludes = [];
let envReport = null;

// ====== Calendar State ======
let calendarYear = 0;
let calendarMonth = 0; // 0-11
let selectedDate = null;

// ====== Init ======
document.addEventListener('DOMContentLoaded', () => {
  if (typeof eel === 'undefined') {
    console.warn('eel not defined — running in preview/demo mode');
    enableDemoMode();
  }

  initCalendar();
  detectSystem();
  loadDrives();
  renderSuggestions();
  renderExcludeList();
  setupTagInput('custom-input', 'custom-list', customPaths);
  setupTagInput('exempt-input', 'exempt-list', exemptPaths);

  const home = getHomePath();
  document.getElementById('output-dir').value = home + (isWindows() ? '\\Desktop\\FrenchExit' : '/Desktop/FrenchExit');

  // Close calendar when clicking outside
  document.addEventListener('click', (e) => {
    const panel = document.getElementById('calendar-panel');
    const field = document.getElementById('calendar-field');
    if (panel && field && !panel.contains(e.target) && !field.contains(e.target)) {
      panel.classList.remove('show');
    }
  });
});

// ====== Demo Mode ======
let isDemo = false;

function enableDemoMode() {
  isDemo = true;
  const banner = document.createElement('div');
  banner.innerHTML = '演示模式：直接打开 HTML 文件预览界面。完整功能请运行 <code>python main.py</code>';
  banner.style.cssText = 'position:fixed;top:0;left:0;right:0;z-index:9999;background:var(--black);color:var(--white);text-align:center;padding:8px;font-size:12px;letter-spacing:0.02em;';
  document.body.appendChild(banner);

  window.eel = {
    py_get_drives: () => () => Promise.resolve(['C:\\', 'D:\\']),
    py_get_env_info: () => () => Promise.resolve({ has_send2trash: true, platform: 'win32' }),
    py_start_scan: (config) => () => { setTimeout(mockScan, 600); return Promise.resolve(); },
    py_stop_scan: () => {},
    py_execute_actions: (cfg) => () => Promise.resolve({
      success: true, take_success: 2, take_failed: 0, del_success: 1, del_failed: 0,
      backup_path: 'C:\\Users\\User\\AppData\\Local\\Temp\\french_exit_backup_20250101_120000.zip',
      log_path: 'C:\\Users\\User\\Desktop\\FrenchExit\\french_exit_log.txt',
      output_dir: 'C:\\Users\\User\\Desktop\\FrenchExit',
      exempt_paths: cfg.exempt_paths || [],
      errors: []
    }),
    py_open_folder: (path) => () => Promise.resolve(true),
    expose: () => {},
  };
}

function mockScan() {
  const demoData = [
    { path: 'C:\\Users\\User\\Pictures\\photo.jpg', name: 'photo.jpg', size: 2048000, size_str: '1.95 MB', mtime: '2024-06-15 14:32', mtime_ts: 1718458320, risk: '高', reason: '个人目录(pictures); 个人文件类型', action: 'ignore', _action: 'ignore' },
    { path: 'C:\\Users\\User\\Documents\\resume.pdf', name: 'resume.pdf', size: 512000, size_str: '500.00 KB', mtime: '2024-03-10 09:15', mtime_ts: 1710063300, risk: '高', reason: '个人目录(documents); 敏感文件名', action: 'ignore', _action: 'ignore' },
    { path: 'C:\\Users\\User\\Downloads\\setup.exe', name: 'setup.exe', size: 10485760, size_str: '10.00 MB', mtime: '2024-08-01 18:00', mtime_ts: 1722531600, risk: '低', reason: '可重下安装包', action: 'ignore', _action: 'ignore' },
    { path: 'C:\\Users\\User\\github\\my-project\\main.py', name: 'main.py', size: 4096, size_str: '4.00 KB', mtime: '2024-09-20 11:45', mtime_ts: 1726823100, risk: '中', reason: '疑似个人项目', action: 'ignore', _action: 'ignore' },
    { path: 'C:\\Users\\User\\Desktop\\notes.txt', name: 'notes.txt', size: 1024, size_str: '1.00 KB', mtime: '2024-10-05 08:22', mtime_ts: 1728102120, risk: '中', reason: '个人文件类型', action: 'ignore', _action: 'ignore' },
  ];
  js_on_scan_done(demoData);
}

// ====== Layer Navigation ======
const LAYERS = [
  'layer-welcome',
  'layer-env',
  'layer-date',
  'layer-drives',
  'layer-suggestions',
  'layer-exclude',
  'layer-exempt',
  'layer-scanning',
  'layer-results',
  'layer-complete'
];

function showLayer(name) {
  LAYERS.forEach(id => {
    const el = document.getElementById(id);
    if (!el) return;
    if (id === name) {
      el.classList.add('active');
    } else {
      el.classList.remove('active');
    }
  });
}

function goToLayer(name) {
  showLayer(name);
}

// ====== Calendar ======
function initCalendar() {
  const now = new Date();
  const sixMonthsAgo = new Date(now.getFullYear(), now.getMonth() - 6, now.getDate());
  calendarYear = sixMonthsAgo.getFullYear();
  calendarMonth = sixMonthsAgo.getMonth();
  selectedDate = new Date(calendarYear, calendarMonth, sixMonthsAgo.getDate());
  updateDateDisplay();
  renderCalendar();
}

function updateDateDisplay() {
  const el = document.getElementById('date-display');
  if (el && selectedDate) {
    const y = selectedDate.getFullYear();
    const m = String(selectedDate.getMonth() + 1).padStart(2, '0');
    const d = String(selectedDate.getDate()).padStart(2, '0');
    el.textContent = `${y}-${m}-${d}`;
  }
}

function toggleCalendar() {
  const panel = document.getElementById('calendar-panel');
  panel.classList.toggle('show');
}

function changeMonth(delta) {
  calendarMonth += delta;
  if (calendarMonth > 11) {
    calendarMonth = 0;
    calendarYear++;
  } else if (calendarMonth < 0) {
    calendarMonth = 11;
    calendarYear--;
  }
  renderCalendar();
}

function renderCalendar() {
  const monthYearEl = document.getElementById('calendar-month-year');
  const daysEl = document.getElementById('calendar-days');
  if (!monthYearEl || !daysEl) return;

  monthYearEl.textContent = `${calendarYear}年 ${calendarMonth + 1}月`;

  const firstDay = new Date(calendarYear, calendarMonth, 1).getDay();
  const daysInMonth = new Date(calendarYear, calendarMonth + 1, 0).getDate();
  const daysInPrevMonth = new Date(calendarYear, calendarMonth, 0).getDate();

  let html = '';

  // Previous month trailing days
  for (let i = firstDay - 1; i >= 0; i--) {
    const day = daysInPrevMonth - i;
    html += `<button class="calendar-day other-month disabled">${day}</button>`;
  }

  // Current month days
  const today = new Date();
  for (let day = 1; day <= daysInMonth; day++) {
    const isToday = day === today.getDate() && calendarMonth === today.getMonth() && calendarYear === today.getFullYear();
    const isSelected = selectedDate && day === selectedDate.getDate() && calendarMonth === selectedDate.getMonth() && calendarYear === selectedDate.getFullYear();
    let cls = 'calendar-day';
    if (isSelected) cls += ' selected';
    else if (isToday) cls += ' today';
    html += `<button class="${cls}" onclick="selectDate(${day})">${day}</button>`;
  }

  // Next month leading days
  const totalCells = firstDay + daysInMonth;
  const remaining = (7 - (totalCells % 7)) % 7;
  for (let day = 1; day <= remaining; day++) {
    html += `<button class="calendar-day other-month disabled">${day}</button>`;
  }

  daysEl.innerHTML = html;
}

function selectDate(day) {
  selectedDate = new Date(calendarYear, calendarMonth, day);
  updateDateDisplay();
  renderCalendar();
  document.getElementById('calendar-panel').classList.remove('show');
}

function getSelectedDate() {
  if (!selectedDate) return '';
  const y = selectedDate.getFullYear();
  const m = String(selectedDate.getMonth() + 1).padStart(2, '0');
  const d = String(selectedDate.getDate()).padStart(2, '0');
  return `${y}-${m}-${d}`;
}

// ====== System & Path Helpers ======
function isWindows() {
  return navigator.platform.indexOf('Win') > -1 || navigator.userAgent.indexOf('Windows') > -1;
}
function isMac() {
  return navigator.platform.indexOf('Mac') > -1;
}
function getHomePath() {
  if (isWindows()) return 'C:\\Users\\User';
  if (isMac()) return '/Users/user';
  return '/home/user';
}
function getSystemName() {
  if (isWindows()) return 'Windows';
  if (isMac()) return 'macOS';
  return 'Linux';
}

function detectSystem() {
  const label = document.getElementById('drives-subtitle');
  if (label) {
    label.textContent = `已识别为 ${getSystemName()} 系统，选择要扫描的磁盘`;
  }
}

// ====== Environment Check ======
async function runEnvCheck() {
  const badge = document.getElementById('env-badge');
  const list = document.getElementById('env-list');
  const actions = document.getElementById('env-actions');
  const continueBtn = document.getElementById('btn-env-continue');

  badge.textContent = '检测中...';
  badge.style.color = 'var(--gray)';
  list.innerHTML = '<div>正在检查 Python 环境...</div>';
  actions.classList.add('hidden');
  continueBtn.classList.add('hidden');

  try {
    await sleep(200);
    list.innerHTML = '<div>Python 版本检查通过</div>';
    await sleep(200);
    list.innerHTML += '<div>检查 eel...</div>';
    await sleep(200);
    list.innerHTML += '<div>检查 send2trash...</div>';

    const info = await eel.py_get_env_info()();
    envReport = info;

    const items = [];
    if (info.has_send2trash) {
      items.push('send2trash 已安装 — 删除将移入回收站');
      badge.textContent = '正常';
      badge.style.color = 'var(--black)';
      continueBtn.classList.remove('hidden');
    } else {
      items.push('send2trash 未安装 — 删除将永久删除');
      badge.textContent = '需修复';
      badge.style.color = 'var(--danger)';
      actions.classList.remove('hidden');
    }

    list.innerHTML = items.map(s => `<div>${escapeHtml(s)}</div>`).join('');
  } catch (e) {
    list.innerHTML = `<div style="color:var(--danger)">检测失败: ${escapeHtml(String(e))}</div>`;
    badge.textContent = '错误';
    badge.style.color = 'var(--danger)';
  }
}

async function fixEnv() {
  alert('请在本程序所在目录的终端中运行:\npip install -r requirements.txt\n\n安装完成后重新启动本程序。');
}

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// ====== Tag Input ======
function setupTagInput(inputId, listId, arr) {
  const input = document.getElementById(inputId);
  const list = document.getElementById(listId);
  if (!input || !list) return;

  function render() {
    list.innerHTML = arr.map((t, i) =>
      `<span class="tag-chip">${escapeHtml(t)} <span class="remove" onclick="removeTag('${inputId}', ${i})">&times;</span></span>`
    ).join('');
  }

  input.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      const val = input.value.trim();
      if (val && !arr.includes(val)) {
        arr.push(val);
        input.value = '';
        render();
      }
    }
  });

  render();
  window['_render_' + inputId] = render;
}

function removeTag(inputId, idx) {
  let arr;
  if (inputId === 'custom-input') arr = customPaths;
  else if (inputId === 'exempt-input') arr = exemptPaths;
  if (arr) {
    arr.splice(idx, 1);
    window['_render_' + inputId] && window['_render_' + inputId]();
  }
}

// ====== Drives ======
async function loadDrives() {
  try {
    const drives = await eel.py_get_drives()();
    const container = document.getElementById('drive-list');
    if (!drives || drives.length === 0) {
      container.innerHTML = '<span class="text-muted">未检测到可用磁盘</span>';
      return;
    }
    container.innerHTML = drives.map(d => {
      const letter = d.replace('\\', '').replace('/', '');
      return `
      <label class="drive-item" onclick="toggleDrive(this)">
        <input type="checkbox" value="${escapeHtml(d)}" style="display:none;">
        <div class="drive-icon">${escapeHtml(letter)}</div>
        <div class="drive-label">${escapeHtml(d)}</div>
      </label>`;
    }).join('');
  } catch (e) {
    console.error('Load drives failed', e);
  }
}

function toggleDrive(el) {
  el.classList.toggle('selected');
  const cb = el.querySelector('input');
  if (cb) cb.checked = el.classList.contains('selected');
}

// ====== Suggestion Paths + Tooltip ======
const SUGGESTIONS = [
  { name: 'Desktop', path: 'Desktop', tooltip: '桌面通常存放临时文件、截图、快捷方式等个人内容' },
  { name: 'Documents', path: 'Documents', tooltip: '文档、简历、合同、个人资料集中存放的位置' },
  { name: 'Downloads', path: 'Downloads', tooltip: '浏览器下载的所有文件，最易积累个人内容' },
  { name: 'Pictures', path: 'Pictures', tooltip: '照片、截图、设计稿等图像类个人文件' },
  { name: 'Home', path: '', tooltip: '用户主目录总入口，涵盖桌面、文档、下载等全部子目录' },
];

function renderSuggestions() {
  const container = document.getElementById('suggestion-row');
  const home = getHomePath();
  const sep = isWindows() ? '\\' : '/';

  container.innerHTML = SUGGESTIONS.map(s => {
    const fullPath = s.path ? home + sep + s.path : home;
    return `
      <button class="suggestion-btn" onclick="addSuggestion('${escapeHtml(fullPath)}')">
        <span class="tooltip">${escapeHtml(s.tooltip)}</span>
        <span class="plus">+</span>${escapeHtml(s.name)}
      </button>`;
  }).join('');
}

function addSuggestion(path) {
  if (!customPaths.includes(path)) {
    customPaths.push(path);
    window['_render_custom-input'] && window['_render_custom-input']();
  }
}

// ====== Exclude List ======
const EXCLUDE_PRESETS = [
  { name: 'node_modules', reason: '前端依赖目录，体积通常很大，且非个人文件', checked: true },
  { name: '__pycache__', reason: 'Python 运行缓存，删除后可自动重建', checked: true },
  { name: '.git', reason: 'Git 版本控制数据，不是个人创作文件', checked: true },
  { name: 'venv', reason: 'Python 虚拟环境，可通过 requirements.txt 重新创建', checked: true },
  { name: '.npm', reason: 'npm 缓存目录，可安全删除', checked: true },
];

function renderExcludeList() {
  const container = document.getElementById('exclude-list');
  if (!container) return;
  container.innerHTML = EXCLUDE_PRESETS.map((item, idx) => `
    <label class="exclude-item">
      <input type="checkbox" ${item.checked ? 'checked' : ''} onchange="togglePresetExclude(${idx}, this.checked)">
      <div class="exclude-info">
        <div class="exclude-name">${escapeHtml(item.name)}</div>
        <div class="exclude-reason">${escapeHtml(item.reason)}</div>
      </div>
    </label>
  `).join('');

  setupExtraExcludeInput();
  syncExcludeArray();
}

function togglePresetExclude(idx, checked) {
  EXCLUDE_PRESETS[idx].checked = checked;
  syncExcludeArray();
}

function setupExtraExcludeInput() {
  const input = document.getElementById('exclude-input');
  const list = document.getElementById('exclude-extra-list');
  if (!input || !list) return;

  function render() {
    list.innerHTML = extraExcludes.map((t, i) =>
      `<span class="tag-chip">${escapeHtml(t)} <span class="remove" onclick="removeExtraExclude(${i})">&times;</span></span>`
    ).join('');
  }

  input.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      const val = input.value.trim();
      if (val && !extraExcludes.includes(val) && !EXCLUDE_PRESETS.some(p => p.name === val)) {
        extraExcludes.push(val);
        input.value = '';
        render();
        syncExcludeArray();
      }
    }
  });

  window._render_extra_exclude = render;
}

function removeExtraExclude(idx) {
  extraExcludes.splice(idx, 1);
  window._render_extra_exclude && window._render_extra_exclude();
  syncExcludeArray();
}

function syncExcludeArray() {
  // This is used by startScan
}

function getExcludeDirs() {
  return [
    ...EXCLUDE_PRESETS.filter(p => p.checked).map(p => p.name),
    ...extraExcludes
  ];
}

// ====== Scan ======
async function startScan() {
  const date = getSelectedDate();
  if (!date) { alert('请选择入职日期'); return; }

  const drives = Array.from(document.querySelectorAll('#drive-list input:checked')).map(cb => cb.value);

  const config = {
    start_date: date,
    drives: drives,
    custom_paths: customPaths,
    exclude_dirs: getExcludeDirs(),
    exempt_paths: exemptPaths,
  };

  showLayer('layer-scanning');
  document.getElementById('scan-fill').style.width = '0%';
  document.getElementById('scan-sub').textContent = '正在初始化扫描...';

  try {
    await eel.py_start_scan(config)();
  } catch (e) {
    alert('启动扫描失败: ' + e);
    showLayer('layer-exempt');
  }
}

function stopScan() {
  eel.py_stop_scan();
  document.getElementById('scan-sub').textContent = '正在停止...';
}

// Callbacks from Python
function js_on_scan_progress(scanned, currentName, found) {
  const fill = document.getElementById('scan-fill');
  fill.style.width = '100%';
  fill.style.opacity = (fill.style.opacity === '1') ? '0.5' : '1';
  document.getElementById('scan-sub').textContent =
    `已扫描 ${scanned} 个目录项，发现 ${found} 个文件`;
  document.getElementById('scan-meta').textContent = currentName ? `当前: ${currentName}` : '';
}

function js_on_scan_done(results) {
  allResults = results || [];
  allResults.forEach(r => { r._action = r.action || 'ignore'; });
  visibleResults = [...allResults];
  selectedPaths.clear();

  showLayer('layer-results');
  updateStats();
  renderTable();
}

function js_on_scan_error(msg) {
  alert('扫描出错: ' + msg);
  showLayer('layer-exempt');
}

eel.expose(js_on_scan_progress);
eel.expose(js_on_scan_done);
eel.expose(js_on_scan_error);

// ====== Results Table ======
function updateStats() {
  const high = allResults.filter(r => r.risk === '高').length;
  const med = allResults.filter(r => r.risk === '中').length;
  const low = allResults.filter(r => r.risk === '低').length;
  const totalSize = allResults.reduce((s, r) => s + (r.size || 0), 0);

  document.getElementById('stats-row').innerHTML = `
    <div class="stat-box"><div class="stat-num">${allResults.length}</div><div class="stat-label">个文件</div></div>
    <div class="stat-box"><div class="stat-num" style="color:var(--black)">${high}</div><div class="stat-label">高风险</div></div>
    <div class="stat-box"><div class="stat-num" style="color:var(--gray)">${med}</div><div class="stat-label">中风险</div></div>
    <div class="stat-box"><div class="stat-num" style="color:var(--light-gray)">${low}</div><div class="stat-label">低风险</div></div>
    <div class="stat-box"><div class="stat-num">${formatSize(totalSize)}</div><div class="stat-label">总计</div></div>
  `;
  document.getElementById('results-sub').textContent = `共发现 ${allResults.length} 个文件`;
}

function formatSize(bytes) {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024*1024) return (bytes/1024).toFixed(2) + ' KB';
  if (bytes < 1024*1024*1024) return (bytes/1024/1024).toFixed(2) + ' MB';
  return (bytes/1024/1024/1024).toFixed(2) + ' GB';
}

function renderTable() {
  const tbody = document.getElementById('results-tbody');
  if (visibleResults.length === 0) {
    tbody.innerHTML = '<tr><td colspan="8" style="text-align:center;padding:48px;color:var(--gray);font-size:14px;">没有匹配的文件</td></tr>';
    return;
  }

  const actionOptions = {
    'ignore': '忽略',
    'take': '带走',
    'delete': '删除',
  };

  tbody.innerHTML = visibleResults.map(r => {
    const isSel = selectedPaths.has(r.path);
    const riskClass = r.risk === '高' ? 'high-risk' : (r.risk === '中' ? 'med-risk' : '');
    return `
      <tr class="${riskClass} ${isSel ? 'selected' : ''}" data-path="${escapeHtml(r.path)}">
        <td><input type="checkbox" ${isSel ? 'checked' : ''} onchange="toggleRow('${escapeHtml(r.path)}', this.checked)"></td>
        <td class="risk">${escapeHtml(r.risk)}</td>
        <td>${escapeHtml(r.name)}</td>
        <td class="size">${escapeHtml(r.size_str)}</td>
        <td>${escapeHtml(r.mtime)}</td>
        <td class="action">
          <select onchange="setAction('${escapeHtml(r.path)}', this.value)">
            ${Object.entries(actionOptions).map(([k,v]) => `<option value="${k}" ${r._action===k?'selected':''}>${v}</option>`).join('')}
          </select>
        </td>
        <td>${escapeHtml(r.reason)}</td>
        <td class="path" title="${escapeHtml(r.path)}">${escapeHtml(r.path)}</td>
      </tr>
    `;
  }).join('');
}

function toggleRow(path, checked) {
  if (checked) selectedPaths.add(path);
  else selectedPaths.delete(path);
  const row = document.querySelector(`tr[data-path="${CSS.escape(path)}"]`);
  if (row) row.classList.toggle('selected', checked);
}

function toggleAll(checkbox) {
  if (checkbox.checked) visibleResults.forEach(r => selectedPaths.add(r.path));
  else visibleResults.forEach(r => selectedPaths.delete(r.path));
  renderTable();
}

function setAction(path, action) {
  const r = allResults.find(x => x.path === path);
  if (r) r._action = action;
}

function selectAllVisible() {
  visibleResults.forEach(r => selectedPaths.add(r.path));
  renderTable();
}

function selectHighRisk() {
  visibleResults.forEach(r => {
    if (r.risk === '高') selectedPaths.add(r.path);
    else selectedPaths.delete(r.path);
  });
  renderTable();
}

function clearSelection() {
  selectedPaths.clear();
  renderTable();
}

function batchAction(action) {
  if (selectedPaths.size === 0) { alert('请先勾选要操作的文件'); return; }
  allResults.forEach(r => { if (selectedPaths.has(r.path)) r._action = action; });
  renderTable();
}

// ====== Filter & Sort ======
function filterResults() {
  const query = document.getElementById('search-input').value.toLowerCase();
  const riskFilter = document.getElementById('filter-risk').value;

  visibleResults = allResults.filter(r => {
    const matchText = !query || r.name.toLowerCase().includes(query) || r.path.toLowerCase().includes(query);
    const matchRisk = !riskFilter || r.risk === riskFilter;
    return matchText && matchRisk;
  });
  doSort();
  renderTable();
}

function sortTable(col) {
  if (sortCol === col) sortAsc = !sortAsc;
  else { sortCol = col; sortAsc = true; }
  doSort();
  renderTable();
}

function doSort() {
  const riskOrder = { '高': 0, '中': 1, '低': 2 };
  visibleResults.sort((a, b) => {
    let va, vb;
    if (sortCol === 'risk') { va = riskOrder[a.risk] ?? 3; vb = riskOrder[b.risk] ?? 3; }
    else if (sortCol === 'size') { va = a.size; vb = b.size; }
    else if (sortCol === 'mtime') { va = a.mtime_ts; vb = b.mtime_ts; }
    else { va = a[sortCol] || ''; vb = b[sortCol] || ''; }
    if (va < vb) return sortAsc ? -1 : 1;
    if (va > vb) return sortAsc ? 1 : -1;
    return 0;
  });
}

// ====== Execute ======
function showConfirm() {
  const take = allResults.filter(r => r._action === 'take');
  const del = allResults.filter(r => r._action === 'delete');
  const ign = allResults.filter(r => r._action === 'ignore');

  if (take.length === 0 && del.length === 0) {
    alert('没有标记任何操作。请先勾选文件并标记「带走」或「删除」。');
    return;
  }

  const outDir = document.getElementById('output-dir').value.trim() || 'Desktop\\FrenchExit';
  const mode = document.getElementById('take-mode').value;

  let html = '';
  if (take.length > 0) {
    const takeSize = take.reduce((s, r) => s + r.size, 0);
    html += `<strong>带走</strong>: ${take.length} 个文件（${formatSize(takeSize)}）<br>方式: ${mode === 'zip' ? '打包成 ZIP' : '原样复制'}<br>输出目录: <code>${escapeHtml(outDir)}</code><br>`;
  }
  if (del.length > 0) {
    const delSize = del.reduce((s, r) => s + r.size, 0);
    html += `<strong>删除</strong>: ${del.length} 个文件（${formatSize(delSize)}）<br>删除前将自动打包备份到临时目录<br>`;
  }
  html += `<strong>忽略</strong>: ${ign.length} 个文件<br>`;
  if (exemptPaths.length > 0) {
    html += `<strong>豁免路径</strong>: ${exemptPaths.map(escapeHtml).join('、')}<br>`;
  }

  document.getElementById('confirm-body').innerHTML = html;
  document.getElementById('confirm-modal').classList.add('show');
}

function hideConfirm() {
  document.getElementById('confirm-modal').classList.remove('show');
}

async function executeActions() {
  hideConfirm();

  const takeFiles = allResults.filter(r => r._action === 'take').map(r => r.path);
  const deleteFiles = allResults.filter(r => r._action === 'delete').map(r => r.path);
  const outDir = document.getElementById('output-dir').value.trim();
  const mode = document.getElementById('take-mode').value;

  const config = {
    take_files: takeFiles,
    delete_files: deleteFiles,
    output_dir: outDir,
    take_mode: mode,
    exempt_paths: exemptPaths,
  };

  showLayer('layer-scanning');
  document.getElementById('scan-sub').textContent = '正在执行操作，请稍候...';

  try {
    const result = await eel.py_execute_actions(config)();
    lastBackupPath = result.backup_path;
    lastOutputDir = result.output_dir;
    showComplete(result);
  } catch (e) {
    alert('执行失败: ' + e);
    showLayer('layer-results');
  }
}

// ====== Complete ======
function showComplete(result) {
  showLayer('layer-complete');

  let html = '';
  html += `<div>带走: 成功 <strong>${result.take_success}</strong>，失败 <strong>${result.take_failed}</strong></div>`;
  html += `<div>删除: 成功 <strong>${result.del_success}</strong>，失败 <strong>${result.del_failed}</strong></div>`;
  if (result.backup_path) {
    html += `<div>删除前备份: <code>${escapeHtml(result.backup_path)}</code></div>`;
  }
  if (result.log_path) {
    html += `<div>操作日志: <code>${escapeHtml(result.log_path)}</code></div>`;
  }
  if (result.exempt_paths && result.exempt_paths.length > 0) {
    html += `<div>豁免路径: ${result.exempt_paths.map(escapeHtml).join('、')}</div>`;
  }
  if (result.errors && result.errors.length > 0) {
    html += `<div style="color:var(--danger);margin-top:8px;">错误信息:<br>${result.errors.map(escapeHtml).join('<br>')}</div>`;
  }

  document.getElementById('complete-detail').innerHTML = html;
  document.getElementById('btn-open-output').style.display = result.output_dir ? '' : 'none';
  document.getElementById('btn-open-backup').style.display = result.backup_path ? '' : 'none';
}

async function openOutput() {
  if (lastOutputDir) await eel.py_open_folder(lastOutputDir)();
}

async function openBackup() {
  if (lastBackupPath) {
    const dir = lastBackupPath.replace(/\\/g, '/').split('/').slice(0, -1).join('/');
    await eel.py_open_folder(dir)();
  }
}

// ====== Utilities ======
function escapeHtml(text) {
  if (!text) return '';
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}
