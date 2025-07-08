import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useState } from "react";

/**
 * TauriDemo component showcasing various Tauri API integrations with React
 */
function TauriDemo() {
	const [greetMsg, setGreetMsg] = useState("");
	const [name, setName] = useState("");
	const [isLoading, setIsLoading] = useState(false);

	/**
	 * Calls the Rust greet command
	 */
	async function greet() {
		if (!name.trim()) {
			setGreetMsg("Please enter a name!");
			return;
		}

		setIsLoading(true);
		try {
			// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
			const result = await invoke("greet", { name: name.trim() });
			setGreetMsg(result as string);
		} catch (error) {
			console.error("Error calling greet command:", error);
			setGreetMsg("Error occurred while greeting!");
		} finally {
			setIsLoading(false);
		}
	}

	/**
	 * Opens external URLs using Tauri's opener plugin
	 */
	async function handleOpenUrl(url: string) {
		try {
			await openUrl(url);
		} catch (error) {
			console.error("Error opening URL:", error);
		}
	}

	return (
		<div className="tauri-demo">
			<h2>Tauri API Demo</h2>

			{/* Greet Command Demo */}
			<section className="demo-section">
				<h3>Rust Command Integration</h3>
				<form
					className="row"
					onSubmit={(e) => {
						e.preventDefault();
						greet();
					}}
				>
					<input
						id="greet-input"
						onChange={(e) => setName(e.currentTarget.value)}
						placeholder="Enter a name..."
						value={name}
						disabled={isLoading}
					/>
					<button type="submit" disabled={isLoading || !name.trim()}>
						{isLoading ? "Greeting..." : "Greet"}
					</button>
				</form>
				{greetMsg && <p className="greet-message">{greetMsg}</p>}
			</section>

			{/* External Links Demo */}
			<section className="demo-section">
				<h3>External Links</h3>
				<div className="links-grid">
					<button
						type="button"
						onClick={() => handleOpenUrl("https://tauri.app")}
					>
						Open Tauri Docs
					</button>
					<button
						type="button"
						onClick={() => handleOpenUrl("https://reactjs.org")}
					>
						Open React Docs
					</button>
					<button
						type="button"
						onClick={() => handleOpenUrl("https://vitejs.dev")}
					>
						Open Vite Docs
					</button>
				</div>
			</section>
		</div>
	);
}

export default TauriDemo;
