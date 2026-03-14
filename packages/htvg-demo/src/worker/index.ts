import { Hono } from "hono";
import { init, compileDocument, version, type HtvgDocument } from "htvg";
// @ts-expect-error Vite handles wasm module imports for the Worker runtime.
import wasmBinary from "htvg/htvg.wasm";
import { exampleMap } from "../shared/examples";

const app = new Hono<{ Bindings: Env }>();
let initPromise: Promise<void> | null = null;

function ensureHtvg(): Promise<void> {
	if (!initPromise) {
		initPromise = init(wasmBinary);
	}

	return initPromise;
}

app.use("/api/*", async (_c, next) => {
	await ensureHtvg();
	await next();
});

app.get("/api/meta", (c) => {
	return c.json({ version: version() });
});

app.post("/api/compile", async (c) => {
	try {
		const body = await c.req.text();
		const doc = JSON.parse(body) as HtvgDocument;
		const result = compileDocument(doc);
		return c.json({ svg: result.svg, width: result.width, height: result.height });
	} catch (error: unknown) {
		const message = error instanceof Error ? error.message : "Compilation failed";
		return c.json({ error: message }, 400);
	}
});

app.get("/api/:name.svg", (c) => {
	const name = c.req.param("name");
	if (typeof name !== "string" || !/^[a-z-]+$/.test(name)) {
		return c.notFound();
	}

	const example = exampleMap.get(name);
	if (!example) {
		return c.notFound();
	}

	const result = compileDocument(example.doc);
	return new Response(result.svg, {
		headers: {
			"Content-Type": "image/svg+xml",
			"Cache-Control": "public, max-age=3600",
		},
	});
});

app.get("/api/:name.json", (c) => {
	const name = c.req.param("name");
	if (typeof name !== "string" || !/^[a-z-]+$/.test(name)) {
		return c.notFound();
	}

	const example = exampleMap.get(name);
	if (!example) {
		return c.notFound();
	}

	return c.json(example.doc);
});

export default app;
