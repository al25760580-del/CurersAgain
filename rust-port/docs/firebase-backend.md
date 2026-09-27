# Official scores backend

Verified against the public client configuration and the reader implementation.

## Product

**Cloud Firestore over REST**:

```
https://firestore.googleapis.com/v1/projects/<ProjectID>/databases/(default)/documents/<doc>
```

Project id: `leaderboardtest-110f4`. Like every client-side config it is public and
is not a secret.

## Minimal authentication

1. `https://identitytoolkit.googleapis.com/v1/accounts:signUp?key=<WebAPIKey>` -
   anonymous sign-up.
2. `https://securetoken.googleapis.com/v1/token?key=<WebAPIKey>` - token refresh.

No Steam credentials and no save files are used.

## Relation to GMEXT-Firebase

`GMEXT-Firebase` (YoYoGames' official extension) ships `Firebase Firestore` with a
"REST API Library" for platforms without a native SDK. That is the same route this
independent client uses, so no game token is required to read the ranking.

## Enforced limits

- Read-only: no publishing, remote rename or deletion.
- URL allow-list: any other request raises `REQUEST_BLOCKED`.
- Interface actions apply only to the port's own profile and log
  `remote_write=skipped`.
- Current coverage: Stage 1 and five characters (`ame, gura, ina, kiara, calli`).

## Never committed

- The session file holding the refresh token.
- Any token, cookie or credential.
- Original save files.
