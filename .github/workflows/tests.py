"""Comprehensive test suite for fastgeotoolkit."""

import json
import tempfile
from pathlib import Path

import pytest

from fastgeotoolkit import (
    process_gpx_files,
    decode_polyline,
    process_polyline,
    process_polylines,
    calculate_track_statistics,
    validate_coordinates,
    find_track_intersections,
    calculate_coverage_area,
    cluster_tracks_by_similarity,
    simplify_coordinates,
    merge_nearby_tracks,
    split_track_by_gaps,
    filter_coordinates_by_bounds,
    coordinates_to_geojson,
    coordinates_to_polyline,
    export_to_gpx,
    calculate_distance_between_points,
    get_bounding_box,
    resample_track,
    get_file_info,
    extract_file_metadata,
    load_gpx_file,
    load_multiple_gpx_files,
    save_heatmap_to_geojson,
    visualize_heatmap,
    create_folium_map,
    calculate_heatmap_statistics,
    HeatmapResult,
    HeatmapTrack,
)
from fastgeotoolkit import __version__

# -------------------- Test Data --------------------

POLYLINE_EXAMPLE = "_p~iF~ps|U_ulLnnqC_mqNvxq`@"
SAMPLE_COORDS = [(48.8584, 2.2945), (40.7128, -74.0060), (34.0522, -118.2437)]

# (0,0) is now considered valid
VALID_COORDS = [(0.0, 0.0), (45.0, 90.0), (-45.0, -90.0)]
INVALID_COORDS = [(91.0, 0.0), (0.0, 181.0), (float('nan'), 0.0)]

GPX_CONTENT = """<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="test">
  <trk>
    <name>Test Track</name>
    <trkseg>
      <trkpt lat="48.8584" lon="2.2945"/>
      <trkpt lat="48.8600" lon="2.2960"/>
    </trkseg>
  </trk>
</gpx>"""

# -------------------- Helpers --------------------

@pytest.fixture
def temp_gpx_file(tmp_path):
    file_path = tmp_path / "test.gpx"
    file_path.write_text(GPX_CONTENT)
    return str(file_path)

@pytest.fixture
def temp_gpx_files(tmp_path):
    paths = []
    for i in range(2):
        p = tmp_path / f"test_{i}.gpx"
        p.write_text(GPX_CONTENT)
        paths.append(str(p))
    return paths

# -------------------- Tests for Core Functions --------------------

def test_version():
    assert __version__ is not None

def test_decode_polyline():
    coords = decode_polyline(POLYLINE_EXAMPLE)
    assert isinstance(coords, list)
    assert len(coords) > 0
    assert all(isinstance(c, tuple) and len(c) == 2 for c in coords)
    assert decode_polyline("") == []

def test_validate_coordinates():
    valid, issues = validate_coordinates(VALID_COORDS)
    assert valid == len(VALID_COORDS)
    assert issues == []
    valid, issues = validate_coordinates(INVALID_COORDS)
    assert valid == 0
    assert len(issues) == 3
    mixed = [(0.0, 0.0), (91.0, 0.0), (45.0, 90.0)]
    valid, issues = validate_coordinates(mixed)
    assert valid == 2
    assert len(issues) == 1

def test_calculate_track_statistics():
    coords = [(0.0, 0.0), (0.0, 1.0), (0.0, 2.0)]
    stats = calculate_track_statistics(coords)
    assert stats is not None
    distance, point_count, bbox = stats
    assert point_count == 3
    assert distance > 0
    assert bbox == [0.0, 0.0, 0.0, 2.0]
    assert calculate_track_statistics([]) is None

def test_simplify_coordinates():
    coords = [(0.0, 0.0), (0.001, 0.001), (0.002, 0.002), (0.01, 0.01)]
    simplified = simplify_coordinates(coords, tolerance=0.005)
    assert len(simplified) >= 2
    assert simplified[0] == coords[0]
    assert simplified[-1] == coords[-1]

def test_filter_coordinates_by_bounds():
    coords = [(0.0, 0.0), (1.0, 1.0), (-1.0, -1.0)]
    filtered = filter_coordinates_by_bounds(coords, [0.0, 0.0, 2.0, 2.0])
    assert filtered == [(0.0, 0.0), (1.0, 1.0)]
    filtered = filter_coordinates_by_bounds(coords, [0.0, 0.0, 0.0, 0.0])
    assert filtered == [(0.0, 0.0)]

