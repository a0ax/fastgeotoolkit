# fastGeoToolkit

[![Crates.io](https://img.shields.io/crates/v/fastgeotoolkit.svg)](https://crates.io/crates/fastgeotoolkit)
[![Documentation](https://docs.rs/fastgeotoolkit/badge.svg)](https://docs.rs/fastgeotoolkit)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A novel high-performance geospatial analysis framework with advanced route density mapping algorithms.

## Features

- **High-Performance GPS Processing**: Optimized algorithms for large-scale GPS trajectory analysis
- **Multi-Format Support**: GPX, FIT, and polyline format parsing
- **Route Density Mapping**: Novel segment-based frequency analysis for route popularity visualization
- **Coordinate Validation**: Robust filtering of unrealistic GPS jumps and invalid coordinates
- **Track Simplification**: Douglas-Peucker-inspired algorithms for efficient data reduction
- **Geospatial Analytics**: Intersection detection, clustering, and coverage analysis
- **Cross-Platform**: Native Rust with optional WebAssembly compilation

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
fastgeotoolkit = "0.1.3"
```

### Basic Usage

```rust
use fastgeotoolkit::{
    process_gpx_files, decode_polyline, 
    calculate_track_statistics, find_track_intersections
};

// Process GPX files
let gpx_data = std::fs::read("track.gpx")?;
let result = process_gpx_files(&[gpx_data]);

// Decode polylines
let coords = decode_polyline("_p~iF~ps|U_ulLnnqC_mqNvxq`@");

// Calculate statistics
let stats = calculate_track_statistics(&coords);
println!("Distance: {:.2} km", stats.distance_km);
```

### Route Density Analysis

```rust
use fastgeotoolkit::{HeatmapResult, create_heatmap_from_tracks};

// Load multiple GPS tracks
let tracks = vec![
    vec![[40.7128, -74.0060], [40.7589, -73.9851]], // NYC example
    vec![[40.7505, -73.9934], [40.7831, -73.9712]], // Central Park
];

// Generate route density heatmap
let heatmap = create_heatmap_from_tracks(tracks);
println!("Max frequency: {}", heatmap.max_frequency);

for track in heatmap.tracks {
    println!("Track with {} points, frequency: {}", 
             track.coordinates.len(), track.frequency);
}
```

### WebAssembly Support

Enable WebAssembly compilation:

```toml
[dependencies]
fastgeotoolkit = { version = "0.1.3", features = ["wasm"] }
```

## API Documentation

### Core Functions

- `process_gpx_files()` - Parse and analyze GPX file data
- `decode_polyline()` - Decode Google polyline format
- `process_polylines()` - Batch process polyline data
- `validate_coordinates()` - Validate GPS coordinate arrays
- `simplify_track()` - Reduce track point density while preserving shape

### Analysis Functions

- `calculate_track_statistics()` - Distance, bounds, and point count analysis
- `find_track_intersections()` - Detect where multiple tracks intersect
- `cluster_tracks_by_similarity()` - Group similar routes together
- `calculate_coverage_area()` - Compute geographic coverage of track data

### Conversion Functions

- `coordinates_to_geojson()` - Export to GeoJSON format
- `export_to_gpx()` - Generate GPX files from coordinate data
- `coordinates_to_polyline()` - Encode coordinates as polylines

## Performance

fastGeoToolkit is optimized for processing large GPS datasets:

- **Memory Efficient**: Streaming processing for large files
- **Fast Parsing**: Optimized GPX and FIT file readers
- **Parallel Processing**: Multi-threaded analysis where applicable
- **WASM Ready**: Near-native performance in web browsers

## Algorithm Details

### Route Density Mapping

Our novel approach uses segment-based frequency analysis:

1. **Segmentation**: Tracks are broken into coordinate pairs
2. **Grid Snapping**: Coordinates are snapped to a tolerance grid
3. **Frequency Counting**: Overlapping segments are identified and counted
4. **Normalization**: Track frequencies are calculated from segment usage

This approach provides more accurate route popularity visualization compared to simple point-based methods.

## Examples

See the `examples/` directory for comprehensive usage examples:

- `basic_usage.rs` - Getting started with core functions
- `route_density.rs` - Advanced heatmap generation
- `batch_processing.rs` - Processing multiple files efficiently

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## Citation

If you use fastGeoToolkit in academic research, please cite:

```bibtex
@software{fastgeotoolkit2024,
  title={fastGeoToolkit: A Novel High-Performance Geospatial Analysis Framework with Advanced Route Density Mapping},
  author={a0a7},
  year={2024},
  url={https://github.com/a0a7/fastgeotoolkit},
  version={0.1.3}
}
```
