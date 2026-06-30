#![allow(deprecated)] // suppress PyObject deprecation warnings

use gpx::read;
use std::io::Cursor;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyModule};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// ========================================
// DATA STRUCTURES
// ========================================

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
    bounding_box: [f64; 4],
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

// ========================================
// INTERNAL HELPERS (must be defined before use)
// ========================================

// ---------- Polyline decoding ----------
pub fn decode_polyline_internal(encoded: &str) -> Vec<[f64; 2]> {
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

        let lat_f64 = lat as f64 * 1e-5;
        let lng_f64 = lng as f64 * 1e-5;
        if is_valid_coordinate(lat_f64, lng_f64) {
            coords.push([lat_f64, lng_f64]);
        }
    }
    coords
}

// ---------- Process polyline (JSON or encoded) ----------
pub fn process_polyline_internal(polyline_str: &str) -> Vec<[f64; 2]> {
    if let Ok(json_coords) = serde_json::from_str::<Vec<[f64; 2]>>(polyline_str) {
        if !json_coords.is_empty() {
            return filter_unrealistic_jumps(&json_coords);
        }
        return Vec::new();
    }
    let coords = decode_polyline_internal(polyline_str);
    if !coords.is_empty() {
        filter_unrealistic_jumps(&coords)
    } else {
        Vec::new()
    }
}

// ---------- Heatmap creation from tracks ----------
pub fn create_heatmap_from_tracks(all_tracks: Vec<Vec<[f64; 2]>>) -> HeatmapResult {
    let mut segment_usage: HashMap<String, u32> = HashMap::new();
    for track in &all_tracks {
        for window in track.windows(2) {
            if let [start, end] = window {
                let key = create_segment_key(*start, *end);
                *segment_usage.entry(key).or_insert(0) += 1;
            }
        }
    }
    let mut heatmap_tracks = Vec::new();
    for track in all_tracks {
        if track.len() < 2 {
            continue;
        }
        let mut total_usage = 0;
        let mut segment_count = 0;
        for window in track.windows(2) {
            if let [start, end] = window {
                let key = create_segment_key(*start, *end);
                if let Some(&usage) = segment_usage.get(&key) {
                    total_usage += usage;
                    segment_count += 1;
                }
            }
        }
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
    let max_frequency = heatmap_tracks.iter().map(|t| t.frequency).max().unwrap_or(1);
    HeatmapResult { tracks: heatmap_tracks, max_frequency }
}

// ---------- Coordinate helpers ----------
pub fn round(value: f64) -> f64 {
    (value * 100000.0).round() / 100000.0
}

pub fn snap_to_grid(point: [f64; 2], tolerance: f64) -> [f64; 2] {
    [
        (point[0] / tolerance).round() * tolerance,
        (point[1] / tolerance).round() * tolerance,
    ]
}

pub fn simplify_track(points: &[[f64; 2]], tolerance: f64) -> Vec<[f64; 2]> {
    if points.len() <= 2 {
        return points.to_vec();
    }
    let mut result = vec![points[0]];
    let mut last_added = 0;
    for i in 1..points.len() {
        let d = distance(points[last_added], points[i]);
        if d > tolerance || i == points.len() - 1 {
            result.push(points[i]);
            last_added = i;
        }
    }
    result
}

pub fn distance(p1: [f64; 2], p2: [f64; 2]) -> f64 {
    let dx = p1[0] - p2[0];
    let dy = p1[1] - p2[1];
    (dx * dx + dy * dy).sqrt()
}

pub fn is_valid_coordinate(lat: f64, lon: f64) -> bool {
    if lat < -90.0 || lat > 90.0 || lon < -180.0 || lon > 180.0 {
        return false;
    }
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
    const MAX_CONSECUTIVE_BAD: usize = 10;

    for i in 1..coords.len() {
        let prev = filtered.last().unwrap();
        let curr = coords[i];
        let distance_km = haversine_distance(prev[0], prev[1], curr[0], curr[1]);
        if distance_km <= max_jump_km {
            filtered.push(curr);
            consecutive_bad_points = 0;
        } else {
            consecutive_bad_points += 1;
            if consecutive_bad_points <= MAX_CONSECUTIVE_BAD {
                let mut found_good_continuation = false;
                for j in (i + 1)..(i + 21).min(coords.len()) {
                    let future = coords[j];
                    let future_dist = haversine_distance(prev[0], prev[1], future[0], future[1]);
                    if future_dist <= max_jump_km * 1.5 {
                        found_good_continuation = true;
                        break;
                    }
                }
                if !found_good_continuation {
                    for k in (i + 1)..coords.len() {
                        let rem = coords[k];
                        let rem_dist = haversine_distance(prev[0], prev[1], rem[0], rem[1]);
                        if rem_dist <= max_jump_km {
                            filtered.push(rem);
                            for m in (k + 1)..coords.len() {
                                let next_prev = filtered.last().unwrap();
                                let next_curr = coords[m];
                                let next_dist = haversine_distance(next_prev[0], next_prev[1], next_curr[0], next_curr[1]);
                                if next_dist <= max_jump_km {
                                    filtered.push(next_curr);
                                }
                            }
                            break;
                        }
                    }
                    break;
                }
            } else {
                break;
            }
        }
    }
    filtered
}

pub fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0;
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let a = (d_lat / 2.0).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c
}

