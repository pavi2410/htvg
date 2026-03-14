import { startTransition, useEffect, useState } from "react";
import CodeMirror from "@uiw/react-codemirror";
import { json } from "@codemirror/lang-json";
import { examples } from "../shared/examples";
import "./App.css";

type CompileResponse =
	| { svg: string; width: number; height: number }
	| { error: string };

type MetaResponse = {
	version: string;
};

type CompileStatus =
	| { state: "idle"; message: string }
	| { state: "loading"; message: string }
	| { state: "success"; message: string }
	| { state: "error"; message: string };

const initialExample = examples[0];
const initialJson = JSON.stringify(initialExample.doc, null, 2);

function downloadBlob(content: string, filename: string, type: string) {
	const url = URL.createObjectURL(new Blob([content], { type }));
	const a = document.createElement("a");
	a.href = url;
	a.download = filename;
	a.click();
	URL.revokeObjectURL(url);
}

function App() {
	const [activeExample, setActiveExample] = useState(initialExample.name);
	const [source, setSource] = useState(initialJson);
	const [svg, setSvg] = useState("");
	const [version, setVersion] = useState("...");
	const [status, setStatus] = useState<CompileStatus>({
		state: "idle",
		message: "Ready to compile",
	});

	useEffect(() => {
		const controller = new AbortController();

		void fetch("/api/meta", { signal: controller.signal })
			.then((response) => response.json() as Promise<MetaResponse>)
			.then((data) => {
				setVersion(data.version);
			})
			.catch((error: unknown) => {
				if (error instanceof DOMException && error.name === "AbortError") {
					return;
				}

				setVersion("unavailable");
			});

		return () => {
			controller.abort();
		};
	}, []);

	useEffect(() => {
		const controller = new AbortController();
		const timer = window.setTimeout(() => {
			void (async () => {
				setStatus({ state: "loading", message: "Compiling document..." });

				try {
					const response = await fetch("/api/compile", {
						method: "POST",
						headers: { "Content-Type": "application/json" },
						body: source,
						signal: controller.signal,
					});
					const data = (await response.json()) as CompileResponse;

					if (!response.ok || "error" in data) {
						setSvg("");
						setStatus({
							state: "error",
							message: "error" in data ? data.error : "Compilation failed",
						});
						return;
					}

					setSvg(data.svg);
					setStatus({
						state: "success",
						message: `${data.width} × ${data.height}`,
					});
				} catch (error: unknown) {
					if (error instanceof DOMException && error.name === "AbortError") {
						return;
					}

					const message = error instanceof Error ? error.message : "Request failed";
					setSvg("");
					setStatus({ state: "error", message });
				}
			})();
		}, 280);

		return () => {
			controller.abort();
			window.clearTimeout(timer);
		};
	}, [source]);

	function loadExample(name: string) {
		const example = examples.find((item) => item.name === name);
		if (!example) return;

		startTransition(() => {
			setActiveExample(example.name);
			setSource(JSON.stringify(example.doc, null, 2));
		});
	}

	return (
		<div className="app-shell">
			<header className="app-header">
				<h1>HTVG Playground</h1>
				<div className="header-meta">
					<span>v{version}</span>
					<span className={`status-badge is-${status.state}`}>{status.message}</span>
					<a href="https://github.com/pavi2410/htvg" target="_blank" rel="noreferrer">
						GitHub
					</a>
				</div>
			</header>

			<div className="workspace-grid">
				<section className="panel">
					<div className="panel-header">
						<span className="panel-title">Source</span>
						<div className="example-rail" role="tablist" aria-label="Example documents">
							{examples.map((example) => (
								<button
									key={example.name}
									type="button"
									className={example.name === activeExample ? "example-pill is-active" : "example-pill"}
									onClick={() => loadExample(example.name)}
								>
									{example.name}
								</button>
							))}
						</div>
					</div>
					<div className="editor-frame">
						<CodeMirror
							value={source}
							height="100%"
							extensions={[json()]}
							onChange={(value) => {
								setActiveExample("");
								setSource(value);
							}}
							basicSetup={{
								lineNumbers: true,
								foldGutter: false,
								dropCursor: false,
								highlightActiveLine: false,
							}}
						/>
					</div>
				</section>

				<section className="panel">
					<div className="panel-header">
						<span className="panel-title">Preview</span>
						<div className="preview-actions">
							{svg && (
								<button
									type="button"
									onClick={() => downloadBlob(svg, `${activeExample || "document"}.svg`, "image/svg+xml")}
								>
									Download SVG
								</button>
							)}
							<button
								type="button"
								onClick={() => downloadBlob(source, `${activeExample || "document"}.json`, "application/json")}
							>
								Download JSON
							</button>
						</div>
					</div>
					<div className="preview-stage">
						{svg ? (
							<div className="preview-surface" dangerouslySetInnerHTML={{ __html: svg }} />
						) : (
							<div className="preview-empty">
								<span>No SVG yet</span>
							</div>
						)}
					</div>
					<div className="preview-footnote">
						<span>POST /api/compile</span>
					</div>
				</section>
			</div>
		</div>
	);
}

export default App;
