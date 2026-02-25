
# fastgeotoolkit  [![npm](https://img.shields.io/npm/v/fastgeotoolkit)](https://www.npmjs.com/package/fastgeotoolkit) [![Docs](https://img.shields.io/badge/Documentation-skyblue?logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCA2NDAgNjQwIj48IS0tIUZvbnQgQXdlc29tZSBGcmVlIDcuMC4wIGJ5IEBmb250YXdlc29tZSAtIGh0dHBzOi8vZm9udGF3ZXNvbWUuY29tIExpY2Vuc2UgLSBodHRwczovL2ZvbnRhd2Vzb21lLmNvbS9saWNlbnNlL2ZyZWUgQ29weXJpZ2h0IDIwMjUgRm9udGljb25zLCBJbmMuLS0+PHBhdGggZD0iTTQ4MCA1NzZMMTkyIDU3NkMxMzkgNTc2IDk2IDUzMyA5NiA0ODBMOTYgMTYwQzk2IDEwNyAxMzkgNjQgMTkyIDY0TDQ5NiA2NEM1MjIuNSA2NCA1NDQgODUuNSA1NDQgMTEyTDU0NCA0MDBDNTQ0IDQyMC45IDUzMC42IDQzOC43IDUxMiA0NDUuM0w1MTIgNTEyQzUyOS43IDUxMiA1NDQgNTI2LjMgNTQ0IDU0NEM1NDQgNTYxLjcgNTI5LjcgNTc2IDUxMiA1NzZMNDgwIDU3NnpNMTkyIDQ0OEMxNzQuMyA0NDggMTYwIDQ2Mi4zIDE2MCA0ODBDMTYwIDQ5Ny43IDE3NC4zIDUxMiAxOTIgNTEyTDQ0OCA1MTJMNDQ4IDQ0OEwxOTIgNDQ4ek0yMjQgMjE2QzIyNCAyMjkuMyAyMzQuNyAyNDAgMjQ4IDI0MEw0MjQgMjQwQzQzNy4zIDI0MCA0NDggMjI5LjMgNDQ4IDIxNkM0NDggMjAyLjcgNDM3LjMgMTkyIDQyNCAxOTJMMjQ4IDE5MkMyMzQuNyAxOTIgMjI0IDIwMi43IDIyNCAyMTZ6TTI0OCAyODhDMjM0LjcgMjg4IDIyNCAyOTguNyAyMjQgMzEyQzIyNCAzMjUuMyAyMzQuNyAzMzYgMjQ4IDMzNkw0MjQgMzM2QzQzNy4zIDMzNiA0NDggMzI1LjMgNDQ4IDMxMkM0NDggMjk4LjcgNDM3LjMgMjg4IDQyNCAyODhMMjQ4IDI4OHoiLz48L3N2Zz4=)](https://fastgeotoolkit.pages.dev/) [![Demo](https://img.shields.io/badge/Try%20the%20demo-lightblue)](https://fastgeotoolkit-demo.pages.dev/)


