// Strava API integration
import { browser } from '$app/environment';

const STRAVA_CLIENT_ID = import.meta.env.VITE_STRAVA_CLIENT_ID;

// Helper function to get redirect URI (only works in browser)
function getRedirectUri(): string {
  if (browser && typeof window !== 'undefined') {
    return `${window.location.origin}/strava-callback`;
  }
  return 'http://localhost:5173/strava-callback'; // fallback for SSR
}

export interface StravaActivity {
  id: number;
  name: string;
  distance: number;
  type: string;
  start_date: string;
  map?: {
    polyline?: string;
    summary_polyline?: string;
  };
}

export interface StravaAthlete {
  id: number;
  firstname: string;
  lastname: string;
  profile: string;
}

export class StravaService {
  private accessToken: string | null = null;
  private athlete: StravaAthlete | null = null;

  constructor() {
    // Check if we have a stored access token (only in browser)
    if (browser && typeof localStorage !== 'undefined') {
      this.accessToken = localStorage.getItem('strava_access_token');
    }
  }

  // Generate OAuth URL for Strava authorization
  getAuthUrl(): string {
    const scope = 'read,activity:read_all';
    const params = new URLSearchParams({
      client_id: STRAVA_CLIENT_ID,
      redirect_uri: getRedirectUri(),
      response_type: 'code',
      scope: scope,
      approval_prompt: 'auto'
    });
    
    return `https://www.strava.com/oauth/authorize?${params.toString()}`;
  }

  // Exchange authorization code for access token
  async exchangeCodeForToken(code: string): Promise<boolean> {
    try {
      // Use our server-side API route to avoid CORS issues
      const response = await fetch('/api/strava-token', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          code: code
        })
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(`Token exchange failed: ${errorData.error || response.statusText}`);
      }

      const data = await response.json();
      
      this.accessToken = data.access_token;
      this.athlete = data.athlete;
      
      // Store tokens in localStorage (only in browser)
      if (browser && typeof localStorage !== 'undefined') {
        localStorage.setItem('strava_access_token', data.access_token);
        if (data.refresh_token) {
          localStorage.setItem('strava_refresh_token', data.refresh_token);
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

  // Get current athlete info
  getAthlete(): StravaAthlete | null {
    return this.athlete;
  }

  // Fetch all activities from Strava
  async fetchAllActivities(onProgress?: (count: number) => void): Promise<StravaActivity[]> {
    if (!this.accessToken) {
      throw new Error('Not authenticated with Strava');
    }

    const allActivities: StravaActivity[] = [];
    let page = 1;
    const perPage = 200; // Maximum allowed by Strava API
    
    try {
      while (true) {
        const response = await fetch(
          `/api/strava-activities?page=${page}&per_page=${perPage}`,
          {
            headers: {
              'Authorization': `Bearer ${this.accessToken}`
            }
          }
        );

        if (!response.ok) {
          if (response.status === 401) {
            // Token expired, clear stored tokens
            this.logout();
            throw new Error('Strava authentication expired. Please reconnect.');
          }
          throw new Error(`Failed to fetch activities: ${response.statusText}`);
        }

        const activities: StravaActivity[] = await response.json();
        
        // Handle case where API returns null/undefined instead of empty array
        if (!activities || !Array.isArray(activities) || activities.length === 0) {
          break; // No more activities
        }

        allActivities.push(...activities);
        
        // Call progress callback if provided
        if (onProgress) {
          onProgress(allActivities.length);
        }

        page++;
        
        // Add a small delay to be respectful to the API
        await new Promise(resolve => setTimeout(resolve, 100));
      }

      console.log(`Fetched ${allActivities.length} activities from Strava`);
      return allActivities;
      
    } catch (error) {
      console.error('Error fetching Strava activities:', error);
      throw error;
    }
  }

  // Get detailed activity with full polyline
  async getActivityDetail(activityId: number): Promise<StravaActivity | null> {
    if (!this.accessToken) {
      throw new Error('Not authenticated with Strava');
    }

    try {
      const response = await fetch(
        `https://www.strava.com/api/v3/activities/${activityId}`,
        {
          headers: {
            'Authorization': `Bearer ${this.accessToken}`
          }
        }
      );

      if (!response.ok) {
        if (response.status === 401) {
          this.logout();
          throw new Error('Strava authentication expired. Please reconnect.');
        }
        throw new Error(`Failed to fetch activity detail: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error(`Error fetching activity ${activityId}:`, error);
      return null;
    }
  }

  // Extract polylines from activities
  getPolylinesFromActivities(activities: StravaActivity[]): string[] {
    const polylines: string[] = [];
    
    // Safety check for null/undefined activities array
    if (!activities || !Array.isArray(activities)) {
      console.warn('getPolylinesFromActivities: activities is not a valid array');
      return polylines;
    }
    
    for (const activity of activities) {
      // Prefer detailed polyline over summary polyline
      const polyline = activity.map?.polyline || activity.map?.summary_polyline;
      if (polyline && polyline.length > 0) {
        polylines.push(polyline);
      }
    }
    
    console.log(`Extracted ${polylines.length} polylines from ${activities.length} activities`);
    return polylines;
  }

  // Logout and clear stored tokens
  logout(): void {
    this.accessToken = null;
    this.athlete = null;
    if (browser && typeof localStorage !== 'undefined') {
      localStorage.removeItem('strava_access_token');
      localStorage.removeItem('strava_refresh_token');
    }
  }

  // Get athlete info from API
  async fetchAthleteInfo(): Promise<StravaAthlete | null> {
    if (!this.accessToken) {
      return null;
    }

    try {
      const response = await fetch('https://www.strava.com/api/v3/athlete', {
        headers: {
          'Authorization': `Bearer ${this.accessToken}`
        }
      });

      if (!response.ok) {
        if (response.status === 401) {
          this.logout();
          return null;
        }
        throw new Error(`Failed to fetch athlete info: ${response.statusText}`);
      }

      this.athlete = await response.json();
      return this.athlete;
    } catch (error) {
      console.error('Error fetching athlete info:', error);
      return null;
    }
  }
}

export const stravaService = new StravaService();
