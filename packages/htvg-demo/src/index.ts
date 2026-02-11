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

function renderExampleCard(name: string, json: string, svg: string): string {
  return `
    <section class="example">
      <h2>${escapeHtml(name)}</h2>
      <div class="example-body">
        <div class="code-pane">
          <pre><code>${escapeHtml(json)}</code></pre>
        </div>
        <div class="svg-pane">
          ${svg}
        </div>
      </div>
    </section>`;
}

function buildPage(cards: string, ver: string): string {
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

    header {
      max-width: 1100px;
      margin: 0 auto;
      padding: 48px 24px 32px;
    }

    header h1 {
      font-size: 32px;
      font-weight: 700;
      letter-spacing: -0.02em;
      color: #f8fafc;
    }

    header p {
      margin-top: 8px;
      font-size: 16px;
      color: #94a3b8;
      line-height: 1.6;
    }

    header .version {
      display: inline-block;
      margin-top: 12px;
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
      gap: 40px;
    }

    .example h2 {
      font-size: 18px;
      font-weight: 600;
      color: #cbd5e1;
      margin-bottom: 16px;
      font-family: 'JetBrains Mono', monospace;
    }

    .example-body {
      display: flex;
      gap: 24px;
      align-items: flex-start;
    }

    .code-pane {
      flex: 1;
      min-width: 0;
      background: #1e293b;
      border: 1px solid #334155;
      border-radius: 12px;
      overflow: auto;
      max-height: 480px;
    }

    .code-pane pre {
      padding: 20px;
      margin: 0;
      font-family: 'JetBrains Mono', monospace;
      font-size: 12.5px;
      line-height: 1.6;
      color: #e2e8f0;
      white-space: pre;
      tab-size: 2;
    }

    .svg-pane {
      flex-shrink: 0;
      background: repeating-conic-gradient(#1e293b 0% 25%, #0f172a 0% 50%) 0 0 / 16px 16px;
      border: 1px solid #334155;
      border-radius: 12px;
      padding: 24px;
      display: flex;
      align-items: flex-start;
      justify-content: center;
    }

    .svg-pane svg {
      display: block;
      max-width: 100%;
      height: auto;
    }

    @media (max-width: 800px) {
      .example-body {
        flex-direction: column;
      }
      .svg-pane {
        width: 100%;
      }
    }
  </style>
</head>
<body>
  <header>
    <h1>HTVG Demo</h1>
    <p>JSON → SVG, rendered server-side with WASM on Cloudflare Workers.</p>
    <span class="version">v${escapeHtml(ver)}</span>
  </header>
  <main>
    ${cards}
  </main>
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
    const cards = examples
      .map((ex) => {
        const json = JSON.stringify(ex.doc, null, 2);
        const result = compileDocument(ex.doc);
        return renderExampleCard(ex.name, json, result.svg);
      })
      .join("\n");

    const html = buildPage(cards, version());
    return new Response(html, {
      headers: { "Content-Type": "text/html; charset=utf-8" },
    });
  },
};
