import reactLogo from "./assets/react.svg";
import tauriLogo from "./assets/tauri.svg";
import viteLogo from "./assets/vite.svg";
import TauriDemo from "./components/TauriDemo";

function App() {
	return (
		<main className="container">
			<h1>Welcome to Tauri + React</h1>

			<div className="row">
				<a href="https://vitejs.dev" target="_blank" rel="noopener noreferrer">
					<img src={viteLogo} className="logo vite" alt="Vite logo" />
				</a>
				<a href="https://tauri.app" target="_blank" rel="noopener noreferrer">
					<img src={tauriLogo} className="logo tauri" alt="Tauri logo" />
				</a>
				<a href="https://reactjs.org" target="_blank" rel="noopener noreferrer">
					<img src={reactLogo} className="logo react" alt="React logo" />
				</a>
			</div>

			<p>Click on the Tauri, Vite, and React logos to learn more.</p>

			<TauriDemo />
		</main>
	);
}

export default App;
