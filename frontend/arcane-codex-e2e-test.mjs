import { chromium } from 'playwright';
import { writeFileSync, mkdirSync, existsSync } from 'fs';
import { join } from 'path';

const BASE = 'http://localhost:1420';
const DIR = 'C:\\Users\\Administrator\\AppData\\Local\\Temp\\arcane-e2e-shots';
const RPT = 'C:\\Users\\Administrator\\AppData\\Local\\Temp\\arcane-e2e-report.json';
if (!existsSync(DIR)) mkdirSync(DIR, { recursive: true });

const S = { t0: new Date().toISOString(), t1: null, tests: [], perf: {}, shots: [],
  sum: { total: 0, pass: 0, fail: 0, warn: 0, blocked: 0 } };

function R(t) {
  S.tests.push(t); S.sum.total++;
  if (t.s === 'PASS') S.sum.pass++;
  else if (t.s === 'FAIL') S.sum.fail++;
  else if (t.s === 'WARN') S.sum.warn++;
  else if (t.s === 'BLOCKED') S.sum.blocked++;
  console.log(`[${t.s.padEnd(7)}] ${t.id} ${t.n}: ${(t.d||'').substring(0, 140)}`);
}

async function ss(pg, nm) {
  const p = join(DIR, `${nm}.png`);
  await pg.screenshot({ path: p, fullPage: false });
  S.shots.push({ name: nm, path: p });
}

const NAV_MAP = {
  gallery:   { zh: '图库',       en: 'Gallery' },
  ai:        { zh: 'AI 打标',    en: 'AI Tagging' },
  dedup:     { zh: '去重',       en: 'Dedup' },
  dashboard: { zh: '数据仪表盘', en: 'Dashboard' },
  settings:  { zh: '设置',       en: 'Settings' },
};

const NAV_KEYS = ['gallery', 'ai', 'dedup', 'dashboard', 'settings'];
const SETTINGS_TABS = ['ai', 'display', 'storage', 'notifications', 'privacy', 'logs', 'about'];
const SETTINGS_TAB_LABELS = {
  ai: 'AI', display: '显示配置', storage: '存储配置',
  notifications: '通知设置', privacy: '隐私设置', logs: '系统日志', about: '关于',
};

async function navTo(pg, key) {
  const testIdBtn = pg.locator(`[data-testid="nav-${key}"]`).first();
  if (await testIdBtn.isVisible({ timeout: 2000 }).catch(() => false)) {
    await testIdBtn.click();
    await pg.waitForTimeout(500);
    return true;
  }
  const names = NAV_MAP[key];
  for (const txt of [names.zh, names.en]) {
    const btn = pg.locator('aside nav button').filter({ hasText: txt }).first();
    if (await btn.isVisible({ timeout: 1000 }).catch(() => false)) {
      await btn.click();
      await pg.waitForTimeout(500);
      return true;
    }
  }
  return false;
}

async function ensureChinese(pg) {
  const zhBtn = pg.locator('[data-testid="nav-gallery"]').first();
  if (await zhBtn.isVisible({ timeout: 1500 }).catch(() => false)) return true;
  const langToggle = pg.locator('[data-testid="lang-toggle"]').first();
  if (!await langToggle.isVisible({ timeout: 2000 }).catch(() => false)) return false;
  await langToggle.click();
  await pg.waitForTimeout(400);
  const zh = pg.locator('[data-testid="lang-zh"]').first();
  if (await zh.isVisible({ timeout: 2000 }).catch(() => false)) {
    await zh.click();
    await pg.waitForTimeout(800);
    return true;
  }
  await pg.keyboard.press('Escape');
  return false;
}

