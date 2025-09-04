import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { env } from '$env/dynamic/private';

export const POST: RequestHandler = async ({ request }) => {
  const RWGPS_CLIENT_ID = env.RWGPS_CLIENT_ID;
  const RWGPS_CLIENT_SECRET = env.RWGPS_CLIENT_SECRET;

  // Check if required environment variables are set
  if (!RWGPS_CLIENT_ID || !RWGPS_CLIENT_SECRET) {
    return json({ 
      error: 'RideWithGPS OAuth credentials not configured' 
    }, { status: 500 });
  }

  try {
    const { code, redirectUri } = await request.json();

    if (!code || !redirectUri) {
      return json({ error: 'Missing code or redirectUri' }, { status: 400 });
    }

    // Exchange the authorization code for an access token
    const tokenResponse = await fetch('https://ridewithgps.com/oauth/token', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        client_id: RWGPS_CLIENT_ID,
        client_secret: RWGPS_CLIENT_SECRET,
        code: code,
        redirect_uri: redirectUri,
      }),
    });

    if (!tokenResponse.ok) {
      const errorText = await tokenResponse.text();
      console.error('RideWithGPS token exchange failed:', errorText);
      return json({ error: 'Token exchange failed' }, { status: tokenResponse.status });
    }

    const tokenData = await tokenResponse.json();
    return json(tokenData);
  } catch (error) {
    console.error('Error in RideWithGPS token exchange:', error);
    return json({ error: 'Internal server error' }, { status: 500 });
  }
};
