import { init, compileDocument, version, type HtvgDocument } from "htvg";
// @ts-ignore — wrangler handles .wasm imports natively
import wasmBinary from "htvg/htvg.wasm";

// ---------------------------------------------------------------------------
// Examples — embedded as plain objects so they're bundled into the worker
// ---------------------------------------------------------------------------

const examples: { name: string; doc: HtvgDocument }[] = [
  {
    name: "hello",
    doc: {
      meta: { width: 400 },
      content: {
        type: "flex",
        style: { width: 400, padding: 32, backgroundColor: "#ffffff", flexDirection: "column", gap: 12 },
        children: [
          { type: "text", content: "Hello, HTVG!", style: { fontSize: 32, fontWeight: "bold", color: "#1a1a1a" } },
          { type: "text", content: "This is a self-contained document rendered to SVG.", style: { fontSize: 16, color: "#666666" } },
        ],
      },
    },
  },
  {
    name: "card",
    doc: {
      meta: { width: 360 },
      content: {
        type: "flex",
        style: { width: 360, padding: 24, backgroundColor: "#f8f9fa", flexDirection: "column", gap: 16 },
        children: [
          {
            type: "flex",
            style: { backgroundColor: "#ffffff", borderRadius: 12, borderWidth: 1, borderColor: "#e0e0e0", padding: 20, flexDirection: "column", gap: 8 },
            children: [
              { type: "text", content: "Card Title", style: { fontSize: 20, fontWeight: 700, color: "#1a1a1a" } },
              { type: "text", content: "This is a card component with a border, rounded corners, and some padding. It demonstrates nested flex containers.", style: { fontSize: 14, color: "#555555", lineHeight: 1.5 } },
              {
                type: "flex",
                style: { flexDirection: "row", gap: 8, padding: "12 0 0 0" },
                children: [
                  { type: "box", style: { width: 80, height: 32, backgroundColor: "#2563eb", borderRadius: 6 } },
                  { type: "box", style: { width: 80, height: 32, backgroundColor: "#e2e8f0", borderRadius: 6 } },
                ],
              },
            ],
          },
          {
            type: "flex",
            style: { backgroundColor: "#ffffff", borderRadius: 12, borderWidth: 1, borderColor: "#e0e0e0", padding: 20, flexDirection: "column", gap: 8 },
            children: [
              { type: "text", content: "Another Card", style: { fontSize: 20, fontWeight: 700, color: "#1a1a1a" } },
              { type: "text", content: "Cards can be stacked vertically in a flex column layout.", style: { fontSize: 14, color: "#555555", lineHeight: 1.5 } },
            ],
          },
        ],
      },
    },
  },
  {
    name: "badge-row",
    doc: {
      meta: { width: 500 },
      content: {
        type: "flex",
        style: { width: 500, padding: 24, backgroundColor: "#1e293b", flexDirection: "column", gap: 16 },
        children: [
          { type: "text", content: "Status Dashboard", style: { fontSize: 24, fontWeight: "bold", color: "#f1f5f9" } },
          {
            type: "flex",
            style: { flexDirection: "row", gap: 10, flexWrap: "wrap" },
            children: [
              { type: "flex", style: { backgroundColor: "#22c55e", borderRadius: 16, padding: "6 14" }, children: [{ type: "text", content: "Passing", style: { fontSize: 13, fontWeight: 600, color: "#ffffff" } }] },
              { type: "flex", style: { backgroundColor: "#ef4444", borderRadius: 16, padding: "6 14" }, children: [{ type: "text", content: "Failing", style: { fontSize: 13, fontWeight: 600, color: "#ffffff" } }] },
              { type: "flex", style: { backgroundColor: "#eab308", borderRadius: 16, padding: "6 14" }, children: [{ type: "text", content: "Warning", style: { fontSize: 13, fontWeight: 600, color: "#ffffff" } }] },
              { type: "flex", style: { backgroundColor: "#3b82f6", borderRadius: 16, padding: "6 14" }, children: [{ type: "text", content: "Info", style: { fontSize: 13, fontWeight: 600, color: "#ffffff" } }] },
            ],
          },
          {
            type: "flex",
            style: { backgroundColor: "#334155", borderRadius: 8, padding: 16, flexDirection: "column", gap: 6 },
            children: [
              { type: "text", content: "Build #1247 completed in 3m 42s", style: { fontSize: 14, color: "#94a3b8" } },
              { type: "text", content: "All 128 tests passed. 0 failures.", style: { fontSize: 14, color: "#22c55e" } },
            ],
          },
        ],
      },
    },
  },
];

