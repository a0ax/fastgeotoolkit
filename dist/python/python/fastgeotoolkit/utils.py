"""
Utility functions for fastGeoToolkit.

This module provides high-level Python utilities for common geospatial tasks,
built on top of the core Rust functions.
"""

import json
from pathlib import Path
from typing import List, Dict, Any, Optional, Union

try:
    import numpy as np
    HAS_NUMPY = True
except ImportError:
    HAS_NUMPY = False

try:
    import folium
    HAS_FOLIUM = True
except ImportError:
    HAS_FOLIUM = False

try:
    import matplotlib.pyplot as plt
    import matplotlib.colors as mcolors
    HAS_MATPLOTLIB = True
except ImportError:
    HAS_MATPLOTLIB = False

from .types import Coordinate, HeatmapResult, HeatmapTrack
from . import process_gpx_files, coordinates_to_geojson


def load_gpx_file(file_path: Union[str, Path]) -> HeatmapResult:
    file_path = Path(file_path)
    if not file_path.exists():
        raise FileNotFoundError(f"GPX file not found: {file_path}")
    with open(file_path, 'rb') as f:
        file_data = f.read()
    try:
        raw = process_gpx_files([file_data])
        tracks = [HeatmapTrack(track['coordinates'], track['frequency']) for track in raw['tracks']]
        return HeatmapResult(tracks, raw['max_frequency'])
    except Exception as e:
        raise ValueError(f"Failed to process GPX file {file_path}: {e}")

def load_multiple_gpx_files(file_paths: List[Union[str, Path]]) -> HeatmapResult:
    file_data_list = []
    for file_path in file_paths:
        file_path = Path(file_path)
        if not file_path.exists():
            raise FileNotFoundError(f"GPX file not found: {file_path}")
        with open(file_path, 'rb') as f:
            file_data_list.append(f.read())
    try:
        raw = process_gpx_files(file_data_list)
        tracks = [HeatmapTrack(track['coordinates'], track['frequency']) for track in raw['tracks']]
        return HeatmapResult(tracks, raw['max_frequency'])
    except Exception as e:
        raise ValueError(f"Failed to process GPX files: {e}")

def save_heatmap_to_geojson(
    heatmap: HeatmapResult, 
    output_path: Union[str, Path],
    include_frequency: bool = True
) -> None:
    """
    Save heatmap result to GeoJSON file.
    
    Args:
        heatmap: HeatmapResult to save
        output_path: Output file path
        include_frequency: Whether to include frequency in properties
    """
    output_path = Path(output_path)
    
    features = []
    for i, track in enumerate(heatmap.tracks):
        properties = {"track_id": i}
        
        if include_frequency:
            properties.update({
                "frequency": track.frequency,
                "relative_frequency": track.frequency / heatmap.max_frequency,
                "point_count": len(track.coordinates)
            })
        
        geojson_feature = coordinates_to_geojson(track.coordinates, properties)
        features.append(geojson_feature)
    
    geojson = {
        "type": "FeatureCollection",
        "features": features,
        "properties": {
            "total_tracks": len(heatmap.tracks),
            "max_frequency": heatmap.max_frequency,
            "generated_by": "fastgeotoolkit"
        }
    }
    
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(geojson, f, indent=2)


def create_folium_map(
    heatmap: HeatmapResult,
    center: Optional[Coordinate] = None,
    zoom_start: int = 12,
    colormap: str = 'YlOrRd'
) -> 'folium.Map':
    """
    Create an interactive Folium map from heatmap data.
    
    Args:
        heatmap: HeatmapResult to visualize
        center: Map center coordinate (lat, lon). If None, auto-calculated
        zoom_start: Initial zoom level
        colormap: Color scheme for frequency visualization
        
    Returns:
        Folium map object
        
    Raises:
        ImportError: If folium is not installed
    """
    if not HAS_FOLIUM:
        raise ImportError("folium is required for map visualization. Install with: pip install folium")
    
    # Calculate center if not provided
    if center is None:
        all_coords = []
        for track in heatmap.tracks:
            all_coords.extend(track.coordinates)
        
        if all_coords:
            center_lat = sum(coord[0] for coord in all_coords) / len(all_coords)
            center_lon = sum(coord[1] for coord in all_coords) / len(all_coords)
            center = (center_lat, center_lon)
        else:
            center = (0, 0)
    
    # Create base map
    m = folium.Map(location=center, zoom_start=zoom_start)
    
    # Create color scale
    max_freq = heatmap.max_frequency
    colormap_obj = folium.LinearColormap(['yellow', 'orange', 'red'], vmin=1, vmax=max_freq)
    
    # Add tracks to map
    for i, track in enumerate(heatmap.tracks):
        color = colormap_obj(track.frequency)
        weight = 2 + (track.frequency / max_freq) * 6  # Line width 2-8 based on frequency
        
        folium.PolyLine(
            locations=track.coordinates,
            color=color,
            weight=weight,
            opacity=0.7,
            popup=f"Track {i+1}: Frequency {track.frequency}"
        ).add_to(m)
    
    # Add colormap legend
    colormap_obj.caption = 'Route Frequency'
    colormap_obj.add_to(m)
    
    return m


