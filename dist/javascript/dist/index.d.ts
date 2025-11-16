/**
 * WASM module initialization utilities
 */
/**
 * Initialize fastgeotoolkit with automatic WASM loading
 * This is a convenience function that handles WASM loading automatically
 */
declare function initWithWasm(): Promise<any>;
/**
 * Load WASM from URL
 */
declare function loadWasmFromUrl(wasmJsUrl: string, wasmBgUrl?: string): Promise<any>;

/**
 * fastGeoToolkit - A novel high-performance geospatial analysis framework
 * with advanced route density mapping algorithms
 */

/**
 * GPS coordinate as [latitude, longitude]
 */
type Coordinate = [number, number];
/**
 * Track with frequency information for heatmap visualization
 */
interface HeatmapTrack {
    coordinates: Coordinate[];
    frequency: number;
}
/**
 * Result of heatmap generation with route density analysis
 */
interface HeatmapResult {
    tracks: HeatmapTrack[];
    max_frequency: number;
}
/**
 * Validation result for coordinate arrays
 */
interface ValidationResult {
    valid_count: number;
    total_count: number;
    issues: string[];
}
/**
 * Track statistics including distance and bounding box
 */
interface TrackStatistics {
    distance_km: number;
    point_count: number;
    bounding_box: [number, number, number, number];
    elevation_gain?: number;
    average_speed?: number;
}
/**
 * File format information
 */
interface FileInfo {
    format: string;
    track_count: number;
    point_count: number;
    valid: boolean;
    file_size: number;
}
/**
 * Initialize the WebAssembly module
 * Must be called before using any WASM-based functions
 * @param wasmInit Pre-loaded WASM module (from loadWasm() helper)
 */
declare function init(wasmInit: any): Promise<void>;
/**
 * Load the WASM module - users call this first, then pass result to init()
 * This is now a convenience wrapper around the improved WASM loader
 */
declare function loadWasm(): Promise<any>;
/**
 * Process GPX files and generate route density heatmap
 * @param files Array of file data as Uint8Array
 * @returns Heatmap result with frequency analysis
 */
declare function processGpxFiles(files: Uint8Array[]): Promise<HeatmapResult>;
/**
 * Decode Google polyline format to coordinates
 * @param encoded Encoded polyline string
 * @returns Array of coordinates
 */
declare function decodePolyline(encoded: string): Promise<Coordinate[]>;
/**
 * Process multiple polylines and generate heatmap
 * @param polylines Array of polyline strings
 * @returns Heatmap result
 */
declare function processPolylines(polylines: string[]): Promise<HeatmapResult>;
/**
 * Validate GPS coordinates
 * @param coordinates Array of coordinates to validate
 * @returns Validation result with issues
 */
declare function validateCoordinates(coordinates: Coordinate[]): Promise<ValidationResult>;
/**
 * Calculate track statistics
 * @param coordinates Track coordinates
 * @returns Statistics including distance and bounding box
 */
declare function calculateTrackStatistics(coordinates: Coordinate[]): Promise<TrackStatistics>;
/**
 * Simplify track by reducing point density
 * @param coordinates Track coordinates
 * @param tolerance Simplification tolerance
 * @returns Simplified coordinate array
 */
declare function simplifyTrack(coordinates: Coordinate[], tolerance: number): Promise<Coordinate[]>;
/**
 * Find intersections between multiple tracks
 * @param tracks Array of track coordinate arrays
 * @param tolerance Distance tolerance for intersection detection
 * @returns Intersection points with track indices
 */
declare function findTrackIntersections(tracks: Coordinate[][], tolerance: number): Promise<Array<{
    coordinate: Coordinate;
    track_indices: number[];
}>>;
/**
 * Convert coordinates to GeoJSON feature
 * @param coordinates Track coordinates
 * @param properties Optional properties object
 * @returns GeoJSON feature
 */
declare function coordinatesToGeojson(coordinates: Coordinate[], properties?: Record<string, any>): Promise<any>;
/**
 * Export tracks to GPX format
 * @param tracks Array of track coordinate arrays
 * @param metadata Optional metadata
 * @returns GPX file content as string
 */
declare function exportToGpx(tracks: Coordinate[][], metadata?: Record<string, any>): Promise<string>;
/**
 * Calculate coverage area of tracks
 * @param tracks Array of track coordinate arrays
 * @returns Coverage information including bounding box and area
 */
declare function calculateCoverageArea(tracks: Coordinate[][]): Promise<{
    bounding_box: [number, number, number, number];
    area_km2: number;
    point_count: number;
}>;
/**
 * Get file information from binary data
 * @param fileData File data as Uint8Array
 * @returns File format information
 */
declare function getFileInfo(fileData: Uint8Array): Promise<FileInfo>;
/**
 * Calculate distance between two points using Haversine formula
 * @param lat1 First point latitude
 * @param lon1 First point longitude
 * @param lat2 Second point latitude
 * @param lon2 Second point longitude
 * @returns Distance in kilometers
 */
declare function calculateDistance(lat1: number, lon1: number, lat2: number, lon2: number): Promise<number>;
/**
 * Utilities for working with coordinates without WebAssembly
 */
declare const utils: {
    /**
     * Check if coordinates are valid (basic validation)
     */
    isValidCoordinate(lat: number, lon: number): boolean;
    /**
     * Calculate simple distance using Haversine formula (JavaScript implementation)
     */
    haversineDistance(lat1: number, lon1: number, lat2: number, lon2: number): number;
    /**
     * Calculate bounding box for coordinate array
     */
    getBoundingBox(coordinates: Coordinate[]): [number, number, number, number];
};

export { calculateCoverageArea, calculateDistance, calculateTrackStatistics, coordinatesToGeojson, decodePolyline, exportToGpx, findTrackIntersections, getFileInfo, init, initWithWasm, loadWasm, loadWasmFromUrl, processGpxFiles, processPolylines, simplifyTrack, utils, validateCoordinates };
export type { Coordinate, FileInfo, HeatmapResult, HeatmapTrack, TrackStatistics, ValidationResult };