// ---------------------------------------------------------------------------
// HTML helpers
// ---------------------------------------------------------------------------

function escapeHtml(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

function buildPage(ver: string, defaultPlaygroundJson: string): string {
  const selectOptions = examples
    .map((ex) => '<option value="' + escapeHtml(ex.name) + '">' + escapeHtml(ex.name) + "</option>")
    .join("");
  const examplesJsonBlob = JSON.stringify(Object.fromEntries(examples.map((ex) => [ex.name, ex.doc])));

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>HTVG Demo</title>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet" />
  <style>
    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

    body {
      font-family: 'Inter', system-ui, sans-serif;
      background: #0f172a;
      color: #e2e8f0;
      min-height: 100vh;
    }

    a { color: #38bdf8; text-decoration: none; }
    a:hover { text-decoration: underline; }

    header {
      max-width: 1100px;
      margin: 0 auto;
      padding: 48px 24px 32px;
    }

    .header-top {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 16px;
    }

    header h1 {
      font-size: 32px;
      font-weight: 700;
      letter-spacing: -0.02em;
      color: #f8fafc;
    }

    .github-link {
      display: flex;
      align-items: center;
      gap: 6px;
      color: #94a3b8;
      font-size: 14px;
      font-weight: 500;
      transition: color 0.15s;
    }
    .github-link:hover { color: #f8fafc; text-decoration: none; }
    .github-link svg { fill: currentColor; }

    header p {
      margin-top: 8px;
      font-size: 16px;
      color: #94a3b8;
      line-height: 1.6;
    }

    .header-meta {
      display: flex;
      align-items: center;
      gap: 12px;
      margin-top: 12px;
    }

    .version {
      font-family: 'JetBrains Mono', monospace;
      font-size: 13px;
      background: #1e293b;
      color: #38bdf8;
      padding: 4px 10px;
      border-radius: 6px;
    }

    main {
      max-width: 1100px;
      margin: 0 auto;
      padding: 0 24px 64px;
      display: flex;
      flex-direction: column;
      gap: 48px;
    }

    /* ---- Playground ---- */
    .playground { display: flex; flex-direction: column; gap: 16px; }

    .playground h2 {
      font-size: 22px;
      font-weight: 700;
      color: #f8fafc;
      letter-spacing: -0.01em;
    }

    .playground-desc {
      font-size: 14px;
      color: #94a3b8;
      line-height: 1.5;
    }

    .playground-body {
      display: flex;
      gap: 24px;
      align-items: stretch;
    }

    .playground-editor {
      flex: 1;
      min-width: 0;
      display: flex;
      flex-direction: column;
      gap: 8px;
    }

    .playground-editor textarea {
      flex: 1;
      min-height: 400px;
      resize: vertical;
      background: #1e293b;
      border: 1px solid #334155;
      border-radius: 12px;
      padding: 20px;
      font-family: 'JetBrains Mono', monospace;
      font-size: 13px;
      line-height: 1.6;
      color: #e2e8f0;
      tab-size: 2;
      outline: none;
      transition: border-color 0.15s;
    }
    .playground-editor textarea:focus {
      border-color: #38bdf8;
    }

    .playground-toolbar {
      display: flex;
      align-items: center;
      gap: 12px;
    }

    .playground-toolbar .status {
      font-family: 'JetBrains Mono', monospace;
      font-size: 12px;
      color: #64748b;
    }
    .playground-toolbar .status.error {
      color: #f87171;
    }
    .playground-toolbar .status.ok {
      color: #4ade80;
    }

    .example-select {
      background: #1e293b;
      border: 1px solid #334155;
      border-radius: 6px;
      color: #e2e8f0;
      font-family: 'Inter', system-ui, sans-serif;
      font-size: 13px;
      padding: 6px 10px;
      outline: none;
      cursor: pointer;
    }
    .example-select:focus { border-color: #38bdf8; }

    .playground-preview {
      flex-shrink: 0;
      width: 540px;
      background: repeating-conic-gradient(#1e293b 0% 25%, #0f172a 0% 50%) 0 0 / 16px 16px;
      border: 1px solid #334155;
      border-radius: 12px;
      padding: 24px;
      display: flex;
      align-items: flex-start;
      justify-content: center;
      overflow: auto;
    }

    .playground-preview svg {
      display: block;
      max-width: 100%;
      height: auto;
    }

    @media (max-width: 900px) {
      .playground-body {
        flex-direction: column;
      }
      .playground-preview {
        width: 100%;
      }
    }
  </style>
</head>
<body>
  <header>
    <div class="header-top">
      <h1>HTVG Demo</h1>
      <a class="github-link" href="https://github.com/pavi2410/htvg" target="_blank" rel="noopener">
        <svg width="20" height="20" viewBox="0 0 16 16"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0016 8c0-4.42-3.58-8-8-8z"/></svg>
        GitHub
      </a>
    </div>
    <p>JSON \u2192 SVG, rendered server-side with WASM on Cloudflare Workers.</p>
    <div class="header-meta">
      <span class="version">v${escapeHtml(ver)}</span>
    </div>
  </header>
  <main>
    <section class="playground">
      <h2>Playground</h2>
      <p class="playground-desc">Edit the JSON below and see the SVG update live.</p>
      <div class="playground-body">
        <div class="playground-editor">
          <div class="playground-toolbar">
            <label for="example-select" style="font-size:13px;color:#94a3b8">Load example:</label>
            <select id="example-select" class="example-select">
              <option value="">— custom —</option>
              ${selectOptions}
            </select>
            <span id="pg-status" class="status"></span>
          </div>
          <textarea id="pg-input" spellcheck="false">${escapeHtml(defaultPlaygroundJson)}</textarea>
        </div>
        <div class="playground-preview" id="pg-preview"></div>
      </div>
    </section>
  </main>

  <script>
    const EXAMPLES = ${examplesJsonBlob};

    const input = document.getElementById('pg-input');
    const preview = document.getElementById('pg-preview');
    const status = document.getElementById('pg-status');
    const select = document.getElementById('example-select');
    let timer = null;

    async function compile() {
      const json = input.value;
      status.textContent = 'compiling...';
      status.className = 'status';
      try {
        const res = await fetch('/api/compile', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: json,
        });
        const data = await res.json();
        if (data.error) {
          status.textContent = data.error;
          status.className = 'status error';
        } else {
          preview.innerHTML = data.svg;
          status.textContent = data.width + ' \u00d7 ' + data.height;
          status.className = 'status ok';
        }
      } catch (e) {
        status.textContent = e.message;
        status.className = 'status error';
      }
    }

    function scheduleCompile() {
      clearTimeout(timer);
      timer = setTimeout(compile, 300);
    }

    input.addEventListener('input', () => {
      select.value = '';
      scheduleCompile();
    });

    select.addEventListener('change', () => {
      const name = select.value;
      if (name && EXAMPLES[name]) {
        input.value = JSON.stringify(EXAMPLES[name], null, 2);
        compile();
      }
    });

    // Load first example on page load
    select.value = Object.keys(EXAMPLES)[0] || '';

    // Initial compile
    compile();
  </script>
</body>
</html>`;
}

// ---------------------------------------------------------------------------
// Worker
// ---------------------------------------------------------------------------

export default {
  async fetch(request: Request): Promise<Response> {
    await init(wasmBinary);

    const url = new URL(request.url);

    // POST /api/compile  — playground live compilation
    if (url.pathname === "/api/compile" && request.method === "POST") {
      try {
        const body = await request.text();
        const doc: HtvgDocument = JSON.parse(body);
        const result = compileDocument(doc);
        return Response.json({ svg: result.svg, width: result.width, height: result.height });
      } catch (e: unknown) {
        const msg = e instanceof Error ? e.message : String(e);
        return Response.json({ error: msg }, { status: 400 });
      }
    }

    // /api/:name.svg  — raw SVG for a single example
    const svgMatch = url.pathname.match(/^\/api\/([a-z-]+)\.svg$/);
    if (svgMatch) {
      const ex = examples.find((e) => e.name === svgMatch[1]);
      if (!ex) return new Response("Not found", { status: 404 });
      const result = compileDocument(ex.doc);
      return new Response(result.svg, {
        headers: { "Content-Type": "image/svg+xml", "Cache-Control": "public, max-age=3600" },
      });
    }

    // /api/:name.json  — raw JSON for a single example
    const jsonMatch = url.pathname.match(/^\/api\/([a-z-]+)\.json$/);
    if (jsonMatch) {
      const ex = examples.find((e) => e.name === jsonMatch[1]);
      if (!ex) return new Response("Not found", { status: 404 });
      return Response.json(ex.doc);
    }

    // Default: demo page
    const defaultJson = JSON.stringify(examples[0].doc, null, 2);
    const html = buildPage(version(), defaultJson);
    return new Response(html, {
      headers: { "Content-Type": "text/html; charset=utf-8" },
    });
  },
};
