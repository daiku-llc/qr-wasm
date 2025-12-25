# ⚡ QR Code Generator - WASM on Cloudflare Workers

A high-performance QR code generator powered by Rust/WebAssembly running on Cloudflare Workers Edge.

## Features

- 🚀 **Edge Computing**: Runs on Cloudflare's global edge network for ultra-low latency
- 🦀 **Rust/WASM**: Compiled from Rust to WebAssembly for maximum performance
- 🎨 **Modern UI**: Beautiful dark-themed interface with responsive design
- 📦 **PNG Format**: Generate QR codes as PNG images
- ⚡ **Fast**: Sub-100ms generation times on edge infrastructure
- 🔒 **Privacy-First**: All processing happens server-side with no data storage

## Tech Stack

- **Backend**: Rust compiled to WebAssembly
- **Runtime**: Cloudflare Workers
- **QR Library**: `qrcode` crate
- **Image Processing**: `image` crate
- **Deployment**: Wrangler CLI

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (v18+)
- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/): `npm install -g wrangler@latest`

### Installation

```bash
# Clone the repository
git clone https://github.com/daiku-llc/qr-wasm.git
cd qr-wasm

# Install dependencies (if using npm)
npm install

# Build the worker
cargo build --release
```

### Development

```bash
# Start local development server
wrangler dev

# Visit http://localhost:8787 in your browser
```

### Deployment

```bash
# Deploy to Cloudflare Workers
wrangler deploy
```

## API Endpoints

### POST `/api/generate`

Generate a QR code from JSON payload.

**Request:**
```json
{
  "data": "https://example.com"
}
```

**Response:**
```json
{
  "format": "png",
  "data_url": "data:image/png;base64,...",
  "size_bytes": 1234
}
```

### GET `/api/qr`

Generate a QR code from query parameters.

**Example:**
```
GET /api/qr?data=https://example.com
```

### GET `/api/health`

Health check endpoint.

**Response:**
```json
{
  "status": "healthy",
  "service": "QR Generator WASM",
  "timestamp": 1234567890
}
```

## Project Structure

```
qr-wasm/
├── src/
│   └── lib.rs          # Rust worker code
├── public/
│   ├── index.html      # Web UI
│   ├── styles.css      # Styling
│   └── app.js          # Frontend logic
├── Cargo.toml          # Rust dependencies
├── wrangler.toml       # Cloudflare Workers config
└── README.md
```

## Configuration

Edit `wrangler.toml` to customize:

- Worker name
- Compatibility date
- Static assets directory
- Custom domains (optional)

## Performance

- **Generation Time**: < 100ms on edge
- **PNG Size**: ~5-15 KB (depending on content)
- **Global Latency**: < 50ms (Cloudflare edge)

## License

ISC

## Contributing

Contributions welcome! Please open an issue or submit a pull request.

## Links

- [Cloudflare Workers Documentation](https://developers.cloudflare.com/workers/)
- [Workers RS Template](https://github.com/cloudflare/workers-rs)
- [QR Code Crate](https://crates.io/crates/qrcode)

