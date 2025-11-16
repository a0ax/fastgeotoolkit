/**
 * WASM module initialization utilities
 */
let wasmModule$1 = null;
let initPromise = null;
/**
 * Initialize fastgeotoolkit with automatic WASM loading
 * This is a convenience function that handles WASM loading automatically
 */
async function initWithWasm() {
    if (wasmModule$1) {
        return wasmModule$1;
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
                wasmModule$1 = relativeImport;
                return wasmModule$1;
            }
            catch (relativeError) {
                console.warn('Relative WASM import failed:', relativeError);
            }
            // Strategy 2: CDN fallback for when bundling fails
            try {
                const cdnUrl = 'https://unpkg.com/fastgeotoolkit@latest/wasm/fastgeotoolkit.js';
                const cdnImport = await import(/* @vite-ignore */ cdnUrl);
                if (typeof cdnImport.default === 'function') {
                    await cdnImport.default();
                }
                wasmModule$1 = cdnImport;
                return wasmModule$1;
            }
            catch (cdnError) {
                console.warn('CDN WASM import failed:', cdnError);
            }
            throw new Error('All WASM loading strategies failed');
        }
        catch (error) {
            initPromise = null; // Reset so we can try again
            throw new Error(`Failed to initialize WASM module: ${error}`);
        }
    })();
    return initPromise;
}
/**
 * Load WASM from URL
 */
async function loadWasmFromUrl(wasmJsUrl, wasmBgUrl) {
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
        }
        else {
            await wasmModule.default();
        }
        URL.revokeObjectURL(moduleUrl);
        return wasmModule;
    }
    catch (error) {
        throw new Error(`Failed to load WASM from URL: ${error}`);
    }
}

var wasmLoader = /*#__PURE__*/Object.freeze({
    __proto__: null,
    initWithWasm: initWithWasm,
    loadWasmFromUrl: loadWasmFromUrl
});

/**
 * fastGeoToolkit - A novel high-performance geospatial analysis framework
 * with advanced route density mapping algorithms
 */
// WebAssembly module import (will be bundled)
let wasmModule = null;
let isInitialized = false;
/**
 * Ensure WASM is initialized before calling WASM functions
 * This will automatically initialize if not already done
 */
async function ensureWasmInitialized() {
    if (isInitialized && wasmModule) {
        return;
    }
    try {
        const { initWithWasm } = await Promise.resolve().then(function () { return wasmLoader; });
        wasmModule = await initWithWasm();
        isInitialized = true;
    }
    catch (error) {
        throw new Error(`Failed to initialize WASM: ${error}`);
    }
}
/**
 * Initialize the WebAssembly module
 * Must be called before using any WASM-based functions
 * @param wasmInit Pre-loaded WASM module (from loadWasm() helper)
 */
async function init(wasmInit) {
    if (!wasmInit) {
        throw new Error('WASM module must be provided to init() function. Use loadWasm() to load it first.');
    }
    wasmModule = wasmInit;
    isInitialized = true;
}
/**
 * Load the WASM module - users call this first, then pass result to init()
 * This is now a convenience wrapper around the improved WASM loader
 */
async function loadWasm() {
    try {
        const { initWithWasm } = await Promise.resolve().then(function () { return wasmLoader; });
        return await initWithWasm();
    }
    catch (error) {
        throw new Error(`Failed to load WASM module: ${error}`);
    }
}
/**
 * Process GPX files and generate route density heatmap
 * @param files Array of file data as Uint8Array
 * @returns Heatmap result with frequency analysis
 */
async function processGpxFiles(files) {
    await ensureWasmInitialized();
    const fileArray = new Array(files.length);
    files.forEach((file, i) => {
        fileArray[i] = file;
    });
    return wasmModule.process_gpx_files(fileArray);
}
/**
 * Decode Google polyline format to coordinates
 * @param encoded Encoded polyline string
 * @returns Array of coordinates
 */
async function decodePolyline(encoded) {
    await ensureWasmInitialized();
    return wasmModule.decode_polyline_string(encoded);
}
/**
 * Process multiple polylines and generate heatmap
 * @param polylines Array of polyline strings
 * @returns Heatmap result
 */
async function processPolylines(polylines) {
    await ensureWasmInitialized();
    return wasmModule.process_polylines(polylines);
}
/**
 * Validate GPS coordinates
 * @param coordinates Array of coordinates to validate
 * @returns Validation result with issues
 */
async function validateCoordinates(coordinates) {
    await ensureWasmInitialized();
    return wasmModule.validate_coordinates(coordinates);
}
/**
 * Calculate track statistics
 * @param coordinates Track coordinates
 * @returns Statistics including distance and bounding box
 */
