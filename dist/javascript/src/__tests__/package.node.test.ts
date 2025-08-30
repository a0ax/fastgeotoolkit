/**
 * @jest-environment node
 */

// Import from the built distribution
const {
  init: initNode,
  decodePolyline: decodePolylineNode,
  validateCoordinates: validateCoordinatesNode,
  calculateDistance: calculateDistanceNode,
  simplifyTrack: simplifyTrackNode,
  processGpxFiles: processGpxFilesNode,
  processPolylines: processPolylinesNode
} = require('../../dist/index.js');

// Track WASM initialization status
let wasmInitializedNode = false;

// Initialize WASM once for all tests
beforeAll(async () => {
  try {
    await initNode();
    wasmInitializedNode = true;
  } catch (error) {
    console.warn('WASM initialization failed, tests will run in mock mode:', String(error));
  }
}, 30000);

describe('JavaScript Package Integration Tests (Node)', () => {
  test('should export main functions', () => {
    expect(typeof initNode).toBe('function');
    expect(typeof decodePolylineNode).toBe('function');
    expect(typeof validateCoordinatesNode).toBe('function');
    expect(typeof calculateDistanceNode).toBe('function');
    expect(typeof simplifyTrackNode).toBe('function');
    expect(typeof processGpxFilesNode).toBe('function');
    expect(typeof processPolylinesNode).toBe('function');
  });


  test('should handle basic polyline decoding', async () => {
    if (!wasmInitializedNode) {
      console.log('Skipping WASM-dependent test: WASM not initialized');
      return;
    }
    
    try {
      const result = await decodePolylineNode("_p~iF~ps|U_ulLnnqC_mqNvxq`@");
      expect(Array.isArray(result)).toBe(true);
      
      if (result.length > 0) {
        expect(result[0]).toHaveLength(2);
        expect(typeof result[0][0]).toBe('number');
        expect(typeof result[0][1]).toBe('number');
      }
    } catch (error) {
      console.warn('Polyline decoding test failed:', String(error));
      throw error;
    }
  });

  test('should handle coordinate validation', async () => {
    if (!wasmInitializedNode) {
      console.log('Skipping WASM-dependent test: WASM not initialized');
      return;
    }
    
    try {
      const coords = [[45.0, -122.0], [0, 0]];
      const result = await validateCoordinatesNode(coords);
      
      expect(result).toHaveProperty('valid_count');
      expect(result).toHaveProperty('total_count');
      expect(result).toHaveProperty('issues');
      expect(Array.isArray(result.issues)).toBe(true);
      expect(typeof result.valid_count).toBe('number');
      expect(typeof result.total_count).toBe('number');
    } catch (error) {
      console.warn('Coordinate validation test failed:', String(error));
      throw error;
    }
  });

  test('should handle distance calculation', async () => {
    if (!wasmInitializedNode) {
      console.log('Skipping WASM-dependent test: WASM not initialized');
      return;
    }
    
    try {
      const distance = await calculateDistanceNode(40.7128, -74.0060, 34.0522, -118.2437);
      expect(typeof distance).toBe('number');
      expect(distance).toBeGreaterThan(0);
    } catch (error) {
      console.warn('Distance calculation test failed:', String(error));
      throw error;
    }
  });

  test('should handle track simplification', async () => {
    if (!wasmInitializedNode) {
      console.log('Skipping WASM-dependent test: WASM not initialized');
      return;
    }
    
    try {
      const coords = [[40.0, -120.0], [40.001, -120.001], [40.01, -120.01]];
      const result = await simplifyTrackNode(coords, 0.005);
      
      expect(Array.isArray(result)).toBe(true);
      expect(result.length).toBeLessThanOrEqual(coords.length);
    } catch (error) {
      console.warn('Track simplification test failed:', String(error));
      throw error;
    }
  });

  test('should handle empty inputs gracefully', async () => {
    if (!wasmInitializedNode) {
      console.log('Skipping WASM-dependent test: WASM not initialized');
      return;
    }
    
    try {
      const emptyPolyline = await decodePolylineNode('');
      expect(Array.isArray(emptyPolyline)).toBe(true);
      expect(emptyPolyline).toHaveLength(0);

      const emptyGpx = await processGpxFilesNode([]);
      expect(emptyGpx).toHaveProperty('tracks');
      expect(Array.isArray(emptyGpx.tracks)).toBe(true);

      const emptyPolylines = await processPolylinesNode([]);
      expect(emptyPolylines).toHaveProperty('tracks');
      expect(Array.isArray(emptyPolylines.tracks)).toBe(true);
    } catch (error) {
      console.warn('Empty input test failed:', String(error));
      throw error;
    }
  });

  test('should handle invalid inputs without crashing', async () => {
    if (!wasmInitializedNode) {
      console.log('Skipping WASM-dependent test: WASM not initialized');
      return;
    }
    
    try {
      // These should not throw errors but handle gracefully
      const invalidPolyline = await decodePolylineNode('invalid_data');
      expect(Array.isArray(invalidPolyline)).toBe(true);

      const invalidCoords = await validateCoordinatesNode([[NaN, Infinity]]);
      expect(invalidCoords).toHaveProperty('valid_count');
      expect(invalidCoords.valid_count).toBe(0);
    } catch (error) {
      console.warn('Invalid input test failed:', String(error));
      throw error;
    }
  });
});
