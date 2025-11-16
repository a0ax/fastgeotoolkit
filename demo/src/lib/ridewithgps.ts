// RideWithGPS API integration
import { browser } from '$app/environment';

const RWGPS_CLIENT_ID = import.meta.env.VITE_RWGPS_CLIENT_ID;

// Helper function to get redirect URI (only works in browser)
function getRedirectUri(): string {
  if (browser && typeof window !== 'undefined') {
    return `${window.location.origin}/rwgps-callback`;
  }
  return 'http://localhost:5173/rwgps-callback'; // fallback for SSR
}

export interface RWGPSTrip {
  id: number;
  name: string;
  distance: number;
  activity_type: string;
  created_at: string;
  track_points?: Array<{
    x: number; // longitude
    y: number; // latitude
    e?: number; // elevation
  }>;
  track_encoded?: string; // encoded polyline
}

export interface RWGPSUser {
  id: number;
  name: string;
  email: string;
  avatar?: string;
}

export class RideWithGPSService {
  private accessToken: string | null = null;
  private user: RWGPSUser | null = null;

  constructor() {
    // Check if we have a stored access token (only in browser)
    if (browser && typeof localStorage !== 'undefined') {
      this.accessToken = localStorage.getItem('rwgps_access_token');
    }
  }

  // Generate OAuth URL for RideWithGPS authorization
  getAuthUrl(): string {
    // RideWithGPS doesn't require a scope parameter, or uses different scope names
    // Let's try without scope first, then try common scopes if needed
    const params = new URLSearchParams({
      client_id: RWGPS_CLIENT_ID,
      redirect_uri: getRedirectUri(),
      response_type: 'code'
    });
    
    return `https://ridewithgps.com/oauth/authorize?${params.toString()}`;
  }

  // Exchange authorization code for access token
  async exchangeCodeForToken(code: string): Promise<boolean> {
    try {
      // Use our server-side API route to avoid CORS issues
      const response = await fetch('/api/rwgps-token', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          code: code,
          redirectUri: getRedirectUri()
        })
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(`Token exchange failed: ${errorData.error || response.statusText}`);
      }

      const data = await response.json();
      
      this.accessToken = data.access_token;
      
      // Store tokens in localStorage (only in browser)
      if (browser && typeof localStorage !== 'undefined') {
        localStorage.setItem('rwgps_access_token', data.access_token);
        if (data.refresh_token) {
          localStorage.setItem('rwgps_refresh_token', data.refresh_token);
        }
      }
      
