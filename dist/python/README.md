# fastGeoToolkit Python

[![PyPI](https://img.shields.io/pypi/v/fastgeotoolkit.svg)](https://pypi.org/project/fastgeotoolkit/)
[![Python Version](https://img.shields.io/pypi/pyversions/fastgeotoolkit.svg)](https://pypi.org/project/fastgeotoolkit/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A novel high-performance geospatial analysis framework with advanced route density mapping algorithms, powered by Rust for maximum performance in Python.

## Installation

```bash
pip install fastgeotoolkit
```

### Optional Dependencies

For enhanced functionality, install with optional dependencies:

```bash
# For visualization
pip install fastgeotoolkit[examples]  # includes matplotlib, folium, etc.

# For development
pip install fastgeotoolkit[dev]       # includes testing and linting tools

# Everything
pip install fastgeotoolkit[all]
```

## Quick Start

### Basic Usage

```python
import fastgeotoolkit as fgt

# Load and process GPX files
heatmap = fgt.load_gpx_file("track.gpx")
print(f"Generated heatmap with {len(heatmap.tracks)} tracks")
print(f"Maximum frequency: {heatmap.max_frequency}")

# Access individual tracks with frequency data
for i, track in enumerate(heatmap.tracks):
    print(f"Track {i}: {track.frequency}x frequency, {len(track.coordinates)} points")
```

### Route Density Analysis

```python
import fastgeotoolkit as fgt

# Process multiple GPX files for route density analysis
gpx_files = ["route1.gpx", "route2.gpx", "route3.gpx"]
heatmap = fgt.load_multiple_gpx_files(gpx_files)

# Analyze route popularity
stats = fgt.calculate_heatmap_statistics(heatmap)
print(f"Total tracks: {stats['total_tracks']}")
print(f"Average frequency: {stats['mean_frequency']:.1f}")
print(f"Frequency distribution: {stats['frequency_distribution']}")

# Find most popular routes
popular_routes = [
    track for track in heatmap.tracks 
    if track.frequency == heatmap.max_frequency
]
print(f"Found {len(popular_routes)} most popular routes")
```

### Interactive Visualization

```python
import fastgeotoolkit as fgt

# Create interactive map with Folium
heatmap = fgt.load_gpx_file("tracks.gpx")
map_viz = fgt.create_folium_map(heatmap)
map_viz.save("heatmap.html")

# Create matplotlib visualization
fgt.visualize_heatmap(heatmap, figsize=(15, 10))
```

### Advanced GPS Processing

```python
import fastgeotoolkit as fgt

# Decode polylines (Google Maps format)
coordinates = fgt.decode_polyline("_p~iF~ps|U_ulLnnqC_mqNvxq`@")
print(f"Decoded {len(coordinates)} points")

# Calculate track statistics
stats = fgt.calculate_track_statistics(coordinates)
print(f"Distance: {stats.distance_km:.2f} km")
print(f"Bounding box: {stats.bounding_box}")

# Validate coordinates
validation = fgt.validate_coordinates(coordinates)
print(f"Valid coordinates: {validation.valid_count}/{validation.total_count}")
for issue in validation.issues:
    print(f"  - {issue}")
```

## Core Features

### Route Density Mapping

Advanced segment-based analysis for route popularity:

```python
import fastgeotoolkit as fgt

# Multiple GPS tracks from the same area
tracks = [
    [(40.7128, -74.0060), (40.7589, -73.9851)],  # NYC to Times Square
    [(40.7128, -74.0060), (40.7589, -73.9851)],  # Same route (increases frequency)
    [(40.7505, -73.9934), (40.7831, -73.9712)],  # Empire State to Central Park
]

# Process with our novel frequency algorithm
result = fgt.process_polylines([fgt.coordinates_to_polyline(track) for track in tracks])

# Analyze route overlap and frequency
for track in result.tracks:
    popularity = "High" if track.frequency > result.max_frequency * 0.7 else "Medium" if track.frequency > result.max_frequency * 0.3 else "Low"
    print(f"Route popularity: {popularity} (frequency: {track.frequency})")
```

### Track Analysis & Statistics

Comprehensive geospatial analysis:

```python
import fastgeotoolkit as fgt

coordinates = [(40.7128, -74.0060), (40.7589, -73.9851), (40.7831, -73.9712)]

# Detailed statistics
stats = fgt.calculate_track_statistics(coordinates)
print(f"Distance: {stats.distance_km:.2f} km")
print(f"Points: {stats.point_count}")
print(f"Bounds: {stats.bounding_box}")

# Find intersections between multiple tracks
tracks = [track1, track2, track3]
intersections = fgt.find_track_intersections(tracks, tolerance=0.001)
for intersection in intersections:
    print(f"Intersection at {intersection.coordinate} between tracks {intersection.track_indices}")

# Calculate geographic coverage
coverage = fgt.calculate_coverage_area(tracks)
print(f"Coverage area: {coverage.area_km2:.2f} km²")
```

### Track Manipulation

Powerful track processing capabilities:

```python
import fastgeotoolkit as fgt

# Simplify tracks (reduce point density while preserving shape)
simplified = fgt.simplify_coordinates(dense_track, tolerance=0.001)
print(f"Reduced from {len(dense_track)} to {len(simplified)} points")

# Split tracks by gaps
split_tracks = fgt.split_track_by_gaps(track_with_gaps, max_gap_km=5.0)
print(f"Split into {len(split_tracks)} continuous segments")

# Filter by geographic bounds
bounds = (40.7, -74.1, 40.8, -73.9)  # NYC area
filtered = fgt.filter_coordinates_by_bounds(coordinates, bounds)

# Merge similar tracks
merged = fgt.merge_nearby_tracks(similar_tracks, distance_threshold=0.5)
```

### 📝 Format Conversion

Convert between popular GPS formats:

```python
import fastgeotoolkit as fgt

# Convert to GeoJSON
geojson = fgt.coordinates_to_geojson(coordinates, {
    "name": "My Route",
    "sport": "cycling",
    "date": "2024-01-15"
})

# Export to GPX
gpx_content = fgt.export_to_gpx([track1, track2], {
    "creator": "fastGeoToolkit",
    "version": "1.1"
})

# Encode as polyline
polyline = fgt.coordinates_to_polyline(coordinates)
print(f"Encoded polyline: {polyline}")
```

## NumPy Integration

Seamless integration with NumPy arrays:

```python
import fastgeotoolkit as fgt
import numpy as np

# Convert between NumPy arrays and coordinate lists
coords_array = np.array([[40.7128, -74.0060], [40.7589, -73.9851]])
coord_list = fgt.numpy_to_coordinates(coords_array)

# Process NumPy data
stats = fgt.calculate_track_statistics(coord_list)
simplified = fgt.simplify_coordinates(coord_list, tolerance=0.001)

# Convert back to NumPy
result_array = fgt.coordinates_to_numpy(simplified)
```

## Performance

fastGeoToolkit leverages Rust's performance for demanding geospatial tasks:

- **10-100x faster** than pure Python implementations
- **Memory efficient** processing of large GPS datasets
- **Parallel processing** for multi-track analysis
- **Optimized algorithms** for route density calculation

### Benchmarks

```python
import fastgeotoolkit as fgt
import time

# Load large dataset
start = time.time()
heatmap = fgt.load_multiple_gpx_files(large_gpx_file_list)
processing_time = time.time() - start

print(f"Processed {len(heatmap.tracks)} tracks in {processing_time:.2f} seconds")
print(f"Performance: {len(heatmap.tracks)/processing_time:.0f} tracks/second")
```

## Advanced Examples

### Route Popularity Analysis

```python
import fastgeotoolkit as fgt

# Load cycling route data
heatmap = fgt.load_multiple_gpx_files(["route1.gpx", "route2.gpx", "route3.gpx"])

# Identify popular segments
popular_segments = [
    track for track in heatmap.tracks 
    if track.frequency >= heatmap.max_frequency * 0.8
]

print(f"Found {len(popular_segments)} highly popular route segments")

# Create frequency distribution analysis
stats = fgt.calculate_heatmap_statistics(heatmap)
for freq, count in stats['frequency_distribution'].items():
    percentage = (count / stats['total_tracks']) * 100
    print(f"Frequency {freq}: {count} routes ({percentage:.1f}%)")
```

### Interactive Web Visualization

```python
import fastgeotoolkit as fgt

# Create web-ready heatmap
heatmap = fgt.load_gpx_file("mountain_bike_trails.gpx")

# Generate interactive map
map_viz = fgt.create_folium_map(
    heatmap, 
    zoom_start=14,
    colormap='viridis'
)

# Add custom styling and save
map_viz.save("trail_heatmap.html")
print("Interactive map saved to trail_heatmap.html")
```

### Batch Processing Pipeline

```python
import fastgeotoolkit as fgt
from pathlib import Path

# Process all GPX files in a directory
gpx_directory = Path("./gpx_data/")
gpx_files = list(gpx_directory.glob("*.gpx"))

# Batch process with progress tracking
print(f"Processing {len(gpx_files)} GPX files...")
heatmap = fgt.load_multiple_gpx_files(gpx_files)

# Export results in multiple formats
fgt.save_heatmap_to_geojson(heatmap, "output.geojson")
gpx_output = fgt.export_to_gpx([track.coordinates for track in heatmap.tracks])
with open("combined_routes.gpx", "w") as f:
    f.write(gpx_output)

print("Batch processing complete!")
```

## API Reference

### Core Functions

- `load_gpx_file(path)` - Load single GPX file
- `load_multiple_gpx_files(paths)` - Load multiple GPX files
- `process_gpx_files(file_data_list)` - Process GPX binary data
- `decode_polyline(encoded)` - Decode Google polyline format
- `process_polylines(polylines)` - Process multiple polylines

### Analysis Functions

- `calculate_track_statistics(coords)` - Track distance, bounds, point count
- `validate_coordinates(coords)` - Validate GPS coordinates
- `find_track_intersections(tracks, tolerance)` - Find intersection points
- `calculate_coverage_area(tracks)` - Geographic coverage analysis
- `cluster_tracks_by_similarity(tracks, threshold)` - Group similar routes

### Manipulation Functions

- `simplify_coordinates(coords, tolerance)` - Reduce point density
- `split_track_by_gaps(coords, max_gap_km)` - Split by spatial gaps
- `merge_nearby_tracks(tracks, threshold)` - Merge similar tracks
- `filter_coordinates_by_bounds(coords, bounds)` - Geographic filtering
- `resample_track(coords, target_points)` - Resample to target density

### Conversion Functions

- `coordinates_to_geojson(coords, properties)` - Export to GeoJSON
- `coordinates_to_polyline(coords)` - Encode as polyline
- `export_to_gpx(tracks, metadata)` - Export to GPX format
- `get_file_info(file_data)` - File format information

### Utility Functions

- `save_heatmap_to_geojson(heatmap, path)` - Save heatmap as GeoJSON
- `create_folium_map(heatmap)` - Create interactive map
- `visualize_heatmap(heatmap)` - Create matplotlib visualization
- `calculate_heatmap_statistics(heatmap)` - Comprehensive statistics
- `numpy_to_coordinates(array)` - NumPy array conversion
- `coordinates_to_numpy(coords)` - Convert to NumPy array

## Type Hints

fastGeoToolkit provides comprehensive type hints for better development experience:

```python
from fastgeotoolkit import (
    Coordinate,
    HeatmapResult,
    HeatmapTrack,
    ValidationResult,
    TrackStatistics,
    FileInfo
)

def process_route(coords: List[Coordinate]) -> TrackStatistics:
    return fgt.calculate_track_statistics(coords)
```

## Requirements

- Python 3.8+
- NumPy (included in requirements)

### Optional Dependencies

- `matplotlib` - For visualization
- `folium` - For interactive maps
- `geopandas` - For advanced geospatial operations
- `pandas` - For data analysis

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Performance Tips

1. **Use batch processing** for multiple files with `load_multiple_gpx_files()`
2. **Simplify tracks** before analysis to reduce computation time
3. **Set appropriate tolerances** for intersection detection
4. **Use NumPy arrays** for large coordinate datasets
5. **Cache results** of expensive operations when processing similar data

## Contributing

Contributions are welcome! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## Citation

If you use fastGeoToolkit in research, please cite:

```bibtex
@software{fastgeotoolkit2024,
  title={fastGeoToolkit: A Novel High-Performance Geospatial Analysis Framework with Advanced Route Density Mapping},
  author={a0a7},
  year={2024},
  url={https://github.com/a0a7/fastgeotoolkit},
  version={0.1.3}
}
```