pub fn create_segment_key(start: [f64; 2], end: [f64; 2]) -> String {
    let tolerance = 0.001;
    let snap_start = snap_to_grid(start, tolerance);
    let snap_end = snap_to_grid(end, tolerance);
    let (p1, p2) = if (snap_start[0], snap_start[1]) < (snap_end[0], snap_end[1]) {
        (snap_start, snap_end)
    } else {
        (snap_end, snap_start)
    };
    format!("{:.4},{:.4}-{:.4},{:.4}", p1[0], p1[1], p2[0], p2[1])
}

// ---------- FIT file parser ----------
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
        Self { data, pos: 0, message_definitions: HashMap::new() }
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
        if self.data.len() < 14 {
            return coordinates;
        }
        let header_size = self.read_u8().unwrap_or(0);
        if header_size < 12 {
            return coordinates;
        }
        let _protocol_version = self.read_u8().unwrap_or(0);
        let _profile_version = self.read_u16_le().unwrap_or(0);
        let data_size = self.read_u32_le().unwrap_or(0);
        let signature = [
            self.read_u8().unwrap_or(0),
            self.read_u8().unwrap_or(0),
            self.read_u8().unwrap_or(0),
            self.read_u8().unwrap_or(0),
        ];
        if signature != [b'.', b'F', b'I', b'T'] {
            return coordinates;
        }
        if header_size == 14 {
            self.skip(2);
        }
        let data_end = (self.pos + data_size as usize).min(self.data.len());
        let file_data_end = self.data.len().saturating_sub(2);
        let data_end = data_end.max(file_data_end);

        let mut consecutive_errors = 0;
        const MAX_CONSECUTIVE_ERRORS: usize = 100;
        let mut last_progress_pos = self.pos;

        while self.pos < data_end && self.pos < self.data.len() && self.pos + 1 < self.data.len() {
            let start_pos = self.pos;
            if self.pos - last_progress_pos > 10000 {
                last_progress_pos = self.pos;
                if coordinates.len() > 100 && self.pos > 50000 {
                    consecutive_errors = 0;
                }
            }
            if self.pos >= self.data.len() {
                break;
            }
            let record_header = match self.read_u8() {
                Some(h) => h,
                None => break,
            };
            let is_definition = (record_header & 0x40) != 0;
            let local_message_type = record_header & 0x0F;

            let parse_success = if is_definition {
                match self.parse_definition_message() {
                    Some(definition) => {
                        self.message_definitions.insert(local_message_type, definition);
                        true
                    }
                    None => false,
                }
            } else {
                if let Some(definition) = self.message_definitions.get(&local_message_type).cloned() {
                    let total_size: usize = definition.fields.iter().map(|f| f.size as usize).sum();
                    if self.pos + total_size > self.data.len() {
                        if total_size < 1000 {
                            self.skip(self.data.len() - self.pos);
                        }
                        break;
                    }
                    match definition.global_message_number {
                        20 => {
                            if let Some(coord) = self.parse_record_message(&definition) {
                                if is_valid_coordinate(coord[0], coord[1]) {
                                    coordinates.push(coord);
                                }
                            }
                            true
                        }
                        19 | 18 => {
                            if let Some(coord) = self.parse_flexible_gps_message(&definition) {
                                if is_valid_coordinate(coord[0], coord[1]) {
                                    coordinates.push(coord);
                                }
                            }
                            true
                        }
                        _ => {
                            let total_size: usize = definition.fields.iter().map(|f| f.size as usize).sum();
                            if total_size < 1000 && self.pos + total_size <= self.data.len() {
                                self.skip(total_size);
                            } else {
                                self.skip(self.data.len() - self.pos);
                                break;
                            }
                            true
                        }
                    }
                } else {
                    false
                }
            };

            if parse_success {
                consecutive_errors = 0;
            } else {
                consecutive_errors += 1;
                if self.pos == start_pos {
                    self.skip(1);
                }
                if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                    if coordinates.len() < 100 {
                        break;
                    } else {
                        consecutive_errors = MAX_CONSECUTIVE_ERRORS / 2;
                    }
                }
            }
        }
        coordinates
    }

    fn parse_definition_message(&mut self) -> Option<MessageDefinition> {
        if self.pos + 5 > self.data.len() {
            return None;
        }
        self.skip(1); // reserved
        self.skip(1); // architecture
        let global_message_number = self.read_u16_le()?;
        let num_fields = self.read_u8()?;
        if num_fields > 100 {
            return None;
        }
        if self.pos + (num_fields as usize * 3) > self.data.len() {
            return None;
        }
        let mut fields = Vec::new();
        for _ in 0..num_fields {
            if self.pos + 3 > self.data.len() {
                return None;
            }
            let field_def_num = self.read_u8()?;
            let size = self.read_u8()?;
            let base_type = self.read_u8()?;
            if size > 100 {
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
            if field.size == 0 || self.pos >= self.data.len() || self.pos + field.size as usize > self.data.len() {
                let safe_skip = (self.data.len() - self.pos).min(field.size as usize);
                self.skip(safe_skip);
                continue;
            }
            match field.field_def_num {
                0 => {
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
        for field in &definition.fields {
            if field.size == 0 || self.pos >= self.data.len() || self.pos + field.size as usize > self.data.len() {
                let safe_skip = (self.data.len() - self.pos).min(field.size as usize);
                self.skip(safe_skip);
                continue;
            }
            if field.size == 4 {
                if let Some(value) = self.read_i32_le() {
                    if value != 0x7FFFFFFF && value != 0 {
                        let degrees = value as f64 * (180.0 / 2147483648.0);
                        if degrees.abs() <= 180.0 {
                            potential_coords.push(degrees);
                        }
                    }
                }
            } else {
                self.skip(field.size as usize);
            }
        }
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
    data.len() >= 12 && data[8] == b'.' && data[9] == b'F' && data[10] == b'I' && data[11] == b'T'
}

// ---------- Rust versions of analysis functions (used internally) ----------
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

pub fn calculate_track_statistics_rust(coords: &[[f64; 2]]) -> Option<(f64, u32, [f64; 4])> {
    if coords.is_empty() {
        return None;
    }
    let bbox = get_bounding_box_rust(coords);
    let mut total_distance = 0.0;
    for window in coords.windows(2) {
        if let [start, end] = window {
            total_distance += haversine_distance(start[0], start[1], end[0], end[1]);
        }
    }
    Some((total_distance, coords.len() as u32, bbox))
}

pub fn simplify_coordinates_rust(coords: &[[f64; 2]], tolerance: f64) -> Vec<[f64; 2]> {
    simplify_track(coords, tolerance)
}

pub fn filter_coordinates_by_bounds_rust(coords: &[[f64; 2]], bounds: [f64; 4]) -> Vec<[f64; 2]> {
    let [min_lat, min_lon, max_lat, max_lon] = bounds;
    coords.iter()
        .filter(|&&[lat, lon]| lat >= min_lat && lat <= max_lat && lon >= min_lon && lon <= max_lon)
        .copied()
        .collect()
}

pub fn coordinates_to_polyline_rust(coords: &[[f64; 2]]) -> String {
    let mut encoded = String::new();
    let mut prev_lat = 0i32;
    let mut prev_lng = 0i32;
    for coord in coords {
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

pub fn get_bounding_box_rust(coords: &[[f64; 2]]) -> [f64; 4] {
    let mut min_lat = f64::MAX;
    let mut max_lat = f64::MIN;
    let mut min_lon = f64::MAX;
    let mut max_lon = f64::MIN;
    for &[lat, lon] in coords {
        if lat < min_lat { min_lat = lat; }
        if lat > max_lat { max_lat = lat; }
        if lon < min_lon { min_lon = lon; }
        if lon > max_lon { max_lon = lon; }
    }
    [min_lat, min_lon, max_lat, max_lon]
}

pub fn split_track_by_gaps_rust(coords: &[[f64; 2]], max_gap_km: f64) -> Vec<Vec<[f64; 2]>> {
    if coords.len() < 2 {
        return vec![coords.to_vec()];
    }
    let mut tracks = Vec::new();
    let mut current = vec![coords[0]];
    for window in coords.windows(2) {
        if let [current_point, next] = window {
            let dist = haversine_distance(current_point[0], current_point[1], next[0], next[1]);
            if dist <= max_gap_km {
                current.push(*next);
            } else {
                if current.len() > 1 {
                    tracks.push(current);
                }
                current = vec![*next];
            }
        }
    }
    if current.len() > 1 {
        tracks.push(current);
    }
    tracks
}

pub fn merge_nearby_tracks_rust(tracks: &[Vec<[f64; 2]>], distance_threshold: f64) -> Vec<Vec<[f64; 2]>> {
    let mut merged = Vec::new();
    let mut used = vec![false; tracks.len()];
    for i in 0..tracks.len() {
        if used[i] { continue; }
        let mut current = tracks[i].clone();
        used[i] = true;
        for j in (i + 1)..tracks.len() {
            if used[j] { continue; }
            if tracks_are_similar(&current, &tracks[j], distance_threshold) {
                current = merge_two_tracks(&current, &tracks[j]);
                used[j] = true;
            }
        }
        merged.push(current);
    }
    merged
}

fn tracks_are_similar(track1: &[[f64; 2]], track2: &[[f64; 2]], threshold: f64) -> bool {
    if track1.len() < 2 || track2.len() < 2 { return false; }
    let start_dist = haversine_distance(track1[0][0], track1[0][1], track2[0][0], track2[0][1]);
    let end_dist = haversine_distance(
        track1[track1.len() - 1][0], track1[track1.len() - 1][1],
        track2[track2.len() - 1][0], track2[track2.len() - 1][1]
    );
    start_dist <= threshold && end_dist <= threshold
}

fn merge_two_tracks(track1: &[[f64; 2]], track2: &[[f64; 2]]) -> Vec<[f64; 2]> {
    if track1.len() >= track2.len() { track1.to_vec() } else { track2.to_vec() }
}

pub fn find_track_intersections_rust(tracks: &[Vec<[f64; 2]>], tolerance: f64) -> Vec<([f64; 2], Vec<u32>)> {
    let mut intersection_map: HashMap<String, Vec<u32>> = HashMap::new();
    for (i, track) in tracks.iter().enumerate() {
        for point in track {
            let key = format!("{:.4},{:.4}",
                (point[0] / tolerance).round() * tolerance,
                (point[1] / tolerance).round() * tolerance
            );
            intersection_map.entry(key).or_insert_with(Vec::new).push(i as u32);
        }
    }
    let mut result = Vec::new();
    for (key_str, indices) in intersection_map {
        if indices.len() > 1 {
            let parts: Vec<f64> = key_str.split(',').map(|s| s.parse().unwrap_or(0.0)).collect();
            if parts.len() == 2 {
                let mut unique = indices;
                unique.sort();
                unique.dedup();
                if unique.len() > 1 {
                    result.push(([parts[0], parts[1]], unique));
                }
            }
        }
    }
    result
}

pub fn calculate_coverage_area_rust(tracks: &[Vec<[f64; 2]>]) -> Option<([f64; 4], f64, usize)> {
    let all_points: Vec<[f64; 2]> = tracks.iter().flatten().copied().collect();
    if all_points.is_empty() { return None; }
    let bbox = get_bounding_box_rust(&all_points);
    let area_km2 = {
        let [min_lat, min_lon, max_lat, max_lon] = bbox;
        let width = haversine_distance(min_lat, min_lon, min_lat, max_lon);
        let height = haversine_distance(min_lat, min_lon, max_lat, min_lon);
        width * height
    };
    Some((bbox, area_km2, all_points.len()))
}

pub fn cluster_tracks_by_similarity_rust(tracks: &[Vec<[f64; 2]>], similarity_threshold: f64) -> Vec<(Vec<[f64; 2]>, Vec<u32>, f64)> {
    let mut clusters = Vec::new();
    let mut assigned = vec![false; tracks.len()];
    for i in 0..tracks.len() {
        if assigned[i] { continue; }
        let mut members = vec![i as u32];
        assigned[i] = true;
        for j in (i + 1)..tracks.len() {
            if assigned[j] { continue; }
            let sim = calculate_track_similarity(&tracks[i], &tracks[j]);
            if sim >= similarity_threshold {
                members.push(j as u32);
                assigned[j] = true;
            }
        }
        clusters.push((tracks[i].clone(), members, 1.0));
    }
    clusters
}

fn calculate_track_similarity(track1: &[[f64; 2]], track2: &[[f64; 2]]) -> f64 {
    if track1.len() < 2 || track2.len() < 2 { return 0.0; }
    let start_dist = haversine_distance(track1[0][0], track1[0][1], track2[0][0], track2[0][1]);
    let end_dist = haversine_distance(
        track1[track1.len() - 1][0], track1[track1.len() - 1][1],
        track2[track2.len() - 1][0], track2[track2.len() - 1][1]
    );
    let max_dist = 10.0;
    let start_sim = (max_dist - start_dist.min(max_dist)) / max_dist;
    let end_sim = (max_dist - end_dist.min(max_dist)) / max_dist;
    (start_sim + end_sim) / 2.0
}

pub fn resample_track_rust(coords: &[[f64; 2]], target_count: usize) -> Vec<[f64; 2]> {
    if coords.len() <= target_count {
        return coords.to_vec();
    }
    let mut resampled = Vec::new();
    let step = coords.len() as f64 / target_count as f64;
    for i in 0..target_count {
        let index = (i as f64 * step) as usize;
        if index < coords.len() {
            resampled.push(coords[index]);
        }
    }
    if let Some(last) = coords.last() {
        if resampled.last() != Some(last) {
            resampled.push(*last);
        }
    }
    resampled
}

pub fn export_to_gpx_rust(tracks: &[Vec<[f64; 2]>]) -> String {
    let mut gpx = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="fastgeotoolkit">
"#);
    for (i, track) in tracks.iter().enumerate() {
        gpx.push_str(&format!("  <trk>\n    <name>Track {}</name>\n    <trkseg>\n", i + 1));
        for coord in track {
            gpx.push_str(&format!("      <trkpt lat=\"{:.6}\" lon=\"{:.6}\"></trkpt>\n", coord[0], coord[1]));
        }
        gpx.push_str("    </trkseg>\n  </trk>\n");
    }
    gpx.push_str("</gpx>");
    gpx
}

// ========================================
// PYTHON BINDINGS (using the helpers)
// ========================================

#[pyfunction]
fn process_gpx_files(files: Vec<Vec<u8>>) -> PyResult<PyObject> {
    let mut all_tracks: Vec<Vec<[f64; 2]>> = Vec::new();

    for bytes in files {
        if let Ok(gpx) = read(Cursor::new(&bytes)) {
            for track in gpx.tracks {
                for segment in track.segments {
                    let mut track_coords = Vec::new();
                    for point in segment.points {
                        let lat = point.point().y().round(); // corrected: use f64::round
                        let lon = point.point().x().round();
                        if is_valid_coordinate(lat, lon) {
                            track_coords.push([lat, lon]);
                        }
                    }
                    if track_coords.len() > 1 {
                        let filtered = filter_unrealistic_jumps(&track_coords);
                        if filtered.len() > 1 {
                            let simplified = simplify_track(&filtered, 0.00005);
                            if simplified.len() > 1 {
                                all_tracks.push(simplified);
                            }
                        }
                    }
                }
            }
        } else if is_fit_file(&bytes) {
            let mut fit_parser = FitParser::new(bytes);
            let fit_coords = fit_parser.parse_gps_coordinates();
            if fit_coords.len() > 1 {
                let filtered = filter_unrealistic_jumps(&fit_coords);
                if filtered.len() > 1 {
                    let simplified = simplify_track(&filtered, 0.00005);
                    if simplified.len() > 1 {
                        all_tracks.push(simplified);
                    }
                }
            }
        }
    }

    let result = create_heatmap_from_tracks(all_tracks);
    Python::with_gil(|py| {
        let dict = PyDict::new(py);
        dict.set_item("max_frequency", result.max_frequency)?;
        let list = PyList::empty(py);
        for track in result.tracks {
            let t_dict = PyDict::new(py);
            t_dict.set_item("frequency", track.frequency)?;
            let coords_list = PyList::empty(py);
            for coord in track.coordinates {
                coords_list.append((coord[0], coord[1]))?;
            }
            t_dict.set_item("coordinates", coords_list)?;
            list.append(t_dict)?;
        }
        dict.set_item("tracks", list)?;
        Ok(dict.into())
    })
}

#[pyfunction]
fn decode_polyline(encoded: &str) -> Vec<(f64, f64)> {
    let coords = decode_polyline_internal(encoded);
    coords.into_iter().map(|[lat, lon]| (lat, lon)).collect()
}

#[pyfunction]
fn validate_coordinates(coordinates: Vec<(f64, f64)>) -> (u32, Vec<String>) {
    let coords: Vec<[f64; 2]> = coordinates.into_iter().map(|(lat, lon)| [lat, lon]).collect();
    validate_coordinates_rust(&coords)
}

#[pyfunction]
fn calculate_track_statistics(coordinates: Vec<(f64, f64)>) -> Option<(f64, u32, [f64; 4])> {
    let coords: Vec<[f64; 2]> = coordinates.into_iter().map(|(lat, lon)| [lat, lon]).collect();
    calculate_track_statistics_rust(&coords)
}

#[pyfunction]
fn simplify_coordinates(coordinates: Vec<(f64, f64)>, tolerance: f64) -> Vec<(f64, f64)> {
    let coords: Vec<[f64; 2]> = coordinates.into_iter().map(|(lat, lon)| [lat, lon]).collect();
    let simplified = simplify_coordinates_rust(&coords, tolerance);
    simplified.into_iter().map(|[lat, lon]| (lat, lon)).collect()
}

#[pyfunction]
fn filter_coordinates_by_bounds(coordinates: Vec<(f64, f64)>, bounds: [f64; 4]) -> Vec<(f64, f64)> {
    let coords: Vec<[f64; 2]> = coordinates.into_iter().map(|(lat, lon)| [lat, lon]).collect();
    let filtered = filter_coordinates_by_bounds_rust(&coords, bounds);
    filtered.into_iter().map(|[lat, lon]| (lat, lon)).collect()
}

#[pyfunction]
fn coordinates_to_polyline(coordinates: Vec<(f64, f64)>) -> String {
    let coords: Vec<[f64; 2]> = coordinates.into_iter().map(|(lat, lon)| [lat, lon]).collect();
    coordinates_to_polyline_rust(&coords)
}

#[pyfunction]
fn get_bounding_box(coordinates: Vec<(f64, f64)>) -> [f64; 4] {
    let coords: Vec<[f64; 2]> = coordinates.into_iter().map(|(lat, lon)| [lat, lon]).collect();
    get_bounding_box_rust(&coords)
}

#[pyfunction]
fn calculate_distance_between_points(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    haversine_distance(lat1, lon1, lat2, lon2)
}

#[pyfunction]
fn process_polyline(polyline_str: &str) -> Vec<(f64, f64)> {
    let coords = process_polyline_internal(polyline_str);
    coords.into_iter().map(|[lat, lon]| (lat, lon)).collect()
}

#[pyfunction]
fn split_track_by_gaps(coordinates: Vec<(f64, f64)>, max_gap_km: f64) -> Vec<Vec<(f64, f64)>> {
    let coords: Vec<[f64; 2]> = coordinates.into_iter().map(|(lat, lon)| [lat, lon]).collect();
    let split = split_track_by_gaps_rust(&coords, max_gap_km);
    split.into_iter()
        .map(|seg| seg.into_iter().map(|[lat, lon]| (lat, lon)).collect())
        .collect()
}

#[pyfunction]
fn merge_nearby_tracks(tracks: Vec<Vec<(f64, f64)>>, distance_threshold: f64) -> Vec<Vec<(f64, f64)>> {
    let tracks_internal: Vec<Vec<[f64; 2]>> = tracks.into_iter()
        .map(|t| t.into_iter().map(|(lat, lon)| [lat, lon]).collect())
        .collect();
    let merged = merge_nearby_tracks_rust(&tracks_internal, distance_threshold);
    merged.into_iter()
        .map(|t| t.into_iter().map(|[lat, lon]| (lat, lon)).collect())
        .collect()
}

#[pyfunction]
fn find_track_intersections(tracks: Vec<Vec<(f64, f64)>>, tolerance: f64) -> Vec<([f64; 2], Vec<u32>)> {
    let tracks_internal: Vec<Vec<[f64; 2]>> = tracks.into_iter()
        .map(|t| t.into_iter().map(|(lat, lon)| [lat, lon]).collect())
        .collect();
    find_track_intersections_rust(&tracks_internal, tolerance)
}

#[pyfunction]
fn calculate_coverage_area(tracks: Vec<Vec<(f64, f64)>>) -> Option<([f64; 4], f64, usize)> {
    let tracks_internal: Vec<Vec<[f64; 2]>> = tracks.into_iter()
        .map(|t| t.into_iter().map(|(lat, lon)| [lat, lon]).collect())
        .collect();
    calculate_coverage_area_rust(&tracks_internal)
}

#[pyfunction]
fn cluster_tracks_by_similarity(tracks: Vec<Vec<(f64, f64)>>, similarity_threshold: f64) -> Vec<(Vec<(f64, f64)>, Vec<u32>, f64)> {
    let tracks_internal: Vec<Vec<[f64; 2]>> = tracks.into_iter()
        .map(|t| t.into_iter().map(|(lat, lon)| [lat, lon]).collect())
        .collect();
    let clusters = cluster_tracks_by_similarity_rust(&tracks_internal, similarity_threshold);
    clusters.into_iter()
        .map(|(rep, members, score)| {
            let rep_conv = rep.into_iter().map(|[lat, lon]| (lat, lon)).collect();
            (rep_conv, members, score)
        })
        .collect()
}

#[pyfunction]
fn resample_track(coordinates: Vec<(f64, f64)>, target_point_count: usize) -> Vec<(f64, f64)> {
    let coords: Vec<[f64; 2]> = coordinates.into_iter().map(|(lat, lon)| [lat, lon]).collect();
    let resampled = resample_track_rust(&coords, target_point_count);
    resampled.into_iter().map(|[lat, lon]| (lat, lon)).collect()
}

#[pyfunction]
fn export_to_gpx(tracks: Vec<Vec<(f64, f64)>>) -> String {
    let tracks_internal: Vec<Vec<[f64; 2]>> = tracks.into_iter()
        .map(|t| t.into_iter().map(|(lat, lon)| [lat, lon]).collect())
        .collect();
    export_to_gpx_rust(&tracks_internal)
}

// ----- New functions that were missing -----
#[pyfunction]
fn coordinates_to_geojson(coordinates: Vec<(f64, f64)>, properties: Option<PyObject>) -> PyResult<PyObject> {
    Python::with_gil(|py| {
        let props = if let Some(p) = properties { p } else { PyDict::new(py).into() };
        let feature = PyDict::new(py);
        feature.set_item("type", "Feature")?;
        let geometry = PyDict::new(py);
        geometry.set_item("type", "LineString")?;
        let coords_list = PyList::new(py, coordinates.iter().map(|(lat, lon)| (lon, lat)))?;
        geometry.set_item("coordinates", coords_list)?;
        feature.set_item("geometry", geometry)?;
        feature.set_item("properties", props)?;
        Ok(feature.into())
    })
}

#[pyfunction]
fn get_file_info(file_bytes: Vec<u8>) -> PyResult<PyObject> {
    Python::with_gil(|py| {
        let dict = PyDict::new(py);
        let mut format = "unknown".to_string();
        let mut track_count = 0;
        let mut point_count = 0;
        let mut valid = false;
        let file_size = file_bytes.len() as u32;

        if let Ok(gpx) = read(Cursor::new(&file_bytes)) {
            format = "gpx".to_string();
            valid = true;
            track_count = gpx.tracks.len() as u32;
            for track in gpx.tracks {
                for segment in track.segments {
                    point_count += segment.points.len() as u32;
                }
            }
        } else if is_fit_file(&file_bytes) {
            format = "fit".to_string();
            valid = true;
            track_count = 1;
            let mut parser = FitParser::new(file_bytes);
            let coords = parser.parse_gps_coordinates();
            point_count = coords.len() as u32;
        }

        dict.set_item("format", format)?;
        dict.set_item("track_count", track_count)?;
        dict.set_item("point_count", point_count)?;
        dict.set_item("valid", valid)?;
        dict.set_item("file_size", file_size)?;
        Ok(dict.into())
    })
}

#[pyfunction]
fn extract_file_metadata(file_bytes: Vec<u8>) -> PyResult<PyObject> {
    Python::with_gil(|py| {
        if let Ok(gpx) = read(Cursor::new(&file_bytes)) {
            let dict = PyDict::new(py);
            dict.set_item("format", "gpx")?;
            dict.set_item("creator", gpx.creator.clone().unwrap_or_default())?;
            dict.set_item("version", format!("{:?}", gpx.version))?;
            let tracks_list = PyList::empty(py);
            for track in gpx.tracks {
                let track_dict = PyDict::new(py);
                track_dict.set_item("name", track.name.clone().unwrap_or_default())?;
                track_dict.set_item("description", track.description.clone().unwrap_or_default())?;
                track_dict.set_item("segment_count", track.segments.len())?;
                tracks_list.append(track_dict)?;
            }
            dict.set_item("tracks", tracks_list)?;
            return Ok(dict.into());
        } else if is_fit_file(&file_bytes) {
            let dict = PyDict::new(py);
            dict.set_item("format", "fit")?;
            dict.set_item("file_size", file_bytes.len())?;
            return Ok(dict.into());
        }
        Ok(py.None())
    })
}

#[pyfunction]
fn process_polylines(polylines: Vec<String>) -> PyResult<PyObject> {
    let mut all_tracks: Vec<Vec<[f64; 2]>> = Vec::new();
    for polyline_str in polylines {
        let coords = process_polyline_internal(&polyline_str);
        if coords.len() > 1 {
            let simplified = simplify_track(&coords, 0.00005);
            if simplified.len() > 1 {
                all_tracks.push(simplified);
            }
        }
    }
    let result = create_heatmap_from_tracks(all_tracks);
    Python::with_gil(|py| {
        let dict = PyDict::new(py);
        dict.set_item("max_frequency", result.max_frequency)?;
        let tracks_list = PyList::empty(py);
        for track in result.tracks {
            let t_dict = PyDict::new(py);
            t_dict.set_item("frequency", track.frequency)?;
            let coords_list = PyList::empty(py);
            for coord in track.coordinates {
                coords_list.append((coord[0], coord[1]))?;
            }
            t_dict.set_item("coordinates", coords_list)?;
            tracks_list.append(t_dict)?;
        }
        dict.set_item("tracks", tracks_list)?;
        Ok(dict.into())
    })
}

// ========================================
// REGISTER MODULE
// ========================================

#[pymodule]
fn _fastgeotoolkit(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(process_gpx_files, m)?)?;
    m.add_function(wrap_pyfunction!(decode_polyline, m)?)?;
    m.add_function(wrap_pyfunction!(validate_coordinates, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_track_statistics, m)?)?;
    m.add_function(wrap_pyfunction!(simplify_coordinates, m)?)?;
    m.add_function(wrap_pyfunction!(filter_coordinates_by_bounds, m)?)?;
    m.add_function(wrap_pyfunction!(coordinates_to_polyline, m)?)?;
    m.add_function(wrap_pyfunction!(get_bounding_box, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_distance_between_points, m)?)?;
    m.add_function(wrap_pyfunction!(process_polyline, m)?)?;
    m.add_function(wrap_pyfunction!(split_track_by_gaps, m)?)?;
    m.add_function(wrap_pyfunction!(merge_nearby_tracks, m)?)?;
    m.add_function(wrap_pyfunction!(find_track_intersections, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_coverage_area, m)?)?;
    m.add_function(wrap_pyfunction!(cluster_tracks_by_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(resample_track, m)?)?;
    m.add_function(wrap_pyfunction!(export_to_gpx, m)?)?;
    m.add_function(wrap_pyfunction!(coordinates_to_geojson, m)?)?;
    m.add_function(wrap_pyfunction!(get_file_info, m)?)?;
    m.add_function(wrap_pyfunction!(extract_file_metadata, m)?)?;
    m.add_function(wrap_pyfunction!(process_polylines, m)?)?;
    Ok(())
}