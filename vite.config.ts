import { createRequire } from "node:module";
import path from "node:path";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { FileSystemIconLoader } from "unplugin-icons/loaders";
import Icons from "unplugin-icons/vite";
import { defineConfig, normalizePath } from "vite";
import { viteStaticCopy } from "vite-plugin-static-copy";

const host = process.env.TAURI_DEV_HOST;

const require = createRequire(import.meta.url);
const pdfjsDistPath = path.dirname(require.resolve("pdfjs-dist/package.json"));
const pdfWorkerPath = normalizePath(path.join(pdfjsDistPath, "build", "pdf.worker.mjs"));
const cMapsDir = normalizePath(path.join(pdfjsDistPath, "cmaps"));
const standardFontsDir = normalizePath(
	path.join(path.dirname(require.resolve("pdfjs-dist/package.json")), "standard_fonts"),
);
const wasmDir = normalizePath(path.join(pdfjsDistPath, "wasm"));

// https://vitejs.dev/config/
export default defineConfig(async () => ({
	plugins: [
		viteStaticCopy({
			targets: [
				{
					src: pdfWorkerPath,
					dest: "",
				},
				{
					src: cMapsDir,
					dest: "",
				},
				{
					src: wasmDir,
					dest: "",
				},
				{
					src: standardFontsDir,
					dest: "",
				},
			],
		}),
		Icons({
			compiler: "jsx",
			jsx: "react",
			customCollections: {
				journey: FileSystemIconLoader("./public/icons"),
			},
		}),
		tailwindcss(),
		react(),
	],
	// Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
	// 1. prevent vite from obscuring rust errors
	clearScreen: false,
	// 2. tauri expects a fixed port, fail if that port is not available
	server: {
		port: 1420,
		strictPort: true,
		host: host || false,
		hmr: host
			? {
					protocol: "ws",
					host,
					port: 1421,
				}
			: undefined,
		watch: {
			// 3. tell vite to ignore watching `src-tauri`
			ignored: ["**/src-tauri/**"],
		},
	},
}));