      return true;
    } catch (error) {
      console.error('Error exchanging code for token:', error);
      return false;
    }
  }

  // Check if user is authenticated
  isAuthenticated(): boolean {
    return !!this.accessToken;
  }

  // Get current user info
  getUser(): RWGPSUser | null {
    return this.user;
  }

  // Fetch all trips from RideWithGPS
  async fetchAllTrips(onProgress?: (count: number) => void): Promise<RWGPSTrip[]> {
    if (!this.accessToken) {
      throw new Error('Not authenticated with RideWithGPS');
    }

    const allTrips: RWGPSTrip[] = [];
    let offset = 0;
    const limit = 100; // RideWithGPS API limit
    
    try {
      while (true) {
        const response = await fetch(
          `/api/rwgps-trips?offset=${offset}&limit=${limit}&token=${encodeURIComponent(this.accessToken)}`,
          {
            headers: {
              'Accept': 'application/json'
            }
          }
        );

        if (!response.ok) {
          if (response.status === 401) {
            // Token expired, clear stored tokens
            this.logout();
            throw new Error('RideWithGPS authentication expired. Please reconnect.');
          }
          throw new Error(`Failed to fetch trips: ${response.statusText}`);
        }

        const data = await response.json();
        const trips: RWGPSTrip[] = data.results || [];
        
        // Handle case where API returns null/undefined instead of empty array
        if (!trips || !Array.isArray(trips) || trips.length === 0) {
          break; // No more trips
        }

        allTrips.push(...trips);
        
        // Call progress callback if provided
        if (onProgress) {
          onProgress(allTrips.length);
        }

        offset += limit;
        
        // Add a small delay to be respectful to the API
        await new Promise(resolve => setTimeout(resolve, 100));
        
        // Break if we got less than the limit (indicates last page)
        if (trips.length < limit) {
          break;
        }
      }

      console.log(`Fetched ${allTrips.length} trips from RideWithGPS`);
      return allTrips;
      
    } catch (error) {
      console.error('Error fetching RideWithGPS trips:', error);
      throw error;
    }
  }

  // Get detailed trip with full track data
  async getTripDetail(tripId: number): Promise<RWGPSTrip | null> {
    if (!this.accessToken) {
      throw new Error('Not authenticated with RideWithGPS');
    }

    try {
      const response = await fetch(
        `https://ridewithgps.com/trips/${tripId}.json`,
        {
          headers: {
            'Authorization': `Bearer ${this.accessToken}`,
            'Accept': 'application/json'
          }
        }
      );

      if (!response.ok) {
        if (response.status === 401) {
          this.logout();
          throw new Error('RideWithGPS authentication expired. Please reconnect.');
        }
        throw new Error(`Failed to fetch trip detail: ${response.statusText}`);
      }

      const data = await response.json();
      return data.trip;
    } catch (error) {
      console.error(`Error fetching trip ${tripId}:`, error);
      return null;
    }
  }

  // Convert track points to polyline format
  private encodePolyline(points: Array<{x: number, y: number}>): string {
    // Simple polyline encoding - this is a basic implementation
    // For production, you might want to use a proper polyline encoding library
    const coordinates = points.map(point => [point.y, point.x]); // [lat, lng]
    
    // This is a simplified encoding - you might want to use the actual Google polyline algorithm
    // For now, we'll return a JSON string that our backend can parse
    return JSON.stringify(coordinates);
  }

  // Extract polylines from trips
  async getPolylinesFromTrips(trips: RWGPSTrip[], onProgress?: (processed: number, total: number) => void): Promise<string[]> {
    const polylines: string[] = [];
    
    // Safety check for null/undefined trips array
    if (!trips || !Array.isArray(trips)) {
      console.warn('getPolylinesFromTrips: trips is not a valid array');
      return polylines;
    }
    
    console.log('Sample trip data structure:', trips.length > 0 ? trips[0] : 'No trips');
    
    for (let i = 0; i < trips.length; i++) {
      const trip = trips[i];
      console.log('Processing trip:', {
        id: trip.id,
        name: trip.name,
        hasTrackId: !!(trip as any).track_id,
        isGps: (trip as any).is_gps,
        allKeys: Object.keys(trip)
      });
      
      let polyline: string | null = null;
      
      if (trip.track_encoded) {
        // Use encoded polyline if available
        polyline = trip.track_encoded;
        console.log('Using track_encoded for trip', trip.id);
      } else if (trip.track_points && trip.track_points.length > 0) {
        // Convert track points to simple coordinate string for our backend
        const coordinates = trip.track_points.map(point => [point.y, point.x]); // [lat, lng]
        polyline = JSON.stringify(coordinates);
        console.log('Using track_points for trip', trip.id, 'with', trip.track_points.length, 'points');
      } else if ((trip as any).track_id && (trip as any).is_gps) {
        // Fetch detailed track data using track_id
        console.log('Fetching track data for trip', trip.id, 'with track_id', (trip as any).track_id);
        try {
          const trackData = await this.getTrackData((trip as any).track_id);
          if (trackData) {
            polyline = trackData;
            console.log('Successfully fetched track data for trip', trip.id);
          }
        } catch (error) {
          console.error('Failed to fetch track data for trip', trip.id, error);
        }
        // Add a small delay to be respectful to the API
        await new Promise(resolve => setTimeout(resolve, 200));
      } else {
        console.log('No GPS data found for trip', trip.id, 'is_gps:', (trip as any).is_gps, 'track_id:', (trip as any).track_id);
      }
      
      if (polyline && polyline.length > 0) {
        polylines.push(polyline);
      }

      // Call progress callback if provided
      if (onProgress) {
        onProgress(i + 1, trips.length);
      }
    }
    
    console.log(`Extracted ${polylines.length} polylines from ${trips.length} trips`);
    return polylines;
  }

  // Fetch track data by track ID
  async getTrackData(trackId: number): Promise<string | null> {
    if (!this.accessToken) {
      throw new Error('Not authenticated with RideWithGPS');
    }

    try {
      const response = await fetch(
        `/api/rwgps-track?id=${trackId}&token=${encodeURIComponent(this.accessToken)}`,
        {
          headers: {
            'Accept': 'application/json'
          }
        }
      );

      if (!response.ok) {
        console.error('Failed to fetch track data:', response.status, response.statusText);
        return null;
      }

      const data = await response.json();
      
      if (data.track_points && data.track_points.length > 0) {
        const coordinates = data.track_points.map((point: any) => [point.y, point.x]);
        return JSON.stringify(coordinates);
      } else if (data.track_encoded) {
        return data.track_encoded;
      }

      return null;
    } catch (error) {
      console.error('Error fetching track data:', error);
      return null;
    }
  }

  // Logout and clear stored tokens
  logout(): void {
    this.accessToken = null;
    this.user = null;
    if (browser && typeof localStorage !== 'undefined') {
      localStorage.removeItem('rwgps_access_token');
      localStorage.removeItem('rwgps_refresh_token');
    }
  }

  // Get user info from API
  async fetchUserInfo(): Promise<RWGPSUser | null> {
    if (!this.accessToken) {
      return null;
    }

    try {
      const response = await fetch(`/api/rwgps-user?token=${encodeURIComponent(this.accessToken)}`, {
        headers: {
          'Accept': 'application/json'
        }
      });

      if (!response.ok) {
        if (response.status === 401) {
          this.logout();
          return null;
        }
        throw new Error(`Failed to fetch user info: ${response.statusText}`);
      }

      const data = await response.json();
      this.user = data.user;
      return this.user;
    } catch (error) {
      console.error('Error fetching user info:', error);
      return null;
    }
  }
}

export const ridewithgpsService = new RideWithGPSService();