async function calculateTrackStatistics(coordinates) {
    await ensureWasmInitialized();
    return wasmModule.calculate_track_statistics(coordinates);
}
/**
 * Simplify track by reducing point density
 * @param coordinates Track coordinates
 * @param tolerance Simplification tolerance
 * @returns Simplified coordinate array
 */
async function simplifyTrack(coordinates, tolerance) {
    await ensureWasmInitialized();
    return wasmModule.simplify_coordinates(coordinates, tolerance);
}
/**
 * Find intersections between multiple tracks
 * @param tracks Array of track coordinate arrays
 * @param tolerance Distance tolerance for intersection detection
 * @returns Intersection points with track indices
 */
async function findTrackIntersections(tracks, tolerance) {
    await ensureWasmInitialized();
    return wasmModule.find_track_intersections(tracks, tolerance);
}
/**
 * Convert coordinates to GeoJSON feature
 * @param coordinates Track coordinates
 * @param properties Optional properties object
 * @returns GeoJSON feature
 */
async function coordinatesToGeojson(coordinates, properties = {}) {
    await ensureWasmInitialized();
    return wasmModule.coordinates_to_geojson(coordinates, properties);
}
/**
 * Export tracks to GPX format
 * @param tracks Array of track coordinate arrays
 * @param metadata Optional metadata
 * @returns GPX file content as string
 */
async function exportToGpx(tracks, metadata = {}) {
    await ensureWasmInitialized();
    return wasmModule.export_to_gpx(tracks, metadata);
}
/**
 * Calculate coverage area of tracks
 * @param tracks Array of track coordinate arrays
 * @returns Coverage information including bounding box and area
 */
async function calculateCoverageArea(tracks) {
    await ensureWasmInitialized();
    return wasmModule.calculate_coverage_area(tracks);
}
/**
 * Get file information from binary data
 * @param fileData File data as Uint8Array
 * @returns File format information
 */
async function getFileInfo(fileData) {
    await ensureWasmInitialized();
    return wasmModule.get_file_info(fileData);
}
/**
 * Calculate distance between two points using Haversine formula
 * @param lat1 First point latitude
 * @param lon1 First point longitude
 * @param lat2 Second point latitude
 * @param lon2 Second point longitude
 * @returns Distance in kilometers
 */
async function calculateDistance(lat1, lon1, lat2, lon2) {
    await ensureWasmInitialized();
    return wasmModule.calculate_distance_between_points(lat1, lon1, lat2, lon2);
}
/**
 * Utilities for working with coordinates without WebAssembly
 */
const utils = {
    /**
     * Check if coordinates are valid (basic validation)
     */
    isValidCoordinate(lat, lon) {
        return (lat >= -90 && lat <= 90 &&
            lon >= -180 && lon <= 180 &&
            !(lat === 0 && lon === 0) &&
            !isNaN(lat) && !isNaN(lon) &&
            isFinite(lat) && isFinite(lon));
    },
    /**
     * Calculate simple distance using Haversine formula (JavaScript implementation)
     */
    haversineDistance(lat1, lon1, lat2, lon2) {
        const R = 6371; // Earth's radius in kilometers
        const dLat = (lat2 - lat1) * Math.PI / 180;
        const dLon = (lon2 - lon1) * Math.PI / 180;
        const lat1Rad = lat1 * Math.PI / 180;
        const lat2Rad = lat2 * Math.PI / 180;
        const a = Math.sin(dLat / 2) * Math.sin(dLat / 2) +
            Math.cos(lat1Rad) * Math.cos(lat2Rad) *
                Math.sin(dLon / 2) * Math.sin(dLon / 2);
        const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
        return R * c;
    },
    /**
     * Calculate bounding box for coordinate array
     */
    getBoundingBox(coordinates) {
        if (coordinates.length === 0) {
            return [0, 0, 0, 0];
        }
        let minLat = coordinates[0][0];
        let maxLat = coordinates[0][0];
        let minLon = coordinates[0][1];
        let maxLon = coordinates[0][1];
        for (const [lat, lon] of coordinates) {
            minLat = Math.min(minLat, lat);
            maxLat = Math.max(maxLat, lat);
            minLon = Math.min(minLon, lon);
            maxLon = Math.max(maxLon, lon);
        }
        return [minLat, minLon, maxLat, maxLon];
    }
};

export { calculateCoverageArea, calculateDistance, calculateTrackStatistics, coordinatesToGeojson, decodePolyline, exportToGpx, findTrackIntersections, getFileInfo, init, initWithWasm, loadWasm, loadWasmFromUrl, processGpxFiles, processPolylines, simplifyTrack, utils, validateCoordinates };
//# sourceMappingURL=index.esm.js.map