def test_coordinates_to_polyline():
    coords = [(38.5, -120.2), (40.7, -73.9)]
    encoded = coordinates_to_polyline(coords)
    assert isinstance(encoded, str)
    assert len(encoded) > 0
    decoded = decode_polyline(encoded)
    for (lat1, lon1), (lat2, lon2) in zip(coords, decoded):
        assert abs(lat1 - lat2) < 1e-5
        assert abs(lon1 - lon2) < 1e-5

def test_get_bounding_box():
    coords = [(0.0, 0.0), (1.0, 2.0), (-1.0, -1.0)]
    bbox = get_bounding_box(coords)
    assert bbox == [-1.0, -1.0, 1.0, 2.0]
    bbox = get_bounding_box([(5.0, 5.0)])
    assert bbox == [5.0, 5.0, 5.0, 5.0]

def test_calculate_distance_between_points():
    dist = calculate_distance_between_points(0.0, 0.0, 0.0, 1.0)
    assert 110.0 < dist < 112.0
    assert calculate_distance_between_points(0.0, 0.0, 0.0, 0.0) == 0.0

def test_process_polyline_json():
    # Use two points close together (≈ 0.2 km apart) so filtering doesn't drop them
    json_str = json.dumps([[48.8584, 2.2945], [48.8600, 2.2960]])
    coords = process_polyline(json_str)
    assert len(coords) == 2
    # Encoded format
    coords = process_polyline(POLYLINE_EXAMPLE)
    assert len(coords) > 0

def test_split_track_by_gaps():
    coords = [(0.0, 0.0), (0.0, 0.1), (0.0, 0.2), (0.0, 1.0), (0.0, 1.1)]
    split = split_track_by_gaps(coords, max_gap_km=50.0)
    assert len(split) == 2
    assert len(split[0]) == 3
    assert len(split[1]) == 2

def test_merge_nearby_tracks():
    track1 = [(0.0, 0.0), (0.0, 1.0)]
    track2 = [(0.001, 0.001), (0.001, 1.001)]
    track3 = [(10.0, 10.0), (10.0, 11.0)]
    merged = merge_nearby_tracks([track1, track2, track3], distance_threshold=1.0)
    assert len(merged) == 2

def test_find_track_intersections():
    # Make both tracks share a common explicit point (0, 0.5)
    track1 = [(0.0, 0.0), (0.0, 0.5), (0.0, 1.0)]
    track2 = [(-0.5, 0.5), (0.0, 0.5), (0.5, 0.5)]
    intersections = find_track_intersections([track1, track2], tolerance=0.1)
    assert len(intersections) >= 1
    point, indices = intersections[0]
    assert len(indices) == 2

def test_calculate_coverage_area():
    tracks = [[(0.0, 0.0), (0.0, 1.0), (1.0, 0.0)]]
    area = calculate_coverage_area(tracks)
    assert area is not None
    bbox, area_km2, point_count = area
    assert area_km2 > 0
    assert point_count == 3

def test_cluster_tracks_by_similarity():
    track1 = [(0.0, 0.0), (0.0, 1.0)]
    track2 = [(0.001, 0.001), (0.001, 1.001)]
    track3 = [(10.0, 10.0), (10.0, 11.0)]
    clusters = cluster_tracks_by_similarity([track1, track2, track3], similarity_threshold=0.8)
    assert len(clusters) == 2

def test_resample_track():
    coords = [(0.0, 0.0), (0.0, 0.1), (0.0, 0.2), (0.0, 0.3)]
    resampled = resample_track(coords, target_point_count=2)
    assert len(resampled) >= 2
    assert resampled[0] == coords[0]
    assert resampled[-1] == coords[-1]

def test_export_to_gpx():
    tracks = [[(48.8584, 2.2945), (48.8600, 2.2960)]]
    gpx_str = export_to_gpx(tracks)
    assert "<gpx" in gpx_str
    assert "trkpt" in gpx_str
    assert 'lat="48.858400"' in gpx_str
    assert 'lon="2.294500"' in gpx_str

def test_coordinates_to_geojson():
    coords = [(48.8584, 2.2945), (48.8600, 2.2960)]
    feature = coordinates_to_geojson(coords, {"name": "test"})
    assert feature["type"] == "Feature"
    assert feature["geometry"]["type"] == "LineString"
    assert len(feature["geometry"]["coordinates"]) == 2
    assert feature["properties"]["name"] == "test"
    feature = coordinates_to_geojson(coords)
    assert feature["properties"] == {}

def test_get_file_info(temp_gpx_file):
    with open(temp_gpx_file, 'rb') as f:
        data = f.read()
    info = get_file_info(data)
    assert info["format"] == "gpx"
    assert info["valid"] is True
    assert info["track_count"] == 1
    assert info["point_count"] >= 2
    assert info["file_size"] > 0

