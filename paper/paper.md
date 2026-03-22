---
title: 'fastgeotoolkit: A High-Performance Geospatial Analysis Library and Segment-Based Route Density Mapping Implementation'
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

fastgeotoolkit is a high-performance library for GPS track analysis that introduces a segment-based approach to route density mapping. Unlike traditional point-based heatmap algorithms, fastgeotoolkit processes GPS tracks as connected line segments to identify overlapping route usage patterns without the spatial clustering artifacts common in existing solutions.

The implementation handles common GPS data formats (GPX, FIT files, and Google polylines) and provides comprehensive track processing capabilities including data validation and statistics.

# Statement of Need

GPS route density visualization is essential for transportation research [@TransportationResearch], urban planning [@UrbanPlanning], trail management [@TrailManagement], and movement ecology [@MovementEcology]. Accurate route usage quantification supports evidence-based infrastructure decisions and mobility behavior analysis.

## Research Gap in Route-Based Analysis

Current geospatial analysis treats GPS trajectories as point collections, applying kernel density estimation or clustering algorithms [@Xu2024Dec]. This introduces several problems when dealing with data where the linear route is the primary feature.

**Sampling bias amplification**: GPS sampling rates vary by device settings and signal conditions [@Muller2022Apr]. Point-based methods make these inconsistencies worse, preventing accurate route comparison across datasets [@Xu2024Dec].

**Parameter sensitivity**: Results change dramatically based on kernel bandwidth and grid size choices [@SimpleHeatmap], making it difficult to establish consistent analysis methods across studies.

## Software Ecosystem Limitations

**Inadequate existing tools**: Popular GIS software (QGIS [@QGIS], R spatial packages [@sf; @sp], Python scipy [@SciPy]) focus on static point analysis, not linear route patterns.

**Computational barriers**: Most trajectory tools require preprocessing or server infrastructure [@TransportationResearch], limiting research accessibility.

**Proprietary algorithms**: Commercial platforms like Strava use linear route processing [@StravaProprietary] but keep implementations closed, creating gaps in open science.

## Research Contribution

fastgeotoolkit provides the first open-source, segment-based route density algorithm for GPS track data, enabling: (1) sampling-rate-independent frequency estimation, (2) browser-native analysis workflows, and (3) standardized trajectory analysis methodologies.

# Implementation

fastgeotoolkit addresses issues with existing heatmap implementations by treating GPS tracks as sequences of connected segments rather than point clouds. This approach provides more accurate route frequency analysis, and fastgeotoolkit implements it in such a way that it enables processing millions of tracks without preprocessing or server-side infrastructure.


## Segment-Based Algorithm

fastgeotoolkit's core algorithm processes GPS tracks in three steps:

**Track segmentation**: GPS tracks are split into consecutive coordinate pairs representing individual route segments. Each segment connects two adjacent GPS points, preserving the linear structure of the original path.

**Coordinate normalization**: To handle GPS measurement noise, coordinates are snapped to a tolerance grid. This reduces minor variations from GPS accuracy limitations while maintaining route integrity with high fidelity.

**Frequency calculation**: Each segment is converted to a normalized string key for efficient storage and lookup. A hash map tracks how many times each unique segment appears across all input tracks. Each track's final frequency is the average frequency of its constituent segments.

This approach ensures route popularity reflects actual overlapping usage rather than GPS sampling artifacts. Routes that share the same path segments will have higher frequencies, while unique routes will have lower frequencies.

## Performance and Architecture

The algorithm runs in O(n×m) time where n is the number of tracks and m is the average track length. Hash map lookups provide O(1) average-case performance for frequency queries.

The core implementation is written in Rust for memory safety and performance, then compiled to WebAssembly using wasm-pack. This enables browser-native execution without server dependencies while maintaining near-native computational speed [@Rust; @wasm].

![Example heatmap produced using fastgeotoolkit and MapLibre GL JS.[]{label="heatmap"}](heatmap.png){#heatmap width="100%"}

The library is distributed as an npm package[^1] with TypeScript definitions, integrating naturally with existing JavaScript mapping libraries like Leaflet and MapLibre GL JS [@leaflet; @maplibre]. This allows for easy use in webapps as seen in \autoref{heatmap}, a screenshot from the demo page for fastgeotoolkit.

# Conclusion

fastgeotoolkit provides a practical solution for GPS route analysis by focusing on segments rather than points. This approach produces more accurate route density visualizations while being accessible through standard JavaScript tooling.

The segment-based algorithm handles the inherent challenges of GPS data, especially measurement noise, variable sampling rates, and device differences, without requiring complex preprocessing. fastgeotoolkit implements this approach while remaining highly performant, which makes it largely unique in the landscape of GIS tooling for the web.


# Acknowledgements

The authors acknowledge the open-source geospatial community and the help of users who provided feedback during development.

# References

[^1]: Available at https://www.npmjs.com/package/fastgeotoolkit