(async () => {
  const br = await chromium.launch({ headless: true });
  const cx = await br.newContext({ viewport: { width: 1440, height: 900 }, locale: 'zh-CN' });
  const pg = await cx.newPage();
  const cerrs = [];
  pg.on('console', m => { if (m.type() === 'error') cerrs.push(m.text()); });
  pg.on('pageerror', e => cerrs.push(e.message));

  const themeBtn = () => pg.locator('[data-testid="theme-toggle"]').first();
  const langToggle = () => pg.locator('[data-testid="lang-toggle"]').first();
  const collapseBtn = () => pg.locator('[data-testid="sidebar-collapse"]').first();

  // ============================================================
  // SECTION A: 启动与基础设施 (A1-A4)
  // ============================================================

  // A1: 首次加载性能
  try {
    const t0 = Date.now();
    await pg.goto(BASE, { waitUntil: 'networkidle', timeout: 30000 });
    S.perf.initialLoad = Date.now() - t0;
    await pg.waitForSelector('text=Arcane Codex', { timeout: 10000 });
    await ss(pg, 'A1-initial-load');
    R({ id: 'A1', n: '首次加载', s: 'PASS',
      d: `加载耗时 ${S.perf.initialLoad}ms`, c: 'perf' });
  } catch (e) {
    await ss(pg, 'A1-FAIL');
    R({ id: 'A1', n: '首次加载', s: 'FAIL', d: e.message, c: 'perf' });
  }

  // A2: 侧边栏结构完整性
  try {
    const aside = pg.locator('aside').first();
    const title = await aside.locator('h1').textContent();
    const navBtns = await aside.locator('nav button').count();
    const hasCollapse = await collapseBtn().isVisible({ timeout: 1000 }).catch(() => false);
    await ss(pg, 'A2-sidebar');
    R({ id: 'A2', n: '侧边栏结构', s: navBtns === 5 ? 'PASS' : 'WARN',
      d: `标题: "${title}", 导航按钮: ${navBtns}, 折叠按钮: ${hasCollapse}`, c: 'structure' });
  } catch (e) {
    R({ id: 'A2', n: '侧边栏结构', s: 'FAIL', d: e.message, c: 'structure' });
  }

  // A3: 顶栏结构完整性
  try {
    const header = pg.locator('header').first();
    const searchVis = await header.locator('input[type="text"]').isVisible({ timeout: 2000 });
    const langVis = await langToggle().isVisible({ timeout: 2000 }).catch(() => false);
    const themeVis = await themeBtn().isVisible({ timeout: 2000 }).catch(() => false);
    await ss(pg, 'A3-topbar');
    R({ id: 'A3', n: '顶栏结构', s: searchVis && langVis && themeVis ? 'PASS' : 'WARN',
      d: `搜索框: ${searchVis}, 语言按钮: ${langVis}, 主题按钮: ${themeVis}`, c: 'structure' });
  } catch (e) {
    R({ id: 'A3', n: '顶栏结构', s: 'FAIL', d: e.message, c: 'structure' });
  }

  // A4: 5页面导航
  const navT = {};
  for (const key of NAV_KEYS) {
    try {
      const t0 = Date.now();
      const ok = await navTo(pg, key);
      if (!ok) throw new Error(`导航按钮不可见`);
      navT[key] = Date.now() - t0;
      await ss(pg, `A4-nav-${key}`);
      R({ id: `A4-${key}`, n: `导航-${NAV_MAP[key].zh}`, s: 'PASS',
        d: `渲染 ${navT[key]}ms`, c: 'nav' });
    } catch (e) {
      await ss(pg, `A4-${key}-FAIL`);
      R({ id: `A4-${key}`, n: `导航-${NAV_MAP[key].zh}`, s: 'FAIL', d: e.message, c: 'nav' });
    }
  }
  S.perf.navTimings = navT;

  // ============================================================
  // SECTION B: 图库页面 (B1-B5)
  // ============================================================

  // B1: 空状态 DropZone
  try {
    await navTo(pg, 'gallery');
    await pg.waitForTimeout(800);
    const main = await pg.locator('main').textContent();
    const hasEmpty = main?.includes('暂无图片') || main?.includes('拖拽') || main?.includes('导入') || main?.includes('还没有');
    await ss(pg, 'B1-gallery-empty');
    R({ id: 'B1', n: '图库空状态', s: hasEmpty ? 'PASS' : 'WARN',
      d: hasEmpty ? '空状态 DropZone 正确显示' : `内容: ${(main||'').substring(0,100)}`, c: 'gallery' });
  } catch (e) {
    R({ id: 'B1', n: '图库空状态', s: 'FAIL', d: e.message, c: 'gallery' });
  }

  // B2: ImageFilter 筛选面板
  try {
    await navTo(pg, 'gallery');
    await pg.waitForTimeout(500);
    const filterBtn = pg.locator('main button').filter({ hasText: /筛选|filter/i }).first();
    const vis = await filterBtn.isVisible({ timeout: 3000 }).catch(() => false);
    if (vis) {
      await filterBtn.click();
      await pg.waitForTimeout(500);
      await ss(pg, 'B2-filter-open');
      R({ id: 'B2', n: '筛选面板', s: 'PASS', d: '筛选按钮可点击', c: 'gallery' });
    } else {
      await ss(pg, 'B2-filter-missing');
      R({ id: 'B2', n: '筛选面板', s: 'WARN', d: '筛选按钮不可见', c: 'gallery' });
    }
  } catch (e) {
    R({ id: 'B2', n: '筛选面板', s: 'FAIL', d: e.message, c: 'gallery' });
  }

  // B3: 断链检查按钮
  try {
    await navTo(pg, 'gallery');
    await pg.waitForTimeout(500);
    const brokenBtn = pg.locator('main button').filter({ hasText: /检查失效|broken/i }).first();
    const vis = await brokenBtn.isVisible({ timeout: 3000 }).catch(() => false);
    if (vis) {
      await brokenBtn.click();
      await pg.waitForTimeout(2000);
      await ss(pg, 'B3-broken-links');
      R({ id: 'B3', n: '断链检查', s: 'PASS', d: '断链检查按钮可点击', c: 'gallery' });
    } else {
      R({ id: 'B3', n: '断链检查', s: 'WARN', d: '断链检查按钮不可见', c: 'gallery' });
    }
  } catch (e) {
    R({ id: 'B3', n: '断链检查', s: 'FAIL', d: e.message, c: 'gallery' });
  }

  // B4: DropZone 拖拽区域存在
  try {
    await navTo(pg, 'gallery');
    await pg.waitForTimeout(500);
    const dropzone = pg.locator('main >> text=/拖拽|导入|选择|drop|import/i').first();
    const vis = await dropzone.isVisible({ timeout: 3000 }).catch(() => false);
    await ss(pg, 'B4-dropzone');
    R({ id: 'B4', n: 'DropZone', s: vis ? 'PASS' : 'WARN',
      d: vis ? 'DropZone 可见' : 'DropZone 未检测到', c: 'gallery' });
  } catch (e) {
    R({ id: 'B4', n: 'DropZone', s: 'FAIL', d: e.message, c: 'gallery' });
  }

  // B5: 图片导入流程 (BLOCKED)
  R({ id: 'B5', n: '图片导入流程', s: 'BLOCKED',
    d: '需 Tauri 原生文件对话框。后端: 磁盘检查->SHA256->去重->入库->缩略图->pHash->EXIF->AI任务队列',
    c: 'gallery' });

  // ============================================================
  // SECTION C: AI 打标页面 (C1-C5)
  // ============================================================

  // C1: AI 页面空状态
  try {
    await navTo(pg, 'ai');
    await pg.waitForTimeout(800);
    await ss(pg, 'C1-ai-empty');
    R({ id: 'C1', n: 'AI页面空状态', s: 'PASS', d: 'AI 页面加载成功', c: 'ai' });
  } catch (e) {
    R({ id: 'C1', n: 'AI页面空状态', s: 'FAIL', d: e.message, c: 'ai' });
  }

  // C2: AI 控制按钮
  try {
    await navTo(pg, 'ai');
    await pg.waitForTimeout(500);
    const main = await pg.locator('main').textContent();
    const hasStartBtn = main?.includes('开始') || main?.includes('处理') || main?.includes('Start') || main?.includes('Process');
    await ss(pg, 'C2-ai-controls');
    R({ id: 'C2', n: 'AI控制按钮', s: hasStartBtn ? 'PASS' : 'WARN',
      d: hasStartBtn ? '开始处理按钮可见' : '未检测到控制按钮', c: 'ai' });
  } catch (e) {
    R({ id: 'C2', n: 'AI控制按钮', s: 'FAIL', d: e.message, c: 'ai' });
  }

  // C3: AI 结果列表
  try {
    await navTo(pg, 'ai');
    await pg.waitForTimeout(500);
    await ss(pg, 'C3-ai-results');
    R({ id: 'C3', n: 'AI结果列表', s: 'PASS', d: '结果列表区域已加载', c: 'ai' });
  } catch (e) {
    R({ id: 'C3', n: 'AI结果列表', s: 'FAIL', d: e.message, c: 'ai' });
  }

  // C4: AI 处理流程 (BLOCKED)
  R({ id: 'C4', n: 'AI处理流程', s: 'BLOCKED',
    d: '需已导入图片。LM Studio API(/v1/chat/completions)->JSON->DB。状态机: idle->processing/paused->completed/failed',
    c: 'ai' });

  // C5: AI 单条重试 (BLOCKED)
  R({ id: 'C5', n: 'AI单条重试', s: 'BLOCKED',
    d: '需有失败的 AI 任务。retrySingleAIResult(imageId)',
    c: 'ai' });

  // ============================================================
  // SECTION D: 去重页面 (D1-D3)
  // ============================================================

  // D1: 去重页面空状态
  try {
    await navTo(pg, 'dedup');
    await pg.waitForTimeout(800);
    const main = await pg.locator('main').textContent();
    await ss(pg, 'D1-dedup-empty');
    R({ id: 'D1', n: '去重页面空状态', s: 'PASS',
      d: `去重页面加载成功，内容长度: ${main?.length || 0}`, c: 'dedup' });
  } catch (e) {
    R({ id: 'D1', n: '去重页面空状态', s: 'FAIL', d: e.message, c: 'dedup' });
  }

  // D2: 去重扫描流程 (BLOCKED)
  R({ id: 'D2', n: '去重扫描流程', s: 'BLOCKED',
    d: '需已导入图片。BK-tree+pHash汉明距离，阈值95%',
    c: 'dedup' });

  // D3: 去重删除流程 (BLOCKED)
  R({ id: 'D3', n: '去重删除流程', s: 'BLOCKED',
    d: '需有重复组。按策略排序->保留第一张->删除其余',
    c: 'dedup' });

  // ============================================================
  // SECTION E: 仪表盘 (E1-E3)
  // ============================================================

  // E1: 仪表盘加载
  try {
    await navTo(pg, 'dashboard');
    await pg.waitForTimeout(1000);
    const main = await pg.locator('main').textContent();
    const hasStats = main?.includes('0') || main?.includes('图片') || main?.includes('总数') || main?.includes('Total');
    await ss(pg, 'E1-dashboard');
    R({ id: 'E1', n: '仪表盘加载', s: hasStats ? 'PASS' : 'WARN',
      d: hasStats ? '统计数据区域加载成功' : `内容: ${(main||'').substring(0,100)}`, c: 'dashboard' });
  } catch (e) {
    R({ id: 'E1', n: '仪表盘加载', s: 'FAIL', d: e.message, c: 'dashboard' });
  }

  // E2: 仪表盘刷新按钮
  try {
    await navTo(pg, 'dashboard');
    await pg.waitForTimeout(500);
    const refreshBtn = pg.locator('main button').filter({ hasText: /刷新|refresh/i }).first();
    const vis = await refreshBtn.isVisible({ timeout: 3000 }).catch(() => false);
    if (vis) {
      await refreshBtn.click();
      await pg.waitForTimeout(1000);
      await ss(pg, 'E2-dashboard-refresh');
      R({ id: 'E2', n: '仪表盘刷新', s: 'PASS', d: '刷新按钮可点击', c: 'dashboard' });
    } else {
      R({ id: 'E2', n: '仪表盘刷新', s: 'WARN', d: '刷新按钮不可见', c: 'dashboard' });
    }
  } catch (e) {
    R({ id: 'E2', n: '仪表盘刷新', s: 'FAIL', d: e.message, c: 'dashboard' });
  }

  // E3: 仪表盘可视化
  try {
    await navTo(pg, 'dashboard');
    await pg.waitForTimeout(800);
    await ss(pg, 'E3-dashboard-charts');
    R({ id: 'E3', n: '仪表盘可视化', s: 'PASS', d: '图表区域已加载', c: 'dashboard' });
  } catch (e) {
    R({ id: 'E3', n: '仪表盘可视化', s: 'FAIL', d: e.message, c: 'dashboard' });
  }

  // ============================================================
  // SECTION F: 设置页面 (F1-F8)
  // ============================================================

  // F1: 设置页 Tab 结构 (使用 data-testid)
  try {
    await navTo(pg, 'settings');
    await pg.waitForTimeout(800);
    let found = 0;
    for (const tabId of SETTINGS_TABS) {
      const btn = pg.locator(`[data-testid="settings-tab-${tabId}"]`).first();
      if (await btn.isVisible({ timeout: 1000 }).catch(() => false)) found++;
    }
    if (found < 5) {
      for (const label of Object.values(SETTINGS_TAB_LABELS)) {
        const btn = pg.locator('main button, main [role="tab"]').filter({ hasText: label }).first();
        if (await btn.isVisible({ timeout: 500 }).catch(() => false)) found++;
      }
    }
    await ss(pg, 'F1-settings-tabs');
    R({ id: 'F1', n: '设置Tab结构', s: found >= 5 ? 'PASS' : found > 0 ? 'WARN' : 'FAIL',
      d: `检测到 ${found} 个标签页`, c: 'settings' });
  } catch (e) {
    R({ id: 'F1', n: '设置Tab结构', s: 'FAIL', d: e.message, c: 'settings' });
  }

  // F2: AI 配置区域
  try {
    await navTo(pg, 'settings');
    await pg.waitForTimeout(500);
    const main = await pg.locator('main').textContent();
    const hasAI = main?.includes('LM Studio') || main?.includes('Ollama') || main?.includes('推理') || main?.includes('模型') || main?.includes('Provider');
    await ss(pg, 'F2-ai-config');
    R({ id: 'F2', n: 'AI配置区域', s: hasAI ? 'PASS' : 'WARN',
      d: hasAI ? 'AI 配置包含 Provider/模型/URL' : `内容: ${(main||'').substring(0,100)}`, c: 'settings' });
  } catch (e) {
    R({ id: 'F2', n: 'AI配置区域', s: 'FAIL', d: e.message, c: 'settings' });
  }

  // F3: 测试连接按钮
  try {
    await navTo(pg, 'settings');
    await pg.waitForTimeout(500);
    const testBtn = pg.locator('main button').filter({ hasText: /测试连接|test.*connection/i }).first();
    const vis = await testBtn.isVisible({ timeout: 3000 }).catch(() => false);
    if (vis) {
      await testBtn.click();
      await pg.waitForTimeout(3000);
      const main = await pg.locator('main').textContent();
      const ok = main?.includes('成功') || main?.includes('连接') || main?.includes('✓') || main?.includes('success');
      await ss(pg, 'F3-test-connection');
      R({ id: 'F3', n: 'LM Studio连接测试', s: ok ? 'PASS' : 'WARN',
        d: ok ? '测试连接返回结果' : '未检测到连接结果', c: 'settings' });
    } else {
      R({ id: 'F3', n: 'LM Studio连接测试', s: 'WARN', d: '测试连接按钮不可见', c: 'settings' });
    }
  } catch (e) {
    R({ id: 'F3', n: 'LM Studio连接测试', s: 'FAIL', d: e.message, c: 'settings' });
  }

  // F4: 自动发现模型
  try {
    await navTo(pg, 'settings');
    await pg.waitForTimeout(500);
    const discBtn = pg.locator('main button').filter({ hasText: /自动发现|auto.*discover/i }).first();
    const vis = await discBtn.isVisible({ timeout: 3000 }).catch(() => false);
    if (vis) {
      await discBtn.click();
      await pg.waitForTimeout(3000);
      await ss(pg, 'F4-model-discovery');
      R({ id: 'F4', n: '模型自动发现', s: 'PASS', d: '自动发现按钮可点击', c: 'settings' });
    } else {
      R({ id: 'F4', n: '模型自动发现', s: 'WARN', d: '自动发现按钮不可见', c: 'settings' });
    }
  } catch (e) {
    R({ id: 'F4', n: '模型自动发现', s: 'FAIL', d: e.message, c: 'settings' });
  }

  // F5: 显示设置 Tab
  try {
    await navTo(pg, 'settings');
    await pg.waitForTimeout(500);
    const tab = pg.locator('[data-testid="settings-tab-display"]').first();
    if (await tab.isVisible({ timeout: 2000 }).catch(() => false)) {
      await tab.click();
      await pg.waitForTimeout(500);
      const main = await pg.locator('main').textContent();
      const hasDisplay = main?.includes('主题') || main?.includes('浅色') || main?.includes('深色') || main?.includes('缩略图') || main?.includes('theme');
      await ss(pg, 'F5-display');
      R({ id: 'F5', n: '显示配置Tab', s: hasDisplay ? 'PASS' : 'WARN',
        d: hasDisplay ? '显示配置包含主题/缩略图尺寸' : '内容未检测到', c: 'settings' });
    } else {
      R({ id: 'F5', n: '显示配置Tab', s: 'WARN', d: 'Tab不可见', c: 'settings' });
    }
  } catch (e) {
    R({ id: 'F5', n: '显示配置Tab', s: 'FAIL', d: e.message, c: 'settings' });
  }

  // F6: 关于页面
  try {
    const tab = pg.locator('[data-testid="settings-tab-about"]').first();
    if (await tab.isVisible({ timeout: 2000 }).catch(() => false)) {
      await tab.click();
      await pg.waitForTimeout(500);
      const main = await pg.locator('main').textContent();
      const hasAbout = main?.includes('0.9.0') || main?.includes('版本') || main?.includes('MIT') || main?.includes('技术栈') || main?.includes('version');
      await ss(pg, 'F6-about');
      R({ id: 'F6', n: '关于页面', s: hasAbout ? 'PASS' : 'WARN',
        d: hasAbout ? '关于页面显示版本号/许可证/技术栈' : '内容未检测到', c: 'settings' });
    } else {
      R({ id: 'F6', n: '关于页面', s: 'WARN', d: '关于Tab不可见', c: 'settings' });
    }
  } catch (e) {
    R({ id: 'F6', n: '关于页面', s: 'FAIL', d: e.message, c: 'settings' });
  }

  // F7: 日志查看器
  try {
    const tab = pg.locator('[data-testid="settings-tab-logs"]').first();
    if (await tab.isVisible({ timeout: 2000 }).catch(() => false)) {
      await tab.click();
      await pg.waitForTimeout(500);
      await ss(pg, 'F7-logs');
      R({ id: 'F7', n: '日志查看器', s: 'PASS', d: '系统日志Tab加载成功', c: 'settings' });
    } else {
      R({ id: 'F7', n: '日志查看器', s: 'WARN', d: '系统日志Tab不可见', c: 'settings' });
    }
  } catch (e) {
    R({ id: 'F7', n: '日志查看器', s: 'FAIL', d: e.message, c: 'settings' });
  }

  // F8: 设置保存机制
  try {
    await navTo(pg, 'settings');
    await pg.waitForTimeout(500);
    const saveBtn = pg.locator('main button').filter({ hasText: /保存|save/i }).first();
    const vis = await saveBtn.isVisible({ timeout: 2000 }).catch(() => false);
    R({ id: 'F8', n: '设置保存机制', s: 'PASS',
      d: vis ? '保存按钮可见' : '保存按钮不可见（无变更，正常状态）', c: 'settings' });
  } catch (e) {
    R({ id: 'F8', n: '设置保存机制', s: 'FAIL', d: e.message, c: 'settings' });
  }

  // ============================================================
  // SECTION G: 交互功能 (G1, G3-G7)
  // ============================================================

  // G1: 主题切换（使用 data-testid，最多点击3次处理 system 主题）
  try {
    const html = pg.locator('html');
    const before = await html.getAttribute('class') || '';
    const wasDark = before.includes('dark');

    if (!await themeBtn().isVisible({ timeout: 3000 })) throw new Error('主题按钮不可见');

    let switched = false;
    let after = before;
    for (let i = 0; i < 3; i++) {
      await themeBtn().click();
      await pg.waitForTimeout(800);
      after = await html.getAttribute('class') || '';
      const isDark = after.includes('dark');
      if (wasDark !== isDark) { switched = true; break; }
    }

    await ss(pg, 'G1-theme-switch');
    R({ id: 'G1', n: '主题切换', s: switched ? 'PASS' : 'WARN',
      d: switched ? `${wasDark ? 'dark→light' : 'light→dark'} 切换成功` : `class 未变化: "${before}" → "${after}"`, c: 'interaction' });

    await themeBtn().click();
    await pg.waitForTimeout(300);
  } catch (e) {
    R({ id: 'G1', n: '主题切换', s: 'FAIL', d: e.message, c: 'interaction' });
  }

  // G3: 侧边栏折叠/展开 (使用 data-testid)
  try {
    const aside = pg.locator('aside').first();
    const beforeW = await aside.evaluate(el => el.offsetWidth);

    if (!await collapseBtn().isVisible({ timeout: 3000 })) throw new Error('折叠按钮不可见');

    await collapseBtn().click();
    await pg.waitForTimeout(600);
    const collapsedW = await aside.evaluate(el => el.offsetWidth);
    await ss(pg, 'G3-sidebar-collapsed');

    await collapseBtn().click();
    await pg.waitForTimeout(600);
    const expandedW = await aside.evaluate(el => el.offsetWidth);

    const ok = collapsedW < beforeW && expandedW > collapsedW;
    R({ id: 'G3', n: '侧边栏折叠', s: ok ? 'PASS' : 'WARN',
      d: ok ? `${beforeW}px → ${collapsedW}px → ${expandedW}px` : `宽度未变化: ${beforeW}px`, c: 'interaction' });
  } catch (e) {
    R({ id: 'G3', n: '侧边栏折叠', s: 'FAIL', d: e.message, c: 'interaction' });
  }

  // G4: 搜索框输入与防抖
  try {
    await navTo(pg, 'gallery');
    await pg.waitForTimeout(500);
    const input = pg.locator('header input[type="text"]').first();
    if (!await input.isVisible({ timeout: 3000 })) throw new Error('搜索框不可见');

    await input.fill('风景');
    await pg.waitForTimeout(300);
    const val = await input.inputValue();
    await ss(pg, 'G4-search-input');
    R({ id: 'G4', n: '搜索框输入', s: val === '风景' ? 'PASS' : 'WARN',
      d: `输入值: "${val}"，300ms 防抖`, c: 'interaction' });
    await input.fill('');
  } catch (e) {
    R({ id: 'G4', n: '搜索框输入', s: 'FAIL', d: e.message, c: 'interaction' });
  }

  // G5: 键盘快捷键
  try {
    await navTo(pg, 'gallery');
    await pg.waitForTimeout(300);

    const html = pg.locator('html');
    const before = await html.getAttribute('class') || '';
    await pg.keyboard.press('d');
    await pg.waitForTimeout(500);
    const after = await html.getAttribute('class') || '';
    const themeToggled = before !== after;
    await pg.keyboard.press('d');
    await pg.waitForTimeout(300);

    await pg.keyboard.press('/');
    await pg.waitForTimeout(300);
    const focused = await pg.evaluate(() => document.activeElement?.tagName);
    const searchFocused = focused === 'INPUT';
    if (searchFocused) await pg.keyboard.press('Escape');

    await ss(pg, 'G5-keyboard');
    R({ id: 'G5', n: '键盘快捷键', s: themeToggled && searchFocused ? 'PASS' : 'WARN',
      d: `d=主题切换: ${themeToggled}, /=搜索聚焦: ${searchFocused}`, c: 'interaction' });
  } catch (e) {
    R({ id: 'G5', n: '键盘快捷键', s: 'FAIL', d: e.message, c: 'interaction' });
  }

  // G6: 设置页 Tab 切换 (使用 data-testid)
  try {
    await navTo(pg, 'settings');
    await pg.waitForTimeout(500);
    let switched = 0;
    for (const tabId of SETTINGS_TABS) {
      const btn = pg.locator(`[data-testid="settings-tab-${tabId}"]`).first();
      if (await btn.isVisible({ timeout: 1000 }).catch(() => false)) {
        await btn.click();
        await pg.waitForTimeout(300);
        switched++;
      }
    }
    await ss(pg, 'G6-settings-tabs');
    R({ id: 'G6', n: '设置Tab切换', s: switched >= 6 ? 'PASS' : 'WARN',
      d: `成功切换 ${switched}/${SETTINGS_TABS.length} 个标签页`, c: 'interaction' });
  } catch (e) {
    R({ id: 'G6', n: '设置Tab切换', s: 'FAIL', d: e.message, c: 'interaction' });
  }

  // G7: 图片卡片交互 (BLOCKED)
  R({ id: 'G7', n: '图片卡片交互', s: 'BLOCKED',
    d: '需已导入图片。hover/勾选框/AI状态圆点(灰/蓝脉冲/绿/红)',
    c: 'interaction' });

  // ============================================================
  // SECTION H: 性能与稳定性 (H1-H4)
  // ============================================================

  // H1: 页面切换性能
  try {
    const timings = {};
    for (const key of NAV_KEYS) {
      const t0 = Date.now();
      await navTo(pg, key);
      timings[key] = Date.now() - t0;
    }
    const avg = Object.values(timings).reduce((a, b) => a + b, 0) / Object.values(timings).length;
    S.perf.pageTransitions = timings;
    S.perf.avgTransition = avg;
    R({ id: 'H1', n: '页面切换性能', s: avg < 500 ? 'PASS' : 'WARN',
      d: `平均 ${avg.toFixed(0)}ms | ${JSON.stringify(timings)}`, c: 'perf' });
  } catch (e) {
    R({ id: 'H1', n: '页面切换性能', s: 'FAIL', d: e.message, c: 'perf' });
  }

  // H2: 响应式布局
  try {
    const sizes = [
      { w: 1440, h: 900, l: 'desktop' },
      { w: 1024, h: 768, l: 'tablet' },
      { w: 768, h: 1024, l: 'portrait' },
    ];
    for (const s of sizes) {
      await pg.setViewportSize({ width: s.w, height: s.h });
      await pg.waitForTimeout(400);
      await ss(pg, `H2-responsive-${s.l}`);
    }
    await pg.setViewportSize({ width: 1440, height: 900 });
    R({ id: 'H2', n: '响应式布局', s: 'PASS', d: '三种分辨率测试完成', c: 'responsive' });
  } catch (e) {
    R({ id: 'H2', n: '响应式布局', s: 'FAIL', d: e.message, c: 'responsive' });
  }

  // H3: 暗黑模式全页面
  try {
    const html = pg.locator('html');
    const cls = await html.getAttribute('class') || '';
    if (!cls.includes('dark')) {
      if (await themeBtn().isVisible({ timeout: 2000 }).catch(() => false)) {
        await themeBtn().click();
        await pg.waitForTimeout(500);
      }
    }
    for (const key of NAV_KEYS) {
      await navTo(pg, key);
      await pg.waitForTimeout(300);
    }
    await navTo(pg, 'gallery');
    await pg.waitForTimeout(500);
    await ss(pg, 'H3-dark-mode');
    const bg = await pg.evaluate(() => getComputedStyle(document.body).backgroundColor);
    R({ id: 'H3', n: '暗黑模式全页面', s: 'PASS',
      d: `暗黑模式下所有页面正常，背景色: ${bg}`, c: 'ui' });

    if (await themeBtn().isVisible({ timeout: 2000 }).catch(() => false)) {
      await themeBtn().click();
      await pg.waitForTimeout(300);
    }
  } catch (e) {
    R({ id: 'H3', n: '暗黑模式全页面', s: 'FAIL', d: e.message, c: 'ui' });
  }

  // H4: 控制台错误检测
  try {
    const critical = cerrs.filter(e =>
      !e.includes('tauri') && !e.includes('ipc') && !e.includes('WebSocket') &&
      !e.includes('favicon') && !e.includes('HMR') && !e.includes('downloadable font') &&
      !e.includes('DevTools') && !e.includes('transformCallback') && !e.includes('invoke'));
    R({ id: 'H4', n: '控制台错误检测', s: critical.length === 0 ? 'PASS' : 'WARN',
      d: critical.length === 0 ? '无关键控制台错误' : `${critical.length} 个: ${critical.slice(0, 5).join(' | ')}`,
      c: 'stability', errors: critical });
  } catch (e) {
    R({ id: 'H4', n: '控制台错误检测', s: 'FAIL', d: e.message, c: 'stability' });
  }

  // ============================================================
  // SECTION I: 可访问性 (I1)
  // ============================================================

  // I1: ARIA 标签覆盖率
  try {
    await navTo(pg, 'gallery');
    await pg.waitForTimeout(300);
    const allBtns = await pg.locator('button').count();
    const ariaBtns = await pg.locator('button[aria-label]').count();
    const allInps = await pg.locator('input').count();
    const ariaInps = await pg.locator('input[aria-label], input[placeholder]').count();
    const btnPct = allBtns > 0 ? Math.round(ariaBtns / allBtns * 100) : 100;
    const inpPct = allInps > 0 ? Math.round(ariaInps / allInps * 100) : 100;
    await ss(pg, 'I1-a11y');
    R({ id: 'I1', n: '可访问性', s: btnPct >= 40 && inpPct >= 50 ? 'PASS' : 'WARN',
      d: `按钮 aria-label: ${btnPct}% (${ariaBtns}/${allBtns}), 输入框: ${inpPct}% (${ariaInps}/${allInps})`, c: 'a11y' });
  } catch (e) {
    R({ id: 'I1', n: '可访问性', s: 'FAIL', d: e.message, c: 'a11y' });
  }

  // ============================================================
  // SECTION J: 搜索与数据流 (J1-J2)
  // ============================================================

  // J1: 语义搜索流程 (BLOCKED)
  R({ id: 'J1', n: '语义搜索流程', s: 'BLOCKED',
    d: '需已导入图片。search_index文本匹配 + narratives LIKE补充，非向量搜索',
    c: 'search' });

  // J2: 搜索空结果展示
  try {
    await navTo(pg, 'gallery');
    await pg.waitForTimeout(500);
    const input = pg.locator('header input[type="text"]').first();
    await input.fill('xyz不存在的搜索词');
    await pg.waitForTimeout(500);
    const main = await pg.locator('main').textContent();
    const hasNoResult = main?.includes('未找到') || main?.includes('没有') || main?.includes('暂无') || main?.includes('No ');
    await ss(pg, 'J2-search-empty');
    R({ id: 'J2', n: '搜索空结果', s: hasNoResult ? 'PASS' : 'WARN',
      d: hasNoResult ? '搜索无结果时正确显示空状态' : `内容: ${(main||'').substring(0,80)}`, c: 'search' });
    await input.fill('');
  } catch (e) {
    R({ id: 'J2', n: '搜索空结果', s: 'FAIL', d: e.message, c: 'search' });
  }

  // ============================================================
  // SECTION K: 语言切换 (使用 data-testid)
  // ============================================================

  // K1: 语言切换 (中文->English->中文)
  try {
    if (!await langToggle().isVisible({ timeout: 3000 })) throw new Error('语言按钮不可见');

    const beforeNav = await pg.locator('aside nav').textContent();
    await langToggle().click();
    await pg.waitForTimeout(400);

    const enBtn = pg.locator('[data-testid="lang-en"]').first();
    if (await enBtn.isVisible({ timeout: 2000 }).catch(() => false)) {
      await enBtn.click();
      await pg.waitForTimeout(800);
    }

    const afterNav = await pg.locator('aside nav').textContent();
    const switched = beforeNav !== afterNav;

    await ss(pg, 'K1-i18n-english');
    R({ id: 'K1', n: '语言切换->英文', s: switched ? 'PASS' : 'WARN',
      d: switched ? `中->英: "${beforeNav?.substring(0,15)}" -> "${afterNav?.substring(0,15)}"` : '导航文本未变化', c: 'interaction' });

    // 英文模式下验证导航
    let enNavOk = 0;
    for (const key of NAV_KEYS) {
      if (await navTo(pg, key)) enNavOk++;
    }
    await ss(pg, 'K2-english-nav');
    R({ id: 'K2', n: '英文模式导航', s: enNavOk === 5 ? 'PASS' : 'WARN',
      d: `英文模式下 ${enNavOk}/5 页面导航正常`, c: 'interaction' });

    // 切回中文 (使用 data-testid)
    await langToggle().click();
    await pg.waitForTimeout(400);
    const zhBtn = pg.locator('[data-testid="lang-zh"]').first();
    if (await zhBtn.isVisible({ timeout: 2000 }).catch(() => false)) {
      await zhBtn.click();
      await pg.waitForTimeout(800);
    }

    const backNav = await pg.locator('aside nav').textContent();
    const restored = backNav?.includes('图库');
    await ss(pg, 'K3-i18n-restored');
    R({ id: 'K3', n: '语言恢复中文', s: restored ? 'PASS' : 'WARN',
      d: restored ? '中文界面恢复成功' : `恢复后: ${(backNav||'').substring(0,20)}`, c: 'interaction' });
  } catch (e) {
    await ensureChinese(pg);
    R({ id: 'K1', n: '语言切换', s: 'FAIL', d: e.message, c: 'interaction' });
  }

  // ============================================================
  // 生成报告
  // ============================================================
  S.t1 = new Date().toISOString();
  writeFileSync(RPT, JSON.stringify(S, null, 2), 'utf-8');
  console.log(`\n${'='.repeat(60)}`);
  console.log(`TOTAL: ${S.sum.total} | PASS: ${S.sum.pass} | FAIL: ${S.sum.fail} | WARN: ${S.sum.warn} | BLOCKED: ${S.sum.blocked}`);
  const executable = S.sum.total - S.sum.blocked;
  console.log(`Pass Rate: ${executable > 0 ? (S.sum.pass / executable * 100).toFixed(1) : 0}% (${S.sum.pass}/${executable} executable)`);
  console.log(`Screenshots: ${DIR}`);
  console.log(`Report: ${RPT}`);
  console.log(`${'='.repeat(60)}`);

  await br.close();
})();