def visualize_heatmap(
    heatmap: HeatmapResult,
    figsize: tuple = (12, 8),
    colormap: str = 'hot',
    show_frequency_distribution: bool = True
) -> None:
    """
    Create matplotlib visualization of heatmap data.
    
    Args:
        heatmap: HeatmapResult to visualize
        figsize: Figure size (width, height)
        colormap: Matplotlib colormap name
        show_frequency_distribution: Whether to show frequency histogram
        
    Raises:
        ImportError: If matplotlib is not installed
    """
    if not HAS_MATPLOTLIB:
        raise ImportError("matplotlib is required for visualization. Install with: pip install matplotlib")
    
    if show_frequency_distribution:
        fig, (ax1, ax2) = plt.subplots(1, 2, figsize=figsize)
    else:
        fig, ax1 = plt.subplots(1, 1, figsize=figsize)
    
    # Main heatmap plot
    max_freq = heatmap.max_frequency
    cmap = plt.get_cmap(colormap)
    
    for track in heatmap.tracks:
        coords = track.coordinates
        if len(coords) < 2:
            continue
            
        lats = [coord[0] for coord in coords]
        lons = [coord[1] for coord in coords]
        
        # Color based on frequency
        color_intensity = track.frequency / max_freq
        color = cmap(color_intensity)
        
        # Line width based on frequency
        linewidth = 1 + (track.frequency / max_freq) * 3
        
        ax1.plot(lons, lats, color=color, linewidth=linewidth, alpha=0.7)
    
    ax1.set_xlabel('Longitude')
    ax1.set_ylabel('Latitude')
    ax1.set_title('Route Density Heatmap')
    ax1.grid(True, alpha=0.3)
    
    # Add colorbar
    norm = mcolors.Normalize(vmin=1, vmax=max_freq)
    sm = plt.cm.ScalarMappable(cmap=cmap, norm=norm)
    sm.set_array([])
    cbar = plt.colorbar(sm, ax=ax1)
    cbar.set_label('Route Frequency')
    
    # Frequency distribution
    if show_frequency_distribution:
        frequencies = [track.frequency for track in heatmap.tracks]
        ax2.hist(frequencies, bins=min(20, max_freq), color='skyblue', alpha=0.7, edgecolor='black')
        ax2.set_xlabel('Frequency')
        ax2.set_ylabel('Number of Routes')
        ax2.set_title('Frequency Distribution')
        ax2.grid(True, alpha=0.3)
    
    plt.tight_layout()
    plt.show()


def calculate_heatmap_statistics(heatmap: HeatmapResult) -> Dict[str, Any]:
    """
    Calculate comprehensive statistics from heatmap data.
    
    Args:
        heatmap: HeatmapResult to analyze
        
    Returns:
        Dictionary with statistical information
    """
    tracks = heatmap.tracks
    frequencies = [track.frequency for track in tracks]
    point_counts = [len(track.coordinates) for track in tracks]
    
    # Basic statistics
    stats = {
        'total_tracks': len(tracks),
        'max_frequency': heatmap.max_frequency,
        'min_frequency': min(frequencies) if frequencies else 0,
        'mean_frequency': sum(frequencies) / len(frequencies) if frequencies else 0,
        'total_points': sum(point_counts),
        'mean_points_per_track': sum(point_counts) / len(point_counts) if point_counts else 0,
    }
    
    # Frequency distribution
    freq_distribution = {}
    for freq in frequencies:
        freq_distribution[freq] = freq_distribution.get(freq, 0) + 1
    
    stats['frequency_distribution'] = freq_distribution
    
    # Calculate bounding box for all tracks
    if tracks:
        all_coords = []
        for track in tracks:
            all_coords.extend(track.coordinates)
        
        if all_coords:
            lats = [coord[0] for coord in all_coords]
            lons = [coord[1] for coord in all_coords]
            stats['bounding_box'] = {
                'min_lat': min(lats),
                'max_lat': max(lats),
                'min_lon': min(lons),
                'max_lon': max(lons)
            }
    
    return stats


def numpy_to_coordinates(array: 'np.ndarray') -> List[Coordinate]:
    """
    Convert numpy array to coordinate list.
    
    Args:
        array: Numpy array of shape (N, 2) with [lat, lon] pairs
        
    Returns:
        List of coordinate tuples
        
    Raises:
        ImportError: If numpy is not installed
        ValueError: If array has wrong shape
    """
    if not HAS_NUMPY:
        raise ImportError("numpy is required for array conversion. Install with: pip install numpy")
    
    if array.ndim != 2 or array.shape[1] != 2:
        raise ValueError("Array must have shape (N, 2) with [lat, lon] pairs")
    
    return [(float(row[0]), float(row[1])) for row in array]


def coordinates_to_numpy(coordinates: List[Coordinate]) -> 'np.ndarray':
    """
    Convert coordinate list to numpy array.
    
    Args:
        coordinates: List of coordinate tuples
        
    Returns:
        Numpy array of shape (N, 2)
        
    Raises:
        ImportError: If numpy is not installed
    """
    if not HAS_NUMPY:
        raise ImportError("numpy is required for array conversion. Install with: pip install numpy")
    
    return np.array(coordinates, dtype=np.float64)