<!--[![codecov](https://codecov.io/gh/a0a7/fastgeotoolkit/branch/main/graph/badge.svg)](https://codecov.io/gh/a0a7/fastgeotoolkit)-->
[![Rust Tests](https://github.com/a0a7/fastgeotoolkit/actions/workflows/rust-tests.yml/badge.svg)](https://github.com/a0a7/fastgeotoolkit/actions/workflows/rust-tests.yml)
[![JavaScript Tests](https://github.com/a0a7/fastgeotoolkit/actions/workflows/javascript-tests.yml/badge.svg)](https://github.com/a0a7/fastgeotoolkit/actions/workflows/javascript-tests.yml)
[![CodeQL](https://github.com/a0a7/fastgeotoolkit/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/a0a7/fastgeotoolkit/actions/workflows/github-code-scanning/codeql)
![License](https://img.shields.io/badge/license-MIT-blue)

fastgeotoolkit is a library for GPS data processing and route density mapping. The core of the library is written in Rust and it's compiled to webassembly for use in the browser and node.

> [!NOTE]
> Only Javascript/Typescript is supported at the moment. Rust and Python releases are planned.

## What it does

The main use case is creating route heatmaps where you want to see which paths/routes are used most frequently. You can test this functionality at [https://fastgeotoolkit-demo.pages.dev/](https://fastgeotoolkit-demo.pages.dev/), using either your own data or sample data. This is an example of what a heatmap produced using fastgeotoolkit looks like:
![https://i.ibb.co/MxpHbVdp/image.png](https://i.ibb.co/MxpHbVdp/image.png)

However, beyond this primary usecase, this library helps you:
- Analyze GPS tracks (distance, statistics, intersections)
- Decode Google polylines 
- Convert between GPS data formats

## Documentation

Docs are available at https://fastgeotoolkit.pages.dev/. 

## Installation

```bash
npm install fastgeotoolkit
# or 
pnpm i fastgeotoolkit
```

## Basic Usage


```typescript
import { processGpxFiles } from 'fastgeotoolkit';

// Process GPX files into a heatmap
const gpxFile1 = new Uint8Array(/* your GPX file data */);
const gpxFile2 = new Uint8Array(/* another GPX file */);

const result = await processGpxFiles([gpxFile1, gpxFile2]);

// Result contains tracks with frequency data
console.log(`Found ${result.tracks.length} unique track segments`);
console.log(`Maximum frequency: ${result.max_frequency}`);

result.tracks.forEach(track => {
  console.log(`Track with ${track.coordinates.length} points, used ${track.frequency} times`);
});
```

## Working with Polylines

```typescript
import { decodePolyline, processPolylines } from 'fastgeotoolkit';

// Decode a single polyline
const coords = await decodePolyline('_p~iF~ps|U_ulLnnqC_mqNvxq`@');
console.log(coords); // [[lat, lng], [lat, lng], ...]

// Process multiple polylines into a heatmap
const polylines = [
  '_p~iF~ps|U_ulLnnqC_mqNvxq`@',
  'another_encoded_polyline',
  'yet_another_one'
];
const heatmap = await processPolylines(polylines);
```

## Track Analysis

```typescript
import { calculateTrackStatistics, validateCoordinates } from 'fastgeotoolkit';

const coordinates = [[37.7749, -122.4194], [37.7849, -122.4094]]; // [lat, lng] pairs

// Get basic statistics
const stats = await calculateTrackStatistics(coordinates);
console.log(`Distance: ${stats.distance_km.toFixed(2)} km`);
console.log(`${stats.point_count} GPS points`);
console.log(`Bounds: ${stats.bounding_box}`); // [min_lat, min_lng, max_lat, max_lng]

// Validate coordinates
const validation = await validateCoordinates(coordinates);
console.log(`${validation.valid_count} out of ${validation.total_count} coordinates are valid`);
if (validation.issues.length > 0) {
  console.log('Issues found:', validation.issues);
}
```

## Data Conversion

```typescript
import { coordinatesToGeojson, exportToGpx } from 'fastgeotoolkit';

// Convert to GeoJSON
const geojson = await coordinatesToGeojson(coordinates, {
  name: 'My Route',
  activity: 'cycling'
});

// Export multiple tracks as GPX
const tracks = [track1_coordinates, track2_coordinates];
const gpxString = await exportToGpx(tracks, {
  creator: 'My App',
  name: 'Route Collection'
});
```

## Real-world Example

Here's an example of how you might use this in a web app to show route popularity:

```typescript
import { processGpxFiles } from 'fastgeotoolkit';

async function createHeatmap(gpxFiles) {
  // Convert files to Uint8Array
  const fileBuffers = await Promise.all(
    gpxFiles.map(file => file.arrayBuffer().then(buf => new Uint8Array(buf)))
  );
  
  // Process into heatmap
  const heatmap = await processGpxFiles(fileBuffers);
  
  // Render on map (example with any mapping library)
  heatmap.tracks.forEach(track => {
    const intensity = track.frequency / heatmap.max_frequency;
    const color = `hsl(${(1-intensity) * 240}, 100%, 50%)`; // blue to red
    
    drawLineOnMap(track.coordinates, {
      color: color,
      weight: Math.max(2, intensity * 8)
    });
  });
}

// Usage
document.getElementById('file-input').addEventListener('change', async (e) => {
  const files = Array.from(e.target.files);
  await createHeatmap(files);
});
```

## TypeScript Support

The library includes full TypeScript definitions:

```typescript
import type { 
  Coordinate,        // [number, number] - [lat, lng]
  HeatmapResult,     // { tracks: HeatmapTrack[], max_frequency: number }
  HeatmapTrack,      // { coordinates: Coordinate[], frequency: number }
  TrackStatistics,   // distance, bounds, point count, etc.
  ValidationResult,  // validation results with issues
  FileInfo          // file format information
} from 'fastgeotoolkit';
```

## JavaScript Utilities

For simple operations that don't rely on WebAssembly:

```typescript
import { utils } from 'fastgeotoolkit';

// Basic coordinate validation
if (utils.isValidCoordinate(37.7749, -122.4194)) {
  console.log('Valid GPS coordinate');
}

// Calculate distance between two points
const distance = utils.haversineDistance(37.7749, -122.4194, 37.7849, -122.4094);
console.log(`Distance: ${distance.toFixed(2)} km`);

// Get bounding box
const bounds = utils.getBoundingBox(coordinates);
console.log(`Bounds: ${bounds}`); // [min_lat, min_lng, max_lat, max_lng]
```

## Browser vs Node.js

Works the same in both environments:

```javascript
// Browser
import { processGpxFiles } from 'fastgeotoolkit';

// Node.js  
const { processGpxFiles } = require('fastgeotoolkit');
// or with ES modules:
import { processGpxFiles } from 'fastgeotoolkit';
```

## Performance Notes

- WebAssembly provides near-native performance for GPS processing
- Large datasets (thousands of tracks) process quickly
- First function call initializes WebAssembly (adds ~100ms startup time)

## Common Issues

**"Cannot resolve module"** errors: Make sure your bundler supports WebAssembly. Modern bundlers (Vite, Webpack 5+, etc.) work out of the box.

**TypeScript errors**: Ensure you're using TypeScript 4.0+ for proper WebAssembly typing support.

**File reading**: Remember to convert File objects to Uint8Array:
```javascript
const buffer = await file.arrayBuffer();
const uint8Array = new Uint8Array(buffer);
```

## Development & Maintenance

This project consists of Rust code compiled to WebAssembly with JavaScript/TypeScript bindings.

### Project Structure

- `/src/` - Rust source code
- `/dist/javascript/` - JavaScript/TypeScript bindings and NPM package
- `/dist/wasm/` - Generated WebAssembly files
- `/demo/` - Demo application (SvelteKit)
- `/docs/` - Generated documentation
> [!NOTE]
> `/dist/python` and `/dist/rust/` contain WIP releases for their respective ecosystems, but they're not in working order yet.

### Compiling Rust to WebAssembly

To compile the Rust code to WebAssembly:

```bash
# Install wasm-pack if you haven't already
cargo install wasm-pack

# Build the WebAssembly module
wasm-pack build --target web --out-dir dist/wasm
```

### Building the NPM Package

To build the complete NPM package with all bindings:

```bash
# From the root directory
npm run build

# Or build individual components:
npm run build:wasm    # Build WebAssembly
npm run build:js      # Build JavaScript bindings
npm run build:docs    # Build documentation
```

### Building Documentation

The documentation is generated using TypeDoc and can be built locally:

```bash
cd dist/javascript
npm run docs
```

This will generate the documentation website in the `docs/` directory.

### Testing

```bash
# Run Rust tests
cargo test

# Run JavaScript tests
cd dist/javascript
npm test
```

## License

MIT
