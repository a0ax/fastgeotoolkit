<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import maplibregl from 'maplibre-gl';
  import 'maplibre-gl/dist/maplibre-gl.css';
  import { browser } from '$app/environment';
  import { PaneGroup, Pane, PaneResizer } from 'paneforge';
  // Import distance calculation for ruler
  import { distance } from '@turf/distance';

  // Custom Ruler Control for MapLibre
  class RulerControl {
	_map: null;
	_container: null;
	_button: null;
	_isActive: boolean;
	_points: never[];
	_lineSource: null;
	_pointSource: null;
	_distanceLabels: never[];
    constructor() {
      this._map = null;
      this._container = null;
      this._button = null;
      this._isActive = false;
      this._points = [];
      this._lineSource = null;
      this._pointSource = null;
      this._distanceLabels = [];
    }

    onAdd(map: null) {
      this._map = map;
      this._container = document.createElement('div');
      this._container.className = 'maplibregl-ctfrl maplibregl-ctrl-group ruler-control';
      
      this._button = document.createElement('button');
      this._button.className = 'ruler-control-button';
      this._button.type = 'button';
      this._button.setAttribute('aria-label', 'Measure distance');
      this._button.title = 'Measure distance';
      
      // Add ruler icon
      this._button.innerHTML = `
        <svg width="24" height="24" viewBox="1 0.5 22 21.5" fill="none" xmlns="http://www.w3.org/2000/svg" >
          <path fill-rule="evenodd" clip-rule="evenodd" d="M16.5352 3.29314C16.1446 2.90261 15.5115 2.90261 15.121 3.29314L3.29314 15.121C2.90261 15.5115 2.90261 16.1447 3.29314 16.5352L7.46471 20.7067C7.85523 21.0973 8.4884 21.0973 8.87892 20.7067L10.5857 18.9999L9.29283 17.7071C8.90231 17.3165 8.90231 16.6834 9.29283 16.2928C9.68336 15.9023 10.3165 15.9023 10.707 16.2928L11.9999 17.5857L14.0857 15.4999L13.2928 14.707C12.9023 14.3165 12.9023 13.6834 13.2928 13.2928C13.6834 12.9023 14.3165 12.9023 14.707 13.2928L15.4999 14.0857L17.5857 11.9999L16.2928 10.707C15.9023 10.3165 15.9023 9.68336 16.2928 9.29284C16.6834 8.90231 17.3165 8.90231 17.707 9.29284L18.9999 10.5857L20.7067 8.87892C21.0973 8.4884 21.0973 7.85524 20.7067 7.46471L16.5352 3.29314ZM13.7067 1.87892C14.8783 0.707353 16.7778 0.707349 17.9494 1.87892L22.121 6.0505C23.2925 7.22207 23.2925 9.12156 22.121 10.2931L10.2931 22.121C9.12156 23.2925 7.22207 23.2925 6.0505 22.121L1.87892 17.9494C0.707349 16.7778 0.707353 14.8783 1.87892 13.7067L13.7067 1.87892Z" fill="#000"></path>
        </svg>
      `;
      
      this._button.addEventListener('click', () => {
        this._toggle();
      });
      
      this._container.appendChild(this._button);
      
      return this._container;
    }

    onRemove() {
      if (this._isActive) {
        this._deactivate();
      }
      
      this._container.parentNode?.removeChild(this._container);
      this._map = null;
    }

    _toggle() {
      if (this._isActive) {
        this._deactivate();
      } else {
        this._activate();
      }
    }

    _activate() {
      this._isActive = true;
      this._button.classList.add('active');
      this._map.getCanvas().style.cursor = 'crosshair';
      
      // Add sources for lines and points
      if (!this._map.getSource('ruler-line')) {
        this._map.addSource('ruler-line', {
          type: 'geojson',
          data: {
            type: 'FeatureCollection',
            features: []
          }
        });
      }
      
      if (!this._map.getSource('ruler-points')) {
        this._map.addSource('ruler-points', {
          type: 'geojson',
          data: {
            type: 'FeatureCollection',
            features: []
          }
        });
      }
      
      // Add layers
      if (!this._map.getLayer('ruler-line-layer')) {
        this._map.addLayer({
          id: 'ruler-line-layer',
          type: 'line',
          source: 'ruler-line',
          paint: {
            'line-color': '#00bcd4', // Cyan
            'line-width': 3,
            'line-dasharray': [2, 2]
          }
        });
      }
      
      if (!this._map.getLayer('ruler-points-layer')) {
        this._map.addLayer({
          id: 'ruler-points-layer',
          type: 'circle',
          source: 'ruler-points',
          paint: {
            'circle-color': '#00bcd4', // Cyan
            'circle-radius': 6,
            'circle-stroke-color': '#ffffff',
            'circle-stroke-width': 2
          }
        });
      }
      
      // Add click handler
      this._map.on('click', this._onClick);
      this._map.on('dblclick', this._onDoubleClick);
      
      this._map.fire('ruler.on');
    }

    _deactivate() {
      this._isActive = false;
      this._button.classList.remove('active');
      this._map.getCanvas().style.cursor = '';
      
      // Remove event handlers
      this._map.off('click', this._onClick);
      this._map.off('dblclick', this._onDoubleClick);
      
      // Clear measurements
      this._clearMeasurements();
      
      this._map.fire('ruler.off');
    }

    _onClick = (e: { lngLat: { lng: any; lat: any; }; }) => {
      if (!this._isActive) return;
      
      const point = [e.lngLat.lng, e.lngLat.lat];
      this._points.push(point);
      
      this._updateDisplay();
    }

    _onDoubleClick = (e: { preventDefault: () => void; }) => {
      if (!this._isActive) return;
      e.preventDefault();
      
      // Finish the current measurement and start a new one
      this._points = [];
      this._updateDisplay();
    }

    _updateDisplay() {
      // Update points
      const pointFeatures = this._points.map((point, index) => ({
        type: 'Feature',
        geometry: {
          type: 'Point',
          coordinates: point
        },
        properties: {
          index: index
        }
      }));
      
      this._map.getSource('ruler-points').setData({
        type: 'FeatureCollection',
        features: pointFeatures
      });
      
      // Update lines and calculate distances
      if (this._points.length > 1) {
        const lineFeature = {
          type: 'Feature',
          geometry: {
            type: 'LineString',
            coordinates: this._points
          },
          properties: {}
        };
        
        this._map.getSource('ruler-line').setData({
          type: 'FeatureCollection',
          features: [lineFeature]
        });
        
        // Add distance labels
        this._updateDistanceLabels();
      } else {
        this._map.getSource('ruler-line').setData({
          type: 'FeatureCollection',
          features: []
        });
        this._clearDistanceLabels();
      }
    }

    _updateDistanceLabels() {
      this._clearDistanceLabels();
      
      let totalDistance = 0;
      
      for (let i = 1; i < this._points.length; i++) {
        const from = this._points[i - 1];
        const to = this._points[i];
        
        const segmentDistance = distance(
          { type: 'Point', coordinates: from },
          { type: 'Point', coordinates: to },
          { units: 'kilometers' }
        );
        
        totalDistance += segmentDistance;
        
        // Calculate midpoint for label placement
        const midpoint = [
          (from[0] + to[0]) / 2,
          (from[1] + to[1]) / 2
        ];
        
        // Create distance label
        const popup = new maplibregl.Popup({
          closeButton: false,
          closeOnClick: false,
          className: 'ruler-distance-popup'
        })
        .setLngLat(midpoint)
        .setHTML(`<div class="ruler-distance-label">${segmentDistance.toFixed(2)} km</div>`)
        .addTo(this._map);
        
        this._distanceLabels.push(popup);
      }
      
      // Add total distance label at the end
      if (this._points.length > 1) {
        const lastPoint = this._points[this._points.length - 1];
        const totalPopup = new maplibregl.Popup({
          closeButton: false,
          closeOnClick: false,
          className: 'ruler-total-popup'
        })
        .setLngLat(lastPoint)
        .setHTML(`<div class="ruler-total-label">Total: ${totalDistance.toFixed(2)} km</div>`)
        .addTo(this._map);
        
        this._distanceLabels.push(totalPopup);
      }
    }

    _clearDistanceLabels() {
      this._distanceLabels.forEach(popup => popup.remove());
      this._distanceLabels = [];
    }

    _clearMeasurements() {
      this._points = [];
      this._clearDistanceLabels();
      
      // Clear sources
      if (this._map.getSource('ruler-line')) {
        this._map.getSource('ruler-line').setData({
          type: 'FeatureCollection',
          features: []
        });
      }
      
      if (this._map.getSource('ruler-points')) {
        this._map.getSource('ruler-points').setData({
          type: 'FeatureCollection',
          features: []
        });
      }
      
      // Remove layers and sources
      if (this._map.getLayer('ruler-line-layer')) {
        this._map.removeLayer('ruler-line-layer');
      }
      if (this._map.getLayer('ruler-points-layer')) {
        this._map.removeLayer('ruler-points-layer');
      }
      if (this._map.getSource('ruler-line')) {
        this._map.removeSource('ruler-line');
      }
      if (this._map.getSource('ruler-points')) {
        this._map.removeSource('ruler-points');
      }
    }
  }

  // Import Strava service conditionally
  let stravaService: any = null;
  
  // Import RideWithGPS service conditionally
  let ridewithgpsService: any = null;

  // WASM module functions - will be loaded dynamically
  let wasmInit: any = null;
  let processGpxFiles: any = null;
  let processPolylines: any = null;

  let mapContainer: HTMLDivElement;
  let map: maplibregl.Map;
  let isLoading = false;
  let error = '';
  let segmentCount = 0;
  let currentBasemap = 'white';
  let currentHeatmapData: any = null; // Store heatmap data persistently
  let layerSwitcherSelect: HTMLSelectElement; // Reference to layer switcher dropdown

  // Global heatmap color constants
  const HEATMAP_COLORS = {
    COLD: '#471894',    // Purple - low intensity
    MEDIUM: '#e03400',  // Orange - medium intensity
    HOT: '#ffe600'      // Yellow - high intensity
  } as const;

  // Strava-related state
  let isStravaAuthenticated = false;
  let stravaAthlete: any = null;
  let isImportingStrava = false;
  let stravaActivityCount = 0;
  let stravaImportProgress = 0;
  let cachedActivities: any[] = [];

  // RideWithGPS-related state
  let isRWGPSAuthenticated = false;
  let rwgpsUser: any = null;
  let isImportingRWGPS = false;
  let rwgpsTripCount = 0;
  let rwgpsImportProgress = 0;
  let cachedTrips: any[] = [];

  // Panel layout state
  let isPanelCollapsed = false;
  let isMobile = false;

  const basemaps = {
    osm: {
      name: 'OpenStreetMap',
      style: {
        version: 8,
        sources: {
          'raster-tiles': {
            type: 'raster',
            tiles: ['https://tile.openstreetmap.org/{z}/{x}/{y}.png'],
            tileSize: 256,
            attribution: '© OpenStreetMap contributors'
          }
        },
        layers: [
          {
            id: 'simple-tiles',
            type: 'raster',
            source: 'raster-tiles',
            minzoom: 0,
            maxzoom: 22
          }
        ]
      }
    },
    white: {
      name: 'White',
      style: {
        version: 8,
        sources: {
          'carto-light': {
            type: 'raster', 
            tiles: ['https://a.basemaps.cartocdn.com/light_all/{z}/{x}/{y}.png'],
            tileSize: 256,
            attribution: '© CARTO'
          }
        },
        layers: [
          {
            id: 'white-tiles',
            type: 'raster',
            source: 'carto-light',
            minzoom: 0,
            maxzoom: 22
          }
        ]
      }
    },
    dark: {
      name: 'Dark',
      style: {
        version: 8,
        sources: {
          'carto-dark': {
            type: 'raster',
            tiles: ['https://a.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}.png'],
            tileSize: 256,
            attribution: '© CARTO'
          }
        },
        layers: [
          {
            id: 'dark-tiles',
            type: 'raster',
            source: 'carto-dark',
            minzoom: 0,
            maxzoom: 22
          }
        ]
      }
    },
  };

  async function loadSampleData() {
    try {
      isLoading = true;
      error = '';
      console.log('Loading sample track data...');

      // Load files 1 through 29.fit from static/tracks
      const filePromises = [];
      for (let i = 1; i <= 29; i++) {
        filePromises.push(
          fetch(`/tracks/${i}.fit`)
            .then(response => {
              if (!response.ok) {
                throw new Error(`Failed to load ${i}.fit: ${response.status}`);
              }
              return response.arrayBuffer();
            })
            .then(buffer => new Uint8Array(buffer))
        );
      }

      const buffers = await Promise.all(filePromises);
      console.log(`Loaded ${buffers.length} sample FIT files`);

      // Create JS array for WASM function (same as handleFiles)
      const jsArray = new Array();
      for (const buffer of buffers) {
        jsArray.push(buffer);
      }

      console.log('Calling WASM function with', jsArray.length, 'buffers');

      // Process the files same as handleFiles
      if (!processGpxFiles) {
        throw new Error('WASM not initialized. Please wait and try again.');
      }

      const result = await processGpxFiles(jsArray);
      console.log('Sample data processing result:', result);

      if (!result) {
        throw new Error('WASM function returned null/undefined result');
      }

      if (result && result.tracks && Array.isArray(result.tracks) && result.tracks.length > 0) {
        const heatmapResult = result as { tracks: any[], max_frequency: number };
        
        // Use the shared rendering function
        await renderHeatmapResult(heatmapResult);
        console.log('Sample data loaded successfully');
      } else {
        throw new Error('No tracks found in sample data.');
      }

    } catch (err) {
      console.error('Error loading sample data:', err);
      error = `Error loading sample data: ${err}`;
    } finally {
      isLoading = false;
    }
  }

  async function handleFiles(event: Event) {
    const input = event.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;

    console.log('Processing', input.files.length, 'files');
    isLoading = true;
    error = '';
    
    try {
      // Convert files to Uint8Array buffers
      const buffers = await Promise.all(
        Array.from(input.files).map(f => f.arrayBuffer().then(buf => new Uint8Array(buf)))
      );

      console.log('Converted files to buffers:', buffers.length);

      // Create JS array for WASM function
      const jsArray = new Array();
      for (const buffer of buffers) {
        jsArray.push(buffer);
      }

      console.log('Calling WASM function with', jsArray.length, 'buffers');

      // Process GPX files with WASM
      if (!processGpxFiles) {
        throw new Error('WASM module not loaded');
      }
      
      const result = await processGpxFiles(jsArray);
      
      if (!result) {
        throw new Error('WASM function returned null/undefined');
      }
      
      const heatmapResult = result as { tracks: any[], max_frequency: number };
      
      console.log('WASM returned', heatmapResult.tracks.length, 'tracks with max frequency:', heatmapResult.max_frequency);
      
      // Use the shared rendering function
      await renderHeatmapResult(heatmapResult);

    } catch (err) {
      error = `Error processing GPX/FIT files: ${err}`;
      console.error(err);
    } finally {
      isLoading = false;
    }
  }

  // Strava functions
  function connectToStrava() {
    if (!stravaService) return;
    const authUrl = stravaService.getAuthUrl();
    window.location.href = authUrl;
  }

  function disconnectFromStrava() {
    if (!stravaService) return;
    stravaService.logout();
    isStravaAuthenticated = false;
    stravaAthlete = null;
  }

  async function importStravaActivities() {
    if (!stravaService || !stravaService.isAuthenticated()) {
      error = 'Not connected to Strava';
      return;
    }

    isImportingStrava = true;
    stravaImportProgress = 0;
    error = '';

    try {
      console.log('Fetching Strava activities...');
      
      let activities: any[] = [];
      
      // Check if we have cached activities in localStorage
      if (browser) {
        const cached = localStorage.getItem('strava_activities');
        const cacheTimestamp = localStorage.getItem('strava_activities_timestamp');
        
        // Use cache if it's less than 1 hour old
        const cacheAge = cacheTimestamp ? Date.now() - parseInt(cacheTimestamp) : Infinity;
        const cacheExpiry = 60 * 60 * 1000; // 1 hour in milliseconds
        
        if (cached && cacheAge < cacheExpiry) {
          activities = JSON.parse(cached);
          // Safety check for parsed data
          if (!activities || !Array.isArray(activities)) {
            activities = [];
          }
          stravaActivityCount = activities.length;
          stravaImportProgress = 50;
          console.log(`Using cached ${activities.length} activities from localStorage`);
        }
      }
      
      // Fetch fresh data if no valid cache
      if (!activities || activities.length === 0) {
        activities = await stravaService.fetchAllActivities((count: number) => {
          stravaActivityCount = count;
          stravaImportProgress = Math.min(50, count); // First 50% for fetching
        });
        
        // Safety check for fetched data
        if (!activities || !Array.isArray(activities)) {
          activities = [];
        }
        
        // Cache the activities in localStorage with safe storage
        if (browser && activities.length > 0) {
          const success = safeLocalStorageSetItem('strava_activities', JSON.stringify(activities));
          if (success) {
            safeLocalStorageSetItem('strava_activities_timestamp', Date.now().toString());
            console.log(`Cached ${activities.length} activities to localStorage`);
          } else {
            console.warn('Failed to cache activities - localStorage quota exceeded or unavailable');
          }
        }
      }

      console.log(`Processing ${activities.length} activities from Strava`);

      if (!activities || activities.length === 0) {
        error = 'No activities found in your Strava account';
        return;
      }

      // Extract polylines from activities
      const polylines = stravaService.getPolylinesFromActivities(activities);
      
      console.log('DEBUG: Extracted polylines:', {
        count: polylines.length,
        firstPolyline: polylines[0]?.substring(0, 100) + '...',
        polylineTypes: polylines.slice(0, 3).map(p => typeof p),
        polyllineLengths: polylines.slice(0, 3).map(p => p?.length)
      });
      
      if (polylines.length === 0) {
        error = 'No GPS data found in your Strava activities';
        return;
      }

      console.log(`Processing ${polylines.length} polylines...`);
      stravaImportProgress = 75;

      // Process polylines with our WASM function
      if (!processPolylines) {
        throw new Error('WASM module not loaded');
      }
      const jsArray = new Array();
      for (const polyline of polylines) {
        jsArray.push(polyline);
      }

      console.log('DEBUG: About to call processPolylines with:', {
        arrayLength: jsArray.length,
        firstPolylineSample: jsArray[0]?.substring(0, 50) + '...',
        arrayType: Array.isArray(jsArray),
        functionType: typeof processPolylines
      });

      const result = await processPolylines(jsArray);
      
      console.log('DEBUG: processPolylines returned:', {
        result: result,
        resultType: typeof result,
        isNull: result === null,
        isUndefined: result === undefined,
        resultKeys: result ? Object.keys(result) : 'N/A',
        stringified: JSON.stringify(result)?.substring(0, 200) + '...'
      });
      
      if (!result) {
        throw new Error('WASM function returned null/undefined result');
      }
      
      const heatmapResult = result as { tracks: any[], max_frequency: number };
      
      console.log('DEBUG: heatmapResult after casting:', {
        hasTracks: 'tracks' in heatmapResult,
        tracksType: typeof heatmapResult.tracks,
        tracksIsArray: Array.isArray(heatmapResult.tracks),
        tracksLength: heatmapResult.tracks?.length,
        maxFrequency: heatmapResult.max_frequency
      });
      
      if (!heatmapResult.tracks || !Array.isArray(heatmapResult.tracks)) {
        throw new Error(`WASM function returned invalid tracks data. Got: ${JSON.stringify(heatmapResult)}`);
      }
      
      console.log('WASM returned', heatmapResult.tracks.length, 'tracks with max frequency:', heatmapResult.max_frequency);
      
      stravaImportProgress = 90;

      // Use the same rendering logic as GPX files
      await renderHeatmapResult(heatmapResult);
      
      stravaImportProgress = 100;
      console.log('Strava import completed successfully');

    } catch (err) {
      // Handle localStorage quota errors more gracefully
      if (err instanceof DOMException && err.name === 'QuotaExceededError') {
        console.warn('localStorage quota exceeded during Strava import');
        error = 'Import completed but could not cache data (storage full). Try clearing browser storage.';
      } else {
        error = `Error importing Strava activities: ${err}`;
      }
      console.error(err);
    } finally {
      isImportingStrava = false;
      stravaImportProgress = 0;
    }
  }

  // RideWithGPS functions
  function connectToRWGPS() {
    if (!ridewithgpsService) return;
    const authUrl = ridewithgpsService.getAuthUrl();
    window.location.href = authUrl;
  }

  function disconnectFromRWGPS() {
    if (!ridewithgpsService) return;
    ridewithgpsService.logout();
    isRWGPSAuthenticated = false;
    rwgpsUser = null;
  }

  async function importRWGPSTrips() {
    if (!ridewithgpsService || !ridewithgpsService.isAuthenticated()) {
      error = 'Not connected to RideWithGPS';
      return;
    }

    isImportingRWGPS = true;
    rwgpsImportProgress = 0;
    error = '';

    try {
      console.log('Fetching RideWithGPS trips...');
      
      let trips: any[] = [];
      
      // Check if we have cached trips in localStorage
      if (browser) {
        const cached = localStorage.getItem('rwgps_trips');
        const cacheTimestamp = localStorage.getItem('rwgps_trips_timestamp');
        
        // Use cache if it's less than 1 hour old
        const cacheAge = cacheTimestamp ? Date.now() - parseInt(cacheTimestamp) : Infinity;
        const cacheExpiry = 60 * 60 * 1000; // 1 hour in milliseconds
        
        if (cached && cacheAge < cacheExpiry) {
          trips = JSON.parse(cached);
          // Safety check for parsed data
          if (!trips || !Array.isArray(trips)) {
            trips = [];
          }
          rwgpsTripCount = trips.length;
          rwgpsImportProgress = 50;
          console.log(`Using cached ${trips.length} trips from localStorage`);
        }
      }
      
      // Fetch fresh data if no valid cache
      if (trips.length === 0) {
        trips = await ridewithgpsService.fetchAllTrips((count: number) => {
          rwgpsTripCount = count;
          rwgpsImportProgress = Math.min(50, count); // First 50% for fetching
        });
        
        // Cache the trips in localStorage with safe storage
        if (browser && trips.length > 0) {
          const success = safeLocalStorageSetItem('rwgps_trips', JSON.stringify(trips));
          if (success) {
            safeLocalStorageSetItem('rwgps_trips_timestamp', Date.now().toString());
            console.log(`Cached ${trips.length} trips to localStorage`);
          } else {
            console.warn('Failed to cache trips - localStorage quota exceeded or unavailable');
          }
        }
      }

      console.log(`Processing ${trips.length} trips from RideWithGPS`);

      if (trips.length === 0) {
        error = 'No trips found in your RideWithGPS account';
        return;
      }

      // Extract polylines from trips (now async because we need to fetch track data)
      console.log('Extracting GPS data from trips...');
      rwgpsImportProgress = 50;
      
      const polylines = await ridewithgpsService.getPolylinesFromTrips(trips, (processed: number, total: number) => {
        // Update progress as we process each trip
        const progressPercent = 50 + Math.floor((processed / total) * 25); // 50-75% range
        rwgpsImportProgress = progressPercent;
      });
      
      if (polylines.length === 0) {
        error = 'No GPS data found in your RideWithGPS trips';
        return;
      }

      console.log(`Processing ${polylines.length} polylines...`);
      rwgpsImportProgress = 75;

      // Process polylines with our WASM function
      if (!processPolylines) {
        throw new Error('WASM module not loaded');
      }
      const jsArray = new Array();
      for (const polyline of polylines) {
        jsArray.push(polyline);
      }

      const result = await processPolylines(jsArray);
      
      if (!result) {
        throw new Error('WASM function returned null/undefined result');
      }
      
      const heatmapResult = result as { tracks: any[], max_frequency: number };
      
      if (!heatmapResult.tracks || !Array.isArray(heatmapResult.tracks)) {
        throw new Error('WASM function returned invalid tracks data');
      }
      
      console.log('WASM returned', heatmapResult.tracks.length, 'tracks with max frequency:', heatmapResult.max_frequency);
      
      rwgpsImportProgress = 90;

      // Use the same rendering logic as GPX files
      await renderHeatmapResult(heatmapResult);
      
      rwgpsImportProgress = 100;
      console.log('RideWithGPS import completed successfully');

    } catch (err) {
      // Handle localStorage quota errors more gracefully
      if (err instanceof DOMException && err.name === 'QuotaExceededError') {
        console.warn('localStorage quota exceeded during RideWithGPS import');
        error = 'Import completed but could not cache data (storage full). Try clearing browser storage.';
      } else {
        error = `Error importing RideWithGPS trips: ${err}`;
      }
      console.error(err);
    } finally {
      isImportingRWGPS = false;
      rwgpsImportProgress = 0;
    }
  }

  // Extract heatmap rendering logic into a reusable function
  async function renderHeatmapResult(heatmapResult: { tracks: any[], max_frequency: number }) {
    // Debug frequency distribution
    const frequencies = heatmapResult.tracks.map(t => t.frequency);
    const freqCounts = frequencies.reduce((acc, freq) => {
      acc[freq] = (acc[freq] || 0) + 1;
      return acc;
    }, {} as Record<number, number>);
    console.log('Frequency distribution:', freqCounts);
    console.log('Unique frequencies:', [...new Set(frequencies)].sort((a, b) => a - b));
    
    // Calculate percentiles for better distribution
    const sortedFreqs = frequencies.slice().sort((a, b) => a - b);
    const p50 = sortedFreqs[Math.floor(sortedFreqs.length * 0.5)];
    const p75 = sortedFreqs[Math.floor(sortedFreqs.length * 0.75)];
    const p90 = sortedFreqs[Math.floor(sortedFreqs.length * 0.9)];
    console.log(`Percentiles: 50th=${p50}, 75th=${p75}, 90th=${p90}, max=${heatmapResult.max_frequency}`);
    
    segmentCount = heatmapResult.tracks.length;

    if (heatmapResult.tracks.length === 0) {
      throw new Error('No tracks found in data. Please check that the files contain valid GPS tracks.');
    }

    // Convert to GeoJSON with percentile-based opacity
    const geojson = {
      type: "FeatureCollection" as const,
      features: heatmapResult.tracks.map(track => ({
        type: "Feature" as const,
        geometry: {
          type: "LineString" as const,
          coordinates: track.coordinates.map((coord: number[]) => [coord[1], coord[0]]) // [lon, lat]
        },
        properties: { 
          frequency: track.frequency,
          opacity_hot: calculateOpacityPercentile(track.frequency, p75, p90, heatmapResult.max_frequency, 'hot'),
          opacity_medium: calculateOpacityPercentile(track.frequency, p50, p75, p90, 'medium')
        }
      }))
    };

    // Store heatmap data persistently
    currentHeatmapData = geojson;

    // Clear existing layers
    ['heatmap-cold', 'heatmap-medium', 'heatmap-hot'].forEach(layerId => {
      if (map.getLayer(layerId)) {
        map.removeLayer(layerId);
      }
    });
    ['heatmap'].forEach(sourceId => {
      if (map.getSource(sourceId)) {
        map.removeSource(sourceId);
      }
    });

    // Add the single source for all layers
    map.addSource('heatmap', {
      type: 'geojson',
      data: geojson
    });

    // Layer 1: Cold (purple, lowest z-index)
    map.addLayer({
      id: 'heatmap-cold',
      type: 'line',
      source: 'heatmap',
      paint: {
        'line-color': HEATMAP_COLORS.COLD,
        'line-width': [
          'interpolate', 
          ['linear'], 
          ['zoom'],
          1, 2,
          5, 3,
          10, 4,
          15, 6
        ],
        'line-opacity': 0.7
      },
      layout: {
        'line-join': 'round',
        'line-cap': 'round'
      }
    });

    // Layer 2: Medium (orange, middle z-index)
    map.addLayer({
      id: 'heatmap-medium',
      type: 'line',
      source: 'heatmap',
      paint: {
        'line-color': HEATMAP_COLORS.MEDIUM,
        'line-width': [
          'interpolate', 
          ['linear'], 
          ['zoom'],
          1, 1.5,
          5, 2.5,
          10, 3.5,
          15, 5.5
        ],
        'line-opacity': ['get', 'opacity_medium']
      },
      layout: {
        'line-join': 'round',
        'line-cap': 'round'
      }
    });

    // Layer 3: Hot (yellow, highest z-index)
    map.addLayer({
      id: 'heatmap-hot',
      type: 'line',
      source: 'heatmap',
      paint: {
        'line-color': HEATMAP_COLORS.HOT,
        'line-width': [
          'interpolate', 
          ['linear'], 
          ['zoom'],
          1, 1,
          5, 2,
          10, 3,
          15, 5
        ],
        'line-opacity': ['get', 'opacity_hot']
      },
      layout: {
        'line-join': 'round',
        'line-cap': 'round'
      }
    });
    
    console.log('heatmap added successfully');

    // Fit map to show all tracks
    if (heatmapResult.tracks.length > 0) {
      const bounds = new maplibregl.LngLatBounds();
      heatmapResult.tracks.forEach(track => {
        track.coordinates.forEach((coord: number[]) => {
          bounds.extend([coord[1], coord[0]]); // [lon, lat]
        });
      });
      map.fitBounds(bounds, { padding: 50 });
    }
  }

  // Calculate opacity based on percentiles for better visual distribution
  function calculateOpacityPercentile(frequency: number, threshold1: number, threshold2: number, threshold3: number, layer: 'hot' | 'medium'): number {
    if (layer === 'hot') {
      // Yellow layer: more generous - show for frequencies > 60th percentile (was 75th)
      const hotThreshold = threshold1 * 0.6; // Lower threshold
      if (frequency <= hotThreshold) {
        return 0;
      }
      // Smooth transition from 60th percentile to max
      const range = threshold3 - hotThreshold;
      if (range <= 0) return 0.8;
      return Math.min(0.9, 0.2 + (frequency - hotThreshold) / range * 0.05);
    } else {
      // Red layer: more generous - show for frequencies > 30th percentile (was 50th)
      const mediumThreshold = threshold1 * 0.1; // Much lower threshold
      if (frequency <= mediumThreshold) {
        return 0;
      }
      // Smooth transition from 30th to 60th percentile
      const range = threshold1 - mediumThreshold;
      if (range <= 0) return 0.6;
      const normalized = Math.min(1, (frequency - mediumThreshold) / range);
      return 0.2 + normalized * 0.05;
    }
  }

  // Calculate opacity based on frequency using Strava's power function approach
  function calculateOpacity(frequency: number, maxFrequency: number, layer: 'hot' | 'medium'): number {
    if (maxFrequency <= 1) {
      return layer === 'hot' ? 0 : 0;
    }
    
    const normalized = frequency / maxFrequency;
    
    if (layer === 'hot') {
      // Yellow layer: only show for frequencies > 60% of max
      if (frequency <= maxFrequency * 0.6) {
        return 0;
      }
      return Math.pow((frequency - maxFrequency * 0.6) / (maxFrequency * 0.4), 2) * 0.9;
    } else {
      // Red layer: show for frequencies > 30% of max
      if (frequency <= maxFrequency * 0.3) {
        return 0;
      }
      return Math.pow((frequency - maxFrequency * 0.3) / (maxFrequency * 0.7), 1.2) * 0.7;
    }
  }

  function clearHeatmap() {
    // Remove all heatmap layers
    ['heatmap-cold', 'heatmap-medium', 'heatmap-hot'].forEach(layerId => {
      if (map.getLayer(layerId)) {
        map.removeLayer(layerId);
      }
    });
    
    // Remove all heatmap sources
    ['heatmap'].forEach(sourceId => {
      if (map.getSource(sourceId)) {
        map.removeSource(sourceId);
      }
    });
    
    segmentCount = 0;
    currentHeatmapData = null; // Clear stored data
  }

  function addCustomAttribution() {
    if (!map) return;
    
    try {
      // Set the custom attribution using MapLibre's method
      // This should be more reliable than DOM manipulation
      const stravaAttribution = '<a href="https://www.strava.com" target="_blank" rel="noopener"><img src="/img/icon/powered_by_strava.svg" alt="Powered by Strava" style="height: 8px; width: 60px; vertical-align: middle; display: inline-block;"></a>';
      
      const sources = map.getStyle().sources;
      const attributions = [];
      
      // Collect existing attributions from sources
      for (const sourceId in sources) {
        const source = sources[sourceId];
        if (source.attribution) {
          attributions.push(source.attribution);
        }
      }
      
      // Add our custom attribution if not already present
      const hasStravaAttribution = attributions.some(attr => 
        attr.includes('strava.com') || attr.includes('Powered by Strava')
      );
      
      if (!hasStravaAttribution) {
        attributions.push(stravaAttribution);
      }
      
      // Force update attribution control
      const attributionControl = map._controls.find((control: any) => 
        control.constructor.name === 'AttributionControl'
      );
      
      if (attributionControl && attributionControl._updateAttributions) {
        // Temporarily add a source with our custom attribution
        if (!map.getSource('strava-attribution')) {
          map.addSource('strava-attribution', {
            type: 'geojson',
            data: { type: 'FeatureCollection', features: [] },
            attribution: stravaAttribution
          });
        }
      }
    } catch (error) {
      console.warn('Failed to add custom attribution:', error);
    }
  }

  // Function to update the layer switcher dropdown
  function updateLayerSwitcherDropdown() {
    if (layerSwitcherSelect) {
      layerSwitcherSelect.value = currentBasemap;
    }
  }

  async function switchBasemap(basemapKey: string) {
    if (!map || currentBasemap === basemapKey) return;
    
    console.log('Switching basemap from', currentBasemap, 'to', basemapKey);
    
    const basemapConfig = basemaps[basemapKey as keyof typeof basemaps];
    
    // Store current heatmap layers before switching
    const hasHeatmap = map.getSource('heatmap');
    let heatmapLayers: any[] = [];
    
    if (hasHeatmap) {
      // Store heatmap layer configs
      ['heatmap-cold', 'heatmap-medium', 'heatmap-hot'].forEach(layerId => {
        const layer = map.getLayer(layerId);
        if (layer) {
          heatmapLayers.push({
            id: layerId,
            type: layer.type,
            source: layer.source,
            paint: layer.paint,
            layout: layer.layout
          });
        }
      });
    }
    
    try {
      let style;
      
      if (basemapConfig.styleUrl) {
        // Load vector style from JSON file
        console.log('Loading vector style from:', basemapConfig.styleUrl);
        const response = await fetch(basemapConfig.styleUrl);
        if (!response.ok) {
          throw new Error(`Failed to load style: ${response.status}`);
        }
        style = await response.json();
      } else {
        // Use inline style
        console.log('Using inline style for:', basemapKey);
        style = basemapConfig.style;
      }
      
      if (hasHeatmap && currentHeatmapData) {
        // If we have heatmap data, add the heatmap source and layers to the new style
        // to prevent them from being removed
        style.sources = style.sources || {};
        style.sources.heatmap = {
          type: 'geojson',
          data: currentHeatmapData
        };
        
        style.layers = style.layers || [];
        
        // Add heatmap layers to the new style
        style.layers.push({
          id: 'heatmap-cold',
          type: 'line',
          source: 'heatmap',
          paint: {
            'line-color': HEATMAP_COLORS.COLD,
            'line-width': [
              'interpolate', 
              ['linear'], 
              ['zoom'],
              1, 2, 5, 3, 10, 4, 15, 6
            ],
            'line-opacity': 0.7
          },
          layout: {
            'line-join': 'round',
            'line-cap': 'round'
          }
        });

        style.layers.push({
          id: 'heatmap-medium',
          type: 'line',
          source: 'heatmap',
          paint: {
            'line-color': HEATMAP_COLORS.MEDIUM,
            'line-width': [
              'interpolate', 
              ['linear'], 
              ['zoom'],
              1, 1.5, 5, 2.5, 10, 3.5, 15, 5.5
            ],
            'line-opacity': ['get', 'opacity_medium']
          },
          layout: {
            'line-join': 'round',
            'line-cap': 'round'
          }
        });

        style.layers.push({
          id: 'heatmap-hot',
          type: 'line',
          source: 'heatmap',
          paint: {
            'line-color': HEATMAP_COLORS.HOT,
            'line-width': [
              'interpolate', 
              ['linear'], 
              ['zoom'],
              1, 1, 5, 2, 10, 3, 15, 5
            ],
            'line-opacity': ['get', 'opacity_hot']
          },
          layout: {
            'line-join': 'round',
            'line-cap': 'round'
          }
        });
        
        console.log('Added heatmap layers to new style');
      }
      
      // Apply the style with heatmap layers already included
      map.setStyle(style);
      
      // Re-add custom attribution after style change
      map.once('styledata', () => {
        addCustomAttribution();
        // Only update currentBasemap after style is successfully loaded
        currentBasemap = basemapKey;
        updateLayerSwitcherDropdown(); // Update dropdown to reflect change
        console.log('Basemap switched successfully to:', basemapKey);
      });
      
    } catch (error) {
      console.error('Error switching basemap:', error);
      // Fallback to a simple style if the vector style fails
      if (basemapConfig.styleUrl) {
        console.log('Falling back to OSM style');
        const osmStyle = JSON.parse(JSON.stringify(basemaps.osm.style)); // Deep copy
        
        if (hasHeatmap && currentHeatmapData) {
          // Add heatmap to fallback style too
          osmStyle.sources.heatmap = {
            type: 'geojson',
            data: currentHeatmapData
          };
          
          osmStyle.layers.push({
            id: 'heatmap-cold',
            type: 'line',
            source: 'heatmap',
            paint: {
              'line-color': '#471894',
              'line-width': [
                'interpolate', 
                ['linear'], 
                ['zoom'],
                1, 2, 5, 3, 10, 4, 15, 6
              ],
              'line-opacity': 0.7
            },
            layout: {
              'line-join': 'round',
              'line-cap': 'round'
            }
          });

          osmStyle.layers.push({
            id: 'heatmap-medium',
            type: 'line',
            source: 'heatmap',
            paint: {
              'line-color': '#F97316',
              'line-width': [
                'interpolate', 
                ['linear'], 
                ['zoom'],
                1, 1.5, 5, 2.5, 10, 3.5, 15, 5.5
              ],
              'line-opacity': ['get', 'opacity_medium']
            },
            layout: {
              'line-join': 'round',
              'line-cap': 'round'
            }
          });

          osmStyle.layers.push({
            id: 'heatmap-hot',
            type: 'line',
            source: 'heatmap',
            paint: {
              'line-color': '#ffe600',
              'line-width': [
                'interpolate', 
                ['linear'], 
                ['zoom'],
                1, 1, 5, 2, 10, 3, 15, 5
              ],
              'line-opacity': ['get', 'opacity_hot']
            },
            layout: {
              'line-join': 'round',
              'line-cap': 'round'
            }
          });
        }
        
        map.setStyle(osmStyle);
        
        // Re-add custom attribution after fallback style change
        map.once('styledata', () => {
          addCustomAttribution();
          // Only update currentBasemap after fallback style is successfully loaded
          currentBasemap = 'osm';
          updateLayerSwitcherDropdown(); // Update dropdown to reflect fallback
          console.log('Fallback to OSM completed');
        });
      }
    }
  }

  function restoreHeatmapLayers(heatmapData: any) {
    if (!heatmapData || !map) {
      console.log('No heatmap data to restore or map not available');
      return;
    }
    
    try {
      console.log('Restoring heatmap layers with', heatmapData.features?.length, 'features');
      
      // Remove existing heatmap layers and sources if they exist
      ['heatmap-cold', 'heatmap-medium', 'heatmap-hot'].forEach(layerId => {
        if (map.getLayer(layerId)) {
          map.removeLayer(layerId);
        }
      });
      
      if (map.getSource('heatmap')) {
        map.removeSource('heatmap');
      }
      
      // Add the source
      map.addSource('heatmap', {
        type: 'geojson',
        data: heatmapData
      });

      // Add all three layers in order (same as in renderHeatmapResult)
      map.addLayer({
        id: 'heatmap-cold',
        type: 'line',
        source: 'heatmap',
        paint: {
          'line-color': '#471894',
          'line-width': [
            'interpolate', 
            ['linear'], 
            ['zoom'],
            1, 2, 5, 3, 10, 4, 15, 6
          ],
          'line-opacity': 0.7
        },
        layout: {
          'line-join': 'round',
          'line-cap': 'round'
        }
      });

      map.addLayer({
        id: 'heatmap-medium',
        type: 'line',
        source: 'heatmap',
        paint: {
          'line-color': '#F97316',
          'line-width': [
            'interpolate', 
            ['linear'], 
            ['zoom'],
            1, 1.5, 5, 2.5, 10, 3.5, 15, 5.5
          ],
          'line-opacity': ['get', 'opacity_medium']
        },
        layout: {
          'line-join': 'round',
          'line-cap': 'round'
        }
      });

      map.addLayer({
        id: 'heatmap-hot',
        type: 'line',
        source: 'heatmap',
        paint: {
          'line-color': '#ffe600',
          'line-width': [
            'interpolate', 
            ['linear'], 
            ['zoom'],
            1, 1, 5, 2, 10, 3, 15, 5
          ],
          'line-opacity': ['get', 'opacity_hot']
        },
        layout: {
          'line-join': 'round',
          'line-cap': 'round'
        }
      });
      
      console.log('Heatmap layers restored successfully');
      
    } catch (error) {
      console.error('Error restoring heatmap layers:', error);
    }
  }

  function getHeatmapPaintConfig(tier: string, maxCount: number) {
    const configs = {
      low: {
        paint: {
          'line-color': [
            'interpolate', 
            ['linear'], 
            ['get', 'count'],
            1, '#471894',
            Math.floor(maxCount * 0.2), '#3366ff'
          ],
          'line-width': [
            'interpolate', 
            ['linear'], 
            ['zoom'],
            1, 1.5, 5, 2, 10, 3, 15, 4
          ],
          'line-opacity': 0.7
        }
      },
      medium: {
        paint: {
          'line-color': [
            'interpolate', 
            ['linear'], 
            ['get', 'count'],
            Math.floor(maxCount * 0.2), '#9929ff',
            Math.floor(maxCount * 0.6), '#ff6600'
          ],
          'line-width': [
            'interpolate', 
            ['linear'], 
            ['zoom'],
            1, 2, 5, 3, 10, 5, 15, 7
          ],
          'line-opacity': 0.8
        }
      },
      high: {
        paint: {
          'line-color': [
            'interpolate', 
            ['linear'], 
            ['get', 'count'],
            Math.floor(maxCount * 0.6), '#ff8800',
            maxCount, '#ffe600'
          ],
          'line-width': [
            'interpolate', 
            ['linear'], 
            ['zoom'],
            1, 3, 5, 5, 10, 8, 15, 12
          ],
          'line-opacity': 0.9
        }
      }
    };
    
    return configs[tier as keyof typeof configs] || configs.low;
  }

  onMount(async () => {
    try {
      console.log('Starting initialization...');
      
      // Set up responsive behavior
      if (browser) {
        handleResize();
        window.addEventListener('resize', handleResize);
      }
      
      // Wait a bit to ensure DOM is ready
      await new Promise(resolve => setTimeout(resolve, 100));
      console.log('Map container:', mapContainer);
      
      if (!mapContainer) {
        throw new Error('Map container not found');
      }
      
      // Initialize WASM module dynamically in browser only
      if (browser) {
        try {
          console.log('Attempting to load fastgeotoolkit...');
          
          // Import the published fastgeotoolkit npm package
          const fastGeoToolkit = await import('fastgeotoolkit');
          console.log('fastgeotoolkit module loaded:', fastGeoToolkit);
          
          // Use the package's built-in WASM initialization if available
          if (typeof fastGeoToolkit.initWithWasm === 'function') {
            console.log('Using built-in WASM initialization...');
            await fastGeoToolkit.initWithWasm();
          } else if (typeof fastGeoToolkit.ensureWasmInitialized === 'function') {
            console.log('Using ensureWasmInitialized...');
            await fastGeoToolkit.ensureWasmInitialized();
          }
          
          // Assign the functions - they should now work with WASM loaded
          processGpxFiles = fastGeoToolkit.processGpxFiles;
          processPolylines = fastGeoToolkit.processPolylines;
          
          console.log('WASM initialized successfully from fastgeotoolkit npm package');
        } catch (err) {
          console.error('Failed to load fastgeotoolkit npm package:', err);
          console.error('Full error details:', err);
          
          // If the automatic WASM loading fails, try to load from a CDN as fallback
          try {
            console.log('Trying CDN fallback...');
            const cdnModule = await import('https://unpkg.com/fastgeotoolkit@latest/dist/index.esm.js');
            
            if (typeof cdnModule.initWithWasm === 'function') {
              await cdnModule.initWithWasm();
            }
            
            processGpxFiles = cdnModule.processGpxFiles;
            processPolylines = cdnModule.processPolylines;
            
            console.log('Successfully loaded from CDN fallback');
          } catch (cdnErr) {
            console.error('CDN fallback also failed:', cdnErr);
            error = `Failed to load fastgeotoolkit module: ${err}. CDN fallback also failed: ${cdnErr}`;
            
            // Provide more specific error information
            if (err instanceof Error) {
              if (err.message.includes('WASM')) {
                error = 'WASM module failed to load. This may be due to Content Security Policy or bundling issues.';
              } else if (err.message.includes('Failed to fetch')) {
                error = 'Network error loading the module. Please check your internet connection.';
              } else if (err.message.includes('Failed to resolve module specifier')) {
                error = 'Module resolution error. The fastgeotoolkit package may not be properly bundled for this environment.';
              }
            }
          }
        }
      }

      // Create map with initial style
      const initialBasemap = basemaps[currentBasemap as keyof typeof basemaps];
      let initialStyle;
      
      if (initialBasemap.styleUrl) {
        // For vector styles, we'll load them dynamically after map creation
        // Start with a simple style first
        initialStyle = basemaps.osm.style;
      } else {
        initialStyle = initialBasemap.style;
      }
      
      map = new maplibregl.Map({
        container: mapContainer,
        style: initialStyle as any,
        center: [-96, 37.8],
        zoom: 3,
        customAttribution: '<a href="https://www.strava.com" target="_blank" rel="noopener"><img src="/img/icon/powered_by_strava.svg" alt="Powered by Strava" style="height: 14px; vertical-align: middle; margin-left: 4px;"></a>'
      });

      console.log('Map created:', map);

      // Ensure custom attribution is added after initial load
      map.once('load', () => {
        addCustomAttribution();
      });

      // If the initial basemap was a vector style, switch to it after map loads
      if (initialBasemap.styleUrl && currentBasemap !== 'osm') {
        map.once('load', () => {
          switchBasemap(currentBasemap);
        });
      }

      // Add navigation controls
      map.addControl(new maplibregl.NavigationControl(), 'top-right');
      
      // Add fullscreen control
      map.addControl(new maplibregl.FullscreenControl(), 'top-right');
      
      // Add ruler control
      map.addControl(new RulerControl(), 'top-right');
      
      // Add ruler event listeners for debugging
      map.on('ruler.on', () => {
        console.log('Ruler activated');
      });
      
      map.on('ruler.off', () => {
        console.log('Ruler deactivated');
      });
      
      // Add layer switcher control (vanilla HTML dropdown)
      let layerSwitcherSelect: HTMLSelectElement;
      const layerSwitcher = {
        onAdd: function(map: maplibregl.Map) {
          this._map = map;
          this._container = document.createElement('div');
          this._container.className = 'maplibregl-ctrl maplibregl-ctrl-group layer-switcher';
          
          // Create select element
          const select = document.createElement('select');
          select.className = 'layer-switcher-select';
          select.setAttribute('aria-label', 'Choose basemap layer');
          layerSwitcherSelect = select; // Store reference for updates
          
          // Add options
          Object.entries(basemaps).forEach(([key, basemap]) => {
            const option = document.createElement('option');
            option.value = key;
            option.textContent = basemap.name;
            option.selected = currentBasemap === key;
            select.appendChild(option);
          });
          
          // Handle selection changes
          select.addEventListener('change', async (e) => {
            const target = e.target as HTMLSelectElement;
            await switchBasemap(target.value);
          });
          
          // Create wrapper with icon
          const wrapper = document.createElement('div');
          wrapper.className = 'layer-switcher-wrapper';
          
          // Add icon
          const icon = document.createElement('div');
          icon.className = 'layer-switcher-icon';
          
          // Load the layer icon SVG
          fetch('/static/img/icon/layer.svg')
            .then(response => response.text())
            .then(svgText => {
              const parser = new DOMParser();
              const svgDoc = parser.parseFromString(svgText, 'image/svg+xml');
              const svgElement = svgDoc.querySelector('svg');
              if (svgElement) {
                svgElement.setAttribute('width', '20');
                svgElement.setAttribute('height', '20');
                // Update the fill color to currentColor for theming
                const paths = svgElement.querySelectorAll('path');
                paths.forEach(path => path.setAttribute('fill', 'currentColor'));
                icon.appendChild(svgElement);
              }
            })
            .catch(() => {
              // Fallback to inline SVG
              icon.innerHTML = `
                <svg width="20" height="20" viewBox="0 0 576 512" fill="currentColor">
                  <path d="M264.5 5.2c14.9-6.9 32.1-6.9 47 0l218.6 101c8.5 3.9 13.9 12.4 13.9 21.8s-5.4 17.9-13.9 21.8l-218.6 101c-14.9 6.9-32.1 6.9-47 0L45.9 149.8C37.4 145.8 32 137.3 32 128s5.4-17.9 13.9-21.8L264.5 5.2zM476.9 209.6l53.2 24.6c8.5 3.9 13.9 12.4 13.9 21.8s-5.4 17.9-13.9 21.8l-218.6 101c-14.9 6.9-32.1 6.9-47 0L45.9 277.8C37.4 273.8 32 265.3 32 256s5.4-17.9 13.9-21.8l53.2-24.6 152 70.2c23.4 10.8 50.4 10.8 73.8 0l152-70.2zm-152 198.2l152-70.2 53.2 24.6c8.5 3.9 13.9 12.4 13.9 21.8s-5.4 17.9-13.9 21.8l-218.6 101c-14.9 6.9-32.1 6.9-47 0L45.9 405.8C37.4 401.8 32 393.3 32 384s5.4-17.9 13.9-21.8l53.2-24.6 152 70.2c23.4 10.8 50.4 10.8 73.8 0z"/>
                </svg>
              `;
            });
          
          wrapper.appendChild(icon);
          wrapper.appendChild(select);
          this._container.appendChild(wrapper);
          
          return this._container;
        },
        
        onRemove: function() {
          this._container.parentNode?.removeChild(this._container);
          this._map = undefined;
        }
      };
      
      map.addControl(layerSwitcher as any, 'bottom-left');

      // Add load event listener to check if map loads
      map.on('load', () => {
        console.log('Map loaded successfully');
      });

      map.on('error', (e) => {
        console.error('Map error:', e);
      });

      map.on('style.load', () => {
        console.log('Map style loaded');
      });

      // Check Strava authentication status
      if (browser) {
        // Import Strava service dynamically to avoid SSR issues
        const { stravaService: service } = await import('../lib/strava');
        stravaService = service;
        
        isStravaAuthenticated = stravaService.isAuthenticated();
        if (isStravaAuthenticated) {
          stravaAthlete = await stravaService.fetchAthleteInfo();
        }
        
        // Import RideWithGPS service dynamically to avoid SSR issues
        const { ridewithgpsService: rwgpsService } = await import('../lib/ridewithgps');
        ridewithgpsService = rwgpsService;
        
        isRWGPSAuthenticated = ridewithgpsService.isAuthenticated();
        if (isRWGPSAuthenticated) {
          rwgpsUser = await ridewithgpsService.fetchUserInfo();
        }
      }

    } catch (err) {
      error = `Failed to initialize: ${err}`;
      console.error('Initialization error:', err);
    }
  });

  onDestroy(() => {
    if (browser) {
      window.removeEventListener('resize', handleResize);
    }
  });

  // Panel functions
  function togglePanel() {
    isPanelCollapsed = !isPanelCollapsed;
  }

  function handleResize() {
    if (browser) {
      isMobile = window.innerWidth < 768;
      // Auto-collapse panel on mobile
      if (isMobile && !isPanelCollapsed) {
        isPanelCollapsed = true;
      }
    }
  }

  // Safe localStorage operations
  function safeLocalStorageSetItem(key: string, value: string): boolean {
    if (!browser || typeof localStorage === 'undefined') {
      return false;
    }
    
    try {
      // Clear potentially large items before setting new data
      const keysToCheck = ['strava_activities', 'rwgps_trips'];
      if (keysToCheck.includes(key)) {
        // Clear old data first to free up space
        keysToCheck.forEach(k => {
          if (k !== key) {
            localStorage.removeItem(k);
            localStorage.removeItem(k + '_timestamp');
          }
        });
      }
      
      localStorage.setItem(key, value);
      return true;
    } catch (error) {
      if (error instanceof DOMException && error.name === 'QuotaExceededError') {
        console.warn('localStorage quota exceeded, clearing cache and retrying...');
        // Clear all cached data and try again
        try {
          localStorage.removeItem('strava_activities');
          localStorage.removeItem('strava_activities_timestamp');
          localStorage.removeItem('rwgps_trips');
          localStorage.removeItem('rwgps_trips_timestamp');
          localStorage.setItem(key, value);
          return true;
        } catch (retryError) {
          console.warn('localStorage still failed after clearing cache:', retryError);
          return false;
        }
      } else {
        console.warn('localStorage error:', error);
        return false;
      }
    }
  }

  // Clear all cached data
  function clearLocalStorageCache() {
    if (!browser || typeof localStorage === 'undefined') {
      return;
    }
    
    try {
      localStorage.removeItem('strava_activities');
      localStorage.removeItem('strava_activities_timestamp');
      localStorage.removeItem('rwgps_trips');
      localStorage.removeItem('rwgps_trips_timestamp');
      console.log('Cleared all cached activity/trip data');
    } catch (error) {
      console.warn('Error clearing localStorage cache:', error);
    }
  }
