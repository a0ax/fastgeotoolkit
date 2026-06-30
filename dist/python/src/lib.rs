use gpx::read;
use std::io::Cursor;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use pyo3::prelude::*;
use pyo3::types::PyModule;
use pyo3::Bound;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::types::{PyList, PyDict};

// DATA STRUCTURES
#[derive(Serialize)]
pub struct HeatmapTrack {
    pub coordinates: Vec<[f64; 2]>,
    pub frequency: u32,
}

#[derive(Serialize)]
pub struct HeatmapResult {
    pub tracks: Vec<HeatmapTrack>,
    pub max_frequency: u32,
}

#[derive(Serialize, Deserialize)]
pub struct ValidationResult {
    valid_count: u32,
    total_count: u32,
    issues: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TrackStatistics {
    distance_km: f64,
    point_count: u32,
    bounding_box: [f64; 4], // [min_lat, min_lng, max_lat, max_lng]
    elevation_gain: Option<f64>,
    average_speed: Option<f64>,
}

#[derive(Serialize, Deserialize)]
pub struct FileInfo {
    format: String,
    track_count: u32,
    point_count: u32,
    valid: bool,
    file_size: u32,
}

#[derive(Serialize, Deserialize)]
pub struct IntersectionPoint {
    coordinate: [f64; 2],
    track_indices: Vec<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct TrackCluster {
    representative_track: Vec<[f64; 2]>,
    member_indices: Vec<u32>,
    similarity_score: f64,
}

// debugging
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// decode Google polyline format
pub fn decode_polyline(encoded: &str) -> Vec<[f64; 2]> {
    let mut coords = Vec::new();
    let mut lat = 0i32;
    let mut lng = 0i32;
    let mut index = 0;
    let bytes = encoded.as_bytes();
    
    while index < bytes.len() {
        // Decode latitude
        let mut shift = 0;
        let mut result = 0i32;
        loop {
            if index >= bytes.len() {
                break;
            }
            let b = bytes[index] as i32 - 63;
            index += 1;
            result |= (b & 0x1f) << shift;
            shift += 5;
            if b < 0x20 {
                break;
            }
        }
        let dlat = if (result & 1) != 0 { !(result >> 1) } else { result >> 1 };
        lat += dlat;
        
        // Decode longitude
        shift = 0;
        result = 0;
        loop {
            if index >= bytes.len() {
                break;
            }
            let b = bytes[index] as i32 - 63;
            index += 1;
            result |= (b & 0x1f) << shift;
            shift += 5;
            if b < 0x20 {
                break;
            }
        }
        let dlng = if (result & 1) != 0 { !(result >> 1) } else { result >> 1 };
        lng += dlng;
        
        // Convert to lat/lng and add to coordinates
        let lat_f64 = lat as f64 * 1e-5;
        let lng_f64 = lng as f64 * 1e-5;
        
        if is_valid_coordinate(lat_f64, lng_f64) {
            coords.push([lat_f64, lng_f64]);
        }
    }
    
    coords
}

// wasm export for polyline decoding
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]

pub fn decode_polyline_string(encoded: &str) -> JsValue {
    let coords = decode_polyline(encoded);
    serde_wasm_bindgen::to_value(&coords).unwrap()
}

// polyline processingstep
pub fn process_polyline(polyline_str: &str) -> Vec<[f64; 2]> {
    // First try to parse as JSON (RideWithGPS format)
    if let Ok(json_coords) = serde_json::from_str::<Vec<[f64; 2]>>(polyline_str) {
        // It's a JSON array of coordinates
        return if !json_coords.is_empty() {
            filter_unrealistic_jumps(&json_coords)
        } else {
            Vec::new()
        };
    }
    
    // If JSON parsing fails, treat as encoded polyline (Strava format)
    let coords = decode_polyline(polyline_str);
    if !coords.is_empty() {
        filter_unrealistic_jumps(&coords)
    } else {
        Vec::new()
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn process_polylines(polylines: js_sys::Array) -> JsValue {
    let mut all_tracks: Vec<Vec<[f64; 2]>> = Vec::new();

    for i in 0..polylines.length() {
        if let Some(polyline_str) = polylines.get(i).as_string() {
            let coords = process_polyline(&polyline_str);
            if coords.len() > 1 {
                let simplified = simplify_track(&coords, 0.00005);
                if simplified.len() > 1 {
                    all_tracks.push(simplified);
                }
            }
        }
    }

    // Apply the same processing logic as GPX files
    let result = create_heatmap_from_tracks(all_tracks);
    
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

// Helper function to create heatmap from coordinate arrays
pub fn create_heatmap_from_tracks(all_tracks: Vec<Vec<[f64; 2]>>) -> HeatmapResult {
    // Create a segment usage map to count overlapping segments
    let mut segment_usage: HashMap<String, u32> = HashMap::new();
    
    // Break each track into segments and count usage
    for track in &all_tracks {
        for window in track.windows(2) {
            if let [start, end] = window {
                let segment_key = create_segment_key(*start, *end);
                *segment_usage.entry(segment_key).or_insert(0) += 1;
            }
        }
    }
    
    // Calculate frequency for each track based on its segments
    let mut heatmap_tracks = Vec::new();
    
    for track in all_tracks {
        if track.len() < 2 {
            continue;
        }
        
        // Calculate track frequency as the average frequency of its segments
        let mut total_usage = 0;
        let mut segment_count = 0;
        
        for window in track.windows(2) {
            if let [start, end] = window {
                let segment_key = create_segment_key(*start, *end);
                if let Some(&usage) = segment_usage.get(&segment_key) {
                    total_usage += usage;
                    segment_count += 1;
                }
            }
        }
        
        // Use average usage, with minimum of 1
        let track_frequency = if segment_count > 0 {
            (total_usage as f64 / segment_count as f64).round() as u32
        } else {
            1
        };
        
        heatmap_tracks.push(HeatmapTrack {
            coordinates: track,
            frequency: track_frequency,
        });
    }
    
    // Find the maximum frequency for normalization
    let max_frequency = heatmap_tracks.iter()
        .map(|track| track.frequency)
        .max()
        .unwrap_or(1);
    
    HeatmapResult {
        tracks: heatmap_tracks,
        max_frequency,
    }
}

pub fn round(value: f64) -> f64 {
    (value * 100000.0).round() / 100000.0
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn process_gpx_files(files: js_sys::Array) -> JsValue {
    let mut all_tracks: Vec<Vec<[f64; 2]>> = Vec::new();
    
    // Parse all GPX and FIT files and extract tracks
    for file_bytes in files.iter() {
        let array = js_sys::Uint8Array::new(&file_bytes);
        let bytes = array.to_vec();

        // Try to parse as GPX first
        if let Ok(gpx) = read(Cursor::new(&bytes)) {
            for track in gpx.tracks {
                for segment in track.segments {
                    let mut track_coords = Vec::new();
                    
                    for point in segment.points {
                        let lat = round(point.point().y());
                        let lon = round(point.point().x());
                        
                        // Validate coordinates to prevent globe-spanning lines
                        if is_valid_coordinate(lat, lon) {
                            track_coords.push([lat, lon]);
                        }
                    }
                    
                    if track_coords.len() > 1 {
                        // Filter out tracks with unrealistic jumps
                        let filtered_coords = filter_unrealistic_jumps(&track_coords);
                        
                        if filtered_coords.len() > 1 {
                            // Less aggressive simplification to preserve track shape
                            let simplified = simplify_track(&filtered_coords, 0.00005);
                            if simplified.len() > 1 {
                                all_tracks.push(simplified);
                            }
                        }
                    }
                }
            }
        }
        // Try to parse as FIT file if GPX parsing fails
        else if is_fit_file(&bytes) {
            // Custom FIT file parser for extracting GPS coordinates
            let mut fit_parser = FitParser::new(bytes);
            let fit_coordinates = fit_parser.parse_gps_coordinates();
            
            // Apply the same validation and filtering as GPX
            if fit_coordinates.len() > 1 {
                let filtered_coords = filter_unrealistic_jumps(&fit_coordinates);
                
                if filtered_coords.len() > 1 {
                    let simplified = simplify_track(&filtered_coords, 0.00005);
                    if simplified.len() > 1 {
                        all_tracks.push(simplified);
                    }
                }
            }
        }
        // Skip files that aren't GPX or FIT
        else {
            continue;
        }
    }
    
    // create segment usage map to count overlapping segments
    let mut segment_usage: HashMap<String, u32> = HashMap::new();
    
    // break each track into segments and count usage
    for track in &all_tracks {
        for window in track.windows(2) {
            if let [start, end] = window {
                let segment_key = create_segment_key(*start, *end);
                *segment_usage.entry(segment_key).or_insert(0) += 1;
            }
        }
    }
    
    // calculate frequency for each track based on its segments
    let mut heatmap_tracks = Vec::new();
    let mut max_frequency = 0;
    
    for track in all_tracks {
        if track.len() < 2 {
            continue;
        }
        
        // calculate track frequency as the average frequency of its segments
        let mut total_usage = 0;
        let mut segment_count = 0;
        
        for window in track.windows(2) {
            if let [start, end] = window {
                let segment_key = create_segment_key(*start, *end);
                if let Some(&usage) = segment_usage.get(&segment_key) {
                    total_usage += usage;
                    segment_count += 1;
                }
            }
        }
        
        // Use average usage, with minimum of 1
        let track_frequency = if segment_count > 0 {
            (total_usage as f64 / segment_count as f64).round() as u32
        } else {
            1
        };
        
        max_frequency = max_frequency.max(track_frequency);
        
        heatmap_tracks.push(HeatmapTrack {
            coordinates: track,
            frequency: track_frequency,
        });
    }
    
    let result = HeatmapResult {
        tracks: heatmap_tracks,
        max_frequency,
    };
    
    serde_wasm_bindgen::to_value(&result).unwrap()
}

pub fn create_segment_key(start: [f64; 2], end: [f64; 2]) -> String {
    // change to a larger tolerance for less aggressive matching
    let tolerance = 0.001; // About 100 meters
    let snap_start = snap_to_grid(start, tolerance);
    let snap_end = snap_to_grid(end, tolerance);
    
    // Normalize direction (smaller coordinate first)
    let (p1, p2) = if (snap_start[0], snap_start[1]) < (snap_end[0], snap_end[1]) {
        (snap_start, snap_end)
    } else {
        (snap_end, snap_start)
    };
    
    format!("{:.4},{:.4}-{:.4},{:.4}", p1[0], p1[1], p2[0], p2[1])
}

pub fn snap_to_grid(point: [f64; 2], tolerance: f64) -> [f64; 2] {
    [
        (point[0] / tolerance).round() * tolerance,
        (point[1] / tolerance).round() * tolerance,
    ]
}

fn simplify_track(points: &[[f64; 2]], tolerance: f64) -> Vec<[f64; 2]> {
    if points.len() <= 2 {
        return points.to_vec();
    }
    
    let mut result = vec![points[0]];
    let mut last_added = 0;
    
    for i in 1..points.len() {
        let distance = distance(points[last_added], points[i]);
        
        // Add point if it's far enough from the last added point
        // or if it's the last point in the track
        if distance > tolerance || i == points.len() - 1 {
            result.push(points[i]);
            last_added = i;
        }
    }
    
    result
}

fn distance(p1: [f64; 2], p2: [f64; 2]) -> f64 {
    let dx = p1[0] - p2[0];
    let dy = p1[1] - p2[1];
    (dx * dx + dy * dy).sqrt()
}

fn is_valid_coordinate(lat: f64, lon: f64) -> bool {
    // Check for valid latitude and longitude ranges
    if lat < -90.0 || lat > 90.0 || lon < -180.0 || lon > 180.0 {
        return false;
    }
    
    // Check for obviously invalid coordinates
    if (lat == 0.0 && lon == 0.0) || lat.is_nan() || lon.is_nan() || lat.is_infinite() || lon.is_infinite() {
        return false;
    }
    
    true
}

pub fn filter_unrealistic_jumps(coords: &[[f64; 2]]) -> Vec<[f64; 2]> {
    if coords.len() <= 1 {
        return coords.to_vec();
    }
    
    let mut filtered = vec![coords[0]];
    let max_jump_km = 100.0;
    let mut consecutive_bad_points = 0;
    const MAX_CONSECUTIVE_BAD: usize = 10; // Allow up to 10 consecutive bad points
    
    for i in 1..coords.len() {
        let prev = filtered.last().unwrap();
        let curr = coords[i];
        
        // Calculate approximate distance in kilometers using Haversine formula
        let distance_km = haversine_distance(prev[0], prev[1], curr[0], curr[1]);
        
        // Only add point if it's within reasonable distance from previous point
        if distance_km <= max_jump_km {
            filtered.push(curr);
            consecutive_bad_points = 0; // Reset bad point counter
        } else {
            consecutive_bad_points += 1;
            
            // If we've seen too many consecutive bad points, try to find good data ahead
            if consecutive_bad_points <= MAX_CONSECUTIVE_BAD {
                // Look ahead up to 20 points to see if we can find a reasonable continuation
                let mut found_good_continuation = false;
                for j in (i + 1)..(i + 21).min(coords.len()) {
                    let future_point = coords[j];
                    let future_distance = haversine_distance(prev[0], prev[1], future_point[0], future_point[1]);
                    
                    // If we find a reasonable point ahead, it suggests this is just a GPS glitch
                    if future_distance <= max_jump_km * 1.5 { // Allow 1.5x distance for bridging
                        found_good_continuation = true;
                        break;
                    }
                }
                
                // If no good continuation found, we might be at the end of good data
                if !found_good_continuation {
                    // Try to find any remaining good segments by continuing to filter the rest
                    for k in (i + 1)..coords.len() {
                        let remaining_point = coords[k];
                        let remaining_distance = haversine_distance(prev[0], prev[1], remaining_point[0], remaining_point[1]);
                        
                        // If we find a reasonable point, start a new segment from there
                        if remaining_distance <= max_jump_km {
                            filtered.push(remaining_point);
                            // Continue filtering from this new point
                            for m in (k + 1)..coords.len() {
                                let next_prev = filtered.last().unwrap();
                                let next_curr = coords[m];
                                let next_distance = haversine_distance(next_prev[0], next_prev[1], next_curr[0], next_curr[1]);
                                
                                if next_distance <= max_jump_km {
                                    filtered.push(next_curr);
                                }
                                // Skip points that are too far, but don't break - keep looking
                            }
                            break; // We've processed the rest of the array
                        }
                    }
                    break; // Exit the main loop as we've processed everything
                }
            } else {
                // Too many consecutive bad points - stop processing to avoid bad data
                break;
            }
            // If there is a good continuation, just skip this point and continue
        }
    }
    
    filtered
}

fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0; // Earth's radius in kilometers
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    
    let a = (d_lat / 2.0).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    
    r * c
}

// FIT file format reference: https://developer.garmin.com/fit/protocol/

pub struct FitParser {
    data: Vec<u8>,
    pos: usize,
    message_definitions: HashMap<u8, MessageDefinition>,
}

#[derive(Clone)]
pub struct MessageDefinition {
    global_message_number: u16,
    fields: Vec<FieldDefinition>,
}

#[derive(Clone)]
pub struct FieldDefinition {
    field_def_num: u8,
    size: u8,
    _base_type: u8,
}

impl FitParser {
    pub fn new(data: Vec<u8>) -> Self {
        Self { 
            data, 
            pos: 0,
            message_definitions: HashMap::new(),
        }
    }

    fn read_u8(&mut self) -> Option<u8> {
        if self.pos < self.data.len() {
            let val = self.data[self.pos];
            self.pos += 1;
            Some(val)
        } else {
            None
        }
    }

    fn read_u16_le(&mut self) -> Option<u16> {
        if self.pos + 1 < self.data.len() {
            let val = u16::from_le_bytes([self.data[self.pos], self.data[self.pos + 1]]);
            self.pos += 2;
            Some(val)
        } else {
            None
        }
    }

    fn read_u32_le(&mut self) -> Option<u32> {
        if self.pos + 3 < self.data.len() {
            let val = u32::from_le_bytes([
                self.data[self.pos],
                self.data[self.pos + 1],
                self.data[self.pos + 2],
                self.data[self.pos + 3],
            ]);
            self.pos += 4;
            Some(val)
        } else {
            None
        }
    }

    fn read_i32_le(&mut self) -> Option<i32> {
        if self.pos + 3 < self.data.len() {
            let val = i32::from_le_bytes([
                self.data[self.pos],
                self.data[self.pos + 1],
                self.data[self.pos + 2],
                self.data[self.pos + 3],
            ]);
            self.pos += 4;
            Some(val)
        } else {
            None
        }
    }

    fn skip(&mut self, bytes: usize) {
        self.pos = (self.pos + bytes).min(self.data.len());
    }

    pub fn parse_gps_coordinates(&mut self) -> Vec<[f64; 2]> {
        let mut coordinates = Vec::new();

        // Check FIT file header
        if self.data.len() < 14 {
            return coordinates;
        }

        // FIT file header (14 bytes)
        let header_size = self.read_u8().unwrap_or(0);
        if header_size < 12 {
            return coordinates;
        }

        let _protocol_version = self.read_u8().unwrap_or(0);
        let _profile_version = self.read_u16_le().unwrap_or(0);
        let data_size = self.read_u32_le().unwrap_or(0);
        
        // Check for ".FIT" signature
        let signature = [
            self.read_u8().unwrap_or(0),
            self.read_u8().unwrap_or(0),
            self.read_u8().unwrap_or(0),
            self.read_u8().unwrap_or(0),
        ];
        if signature != [b'.', b'F', b'I', b'T'] {
            return coordinates;
        }

        // Skip header CRC if present
        if header_size == 14 {
            self.skip(2);
        }

        // Calculate data end position, but also consider that some FIT files 
        // might have the data_size field incorrect, so we'll try to parse until
        // we reach the actual end of the file (minus CRC bytes)
        let header_data_end = (self.pos + data_size as usize).min(self.data.len());
        let file_data_end = self.data.len().saturating_sub(2); // Leave 2 bytes for CRC at end
        let data_end = header_data_end.max(file_data_end); // Use the larger of the two
        
        let mut consecutive_errors = 0;
        const MAX_CONSECUTIVE_ERRORS: usize = 100; // Allow more errors before giving up
        let mut processed_bytes = 0;
        let mut last_progress_pos = self.pos;

        // Parse data records - continue until we reach the end or hit too many errors
        while self.pos < data_end && self.pos < self.data.len() && self.pos + 1 < self.data.len() {
            let start_pos = self.pos;
            
            // Every 10,000 bytes, check if we're making progress
            if self.pos - last_progress_pos > 10000 {
                processed_bytes += self.pos - last_progress_pos;
                last_progress_pos = self.pos;
                
                // If we've processed a lot of data and found some coordinates, we're probably doing well
                if coordinates.len() > 100 && processed_bytes > 50000 {
                    consecutive_errors = 0; // Reset error count as we're clearly making progress
                }
            }
            
            // Ensure we have at least 1 byte to read
            if self.pos >= self.data.len() {
                break;
            }
            
            let record_header = match self.read_u8() {
                Some(header) => header,
                None => break, // End of data
            };

            let is_definition = (record_header & 0x40) != 0;
            let local_message_type = record_header & 0x0F;

            let parse_success = if is_definition {
                // Parse definition message
                match self.parse_definition_message() {
                    Some(definition) => {
                        self.message_definitions.insert(local_message_type, definition);
                        true
                    }
                    None => {
                        // Definition parsing failed, skip ahead a bit
                        false
                    }
                }
            } else {
                // Parse data message
                if let Some(definition) = self.message_definitions.get(&local_message_type).cloned() {
                    // Verify we have enough bytes for this message
                    let total_size: usize = definition.fields.iter().map(|f| f.size as usize).sum();
                    if self.pos + total_size > self.data.len() {
                        // Not enough bytes left, try to parse what we can or skip this message
                        if total_size < 1000 { // Only try if it's a reasonable size
                            self.skip(self.data.len() - self.pos); // Skip to end
                        }
                        break;
                    }
                    
                    // Look for GPS data in multiple message types
                    match definition.global_message_number {
                        20 => {
                            // Record message (primary GPS data)
                            if let Some(coord) = self.parse_record_message(&definition) {
                                if is_valid_coordinate(coord[0], coord[1]) {
                                    coordinates.push(coord);
                                }
                            }
                            true
                        }
                        19 => {
                            // Lap message (might contain GPS data)
                            if let Some(coord) = self.parse_flexible_gps_message(&definition) {
                                if is_valid_coordinate(coord[0], coord[1]) {
                                    coordinates.push(coord);
                                }
                            }
                            true
                        }
                        18 => {
                            // Session message (might contain GPS data)
                            if let Some(coord) = self.parse_flexible_gps_message(&definition) {
                                if is_valid_coordinate(coord[0], coord[1]) {
                                    coordinates.push(coord);
                                }
                            }
                            true
                        }
                        _ => {
                            // Skip other message types but don't count as error
                            let total_size: usize = definition.fields.iter().map(|f| f.size as usize).sum();
                            if total_size < 1000 && self.pos + total_size <= self.data.len() {
                                self.skip(total_size);
                            } else {
                                // Skip to end if message is too large or would overflow
                                self.skip(self.data.len() - self.pos);
                                break;
                            }
                            true
                        }
                    }
                } else {
                    // Unknown message type - this might be an error, but try to continue
                    false
                }
            };

            if parse_success {
                consecutive_errors = 0; // Reset error counter on success
            } else {
                consecutive_errors += 1;
                
                // If we can't parse this message, try to advance by a small amount and continue
                if self.pos == start_pos {
                    // We didn't advance at all, force advancement to prevent infinite loop
                    self.skip(1);
                }
                
                // Only give up if we hit way too many consecutive errors AND we haven't found much data
                if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                    // If we have a decent amount of coordinates, maybe this is just the end of useful data
                    if coordinates.len() < 100 {
                        break; // Give up if we don't have much data
                    } else {
                        // We have good data, try to continue a bit more
                        consecutive_errors = MAX_CONSECUTIVE_ERRORS / 2; // Reset to half
                    }
                }
            }
        }

        coordinates
    }

    fn parse_definition_message(&mut self) -> Option<MessageDefinition> {
        let _start_pos = self.pos;
        
        // Check we have enough bytes for the basic structure
        if self.pos + 5 > self.data.len() {
            return None;
        }
        
        self.skip(1); // reserved byte
        self.skip(1); // architecture
        let global_message_number = self.read_u16_le()?;
        let num_fields = self.read_u8()?;

        // Sanity check on number of fields
        if num_fields > 100 {
            // This seems unreasonable, likely a parsing error
            return None;
        }

        // Check we have enough bytes for all field definitions
        if self.pos + (num_fields as usize * 3) > self.data.len() {
            return None;
        }

        let mut fields = Vec::new();
        for _ in 0..num_fields {
            // Check bounds before each field
            if self.pos + 3 > self.data.len() {
                // Not enough bytes for this field definition
                return None;
            }
            
            let field_def_num = self.read_u8()?;
            let size = self.read_u8()?;
            let base_type = self.read_u8()?;
            
            // Sanity check on field size
            if size > 100 {
                // Field size seems unreasonable, likely a parsing error
                return None;
            }
            
            fields.push(FieldDefinition {
                field_def_num,
                size,
                _base_type: base_type,
            });
        }

        Some(MessageDefinition {
            global_message_number,
            fields,
        })
    }

    fn parse_record_message(&mut self, definition: &MessageDefinition) -> Option<[f64; 2]> {
        let mut lat: Option<f64> = None;
        let mut lon: Option<f64> = None;

        for field in &definition.fields {
            // More defensive bounds checking
            if field.size == 0 || self.pos >= self.data.len() || self.pos + field.size as usize > self.data.len() {
                // Skip this field if we can't read it safely
                let safe_skip = (self.data.len() - self.pos).min(field.size as usize);
                self.skip(safe_skip);
                continue;
            }
            
            match field.field_def_num {
                0 => {
                    // Latitude field
                    if field.size == 4 {
                        if let Some(lat_raw) = self.read_i32_le() {
                            if lat_raw != 0x7FFFFFFF && lat_raw != 0 {
                                let lat_degrees = lat_raw as f64 * (180.0 / 2147483648.0);
                                if lat_degrees.abs() <= 90.0 {
                                    lat = Some(lat_degrees);
                                }
                            }
                        }
                    } else {
                        self.skip(field.size as usize);
                    }
                }
                1 => {
                    // Longitude field
                    if field.size == 4 {
                        if let Some(lon_raw) = self.read_i32_le() {
                            if lon_raw != 0x7FFFFFFF && lon_raw != 0 {
                                let lon_degrees = lon_raw as f64 * (180.0 / 2147483648.0);
                                if lon_degrees.abs() <= 180.0 {
                                    lon = Some(lon_degrees);
                                }
                            }
                        }
                    } else {
                        self.skip(field.size as usize);
                    }
                }
                _ => {
                    // Skip other fields
                    self.skip(field.size as usize);
                }
            }
        }

        if let (Some(lat_val), Some(lon_val)) = (lat, lon) {
            Some([round(lat_val), round(lon_val)])
        } else {
            None
        }
    }

    fn parse_flexible_gps_message(&mut self, definition: &MessageDefinition) -> Option<[f64; 2]> {
        let mut lat: Option<f64> = None;
        let mut lon: Option<f64> = None;
        let mut potential_coords = Vec::new();

        // Collect all potential coordinate values
        for field in &definition.fields {
            // More defensive bounds checking
            if field.size == 0 || self.pos >= self.data.len() || self.pos + field.size as usize > self.data.len() {
                // Skip this field if we can't read it safely
                let safe_skip = (self.data.len() - self.pos).min(field.size as usize);
                self.skip(safe_skip);
                continue;
            }
            
            if field.size == 4 {
                if let Some(value) = self.read_i32_le() {
                    if value != 0x7FFFFFFF && value != 0 {
                        let degrees = value as f64 * (180.0 / 2147483648.0);
                        // Only consider reasonable coordinate values
                        if degrees.abs() <= 180.0 {
                            potential_coords.push(degrees);
                        }
                    }
                }
            } else {
                self.skip(field.size as usize);
            }
        }

        // Try to identify lat/lon from potential coordinates
        for coord in &potential_coords {
            if coord.abs() <= 90.0 && lat.is_none() {
                lat = Some(*coord);
            } else if coord.abs() <= 180.0 && lon.is_none() && Some(*coord) != lat {
                lon = Some(*coord);
            }
        }

        if let (Some(lat_val), Some(lon_val)) = (lat, lon) {
            Some([round(lat_val), round(lon_val)])
        } else {
            None
        }
    }
}

pub fn is_fit_file(data: &[u8]) -> bool {
    if data.len() < 12 {
        return false;
    }
    
    // Check for FIT signature at bytes 8-11
    data[8] == b'.' && data[9] == b'F' && data[10] == b'I' && data[11] == b'T'
}

// #################################################
// 
//     DATA PROCESSING & VALIDATION FUNCTIONS 
//
// #################################################

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn validate_coordinates(coords: js_sys::Array) -> JsValue {
    let mut issues = Vec::new();
    let mut valid_count = 0;
    
    for i in 0..coords.length() {
        if let Ok(coord_array) = serde_wasm_bindgen::from_value::<[f64; 2]>(coords.get(i)) {
            let [lat, lon] = coord_array;
            
            if !is_valid_coordinate(lat, lon) {
                issues.push(format!("invalid coordinate at index {}: [{}, {}]", i, lat, lon));
            } else {
                valid_count += 1;
            }
        }
    }
    
    let result = ValidationResult {
        valid_count,
        total_count: coords.length(),
        issues,
    };
    
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

// validation
pub fn validate_coordinates_rust(coords: &[[f64; 2]]) -> (u32, Vec<String>) {
    let mut issues = Vec::new();
    let mut valid_count = 0;
    
    for (i, &[lat, lon]) in coords.iter().enumerate() {
        if !is_valid_coordinate(lat, lon) {
            issues.push(format!("invalid coordinate at index {}: [{}, {}]", i, lat, lon));
        } else {
            valid_count += 1;
        }
    }
    
    (valid_count, issues)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn filter_coordinates_by_bounds(coords: js_sys::Array, bounds: js_sys::Array) -> JsValue {
    // bounds format: [min_lat, min_lon, max_lat, max_lon]
    if bounds.length() != 4 {
        return JsValue::NULL;
    }
    
    let min_lat = bounds.get(0).as_f64().unwrap_or(-90.0);
    let min_lon = bounds.get(1).as_f64().unwrap_or(-180.0);
    let max_lat = bounds.get(2).as_f64().unwrap_or(90.0);
    let max_lon = bounds.get(3).as_f64().unwrap_or(180.0);
    
    let mut filtered = Vec::new();
    
    for i in 0..coords.length() {
        if let Ok(coord_array) = serde_wasm_bindgen::from_value::<[f64; 2]>(coords.get(i)) {
            let [lat, lon] = coord_array;
            
            if lat >= min_lat && lat <= max_lat && lon >= min_lon && lon <= max_lon {
                filtered.push([lat, lon]);
            }
        }
    }
    
    serde_wasm_bindgen::to_value(&filtered).unwrap_or(JsValue::NULL)
}

// bounds filtering
pub fn filter_coordinates_by_bounds_rust(coords: &[[f64; 2]], bounds: [f64; 4]) -> Vec<[f64; 2]> {
    let [min_lat, min_lon, max_lat, max_lon] = bounds;
    
    coords.iter()
        .filter(|&&[lat, lon]| lat >= min_lat && lat <= max_lat && lon >= min_lon && lon <= max_lon)
        .copied()
        .collect()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn calculate_track_statistics(coords: js_sys::Array) -> JsValue {
    if coords.length() == 0 {
        return JsValue::NULL;
    }
    
    let mut coordinates = Vec::new();
    let mut min_lat = f64::MAX;
    let mut max_lat = f64::MIN;
    let mut min_lng = f64::MAX;
    let mut max_lng = f64::MIN;
    
    // extract coordinates and calculate bounds
    for i in 0..coords.length() {
        if let Ok(coord_array) = serde_wasm_bindgen::from_value::<[f64; 2]>(coords.get(i)) {
            let [lat, lon] = coord_array;
            coordinates.push([lat, lon]);
            
            min_lat = min_lat.min(lat);
            max_lat = max_lat.max(lat);
            min_lng = min_lng.min(lon);
            max_lng = max_lng.max(lon);
        }
    }
    
    let mut total_distance = 0.0;
    for window in coordinates.windows(2) {
        if let [start, end] = window {
            total_distance += haversine_distance(start[0], start[1], end[0], end[1]);
        }
    }
    
    let result = TrackStatistics {
        distance_km: total_distance,
        point_count: coordinates.len() as u32,
        bounding_box: [min_lat, min_lng, max_lat, max_lng],
        elevation_gain: None, // would require elevation data
        average_speed: None,  // would require timestamp data
    };
    
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

// track statistics
pub fn calculate_track_statistics_rust(coords: &[[f64; 2]]) -> Option<(f64, u32, [f64; 4])> {
    if coords.is_empty() {
        return None;
    }
    
    let bbox = calculate_bounding_box(coords);
    let mut total_distance = 0.0;
    
    for window in coords.windows(2) {
        if let [start, end] = window {
            total_distance += haversine_distance(start[0], start[1], end[0], end[1]);
        }
    }
    
    Some((total_distance, coords.len() as u32, bbox))
}

// #################################################
// 
//          TRACK MANIPULATION FUNCTIONS 
//
// #################################################

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn simplify_coordinates(coords: js_sys::Array, tolerance: f64) -> JsValue {
    let mut coordinates = Vec::new();
    
    for i in 0..coords.length() {
        if let Ok(coord_array) = serde_wasm_bindgen::from_value::<[f64; 2]>(coords.get(i)) {
            coordinates.push(coord_array);
        }
    }
    
    let simplified = simplify_track(&coordinates, tolerance);
    serde_wasm_bindgen::to_value(&simplified).unwrap_or(JsValue::NULL)
}


// simplify coordinates
pub fn simplify_coordinates_rust(coords: &[[f64; 2]], tolerance: f64) -> Vec<[f64; 2]> {
    simplify_track(coords, tolerance)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn merge_nearby_tracks(tracks: js_sys::Array, distance_threshold: f64) -> JsValue {
    let mut track_list = Vec::new();
    
    for i in 0..tracks.length() {
        if let Ok(track) = serde_wasm_bindgen::from_value::<Vec<[f64; 2]>>(tracks.get(i)) {
            track_list.push(track);
        }
    }
    
    let merged = merge_similar_tracks(&track_list, distance_threshold);
    serde_wasm_bindgen::to_value(&merged).unwrap_or(JsValue::NULL)
}
// merge nearby tracks
pub fn merge_nearby_tracks_rust(tracks: &[Vec<[f64; 2]>], distance_threshold: f64) -> Vec<Vec<[f64; 2]>> {
    merge_similar_tracks(tracks, distance_threshold)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn split_track_by_gaps(coords: js_sys::Array, max_gap_km: f64, _max_time_gap_seconds: u32) -> JsValue {
    let mut coordinates = Vec::new();
    
    for i in 0..coords.length() {
        if let Ok(coord_array) = serde_wasm_bindgen::from_value::<[f64; 2]>(coords.get(i)) {
            coordinates.push(coord_array);
        }
    }
    
    let split_tracks = split_by_spatial_gaps(&coordinates, max_gap_km);
    serde_wasm_bindgen::to_value(&split_tracks).unwrap_or(JsValue::NULL)
}

// split track by gaps
pub fn split_track_by_gaps_rust(coords: &[[f64; 2]], max_gap_km: f64) -> Vec<Vec<[f64; 2]>> {
    split_by_spatial_gaps(coords, max_gap_km)
}

// #################################################
// 
//          FORMAT CONVERSION FUNCTIONS 
//
// #################################################

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn coordinates_to_polyline(coords: js_sys::Array) -> String {
    let mut coordinates = Vec::new();
    
    for i in 0..coords.length() {
        if let Ok(coord_array) = serde_wasm_bindgen::from_value::<[f64; 2]>(coords.get(i)) {
            coordinates.push(coord_array);
        }
    }
    
    encode_polyline(&coordinates)
}

// coordinates to polyline
pub fn coordinates_to_polyline_rust(coords: &[[f64; 2]]) -> String {
    encode_polyline(coords)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn coordinates_to_geojson(coords: js_sys::Array, properties: JsValue) -> JsValue {
    let mut coordinates = Vec::new();
    
    for i in 0..coords.length() {
        if let Ok(coord_array) = serde_wasm_bindgen::from_value::<[f64; 2]>(coords.get(i)) {
            // geojson uses [lng, lat] order
            coordinates.push([coord_array[1], coord_array[0]]);
        }
    }
    
    let props = if properties.is_null() || properties.is_undefined() {
        serde_json::json!({})
    } else {
        serde_wasm_bindgen::from_value(properties).unwrap_or(serde_json::json!({}))
    };
    
    let geojson = serde_json::json!({
        "type": "Feature",
        "geometry": {
            "type": "LineString",
            "coordinates": coordinates
        },
        "properties": props
    });
    
    serde_wasm_bindgen::to_value(&geojson).unwrap_or(JsValue::NULL)
}

// coordinates to geojson (returns json value)
pub fn coordinates_to_geojson_rust(coords: &[[f64; 2]], properties: serde_json::Value) -> serde_json::Value {
    let coordinates: Vec<[f64; 2]> = coords.iter()
        .map(|&[lat, lon]| [lon, lat]) // geojson uses [lng, lat] order
        .collect();
    
    serde_json::json!({
        "type": "Feature",
        "geometry": {
            "type": "LineString",
            "coordinates": coordinates
        },
        "properties": properties
    })
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn export_to_gpx(tracks: js_sys::Array, _metadata: JsValue) -> String {
    let mut gpx_content = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="fastgeotoolkit">
"#);
    
    for i in 0..tracks.length() {
        if let Ok(track) = serde_wasm_bindgen::from_value::<Vec<[f64; 2]>>(tracks.get(i)) {
            gpx_content.push_str(&format!("  <trk>\n    <name>Track {}</name>\n    <trkseg>\n", i + 1));
            
            for coord in track {
                gpx_content.push_str(&format!(
                    "      <trkpt lat=\"{:.6}\" lon=\"{:.6}\"></trkpt>\n",
                    coord[0], coord[1]
                ));
            }
            
            gpx_content.push_str("    </trkseg>\n  </trk>\n");
        }
    }
    
    gpx_content.push_str("</gpx>");
    gpx_content
}
// gpx export
pub fn export_to_gpx_rust(tracks: &[Vec<[f64; 2]>]) -> String {
    let mut gpx_content = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="fastgeotoolkit">
"#);
    
    for (i, track) in tracks.iter().enumerate() {
        gpx_content.push_str(&format!("  <trk>\n    <name>Track {}</name>\n    <trkseg>\n", i + 1));
        
        for coord in track {
            gpx_content.push_str(&format!(
                "      <trkpt lat=\"{:.6}\" lon=\"{:.6}\"></trkpt>\n",
                coord[0], coord[1]
            ));
        }
        
        gpx_content.push_str("    </trkseg>\n  </trk>\n");
    }
    
    gpx_content.push_str("</gpx>");
    gpx_content
}

// #################################################
// 
//      c    TRACK ANALYSIS FUNCTIONS
//
// #################################################

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn find_track_intersections(tracks: js_sys::Array, tolerance: f64) -> JsValue {
    let mut track_list = Vec::new();
    
    for i in 0..tracks.length() {
        if let Ok(track) = serde_wasm_bindgen::from_value::<Vec<[f64; 2]>>(tracks.get(i)) {
            track_list.push(track);
        }
    }
    
    let intersections = find_intersections(&track_list, tolerance);
    serde_wasm_bindgen::to_value(&intersections).unwrap_or(JsValue::NULL)
}

// track intersections finder
pub fn find_track_intersections_rust(tracks: &[Vec<[f64; 2]>], tolerance: f64) -> Vec<([f64; 2], Vec<u32>)> {
    let intersections = find_intersections(tracks, tolerance);
    intersections.into_iter()
        .map(|ip| (ip.coordinate, ip.track_indices))
        .collect()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn calculate_coverage_area(tracks: js_sys::Array) -> JsValue {
    let mut all_points = Vec::new();
    
    for i in 0..tracks.length() {
        if let Ok(track) = serde_wasm_bindgen::from_value::<Vec<[f64; 2]>>(tracks.get(i)) {
            all_points.extend(track);
        }
    }
    
    if all_points.is_empty() {
        return JsValue::NULL;
    }
    
    let bbox = calculate_bounding_box(&all_points);
    let area_km2 = calculate_area_km2(&bbox);
    
    let result = serde_json::json!({
        "bounding_box": bbox,
        "area_km2": area_km2,
        "point_count": all_points.len()
    });
    
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}
// calculate coverage area
pub fn calculate_coverage_area_rust(tracks: &[Vec<[f64; 2]>]) -> Option<([f64; 4], f64, usize)> {
    let all_points: Vec<[f64; 2]> = tracks.iter().flatten().copied().collect();
    
    if all_points.is_empty() {
        return None;
    }
    
    let bbox = calculate_bounding_box(&all_points);
    let area_km2 = calculate_area_km2(&bbox);
    
    Some((bbox, area_km2, all_points.len()))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn cluster_tracks_by_similarity(tracks: js_sys::Array, similarity_threshold: f64) -> JsValue {
    let mut track_list = Vec::new();
    
    for i in 0..tracks.length() {
        if let Ok(track) = serde_wasm_bindgen::from_value::<Vec<[f64; 2]>>(tracks.get(i)) {
            track_list.push(track);
        }
    }
    
    let clusters = cluster_similar_tracks(&track_list, similarity_threshold);
    serde_wasm_bindgen::to_value(&clusters).unwrap_or(JsValue::NULL)
}

// cluster tracks by similarity
pub fn cluster_tracks_by_similarity_rust(tracks: &[Vec<[f64; 2]>], similarity_threshold: f64) -> Vec<(Vec<[f64; 2]>, Vec<u32>, f64)> {
    let clusters = cluster_similar_tracks(tracks, similarity_threshold);
    clusters.into_iter()
        .map(|tc| (tc.representative_track, tc.member_indices, tc.similarity_score))
        .collect()
}

// #################################################
// 
//     (SIMPLE) UTILITY FUNCTIONS (public) 
//
// #################################################

pub fn calculate_distance_between_points(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    haversine_distance(lat1, lon1, lat2, lon2)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_bounding_box(coords: js_sys::Array) -> JsValue {
    let mut coordinates = Vec::new();
    
    for i in 0..coords.length() {
        if let Ok(coord_array) = serde_wasm_bindgen::from_value::<[f64; 2]>(coords.get(i)) {
            coordinates.push(coord_array);
        }
    }
    
    if coordinates.is_empty() {
        return JsValue::NULL;
    }
    
    let bbox = calculate_bounding_box(&coordinates);
    serde_wasm_bindgen::to_value(&bbox).unwrap_or(JsValue::NULL)
}

// bounding box
pub fn get_bounding_box_rust(coords: &[[f64; 2]]) -> [f64; 4] {
    calculate_bounding_box(coords)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn resample_track(coords: js_sys::Array, target_point_count: u32) -> JsValue {
    let mut coordinates = Vec::new();
    
    for i in 0..coords.length() {
        if let Ok(coord_array) = serde_wasm_bindgen::from_value::<[f64; 2]>(coords.get(i)) {
            coordinates.push(coord_array);
        }
    }
    
    let resampled = resample_coordinates(&coordinates, target_point_count as usize);
    serde_wasm_bindgen::to_value(&resampled).unwrap_or(JsValue::NULL)
}

// resample track
pub fn resample_track_rust(coords: &[[f64; 2]], target_point_count: usize) -> Vec<[f64; 2]> {
    resample_coordinates(coords, target_point_count)
}

// #################################################
// 
//        FILE INFO/METADATA FUNCTIONS 
//
// #################################################

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_file_info(file_bytes: js_sys::Uint8Array) -> JsValue {
    let bytes = file_bytes.to_vec();
    let mut info = FileInfo {
        format: "unknown".to_string(),
        track_count: 0,
        point_count: 0,
        valid: false,
        file_size: bytes.len() as u32,
    };
    
    // check for gpx
    if let Ok(gpx) = read(Cursor::new(&bytes)) {
        info.format = "gpx".to_string();
        info.valid = true;
        info.track_count = gpx.tracks.len() as u32;
        
        for track in gpx.tracks {
            for segment in track.segments {
                info.point_count += segment.points.len() as u32;
            }
        }
    }
    // check for fit
    else if is_fit_file(&bytes) {
        info.format = "fit".to_string();
        info.valid = true;
        info.track_count = 1; // assume single track for fit files
        
        let mut parser = FitParser::new(bytes);
        let coords = parser.parse_gps_coordinates();
        info.point_count = coords.len() as u32;
    }
    
    serde_wasm_bindgen::to_value(&info).unwrap_or(JsValue::NULL)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn extract_file_metadata(file_bytes: js_sys::Uint8Array) -> JsValue {
    let bytes = file_bytes.to_vec();
    
    if let Ok(gpx) = read(Cursor::new(&bytes)) {
        let metadata = serde_json::json!({
            "format": "gpx",
            "creator": gpx.creator,
            "version": format!("{:?}", gpx.version),
            "tracks": gpx.tracks.iter().map(|t| serde_json::json!({
                "name": t.name,
                "description": t.description,
                "segment_count": t.segments.len()
            })).collect::<Vec<_>>()
        });
        
        return serde_wasm_bindgen::to_value(&metadata).unwrap_or(JsValue::NULL);
    }
    
    if is_fit_file(&bytes) {
        let metadata = serde_json::json!({
            "format": "fit",
            "file_size": bytes.len()
        });
        
        return serde_wasm_bindgen::to_value(&metadata).unwrap_or(JsValue::NULL);
    }
    
    JsValue::NULL
}

// #################################################
// 
//        HELPER FUNCTIONS (private)
//
// #################################################

fn encode_polyline(coordinates: &[[f64; 2]]) -> String {
    let mut encoded = String::new();
    let mut prev_lat = 0i32;
    let mut prev_lng = 0i32;
    
    for coord in coordinates {
        let lat = (coord[0] * 1e5).round() as i32;
        let lng = (coord[1] * 1e5).round() as i32;
        
        let d_lat = lat - prev_lat;
        let d_lng = lng - prev_lng;
        
        encoded.push_str(&encode_number(d_lat));
        encoded.push_str(&encode_number(d_lng));
        
        prev_lat = lat;
        prev_lng = lng;
    }
    
    encoded
}

fn encode_number(num: i32) -> String {
    let mut value = if num < 0 { (!num) << 1 | 1 } else { num << 1 };
    let mut encoded = String::new();
    
    while value >= 0x20 {
        encoded.push(((0x20 | (value & 0x1f)) + 63) as u8 as char);
        value >>= 5;
    }
    encoded.push((value + 63) as u8 as char);
    
    encoded
}

fn merge_similar_tracks(tracks: &[Vec<[f64; 2]>], distance_threshold: f64) -> Vec<Vec<[f64; 2]>> {
    let mut merged = Vec::new();
    let mut used = vec![false; tracks.len()];
    
    for i in 0..tracks.len() {
        if used[i] {
            continue;
        }
        
        let mut current_track = tracks[i].clone();
        used[i] = true;
        
        // look for similar tracks to merge
        for j in (i + 1)..tracks.len() {
            if used[j] {
                continue;
            }
            
            if tracks_are_similar(&current_track, &tracks[j], distance_threshold) {
                // merge tracks by interleaving points
                current_track = merge_two_tracks(&current_track, &tracks[j]);
                used[j] = true;
            }
        }
        
        merged.push(current_track);
    }
    
    merged
}

fn tracks_are_similar(track1: &[[f64; 2]], track2: &[[f64; 2]], threshold: f64) -> bool {
    if track1.len() < 2 || track2.len() < 2 {
        return false;
    }
    
    // simple similarity check: compare start and end points
    let start_dist = haversine_distance(track1[0][0], track1[0][1], track2[0][0], track2[0][1]);
    let end_dist = haversine_distance(
        track1[track1.len() - 1][0], track1[track1.len() - 1][1],
        track2[track2.len() - 1][0], track2[track2.len() - 1][1]
    );
    
    start_dist <= threshold && end_dist <= threshold
}

fn merge_two_tracks(track1: &[[f64; 2]], track2: &[[f64; 2]]) -> Vec<[f64; 2]> {
    // simple merge: take the longer track as base
    if track1.len() >= track2.len() {
        track1.to_vec()
    } else {
        track2.to_vec()
    }
}

fn split_by_spatial_gaps(coordinates: &[[f64; 2]], max_gap_km: f64) -> Vec<Vec<[f64; 2]>> {
    if coordinates.len() < 2 {
        return vec![coordinates.to_vec()];
    }
    
    let mut tracks = Vec::new();
    let mut current_track = vec![coordinates[0]];
    
    for window in coordinates.windows(2) {
        if let [current, next] = window {
            let distance = haversine_distance(current[0], current[1], next[0], next[1]);
            
            if distance <= max_gap_km {
                current_track.push(*next);
            } else {
                // gap detected, start new track
                if current_track.len() > 1 {
                    tracks.push(current_track);
                }
                current_track = vec![*next];
            }
        }
    }
    
    if current_track.len() > 1 {
        tracks.push(current_track);
    }
    
    tracks
}

fn find_intersections(tracks: &[Vec<[f64; 2]>], tolerance: f64) -> Vec<IntersectionPoint> {
    let mut intersections = Vec::new();
    let mut intersection_map: HashMap<String, Vec<u32>> = HashMap::new();
    
    // find points that are close to each other across different tracks
    for (i, track1) in tracks.iter().enumerate() {
        for point1 in track1 {
            let grid_key = format!("{:.4},{:.4}", 
                (point1[0] / tolerance).round() * tolerance,
                (point1[1] / tolerance).round() * tolerance
            );
            
            intersection_map.entry(grid_key).or_insert_with(Vec::new).push(i as u32);
        }
    }
    
    // collect intersections where multiple tracks meet
    for (key_str, track_indices) in intersection_map {
        if track_indices.len() > 1 {
            let coords: Vec<f64> = key_str.split(',').map(|s| s.parse().unwrap_or(0.0)).collect();
            if coords.len() == 2 {
                let mut unique_tracks = track_indices;
                unique_tracks.sort();
                unique_tracks.dedup();
                
                if unique_tracks.len() > 1 {
                    intersections.push(IntersectionPoint {
                        coordinate: [coords[0], coords[1]],
                        track_indices: unique_tracks,
                    });
                }
            }
        }
    }
    
    intersections
}

fn calculate_bounding_box(coordinates: &[[f64; 2]]) -> [f64; 4] {
    let mut min_lat = f64::MAX;
    let mut max_lat = f64::MIN;
    let mut min_lng = f64::MAX;
    let mut max_lng = f64::MIN;
    
    for coord in coordinates {
        min_lat = min_lat.min(coord[0]);
        max_lat = max_lat.max(coord[0]);
        min_lng = min_lng.min(coord[1]);
        max_lng = max_lng.max(coord[1]);
    }
    
    [min_lat, min_lng, max_lat, max_lng]
}
fn calculate_area_km2(bbox: &[f64; 4]) -> f64 {
    let [min_lat, min_lng, max_lat, max_lng] = *bbox;
    
    // approximate area calculation using haversine for edges
    let width_km = haversine_distance(min_lat, min_lng, min_lat, max_lng);
    let height_km = haversine_distance(min_lat, min_lng, max_lat, min_lng);
    
    width_km * height_km
}

fn cluster_similar_tracks(tracks: &[Vec<[f64; 2]>], similarity_threshold: f64) -> Vec<TrackCluster> {
    let mut clusters = Vec::new();
    let mut assigned = vec![false; tracks.len()];
    
    for i in 0..tracks.len() {
        if assigned[i] {
            continue;
        }
        
        let mut cluster_members = vec![i as u32];
        assigned[i] = true;
        
        // find similar tracks
        for j in (i + 1)..tracks.len() {
            if assigned[j] {
                continue;
            }
            
            let similarity = calculate_track_similarity(&tracks[i], &tracks[j]);
            if similarity >= similarity_threshold {
                cluster_members.push(j as u32);
                assigned[j] = true;
            }
        }
        
        clusters.push(TrackCluster {
            representative_track: tracks[i].clone(),
            member_indices: cluster_members,
            similarity_score: 1.0, // placeholder
        });
    }
    
    clusters
}

fn calculate_track_similarity(track1: &[[f64; 2]], track2: &[[f64; 2]]) -> f64 {
    if track1.len() < 2 || track2.len() < 2 {
        return 0.0;
    }
    
    // simple similarity based on start/end point proximity
    let start_dist = haversine_distance(track1[0][0], track1[0][1], track2[0][0], track2[0][1]);
    let end_dist = haversine_distance(
        track1[track1.len() - 1][0], track1[track1.len() - 1][1],
        track2[track2.len() - 1][0], track2[track2.len() - 1][1]
    );
    
    // convert distance to similarity score (closer = more similar)
    let max_reasonable_distance = 10.0; // 10km
    let start_similarity = (max_reasonable_distance - start_dist.min(max_reasonable_distance)) / max_reasonable_distance;
    let end_similarity = (max_reasonable_distance - end_dist.min(max_reasonable_distance)) / max_reasonable_distance;
    
    (start_similarity + end_similarity) / 2.0
}
fn resample_coordinates(coordinates: &[[f64; 2]], target_count: usize) -> Vec<[f64; 2]> {
    if coordinates.len() <= target_count {
        return coordinates.to_vec();
    }
    
    let mut resampled = Vec::new();
    let step = coordinates.len() as f64 / target_count as f64;
    
    for i in 0..target_count {
        let index = (i as f64 * step) as usize;
        if index < coordinates.len() {
            resampled.push(coordinates[index]);
        }
    }
    
    // always include the last point
    if let Some(last) = coordinates.last() {
        if resampled.last() != Some(last) {
            resampled.push(*last);
        }
    }
    
    resampled
}

// #################################################
// 
//        PYTHON BINDINGS (PyO3)
//
// #################################################

/// Decode a polyline string into coordinates
#[pyfunction]
fn py_decode_polyline(encoded: &str) -> Vec<[f64; 2]> {
    decode_polyline(encoded)
}

/// Validate coordinates and return validation results
#[pyfunction]
fn py_validate_coordinates(coordinates: Vec<[f64; 2]>) -> (u32, Vec<String>) {
    validate_coordinates_rust(&coordinates)
}

/// Calculate track statistics
#[pyfunction]
fn py_calculate_track_statistics(coordinates: Vec<[f64; 2]>) -> Option<(f64, u32, [f64; 4])> {
    calculate_track_statistics_rust(&coordinates)
}

/// Simplify coordinates using tolerance
#[pyfunction]
fn py_simplify_coordinates(coordinates: Vec<[f64; 2]>, tolerance: f64) -> Vec<[f64; 2]> {
    simplify_coordinates_rust(&coordinates, tolerance)
}

/// Filter coordinates by bounding box
#[pyfunction]
fn py_filter_coordinates_by_bounds(coordinates: Vec<[f64; 2]>, bounds: [f64; 4]) -> Vec<[f64; 2]> {
    filter_coordinates_by_bounds_rust(&coordinates, bounds)
}

/// Convert coordinates to polyline string
#[pyfunction]
fn py_coordinates_to_polyline(coordinates: Vec<[f64; 2]>) -> String {
    coordinates_to_polyline_rust(&coordinates)
}

/// Get bounding box for coordinates
#[pyfunction]
fn py_get_bounding_box(coordinates: Vec<[f64; 2]>) -> [f64; 4] {
    get_bounding_box_rust(&coordinates)
}

/// Calculate distance between two points
#[pyfunction]
fn py_calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    calculate_distance_between_points(lat1, lon1, lat2, lon2)
}

/// Process polyline string (handles both JSON and encoded formats)
#[pyfunction]
fn py_process_polyline(polyline_str: &str) -> Vec<[f64; 2]> {
    process_polyline(polyline_str)
}

/// Split track by spatial gaps
#[pyfunction]
fn py_split_track_by_gaps(coordinates: Vec<[f64; 2]>, max_gap_km: f64) -> Vec<Vec<[f64; 2]>> {
    split_track_by_gaps_rust(&coordinates, max_gap_km)
}

/// Merge nearby tracks
#[pyfunction]
fn py_merge_nearby_tracks(tracks: Vec<Vec<[f64; 2]>>, distance_threshold: f64) -> Vec<Vec<[f64; 2]>> {
    merge_nearby_tracks_rust(&tracks, distance_threshold)
}

/// Find track intersections
#[pyfunction]
fn py_find_track_intersections(tracks: Vec<Vec<[f64; 2]>>, tolerance: f64) -> Vec<([f64; 2], Vec<u32>)> {
    find_track_intersections_rust(&tracks, tolerance)
}

/// Calculate coverage area for tracks
#[pyfunction]
fn py_calculate_coverage_area(tracks: Vec<Vec<[f64; 2]>>) -> Option<([f64; 4], f64, usize)> {
    calculate_coverage_area_rust(&tracks)
}

/// Cluster tracks by similarity
#[pyfunction]
fn py_cluster_tracks_by_similarity(tracks: Vec<Vec<[f64; 2]>>, similarity_threshold: f64) -> Vec<(Vec<[f64; 2]>, Vec<u32>, f64)> {
    cluster_tracks_by_similarity_rust(&tracks, similarity_threshold)
}

/// Resample track to target point count
#[pyfunction]
fn py_resample_track(coordinates: Vec<[f64; 2]>, target_point_count: usize) -> Vec<[f64; 2]> {
    resample_track_rust(&coordinates, target_point_count)
}

/// Export tracks to GPX format
#[pyfunction]
fn py_export_to_gpx(tracks: Vec<Vec<[f64; 2]>>) -> String {
    export_to_gpx_rust(&tracks)
}

#[pymodule]
fn fastgeotoolkit(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_decode_polyline, m)?)?;
    m.add_function(wrap_pyfunction!(py_validate_coordinates, m)?)?;
    m.add_function(wrap_pyfunction!(py_calculate_track_statistics, m)?)?;
    m.add_function(wrap_pyfunction!(py_simplify_coordinates, m)?)?;
    m.add_function(wrap_pyfunction!(py_filter_coordinates_by_bounds, m)?)?;
    m.add_function(wrap_pyfunction!(py_coordinates_to_polyline, m)?)?;
    m.add_function(wrap_pyfunction!(py_get_bounding_box, m)?)?;
    m.add_function(wrap_pyfunction!(py_calculate_distance, m)?)?;
    m.add_function(wrap_pyfunction!(py_process_polyline, m)?)?;
    m.add_function(wrap_pyfunction!(py_split_track_by_gaps, m)?)?;
    m.add_function(wrap_pyfunction!(py_merge_nearby_tracks, m)?)?;
    m.add_function(wrap_pyfunction!(py_find_track_intersections, m)?)?;
    m.add_function(wrap_pyfunction!(py_calculate_coverage_area, m)?)?;
    m.add_function(wrap_pyfunction!(py_cluster_tracks_by_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(py_resample_track, m)?)?;
    m.add_function(wrap_pyfunction!(py_export_to_gpx, m)?)?;
    Ok(())
}