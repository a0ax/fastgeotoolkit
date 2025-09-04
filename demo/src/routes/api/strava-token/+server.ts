import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { env } from '$env/dynamic/private';

export const POST: RequestHandler = async ({ request }) => {
  const STRAVA_CLIENT_ID = env.STRAVA_CLIENT_ID;
  const STRAVA_CLIENT_SECRET = env.STRAVA_CLIENT_SECRET;

  // Check if required environment variables are set
  if (!STRAVA_CLIENT_ID || !STRAVA_CLIENT_SECRET) {
    return json({ 
      error: 'Strava OAuth credentials not configured' 
    }, { status: 500 });
  }

  try {
    const { code } = await request.json();

    if (!code) {
      return json({ error: 'Missing authorization code' }, { status: 400 });
    }

    // Exchange the authorization code for an access token
    const tokenResponse = await fetch('https://www.strava.com/oauth/token', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        client_id: STRAVA_CLIENT_ID,
        client_secret: STRAVA_CLIENT_SECRET,
        code: code,
        grant_type: 'authorization_code',
      }),
    });

    if (!tokenResponse.ok) {
      const errorText = await tokenResponse.text();
      console.error('Strava token exchange failed:', errorText);
      return json({ error: 'Token exchange failed' }, { status: tokenResponse.status });
    }

    const tokenData = await tokenResponse.json();
    return json(tokenData);
  } catch (error) {
    console.error('Error in Strava token exchange:', error);
    return json({ error: 'Internal server error' }, { status: 500 });
  }
};