</script>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  }

  /* PaneForge customizations */
  :global([data-pane-group]) {
    display: flex !important;
    height: 100% !important;
  }

  :global([data-pane]) {
    overflow: hidden !important;
  }

  :global([data-pane-resizer]) {
    background: #e5e7eb !important;
    transition: background-color 0.2s !important;
    width: 4px !important;
    cursor: ew-resize !important;
  }

  :global([data-pane-resizer]:hover) {
    background: #d1d5db !important;
  }

  :global([data-pane-resizer][data-resizing]) {
    background: #3b82f6 !important;
  }

  /* Hide resizer on mobile */
  @media (max-width: 767px) {
    :global([data-pane-resizer]) {
      display: none !important;
    }
  }

  /* Layer switcher control styles */
  :global(.layer-switcher) {
    position: relative;
    min-width: 120px;
  }

  :global(.layer-switcher-wrapper) {
    display: flex;
    align-items: center;
    background: white;
    border-radius: 2px;
    padding: 2px;
    gap: 4px;
  }

  :global(.layer-switcher-icon) {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    color: #333;
    pointer-events: none;
  }

  :global(.layer-switcher-icon svg) {
    width: 20px;
    height: 20px;
    display: block;
  }

  :global(.layer-switcher-select) {
    background: transparent;
    border: none;
    font-size: 12px;
    color: #333;
    cursor: pointer;
    padding: 4px 8px 4px 2px;
    outline: none;
    appearance: none;
    -webkit-appearance: none;
    -moz-appearance: none;
    font-family: inherit;
    font-weight: 500;
    min-width: 80px;
  }

  :global(.layer-switcher-select:focus) {
    background-color: rgba(0, 0, 0, 0.05);
    border-radius: 2px;
  }

  :global(.layer-switcher:hover) {
    background-color: rgba(0, 0, 0, 0.02);
  }

  /* Custom Ruler control styles */
  :global(.maplibregl-ctrl-icon) {
    width: 29px;
    height: 29px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: white;
    border: none;
    cursor: pointer;
    border-radius: 2px;
    transition: background-color 0.2s;
    color: #333;
  }

  :global(.maplibregl-ctrl-icon:hover) {
    background-color: rgba(0, 0, 0, 0.05);
  }

  :global(.maplibregl-ctrl-icon.active) {
    background-color: #3b82f6 !important;
    color: white !important;
  }

  :global(.maplibregl-ctrl-icon svg) {
    width: 20px;
    height: 20px;
    pointer-events: none;
  }

  /* Ruler Control Styles */
  :global(.ruler-control) {
    margin: 0;
  }

  :global(.ruler-control-button) {
    background: #fff;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    height: 29px;
    width: 29px;
    padding: 0;
    transition: background-color 0.2s ease;
  }

  :global(.ruler-control-button:hover) {
    background: #f0f0f0;
  }

  :global(.ruler-control-button.active) {
    background: #00bcd4; /* Cyan active state */
    color: white;
  }

  :global(.ruler-control-button svg) {
    width: 18px;
    height: 18px;
    display: block;
    margin: auto;
  }

  /* Ruler distance labels with cyan theme */
  :global(.ruler-distance-popup .maplibregl-popup-content) {
    background: rgba(0, 188, 212, 0.9); /* Cyan background */
    color: white;
    border-radius: 4px;
    padding: 4px 8px;
    font-size: 12px;
    font-weight: bold;
    pointer-events: none;
  }

  :global(.ruler-total-popup .maplibregl-popup-content) {
    background: rgba(0, 150, 171, 0.95); /* Darker cyan for total */
    color: white;
    border-radius: 4px;
    padding: 6px 10px;
    font-size: 14px;
    font-weight: bold;
    pointer-events: none;
  }

  :global(.ruler-distance-popup .maplibregl-popup-tip),
  :global(.ruler-total-popup .maplibregl-popup-tip) {
    display: none;
  }

  :global(.ruler-distance-label),
  :global(.ruler-total-label) {
    margin: 0;
    white-space: nowrap;
  }
