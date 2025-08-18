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
const resolvePath = (p: string, name: string) => normalizePath(path.join(path.dirname(require.resolve(p)), name));

// https://vitejs.dev/config/
export default defineConfig(async () => ({
	plugins: [
		viteStaticCopy({
			targets: [
				{ src: resolvePath("pdfjs-dist/package.json", "cmaps"), dest: "" },
				{ src: resolvePath("pdfjs-dist/package.json", "standard_fonts"), dest: "" },
				{ src: resolvePath("pdfjs-dist/package.json", "wasm"), dest: "" },
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
