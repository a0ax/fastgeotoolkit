"""
Type definitions for fastgeotoolkit.

This module provides Python type hints for all data structures used in the library.
"""

from typing import List, Tuple, Dict, Any, Optional, Union
import numpy as np

# Basic types
Coordinate = Tuple[float, float]  # (latitude, longitude)
CoordinateArray = Union[List[Coordinate], np.ndarray]
BoundingBox = Tuple[float, float, float, float]  # (min_lat, min_lng, max_lat, max_lng)

class HeatmapTrack:
    """
    A GPS track with frequency information for heatmap visualization.
    
    Attributes:
        coordinates: List of coordinate pairs [(lat, lon), ...]
        frequency: Usage frequency of this track segment
    """
    coordinates: List[Coordinate]
    frequency: int
    
    def __init__(self, coordinates: List[Coordinate], frequency: int):
        self.coordinates = coordinates
        self.frequency = frequency

class HeatmapResult:
    """
    Result of route density heatmap generation.
    
    Attributes:
        tracks: List of tracks with frequency information
        max_frequency: Maximum frequency value across all tracks
    """
    tracks: List[HeatmapTrack]
    max_frequency: int
    
    def __init__(self, tracks: List[HeatmapTrack], max_frequency: int):
        self.tracks = tracks
        self.max_frequency = max_frequency

class ValidationResult:
    """
    Result of coordinate validation.
    
    Attributes:
        valid_count: Number of valid coordinates
        total_count: Total number of coordinates checked
        issues: List of validation issue descriptions
    """
    valid_count: int
    total_count: int
    issues: List[str]
    
    def __init__(self, valid_count: int, total_count: int, issues: List[str]):
        self.valid_count = valid_count
        self.total_count = total_count
        self.issues = issues

class TrackStatistics:
    """
    Statistical information about a GPS track.
    
    Attributes:
        distance_km: Total distance in kilometers
        point_count: Number of coordinate points
        bounding_box: Geographic bounds (min_lat, min_lng, max_lat, max_lng)
        elevation_gain: Optional elevation gain in meters
        average_speed: Optional average speed in km/h
    """
    distance_km: float
    point_count: int
    bounding_box: BoundingBox
    elevation_gain: Optional[float]
    average_speed: Optional[float]
    
    def __init__(
        self, 
        distance_km: float, 
        point_count: int, 
        bounding_box: BoundingBox,
        elevation_gain: Optional[float] = None,
        average_speed: Optional[float] = None
    ):
        self.distance_km = distance_km
        self.point_count = point_count
        self.bounding_box = bounding_box
        self.elevation_gain = elevation_gain
        self.average_speed = average_speed

class FileInfo:
    """
    Information about a GPS file format.
    
    Attributes:
        format: File format ('gpx', 'fit', etc.)
        track_count: Number of tracks in the file
        point_count: Total number of coordinate points
        valid: Whether the file is valid and parseable
        file_size: File size in bytes
    """
    format: str
    track_count: int
    point_count: int
    valid: bool
    file_size: int
    
    def __init__(
        self, 
        format: str, 
        track_count: int, 
        point_count: int, 
        valid: bool, 
        file_size: int
    ):
        self.format = format
        self.track_count = track_count
        self.point_count = point_count
        self.valid = valid
        self.file_size = file_size

class IntersectionPoint:
    """
    Point where multiple GPS tracks intersect.
    
    Attributes:
        coordinate: The intersection coordinate (lat, lon)
        track_indices: List of track indices that intersect at this point
    """
    coordinate: Coordinate
    track_indices: List[int]
    
    def __init__(self, coordinate: Coordinate, track_indices: List[int]):
        self.coordinate = coordinate
        self.track_indices = track_indices

class TrackCluster:
    """
    A cluster of similar GPS tracks.
    
    Attributes:
        representative_track: The representative track for this cluster
        member_indices: Indices of tracks belonging to this cluster
        similarity_score: Average similarity score within the cluster
    """
    representative_track: List[Coordinate]
    member_indices: List[int]
    similarity_score: float
    
    def __init__(
        self, 
        representative_track: List[Coordinate], 
        member_indices: List[int], 
        similarity_score: float
    ):
        self.representative_track = representative_track
        self.member_indices = member_indices
        self.similarity_score = similarity_score
