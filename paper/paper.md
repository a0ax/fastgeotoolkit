---
title: 'fastGeoToolkit: A High-Performance Geospatial Analysis Library and Segment-Based Route Density Mapping Implementation'
tags:
  - Geospatial computing
  - heatmap visualization
  - GPS
  - WebAssembly
  - JavaScript
authors:
  - name: Alexander Akira Weimer
    orcid: 0009-0008-5679-3042
    affiliation: 1
  - name: Justin Abraham
    orcid: 0009-0009-1472-0408
    affiliation: 1
affiliations:
 - name: University of Minnesota
   index: 1
   ror: 017zqws13
date: 1 September 2025
bibliography: paper.bib

---

# Summary

fastGeoToolkit is a JavaScript library for GPS track analysis that introduces a segment-based approach to route density mapping. Unlike traditional point-based heatmap algorithms, fastGeoToolkit processes GPS tracks as connected line segments to identify overlapping route usage patterns without the spatial clustering artifacts common in existing solutions.

The library handles common GPS data formats (GPX, FIT files, and Google polylines) and provides comprehensive track processing capabilities including distance calculations, track statistics, and coordinate validation.


# Statement of Need

GPS route density visualization is important for urban planning, transportation analysis, trail management, and fitness applications. However, existing approaches have significant limitations:

**Point-based methods create misleading results**: Traditional heatmap algorithms treat GPS tracks as collections of points, using circular density kernels that poorly represent linear features like roads and trails [@PLACEHOLDER]. This approach creates artificial hotspots where GPS devices record more frequent updates, regardless of actual route usage.

**Variable sampling rates distort analysis**: GPS devices record data at different frequencies depending on device settings, battery optimization, and signal conditions [@PLACEHOLDER]. Point-based methods amplify these inconsistencies, making it difficult to accurately compare route popularity.

**Existing tools lack specialized algorithms**: Popular GIS software and libraries like QGIS, R's spatial packages, and Python's scipy focus on point data analysis [@PLACEHOLDER]. While these tools can process GPS tracks, they don't account for the linear nature of route data.

**Commercial solutions are inaccessible**: Platforms like Strava use proprietary algorithms for route analysis that aren't available for research or custom applications [@PLACEHOLDER]. This creates a gap for developers who need similar functionality in their own projects.

fastGeoToolkit addresses these issues by treating GPS tracks as sequences of connected segments rather than point clouds. This approach provides more accurate route frequency analysis without requiring complex preprocessing or server-side infrastructure.

# Implementation

## Segment-Based Algorithm

fastGeoToolkit's core algorithm processes GPS tracks in three steps:

**Track segmentation**: GPS tracks are split into consecutive coordinate pairs representing individual route segments. Each segment connects two adjacent GPS points, preserving the linear structure of the original path.

**Coordinate normalization**: To handle GPS measurement noise, coordinates are snapped to a tolerance grid. This reduces minor variations from GPS accuracy limitations while maintaining route integrity.

**Frequency calculation**: Each segment is converted to a normalized string key for efficient storage and lookup. A hash map tracks how many times each unique segment appears across all input tracks. Each track's final frequency is the average frequency of its constituent segments.

This approach ensures route popularity reflects actual overlapping usage rather than GPS sampling artifacts. Routes that share the same path segments will have higher frequencies, while unique routes will have lower frequencies.

## Performance and Architecture

The algorithm runs in O(n×m) time where n is the number of tracks and m is the average track length. Hash map lookups provide O(1) average-case performance for frequency queries.

The core implementation is written in Rust for memory safety and performance, then compiled to WebAssembly using wasm-pack. This enables browser-native execution without server dependencies while maintaining near-native computational speed.

![Example heatmap produced using fastgeotoolkit and MapLibre GL.](heatmap.png){#heatmap width="80%"}

The library is distributed as an npm package[^1] with TypeScript definitions, integrating naturally with existing JavaScript mapping libraries like Leaflet and MapLibre GL, allowing for its use in webapps as in the above example.



# Conclusion

fastGeoToolkit provides a practical solution for GPS route analysis by focusing on segments rather than points. This approach produces more accurate route density visualizations while being accessible through standard JavaScript tooling.

The segment-based algorithm handles the inherent challenges of GPS data - measurement noise, variable sampling rates, and device differences - without requiring complex preprocessing. Combined with WebAssembly performance and npm distribution, the library enables developers to build route analysis applications that run entirely in the browser.

The library is available on npm as `fastgeotoolkit` and includes comprehensive documentation and examples for common use cases.

# Acknowledgements

The author acknowledges the open-source geospatial community and the contributors who provided feedback during development.

# References

[^1]: Available at https://www.npmjs.com/package/fastgeotoolkit