def test_extract_file_metadata(temp_gpx_file):
    with open(temp_gpx_file, 'rb') as f:
        data = f.read()
    metadata = extract_file_metadata(data)
    assert metadata["format"] == "gpx"
    assert metadata["creator"] == "test"
    assert "tracks" in metadata
    assert len(metadata["tracks"]) == 1
    assert metadata["tracks"][0]["segment_count"] == 1

def test_process_gpx_files(temp_gpx_file):
    with open(temp_gpx_file, 'rb') as f:
        data = f.read()
    result = process_gpx_files([data])
    assert "tracks" in result
    assert "max_frequency" in result
    assert len(result["tracks"]) > 0

def test_process_polylines():
    coords = [(0.0, 0.0), (0.0, 0.1)]
    poly = coordinates_to_polyline(coords)
    result = process_polylines([poly, poly])
    assert "tracks" in result
    assert "max_frequency" in result
    assert len(result["tracks"]) >= 2

# -------------------- Tests for Python Utilities --------------------

def test_load_gpx_file(temp_gpx_file):
    result = load_gpx_file(temp_gpx_file)
    assert isinstance(result, HeatmapResult)
    assert len(result.tracks) >= 1
    assert result.max_frequency >= 1
    track = result.tracks[0]
    assert isinstance(track, HeatmapTrack)
    assert len(track.coordinates) >= 2
    assert track.frequency >= 1

def test_load_multiple_gpx_files(temp_gpx_files):
    result = load_multiple_gpx_files(temp_gpx_files)
    assert isinstance(result, HeatmapResult)
    assert len(result.tracks) >= len(temp_gpx_files)

def test_save_heatmap_to_geojson(tmp_path, temp_gpx_file):
    heatmap = load_gpx_file(temp_gpx_file)
    output = tmp_path / "output.geojson"
    save_heatmap_to_geojson(heatmap, output)
    assert output.exists()
    with open(output) as f:
        data = json.load(f)
    assert data["type"] == "FeatureCollection"
    assert "features" in data
    assert len(data["features"]) >= 1
    assert data["properties"]["total_tracks"] >= 1
    assert data["properties"]["max_frequency"] >= 1

def test_calculate_heatmap_statistics(temp_gpx_file):
    heatmap = load_gpx_file(temp_gpx_file)
    stats = calculate_heatmap_statistics(heatmap)
    assert stats["total_tracks"] >= 1
    assert stats["max_frequency"] >= 1
    assert stats["total_points"] >= 2
    assert "bounding_box" in stats

# -------------------- Tests for Optional Dependencies --------------------

@pytest.mark.skipif(not hasattr(visualize_heatmap, '__call__'), reason="matplotlib not installed")
def test_visualize_heatmap(temp_gpx_file):
    heatmap = load_gpx_file(temp_gpx_file)
    visualize_heatmap(heatmap, show_frequency_distribution=True)

@pytest.mark.skipif(not hasattr(create_folium_map, '__call__'), reason="folium not installed")
def test_create_folium_map(temp_gpx_file):
    heatmap = load_gpx_file(temp_gpx_file)
    m = create_folium_map(heatmap)
    assert m is not None

# -------------------- Tests for Error Handling --------------------

def test_load_gpx_file_not_found():
    with pytest.raises(FileNotFoundError):
        load_gpx_file("nonexistent.gpx")

def test_load_multiple_gpx_files_not_found():
    with pytest.raises(FileNotFoundError):
        load_multiple_gpx_files(["nonexistent1.gpx", "nonexistent2.gpx"])

def test_invalid_bounds():
    with pytest.raises(Exception):
        filter_coordinates_by_bounds(SAMPLE_COORDS, [0, 0])

# -------------------- NumPy integration tests (optional) --------------------

@pytest.mark.skipif(not hasattr(load_gpx_file, '__call__'), reason="numpy not installed")
def test_numpy_conversion():
    from fastgeotoolkit import numpy_to_coordinates, coordinates_to_numpy
    import numpy as np
    arr = np.array([[48.8584, 2.2945], [48.8600, 2.2960]])
    coords = numpy_to_coordinates(arr)
    assert len(coords) == 2
    assert isinstance(coords[0], tuple)
    arr2 = coordinates_to_numpy(coords)
    np.testing.assert_array_almost_equal(arr, arr2)

# -------------------- Run Tests --------------------

if __name__ == "__main__":
    pytest.main([__file__])