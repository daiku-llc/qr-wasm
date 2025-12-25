# API Security Configuration

This document explains how API access is restricted to only allow requests from your app.

## Security Features

### 1. Origin Validation

The API endpoints (`/api/generate` and `/api/qr`) validate that requests come from the same origin as your app:

- **Same-Origin Requests**: Automatically allowed (browsers don't send Origin header for same-origin requests)
- **Cross-Origin Requests**: Must have an `Origin` or `Referer` header that matches your app's domain
- **Direct API Calls**: Blocked by default (no Origin/Referer headers)

### 2. Optional API Key Authentication

For programmatic access, you can set an API key:

1. **Set Environment Variable** in Cloudflare Dashboard:
   - Go to Workers & Pages → Your Worker → Settings → Variables
   - Add a secret variable: `ALLOWED_API_KEY` with your desired API key value

2. **Use API Key in Requests**:
   ```bash
   curl -X POST https://your-worker.workers.dev/api/generate \
     -H "Content-Type: application/json" \
     -H "X-API-Key: your-api-key-here" \
     -d '{"data":"test"}'
   ```

## How It Works

### Request Flow

1. **Request arrives** at API endpoint
2. **Check for API Key** (if `ALLOWED_API_KEY` is set):
   - If `X-API-Key` header matches → Allow request
3. **Check Origin/Referer headers**:
   - If Origin matches request host → Allow request
   - If Referer contains request host → Allow request
   - If no Origin/Referer → Block request (direct API call)
4. **If validation fails** → Return 401 Unauthorized error

### Example Scenarios

#### ✅ Allowed: Request from your web app
```
Origin: https://your-worker.workers.dev
→ Allowed (same origin)
```

#### ✅ Allowed: Request with valid API key
```
X-API-Key: your-secret-key
→ Allowed (valid API key)
```

#### ❌ Blocked: Direct API call (curl, Postman, etc.)
```
No Origin header
No Referer header
→ Blocked (direct API call)
```

#### ❌ Blocked: Request from different domain
```
Origin: https://evil-site.com
→ Blocked (origin mismatch)
```

## Configuration

### Environment Variables

Set these in Cloudflare Dashboard → Workers & Pages → Your Worker → Settings → Variables:

- `ALLOWED_API_KEY` (Secret, optional): API key for programmatic access

### Testing

**Test from your app** (should work):
```javascript
// In your app.js or browser console
fetch('/api/generate', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ data: 'test' })
})
```

**Test direct API call** (should be blocked):
```bash
curl -X POST https://your-worker.workers.dev/api/generate \
  -H "Content-Type: application/json" \
  -d '{"data":"test"}'
# Expected: 401 Unauthorized error
```

**Test with API key** (should work if configured):
```bash
curl -X POST https://your-worker.workers.dev/api/generate \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{"data":"test"}'
```

## Security Notes

- ⚠️ **Origin/Referer headers can be spoofed** by malicious clients, but this provides basic protection against casual abuse
- ✅ **API key authentication** is more secure for programmatic access
- ✅ **Same-origin policy** protects requests from your web app automatically
- 🔒 **Direct API calls are blocked** to prevent unauthorized usage

## Bypassing for Development

If you need to test API endpoints directly during development, you can:

1. Temporarily comment out the `validate_origin` calls in `src/lib.rs`
2. Or set an `ALLOWED_API_KEY` and use it in your requests
3. Or use `wrangler dev` which runs locally and may have different security behavior

## Recommendations

1. **For Production**: Always use API key authentication for any programmatic access
2. **For Web App**: Same-origin validation is sufficient (browser enforces it)
3. **Monitor**: Check Cloudflare Workers logs for unauthorized access attempts
4. **Rate Limiting**: Consider adding rate limiting via Cloudflare's built-in features

