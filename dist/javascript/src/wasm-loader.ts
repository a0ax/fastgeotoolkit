/**
 * WASM module initialization utilities
 */

let wasmModule: any = null;
let initPromise: Promise<any> | null = null;

/**
 * Initialize fastgeotoolkit with automatic WASM loading
 * This is a convenience function that handles WASM loading automatically
 */
export async function initWithWasm(): Promise<any> {
  if (wasmModule) {
    return wasmModule;
  }
  
  if (initPromise) {
    return initPromise;
  }
  
  initPromise = (async () => {
    try {
      // Strategy 1: Try relative path from built package (this should work with bundlers)
      try {
        const relativePath = '../wasm/fastgeotoolkit.js';
        const relativeImport = await import(/* @vite-ignore */ relativePath);
        if (typeof relativeImport.default === 'function') {
          await relativeImport.default();
        }
        wasmModule = relativeImport;
        return wasmModule;
      } catch (relativeError) {
        console.warn('Relative WASM import failed:', relativeError);
      }
      
      // Strategy 2: CDN fallback for when bundling fails
      try {
        const cdnUrl = 'https://unpkg.com/fastgeotoolkit@latest/wasm/fastgeotoolkit.js';
        const cdnImport = await import(/* @vite-ignore */ cdnUrl);
        if (typeof cdnImport.default === 'function') {
          await cdnImport.default();
        }
        wasmModule = cdnImport;
        return wasmModule;
      } catch (cdnError) {
        console.warn('CDN WASM import failed:', cdnError);
      }
      
      throw new Error('All WASM loading strategies failed');
    } catch (error) {
      initPromise = null; // Reset so we can try again
      throw new Error(`Failed to initialize WASM module: ${error}`);
    }
  })();
  
  return initPromise;
}

/**
 * Load WASM from URL
 */
export async function loadWasmFromUrl(wasmJsUrl: string, wasmBgUrl?: string): Promise<any> {
  try {
    const response = await fetch(wasmJsUrl);
    if (!response.ok) {
      throw new Error(`Failed to fetch WASM JS module: ${response.status}`);
    }
    
    const moduleText = await response.text();
    const moduleBlob = new Blob([moduleText], { type: 'application/javascript' });
    const moduleUrl = URL.createObjectURL(moduleBlob);
    
    const wasmModule = await import(/* @vite-ignore */ moduleUrl);
    
    if (wasmBgUrl) {
      await wasmModule.default(wasmBgUrl);
    } else {
      await wasmModule.default();
    }
    
    URL.revokeObjectURL(moduleUrl);
    
    return wasmModule;
  } catch (error) {
    throw new Error(`Failed to load WASM from URL: ${error}`);
  }
}