</style>

<div class="h-screen overflow-hidden">
 
  <!-- Mobile panel toggle button -->
  <button 
    class="absolute top-4 left-4 z-[1000] bg-white border border-gray-300 rounded-lg p-3 shadow-lg cursor-pointer transition-all duration-200 hover:bg-gray-50 block md:hidden"
    on:click={togglePanel} 
    aria-label="Toggle control panel"
  >
    {#if isPanelCollapsed}
      <!-- Expand icon -->
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="none" viewBox="0 0 24 24">
        <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 18l6-6-6-6"/>
      </svg>
    {:else}
      <!-- Collapse icon -->
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="none" viewBox="0 0 24 24">
        <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 18l-6-6 6-6"/>
      </svg>
    {/if}
  </button>

  <div class="h-full">
    <PaneGroup direction="horizontal">
    <!-- Controls Panel -->
    {#if !isPanelCollapsed}
      <Pane defaultSize={isMobile ? 100 : 25} minSize={15} maxSize={isMobile ? 100 : 50}>
        <div class="h-full overflow-y-auto bg-white p-4 border-r border-gray-200">
          <div class="flex items-center justify-between mb-4">
            <button 
              class="bg-gray-100 border border-gray-300 rounded-md p-2 cursor-pointer transition-all duration-200 hover:bg-gray-200 block md:hidden"
              on:click={togglePanel} 
              aria-label="Collapse panel"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="none" viewBox="0 0 24 24">
                <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 18l-6-6 6-6"/>
              </svg>
            </button>
          </div>
          <h2>This is a demo for fastgeotoolkit, a high-performance GPS route processing library. For more information:
<a 
              href="https://github.com/a0a7/fastgeotoolkit" 
              target="_blank" 
              rel="noopener noreferrer"
              class="inline-flex items-center gap-2 bg-gray-900 text-white border-none rounded-md py-2 px-4 text-sm font-medium cursor-pointer transition-colors duration-200 hover:bg-gray-700"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 16 16">
                <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.012 8.012 0 0 0 16 8c0-4.42-3.58-8-8-8z"/>
              </svg>
              a0a7/fastgeotoolkit
            </a>
        </h2>
            <div class="mx-1 w-full h-px bg-gray-500 my-4"></div>

          <!-- Strava Integration Section -->
          {#if browser && stravaService}
            <div class="mb-4">
            {#if !isStravaAuthenticated}
              <button 
                class="w-full bg-orange-600 text-white border-none rounded-md py-3 px-4 text-sm font-semibold cursor-pointer transition-colors duration-200 flex items-center justify-center gap-2 hover:bg-orange-700 disabled:bg-gray-400 disabled:cursor-not-allowed"
                on:click={connectToStrava}
                disabled={isLoading || isImportingStrava}
              >
              <svg class="h-5" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 17 193 18" version="1.1">
                <desc>Created with Sketch.</desc>
                <defs/>
                <g id="Strava-Button_outlined" stroke="none" stroke-width="1" fill="none" fill-rule="evenodd">
                    <g id="btn_strava_connectwith_orange">
                        <path d="M27,25.164 L28.736,25.514 C28.6239994,26.6153388 28.2226701,27.5066633 27.532,28.188 C26.8413299,28.8693367 25.9500055,29.21 24.858,29.21 C23.6166605,29.21 22.6016706,28.7760043 21.813,27.908 C21.0243294,27.0399957 20.63,25.7426753 20.63,24.016 C20.63,22.4106586 21.0429959,21.171671 21.869,20.299 C22.6950041,19.426329 23.6866609,18.99 24.844,18.99 C25.8613384,18.99 26.7199965,19.3096635 27.42,19.949 C28.1200035,20.5883365 28.5306661,21.4306614 28.652,22.476 L26.944,22.742 C26.7013321,21.2579926 26.0060057,20.516 24.858,20.516 C24.1393297,20.516 23.5396691,20.8053304 23.059,21.384 C22.5783309,21.9626696 22.338,22.8493274 22.338,24.044 C22.338,25.2666728 22.5736643,26.178997 23.045,26.781 C23.5163357,27.383003 24.120663,27.684 24.858,27.684 C26.0806728,27.684 26.7946656,26.8440084 27,25.164 L27,25.164 Z M33.51875,27.768 C34.0694194,27.768 34.5150816,27.5510022 34.85575,27.117 C35.1964184,26.6829978 35.36675,26.0740039 35.36675,25.29 C35.36675,24.5059961 35.1964184,23.8970022 34.85575,23.463 C34.5150816,23.0289978 34.0694194,22.812 33.51875,22.812 C32.9587472,22.812 32.5084184,23.0266645 32.16775,23.456 C31.8270816,23.8853355 31.65675,24.4966627 31.65675,25.29 C31.65675,26.0926707 31.8270816,26.7063312 32.16775,27.131 C32.5084184,27.5556688 32.9587472,27.768 33.51875,27.768 L33.51875,27.768 Z M33.51875,29.21 C32.5200783,29.21 31.6964199,28.8646701 31.04775,28.174 C30.3990801,27.4833299 30.07475,26.5220062 30.07475,25.29 C30.07475,24.0766606 30.4084133,23.1200035 31.07575,22.42 C31.7430867,21.7199965 32.5574119,21.37 33.51875,21.37 C34.4800881,21.37 35.2897467,21.7199965 35.94775,22.42 C36.6057533,23.1200035 36.93475,24.0766606 36.93475,25.29 C36.93475,26.5220062 36.6150865,27.4833299 35.97575,28.174 C35.3364135,28.8646701 34.5174217,29.21 33.51875,29.21 L33.51875,29.21 Z M38.7635,29 L38.7635,21.58 L40.3035,21.58 L40.3035,22.294 L40.3315,22.294 C40.5368344,22.0326654 40.8214982,21.8133342 41.1855,21.636 C41.5495018,21.4586658 41.9321647,21.37 42.3335,21.37 C43.1548374,21.37 43.8011643,21.6149976 44.2725,22.105 C44.7438357,22.5950024 44.9795,23.2739957 44.9795,24.142 L44.9795,29 L43.3975,29 L43.3975,24.562 C43.3975,23.4139943 42.9168381,22.84 41.9555,22.84 C41.4701642,22.84 41.0828348,22.9799986 40.7935,23.26 C40.5041652,23.5400014 40.3595,23.9179976 40.3595,24.394 L40.3595,29 L38.7635,29 Z M47.22825,29 L47.22825,21.58 L48.76825,21.58 L48.76825,22.294 L48.79625,22.294 C49.0015844,22.0326654 49.2862482,21.8133342 49.65025,21.636 C50.0142518,21.4586658 50.3969147,21.37 50.79825,21.37 C51.6195874,21.37 52.2659143,21.6149976 52.73725,22.105 C53.2085857,22.5950024 53.44425,23.2739957 53.44425,24.142 L53.44425,29 L51.86225,29 L51.86225,24.562 C51.86225,23.4139943 51.3815881,22.84 50.42025,22.84 C49.9349142,22.84 49.5475848,22.9799986 49.25825,23.26 C48.9689152,23.5400014 48.82425,23.9179976 48.82425,24.394 L48.82425,29 L47.22825,29 Z M60.621,26.536 L61.769,27.474 C61.0503297,28.6313391 60.0236733,29.21 58.689,29.21 C57.6623282,29.21 56.8246699,28.8530036 56.176,28.139 C55.5273301,27.4249964 55.203,26.4753393 55.203,25.29 C55.203,24.1046607 55.5249968,23.1550036 56.169,22.441 C56.8130032,21.7269964 57.6296617,21.37 58.619,21.37 C59.6083383,21.37 60.4179968,21.7246631 61.048,22.434 C61.6780031,23.1433369 61.993,24.0953274 61.993,25.29 L61.993,25.766 L56.813,25.766 C56.8316668,26.3820031 57.0019984,26.8743315 57.324,27.243 C57.6460016,27.6116685 58.0916638,27.796 58.661,27.796 C58.931668,27.796 59.1743323,27.758667 59.389,27.684 C59.6036677,27.609333 59.7926658,27.4973341 59.956,27.348 C60.1193342,27.1986659 60.2429996,27.0680006 60.327,26.956 C60.4110004,26.8439994 60.5089994,26.7040008 60.621,26.536 L60.621,26.536 Z M56.827,24.562 L60.439,24.562 C60.4109999,24.0393307 60.2430015,23.612335 59.935,23.281 C59.6269985,22.949665 59.1883362,22.784 58.619,22.784 C58.0869973,22.784 57.6623349,22.9613316 57.345,23.316 C57.0276651,23.6706684 56.8550001,24.0859976 56.827,24.562 L56.827,24.562 Z M68.32975,26.046 L69.91175,26.382 C69.7624159,27.2780045 69.4217527,27.9733309 68.88975,28.468 C68.3577473,28.9626691 67.6530877,29.21 66.77575,29.21 C65.767745,29.21 64.9417532,28.8646701 64.29775,28.174 C63.6537468,27.4833299 63.33175,26.5220062 63.33175,25.29 C63.33175,24.1046607 63.6560801,23.1550036 64.30475,22.441 C64.9534199,21.7269964 65.7724117,21.37 66.76175,21.37 C67.6110876,21.37 68.3110806,21.6243308 68.86175,22.133 C69.4124194,22.6416692 69.7344162,23.3019959 69.82775,24.114 L68.32975,24.366 C68.1710825,23.3299948 67.6530877,22.812 66.77575,22.812 C66.2064138,22.812 65.7537517,23.0289978 65.41775,23.463 C65.0817483,23.8970022 64.91375,24.5059961 64.91375,25.29 C64.91375,26.0740039 65.0770817,26.6829978 65.40375,27.117 C65.7304183,27.5510022 66.1877471,27.768 66.77575,27.768 C67.6437543,27.768 68.1617492,27.1940057 68.32975,26.046 L68.32975,26.046 Z M72.0205,26.522 L72.0205,22.952 L70.9005,22.952 L70.9005,21.58 L72.0625,21.58 L72.0625,19.76 L73.5745,19.76 L73.5745,21.58 L75.4365,21.58 L75.4365,22.952 L73.5885,22.952 L73.5885,26.354 C73.5885,26.7646687 73.6514994,27.0516658 73.7775,27.215 C73.9035006,27.3783342 74.162498,27.46 74.5545,27.46 L75.1425,27.46 L75.1425,29 L74.4285,29 C73.5324955,29 72.9071684,28.8016686 72.5525,28.405 C72.1978316,28.0083313 72.0205,27.380671 72.0205,26.522 L72.0205,26.522 Z M81.992,29 L80.354,21.58 L81.922,21.58 L82.972,26.746 L83,26.746 L84.764,21.58 L86.206,21.58 L87.858,26.732 L87.886,26.732 L89.076,21.58 L90.616,21.58 L88.838,29 L87.298,29 L85.492,23.428 L85.464,23.428 L83.518,29 L81.992,29 Z M92.40275,29 L92.40275,21.58 L93.99875,21.58 L93.99875,29 L92.40275,29 Z M92.37475,20.362 L92.37475,18.78 L94.02675,18.78 L94.02675,20.362 L92.37475,20.362 Z M96.6955,26.522 L96.6955,22.952 L95.5755,22.952 L95.5755,21.58 L96.7375,21.58 L96.7375,19.76 L98.2495,19.76 L98.2495,21.58 L100.1115,21.58 L100.1115,22.952 L98.2635,22.952 L98.2635,26.354 C98.2635,26.7646687 98.3264994,27.0516658 98.4525,27.215 C98.5785006,27.3783342 98.837498,27.46 99.2295,27.46 L99.8175,27.46 L99.8175,29 L99.1035,29 C98.2074955,29 97.5821684,28.8016686 97.2275,28.405 C96.8728316,28.0083313 96.6955,27.380671 96.6955,26.522 L96.6955,26.522 Z M101.87025,29 L101.87025,18.78 L103.46625,18.78 L103.46625,22.308 L103.49425,22.308 C103.662251,22.0559987 103.937581,21.8366676 104.32025,21.65 C104.702919,21.4633324 105.090248,21.37 105.48225,21.37 C106.275587,21.37 106.919581,21.6126642 107.41425,22.098 C107.908919,22.5833358 108.15625,23.2459958 108.15625,24.086 L108.15625,29 L106.57425,29 L106.57425,24.464 C106.57425,23.9786642 106.443585,23.5866682 106.18225,23.288 C105.920915,22.9893318 105.542919,22.84 105.04825,22.84 C104.572248,22.84 104.189585,22.9799986 103.90025,23.26 C103.610915,23.5400014 103.46625,23.8993311 103.46625,24.338 L103.46625,29 L101.87025,29 Z" id="Connect-with" fill="#FFFFFF"/>
                        <path d="M160.015559,18.7243818 L157.573637,23.6936411 L155.130184,18.7243818 L151.538762,18.7243818 L157.573637,31 L163.604197,18.7243818 L160.015559,18.7243818 Z M140.167341,23.0633572 C140.167341,22.6899778 140.038474,22.4112701 139.782411,22.2312505 C139.527323,22.049653 139.178854,21.959428 138.742573,21.959428 L137.108085,21.959428 L137.108085,24.220073 L138.726013,24.220073 C139.17454,24.220073 139.527323,24.1208112 139.782411,23.9228613 C140.038474,23.7244811 140.167341,23.4484988 140.167341,23.0966357 L140.167341,23.0633572 Z M149.175468,18 L155.208534,30.2756182 L151.617112,30.2756182 L149.175468,25.306072 L146.735216,30.2756182 L144.297747,30.2756182 L143.145603,30.2756182 L140.022749,30.2756182 L137.908281,26.9753059 L137.877804,26.9753059 L137.108085,26.9753059 L137.108085,30.2756182 L133.360798,30.2756182 L133.360798,18.7243818 L138.838458,18.7243818 C139.841696,18.7243818 140.666246,18.8428649 141.311553,19.0794006 C141.958668,19.3155061 142.477752,19.638107 142.87298,20.0451952 C143.215325,20.3961977 143.471249,20.7933884 143.642978,21.231747 C143.812619,21.6721138 143.898902,22.1909433 143.898902,22.7824979 L143.898902,22.8174977 C143.898902,23.6638052 143.701288,24.3792936 143.305086,24.9618113 C142.911389,25.5449028 142.372405,26.0083638 141.687715,26.3481777 L143.635324,29.2788401 L149.175468,18 Z M165.966934,18 L159.934565,30.2756182 L163.525987,30.2756182 L165.966934,25.306072 L168.409552,30.2756182 L172,30.2756182 L165.966934,18 Z M122.487587,21.9899811 L125.786345,21.9899811 L125.786345,30.2756182 L129.534327,30.2756182 L129.534327,21.9899811 L132.833921,21.9899811 L132.833921,18.7243818 L122.487587,18.7243818 L122.487587,21.9899811 Z M122.352597,25.0606428 C122.581523,25.4677311 122.696612,25.9636099 122.696612,26.5455539 L122.696612,26.5794062 C122.696612,27.1838705 122.579853,27.7295237 122.343829,28.212923 C122.109615,28.6976133 121.777847,29.1069967 121.351168,29.4430811 C120.923515,29.7780181 120.405405,30.036357 119.797395,30.2182414 C119.189663,30.400843 118.505112,30.4919286 117.748474,30.4919286 C116.606767,30.4919286 115.540905,30.3282614 114.553254,30.0046563 C113.565741,29.6791866 112.715028,29.1923446 112,28.5447042 L114.001188,26.0865396 C114.609059,26.5697955 115.250886,26.9167816 115.922216,27.1254896 C116.594521,27.334628 117.262511,27.4391972 117.924378,27.4391972 C118.266584,27.4391972 118.511235,27.3947302 118.660976,27.3078043 C118.811692,27.2194441 118.884892,27.0980922 118.884892,26.9447528 L118.884892,26.9119046 C118.884892,26.7453685 118.774256,26.6062298 118.556603,26.4980746 C118.338949,26.388198 117.928692,26.273301 117.331258,26.1525229 C116.701677,26.0195522 116.100764,25.8647784 115.530329,25.6893489 C114.959058,25.51478 114.457508,25.2881418 114.025541,25.0125898 C113.592878,24.7383288 113.248863,24.3904821 112.991966,23.9727791 C112.735903,23.554359 112.607871,23.0422712 112.607871,22.4378069 L112.607871,22.4045284 C112.607871,21.8538547 112.711549,21.341767 112.920435,20.8692692 C113.126816,20.3961977 113.431726,19.982798 113.832242,19.6317956 C114.232897,19.2795021 114.730132,19.0049543 115.323112,18.80557 C115.913588,18.6076201 116.594521,18.508932 117.364379,18.508932 C118.45209,18.508932 119.404811,18.6413289 120.221569,18.9062662 C121.038465,19.1683346 121.771724,19.5649516 122.423988,20.092818 L120.598705,22.7013097 C120.064869,22.3039755 119.501531,22.0153704 118.909246,21.8333425 C118.315431,21.6514581 117.759746,21.5603725 117.235513,21.5603725 C116.958575,21.5603725 116.753168,21.6044092 116.619849,21.6927694 C116.484024,21.7808428 116.419591,21.8968873 116.419591,22.0391817 L116.419591,22.0720299 C116.419591,22.2273776 116.51965,22.3593441 116.723526,22.469651 C116.927263,22.5790972 117.321656,22.6949983 117.907817,22.8160633 C118.623541,22.9471692 119.274136,23.1073939 119.862802,23.2932946 C120.448825,23.4822077 120.954827,23.7204648 121.375383,24.012656 C121.797052,24.3042735 122.123532,24.6521202 122.352597,25.0606428 L122.352597,25.0606428 Z" id="Strava-logo-Copy-6" fill="#FFFFFF"/>
                    </g>
                </g>
            </svg>
              </button>
            {:else}
              <div class="bg-gray-50 border border-gray-200 rounded-md p-4">
                <div class="flex items-center gap-3 mb-4">
                  {#if stravaAthlete}
                    <img src={stravaAthlete.profile} alt="Profile" class="w-10 h-10 rounded-full object-cover">
                    <div>
                      <strong>{stravaAthlete.firstname} {stravaAthlete.lastname}</strong>
                      <div class="text-xs text-green-600">Connected to Strava</div>
                    </div>
                  {:else}
                    <div>
                      <strong>Connected to Strava</strong>
                      <div class="text-xs text-green-600">Ready to import activities</div>
                    </div>
                  {/if}
                </div>
                
                <div class="flex gap-2 mb-4">
                  <button 
                    class="flex-1 bg-green-600 text-white border-none rounded py-2 px-4 text-sm cursor-pointer flex items-center justify-center gap-2 transition-colors duration-200 hover:bg-green-700 disabled:bg-gray-500 disabled:cursor-not-allowed"
                    on:click={importStravaActivities}
                    disabled={isLoading || isImportingStrava}
                  >
                    {#if isImportingStrava}
                      <div class="w-3.5 h-3.5 border-2 border-transparent border-t-current rounded-full animate-spin"></div>
                      Importing Activities...
                    {:else}
                      Import All Activities
                    {/if}
                  </button>
                  
                  <button 
                    class="bg-gray-500 text-white border-none rounded py-2 px-4 text-sm cursor-pointer transition-colors duration-200 hover:bg-gray-600 disabled:bg-gray-300 disabled:cursor-not-allowed"
                    on:click={disconnectFromStrava}
                    disabled={isLoading || isImportingStrava}
                  >
                    Disconnect
                  </button>
                </div>
                
                {#if isImportingStrava}
                  <div class="mt-2">
                    <div class="w-full h-1.5 bg-gray-200 rounded-full overflow-hidden mb-2">
                      <div class="h-full bg-orange-600 transition-all duration-300 ease-out" style="width: {stravaImportProgress}%"></div>
                    </div>
                    <div class="text-xs text-gray-600 text-center">
                      {#if stravaActivityCount > 0}
                        Fetched {stravaActivityCount} activities... {stravaImportProgress}%
                      {:else}
                        Connecting to Strava... {stravaImportProgress}%
                      {/if}
                    </div>
                  </div>
                {/if}
              </div>
            {/if}
          </div>
          {/if}

          <!-- RideWithGPS Integration Section -->
          {#if browser && ridewithgpsService}
            <div class="mb-4">
            {#if !isRWGPSAuthenticated}
              <button 
                class="w-full bg-orange-700 text-white border-none rounded-md py-3 px-4 text-sm font-semibold cursor-pointer transition-colors duration-200 flex items-center justify-center gap-2 hover:bg-orange-800 disabled:bg-gray-400 disabled:cursor-not-allowed"
                on:click={connectToRWGPS}
                disabled={isLoading || isImportingRWGPS}
              >
                <div class="w-8 h-4 transform -translate-y-2 py-0">
                  <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" version="1.1" viewBox="0 0 512 512"><g fill="none" fill-rule="evenodd" stroke="none" stroke-width="1"><g fill="currentColor" fill-rule="nonzero"><g id="cyclist-logo-small" transform="translate(29.000000, 96.000000)"><path d="M265.535248,197.777784 C264.94517,198.368229 264.94517,198.368229 264.355091,198.958675 C251.373368,214.900703 223.639687,245.603868 219.509138,250.327432 C173.483029,305.829308 146.929504,316.457327 117.425587,318.819109 C112.114883,319.409555 106.804178,319.409555 100.313316,319.409555 C49.5665796,319.409555 0,276.897479 0,222.576494 C0,172.388628 35.4046997,131.057443 83.2010444,125.743434 C86.7415144,125.152988 94.4125326,124.562543 103.263708,125.743434 C111.524804,126.924325 126.866841,133.419225 126.866841,141.685462 C126.276762,141.685462 125.686684,141.685462 125.096606,141.095017 C112.704961,138.142789 106.214099,138.733235 99.7232376,139.32368 C83.2010444,141.095017 71.9895561,146.409026 63.1383812,152.903926 C45.4360313,167.665064 32.4543081,188.921101 32.4543081,214.900703 C32.4543081,256.231887 65.4986945,289.296835 105.033943,289.296835 C125.686684,289.296835 144.569191,281.030598 158.140992,266.859906 C160.501305,264.498124 162.861619,262.726788 164.631854,260.955451 C183.51436,240.880304 223.049608,197.187338 223.049608,187.74021 C223.049608,177.112192 199.446475,152.903926 175.253264,138.733235 C140.438642,118.067643 139.258486,102.125614 139.258486,89.7262591 C139.258486,50.7568568 199.446475,30.0912646 257.274151,35.9957195 C271.436031,87.9549226 312.151436,90.9071501 319.232376,91.4975955 C340.475196,93.268932 351.096606,91.4975955 363.488251,88.5453681 C376.469974,85.5931407 376.469974,82.0504677 376.469974,82.0504677 C376.469974,82.0504677 380.600522,86.7740316 370.569191,93.268932 C360.537859,99.7638324 333.394256,109.801406 315.101828,109.801406 C276.156658,109.801406 254.913838,93.268932 234.851175,71.4224489 C190.5953,74.9651218 186.464752,87.9549226 188.825065,92.088041 C195.315927,101.535169 263.765013,148.770808 270.255875,147.589917 C277.926893,146.409026 308.610966,120.429425 355.817232,124.562543 C408.924282,125.743434 452,168.845955 452,221.986049 C452,276.307034 409.51436,320 353.456919,320 C305.070496,320 274.386423,282.211489 283.237598,268.631242 C283.827676,269.221688 284.417755,269.812133 285.007833,269.812133 C298.579634,282.211489 321.592689,295.791735 344.605744,295.791735 C388.27154,295.791735 423.67624,263.317233 423.67624,223.16694 C423.67624,183.016646 394.172324,155.856154 351.096606,151.723035 C332.214099,149.951699 321.592689,153.494372 309.791123,159.398827 C298.579634,164.712836 283.827676,178.293083 272.616188,189.511547 C272.616188,189.511547 272.616188,189.511547 272.616188,188.921101 C272.02611,189.511547 271.436031,190.101992 270.845953,190.692438 C268.48564,193.644665 266.715405,196.006447 265.535248,197.777784 Z" id="Combined-Shape"/><path d="M315.244484,67 C329.519433,66.4040248 345.578751,53.8885463 344.983962,34.8173409 C344.389172,15.7461356 325.355907,4.42260736 308.10701,1.44273152 C273.609216,-3.92104499 238.516633,6.21053286 229,22.8978376 C229,22.8978376 232.568737,21.7058872 237.921843,19.9179617 C251.007213,16.3421107 264.687373,18.1300362 275.393585,24.6857631 C280.151901,27.6656389 281.34148,33.6253906 281.34148,34.8173409 C280.746691,57.4643973 300.969535,67 315.244484,67 Z" id="Oval-3"/></g></g></g></svg>
                </div>
                Connect to RideWithGPS
              </button>
            {:else}
              <div class="bg-gray-50 border border-gray-200 rounded-md p-4">
                <div class="flex items-center gap-3 mb-4">
                  {#if rwgpsUser}
                    <img src={rwgpsUser.profile} alt="Profile" class="w-10 h-10 rounded-full object-cover">
                    <div>
                      <strong>{rwgpsUser.firstname} {rwgpsUser.lastname}</strong>
                      <div class="text-xs text-green-600">Connected to RideWithGPS</div>
                    </div>
                  {:else}
                    <div>
                      <strong>Connected to RideWithGPS</strong>
                      <div class="text-xs text-green-600">Ready to import trips</div>
                    </div>
                  {/if}
                </div>
                
                <div class="flex gap-2 mb-4">
                  <button 
                    class="flex-1 bg-green-600 text-white border-none rounded py-2 px-4 text-sm cursor-pointer flex items-center justify-center gap-2 transition-colors duration-200 hover:bg-green-700 disabled:bg-gray-500 disabled:cursor-not-allowed"
                    on:click={importRWGPSTrips}
                    disabled={isLoading || isImportingRWGPS}
                  >
                    {#if isImportingRWGPS}
                      <div class="w-3.5 h-3.5 border-2 border-transparent border-t-current rounded-full animate-spin"></div>
                      Importing Trips...
                    {:else}
                      Import All Trips
                    {/if}
                  </button>
                  
                  <button 
                    class="bg-gray-500 text-white border-none rounded py-2 px-4 text-sm cursor-pointer transition-colors duration-200 hover:bg-gray-600 disabled:bg-gray-300 disabled:cursor-not-allowed"
                    on:click={disconnectFromRWGPS}
                    disabled={isLoading || isImportingRWGPS}
                  >
                    Disconnect
                  </button>
                </div>
                
                {#if isImportingRWGPS}
                  <div class="mt-2">
                    <div class="w-full h-1.5 bg-gray-200 rounded-full overflow-hidden mb-2">
                      <div class="h-full bg-orange-700 transition-all duration-300 ease-out" style="width: {rwgpsImportProgress}%"></div>
                    </div>
                    <div class="text-xs text-gray-600 text-center">
                      {#if rwgpsTripCount > 0}
                        Fetched {rwgpsTripCount} trips... {rwgpsImportProgress}%
                      {:else}
                        Connecting to RideWithGPS... {rwgpsImportProgress}%
                      {/if}
                    </div>
                  </div>
                {/if}
              </div>
            {/if}
          </div>
          {/if}



          <div class="mb-4">
            <label for="gpx-files" class="block mb-2 font-medium text-gray-700">Upload GPX/FIT Files:</label>
            <input 
              id="gpx-files"
              type="file" 
              multiple 
              accept=".gpx,.fit" 
              class="w-full p-2 border-2 border-dashed border-gray-300 rounded cursor-pointer transition-colors duration-200 hover:border-slate-600 file:mr-4 file:py-2 file:px-4 file:rounded file:border-0 file:text-sm file:font-semibold file:bg-slate-50 file:text-slate-700 hover:file:bg-slate-100"
              on:change={handleFiles}
              disabled={isLoading}
            >
            
            <div class="mt-2 text-sm">
              {#if isLoading}
                <div class="text-slate-600">Processing GPX/FIT files...</div>
              {:else if error}
                <div class="text-red-600">{error}</div>
              {:else if segmentCount > 0}
                <div class="text-green-600">
                  Loaded {segmentCount} route segments
                </div>
              {/if}
            </div>
          </div>
                    <!-- Sample Data Section -->
                     <h3>OR</h3>
          <div class="mb-6">
            <div class="flex items-center justify-between mb-3">
            </div>
            <button 
              class="w-full bg-slate-800 text-white py-3 px-4 rounded-lg font-medium transition-colors duration-200 hover:bg-slate-900 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
              on:click={loadSampleData}
              disabled={isLoading}
            >
              {#if isLoading}
                <div class="w-4 h-4 border-2 border-transparent border-t-white rounded-full animate-spin"></div>
                Loading Sample Data...
              {:else}
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 16 16">
                  <path d="M4 1.5H3a2 2 0 0 0-2 2V14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V3.5a2 2 0 0 0-2-2h-1v1h1a1 1 0 0 1 1 1V14a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V3.5a1 1 0 0 1 1-1h1v-1z"/>
                  <path d="M9.5 1a.5.5 0 0 1 .5.5v1a.5.5 0 0 1-.5.5h-3a.5.5 0 0 1-.5-.5v-1a.5.5 0 0 1 .5-.5h3zm-3-1A1.5 1.5 0 0 0 5 1.5v1A1.5 1.5 0 0 0 6.5 4h3A1.5 1.5 0 0 0 11 2.5v-1A1.5 1.5 0 0 0 9.5 0h-3z"/>
                </svg>
                Load Sample Data
              {/if}
            </button>
            <p class="text-xs text-gray-500 mt-2">
              Loads a small set of 20 sample files.
            </p>
          </div>

          {#if segmentCount > 0}
            <button 
              class="bg-red-600 text-white border-none py-2 px-4 rounded cursor-pointer text-sm transition-colors duration-200 hover:bg-red-700 disabled:bg-gray-400 disabled:cursor-not-allowed"
              on:click={clearHeatmap}
              disabled={isLoading}
            >
              Clear Heatmap
            </button>
          {/if}

        </div>
      </Pane>

      <PaneResizer />
    {/if}

    <!-- Map Panel -->
    <Pane defaultSize={isPanelCollapsed ? 100 : (isMobile ? 0 : 75)}>
      <div class="relative h-full w-full">
        <div bind:this={mapContainer} id="map" class="w-full h-full"></div>
      </div>
    </Pane>
  </PaneGroup>
  </div> <!-- Close padding div -->
</div>
