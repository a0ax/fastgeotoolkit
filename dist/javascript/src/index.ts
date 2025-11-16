/**
 * fastGeoToolkit - A novel high-performance geospatial analysis framework
 * with advanced route density mapping algorithms
 */

// WebAssembly module import (will be bundled)
let wasmModule: any = null;
let isInitialized = false;

// Export WASM loading utilities
export { initWithWasm, loadWasmFromUrl } from './wasm-loader.js';

/**
 * GPS coordinate as [latitude, longitude]
 */
export type Coordinate = [number, number];

/**
 * Track with frequency information for heatmap visualization
 */
export interface HeatmapTrack {
  coordinates: Coordinate[];
  frequency: number;
}

/**
 * Result of heatmap generation with route density analysis
 */
export interface HeatmapResult {
  tracks: HeatmapTrack[];
  max_frequency: number;
}

/**
 * Validation result for coordinate arrays
 */
export interface ValidationResult {
  valid_count: number;
  total_count: number;
  issues: string[];
}

/**
 * Track statistics including distance and bounding box
 */
export interface TrackStatistics {
  distance_km: number;
  point_count: number;
  bounding_box: [number, number, number, number]; // [min_lat, min_lng, max_lat, max_lng]
  elevation_gain?: number;
  average_speed?: number;
}

/**
 * File format information
 */
export interface FileInfo {
  format: string;
  track_count: number;
  point_count: number;
  valid: boolean;
  file_size: number;
}

/**
 * Ensure WASM is initialized before calling WASM functions
 * This will automatically initialize if not already done
 */
async function ensureWasmInitialized(): Promise<void> {
  if (isInitialized && wasmModule) {
    return;
  }
  
  try {
    const { initWithWasm } = await import('./wasm-loader.js');
    wasmModule = await initWithWasm();
    isInitialized = true;
  } catch (error) {
    throw new Error(`Failed to initialize WASM: ${error}`);
  }
}

/**
 * Initialize the WebAssembly module
 * Must be called before using any WASM-based functions
 * @param wasmInit Pre-loaded WASM module (from loadWasm() helper)
 */
export async function init(wasmInit: any): Promise<void> {
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
export async function loadWasm(): Promise<any> {
  try {
    const { initWithWasm } = await import('./wasm-loader.js');
    return await initWithWasm();
  } catch (error) {
    throw new Error(`Failed to load WASM module: ${error}`);
  }
}

/**
 * Process GPX files and generate route density heatmap
 * @param files Array of file data as Uint8Array
 * @returns Heatmap result with frequency analysis
 */
export async function processGpxFiles(files: Uint8Array[]): Promise<HeatmapResult> {
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
export async function decodePolyline(encoded: string): Promise<Coordinate[]> {
  await ensureWasmInitialized();
  return wasmModule.decode_polyline_string(encoded);
}

/**
 * Process multiple polylines and generate heatmap
 * @param polylines Array of polyline strings
 * @returns Heatmap result
 */
export async function processPolylines(polylines: string[]): Promise<HeatmapResult> {
  await ensureWasmInitialized();
  return wasmModule.process_polylines(polylines);
}

/**
 * Validate GPS coordinates
 * @param coordinates Array of coordinates to validate
 * @returns Validation result with issues
 */
export async function validateCoordinates(coordinates: Coordinate[]): Promise<ValidationResult> {
  await ensureWasmInitialized();
  return wasmModule.validate_coordinates(coordinates);
}

/**
 * Calculate track statistics
 * @param coordinates Track coordinates
 * @returns Statistics including distance and bounding box
 */
export async function calculateTrackStatistics(coordinates: Coordinate[]): Promise<TrackStatistics> {
  await ensureWasmInitialized();
  return wasmModule.calculate_track_statistics(coordinates);
}

/**
 * Simplify track by reducing point density
 * @param coordinates Track coordinates
 * @param tolerance Simplification tolerance
 * @returns Simplified coordinate array
 */
export async function simplifyTrack(coordinates: Coordinate[], tolerance: number): Promise<Coordinate[]> {
  await ensureWasmInitialized();
  return wasmModule.simplify_coordinates(coordinates, tolerance);
}

/**
 * Find intersections between multiple tracks
 * @param tracks Array of track coordinate arrays
 * @param tolerance Distance tolerance for intersection detection
 * @returns Intersection points with track indices
 */
export async function findTrackIntersections(
  tracks: Coordinate[][],
  tolerance: number
): Promise<Array<{ coordinate: Coordinate; track_indices: number[] }>> {
  await ensureWasmInitialized();
  return wasmModule.find_track_intersections(tracks, tolerance);
}

/**
 * Convert coordinates to GeoJSON feature
 * @param coordinates Track coordinates
 * @param properties Optional properties object
 * @returns GeoJSON feature
 */
export async function coordinatesToGeojson(
  coordinates: Coordinate[],
  properties: Record<string, any> = {}
): Promise<any> {
  await ensureWasmInitialized();
  return wasmModule.coordinates_to_geojson(coordinates, properties);
}

/**
 * Export tracks to GPX format
 * @param tracks Array of track coordinate arrays
 * @param metadata Optional metadata
 * @returns GPX file content as string
 */
export async function exportToGpx(
  tracks: Coordinate[][],
  metadata: Record<string, any> = {}
): Promise<string> {
  await ensureWasmInitialized();
  return wasmModule.export_to_gpx(tracks, metadata);
}

/**
 * Calculate coverage area of tracks
 * @param tracks Array of track coordinate arrays
 * @returns Coverage information including bounding box and area
 */
export async function calculateCoverageArea(
  tracks: Coordinate[][]
): Promise<{
  bounding_box: [number, number, number, number];
  area_km2: number;
  point_count: number;
}> {
  await ensureWasmInitialized();
  return wasmModule.calculate_coverage_area(tracks);
}

/**
 * Get file information from binary data
 * @param fileData File data as Uint8Array
 * @returns File format information
 */
export async function getFileInfo(fileData: Uint8Array): Promise<FileInfo> {
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
export async function calculateDistance(
  lat1: number,
  lon1: number,
  lat2: number,
  lon2: number
): Promise<number> {
  await ensureWasmInitialized();
  return wasmModule.calculate_distance_between_points(lat1, lon1, lat2, lon2);
}

/**
 * Utilities for working with coordinates without WebAssembly
 */
export const utils = {
  /**
   * Check if coordinates are valid (basic validation)
   */
  isValidCoordinate(lat: number, lon: number): boolean {
    return (
      lat >= -90 && lat <= 90 &&
      lon >= -180 && lon <= 180 &&
      !(lat === 0 && lon === 0) &&
      !isNaN(lat) && !isNaN(lon) &&
      isFinite(lat) && isFinite(lon)
    );
  },

  /**
   * Calculate simple distance using Haversine formula (JavaScript implementation)
   */
  haversineDistance(lat1: number, lon1: number, lat2: number, lon2: number): number {
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
  getBoundingBox(coordinates: Coordinate[]): [number, number, number, number] {
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
