import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	optimizeDeps: {
		exclude: ['fastgeotoolkit'],
		include: [
			'maplibre-gl',
			'svelte-maplibre'
		]
	},
	ssr: {
		noExternal: []
	},
	server: {
		fs: {
			allow: ['..']
		}
	},
	build: {
		rollupOptions: {
			external: ['zlib', 'fs', 'path', 'stream', 'util']
		},
		commonjsOptions: {
			include: [
				/node_modules/,
				/maplibre-gl/,
			],
			transformMixedEsModules: true
		},
		target: 'esnext'
	},
	// Handle WASM files properly for fastgeotoolkit
	assetsInclude: ['**/*.wasm'],
	worker: {
		format: 'es'
	},
	define: {
		global: 'globalThis',
	}
});
