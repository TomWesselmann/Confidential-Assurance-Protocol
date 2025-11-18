# CAP Verifier WebUI

**Production-Ready Verification Interface for Confidential Assurance Protocol**

[![TypeScript](https://img.shields.io/badge/TypeScript-5.9-blue)](https://www.typescriptlang.org/)
[![React](https://img.shields.io/badge/React-19.2-blue)](https://react.dev/)
[![Vite](https://img.shields.io/badge/Vite-7.2-purple)](https://vite.dev/)
[![TailwindCSS](https://img.shields.io/badge/TailwindCSS-4.0-blue)](https://tailwindcss.com/)

---

## 📖 Overview

The **CAP Verifier WebUI** is a web-based interface for verifying LkSG compliance proofs using the Confidential Assurance Protocol (CAP). It enables auditors to upload proof bundles, visualize manifests, and verify cryptographic proofs without revealing sensitive business data.

### Key Features

✅ **Drag-and-Drop Upload** - Upload proof bundles (.zip) or manifest.json files
✅ **Manifest Visualization** - View cryptographic commitments, policy details, and audit trails
✅ **Proof Verification** - Verify Zero-Knowledge Proofs via CAP backend API
✅ **Constraint Display** - Show detailed verification results for each constraint
✅ **CAP-Compliant Architecture** - Functional Core, Imperative Shell design pattern
✅ **Type-Safe** - Full TypeScript coverage with strict type checking
✅ **Tested** - Unit tests with Vitest and Testing Library
✅ **Production-Ready** - Docker deployment with nginx

---

## 🏗️ Architecture

This WebUI follows **CAP Engineering Guide** principles:

```
┌────────────────────────────────────────┐
│  React Components (Imperative Shell)   │  ← UI, I/O, State Management
├────────────────────────────────────────┤
│  Core Layer (Functional Core)         │  ← API Client, Models, Utils
│  - API Client (axios)                 │     (I/O-free, Deterministic)
│  - Data Models (Manifest, Proof)      │
│  - Validation Utils (Pure Functions)  │
└────────────────────────────────────────┘
```

### Project Structure

```
webui/
├── src/
│   ├── core/                # Functional Core (I/O-free)
│   │   ├── api/            # API Client + Types
│   │   ├── models/         # Data Models
│   │   └── utils/          # Pure Functions
│   ├── components/         # React Components (Imperative Shell)
│   │   ├── upload/         # Bundle Uploader
│   │   ├── verification/   # Verification View
│   │   └── manifest/       # Manifest Viewer
│   ├── hooks/              # React Hooks
│   ├── store/              # Zustand State Management
│   └── __tests__/          # Unit Tests
├── Dockerfile              # Production Docker Image
├── nginx.conf              # Nginx Configuration
└── vitest.config.ts        # Test Configuration
```

---

## 🚀 Quick Start

### Prerequisites

- **Node.js** 24+ (LTS)
- **npm** 11+
- **CAP Backend** running on `http://localhost:8080` (see [agent/README.md](../agent/README.md))

### 1. Install Dependencies

```bash
cd webui
npm install
```

### 2. Run Development Server

```bash
npm run dev
```

Open [http://localhost:5173](http://localhost:5173) in your browser.

### 3. Build for Production

```bash
npm run build
```

Built files will be in `dist/`.

### 4. Preview Production Build

```bash
npm run preview
```

---

## 🐳 Docker Deployment

### Build Docker Image

```bash
docker build -t cap-verifier-webui:latest .
```

### Run Docker Container

```bash
docker run -d -p 3000:80 --name cap-webui cap-verifier-webui:latest
```

Open [http://localhost:3000](http://localhost:3000).

### Docker Compose

```bash
docker compose up -d
```

---

## 🧪 Testing

### Run Unit Tests

```bash
npm test
```

### Run Tests in Watch Mode

```bash
npm run test:watch
```

### Generate Coverage Report

```bash
npm run test:coverage
```

---

## 🔧 Configuration

### API Backend URL

By default, the WebUI connects to `http://localhost:8080`. You can configure this in the UI under **API-Konfiguration**.

### Environment Variables (Optional)

Create a `.env` file for custom configuration:

```env
VITE_API_BASE_URL=http://localhost:8080
```

Access in code:

```typescript
const apiUrl = import.meta.env.VITE_API_BASE_URL;
```

---

## 📚 Usage

### 1. Upload Proof Bundle

- Drag-and-drop a `.zip` proof bundle or click to select a file
- For testing, you can also upload a `manifest.json` file directly

### 2. View Manifest

- Once uploaded, the manifest is automatically extracted and displayed
- Sections: Cryptographic Commitments, Policy, Proof, Audit Trail, Signatures

### 3. Verify Proof

- Click **"🔍 Proof Verifizieren"**
- The WebUI sends a request to the CAP backend API
- Verification results are displayed with constraint-level details

### 4. Reset

- Click **"🔄 Reset"** to clear the current session and upload a new bundle

---

## 🔐 Security

### Authentication

- The WebUI supports **OAuth2 Bearer Token** authentication
- Enter your JWT token in the **API-Konfiguration** section
- Tokens are stored in memory only (not persisted)

### HTTPS/TLS

- For production, deploy behind a reverse proxy (nginx, Traefik) with TLS
- Update `nginx.conf` to enforce HTTPS and add HSTS headers

### Content Security Policy

Recommended CSP headers (add to `nginx.conf`):

```nginx
add_header Content-Security-Policy "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';" always;
```

---

## 🛠️ Development

### Code Quality

```bash
# Linting
npm run lint

# Type Checking
npx tsc --noEmit
```

### Adding New Features

1. **Functional Core**: Add pure functions to `src/core/`
2. **Imperative Shell**: Add React components to `src/components/`
3. **Tests**: Add tests to `src/__tests__/`
4. **Ensure CAP-Compliance**: Follow CAP Engineering Guide principles

---

## 📦 Dependencies

### Runtime

- **React** 19.2 - UI framework
- **React DOM** 19.2 - DOM rendering
- **axios** 1.13 - HTTP client
- **zustand** 5.0 - State management
- **react-dropzone** 14.3 - File upload
- **@tanstack/react-query** 5.90 - Data fetching

### Development

- **TypeScript** 5.9 - Type safety
- **Vite** 7.2 - Build tool
- **Vitest** 4.0 - Unit testing
- **@testing-library/react** 16.3 - Component testing
- **TailwindCSS** 4.0 - Styling
- **ESLint** 9.39 - Linting

---

## 🗺️ Roadmap

### ✅ Completed (v0.1.0)

- Proof bundle upload with drag-and-drop
- Manifest visualization (commitments, policy, audit trail)
- Verification view with constraint-level results
- OAuth2 Bearer Token authentication
- Docker deployment
- Unit tests for core utilities

### 🔄 In Progress

- Integration tests for React components
- E2E tests with Playwright
- Multi-language support (DE/EN)

### 📅 Planned

- Batch verification (multiple bundles)
- Proof comparison view
- Export verification report (PDF)
- Dark mode toggle
- Advanced manifest filtering

---

## 🤝 Contributing

Contributions are welcome! Please follow the CAP Engineering Guide:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Follow **Functional Core, Imperative Shell** architecture
4. Write tests for new features
5. Ensure `npm test` and `npm run build` pass
6. Submit a Pull Request

---

## 📄 License

**All Rights Reserved**

Copyright © 2025 Tom Wesselmann

This project is proprietary software. Unauthorized copying, distribution, or modification is prohibited.

---

## 🙏 Acknowledgments

- Built with ❤️ using [React](https://react.dev/), [TypeScript](https://www.typescriptlang.org/), and [Vite](https://vite.dev/)
- Styled with [TailwindCSS](https://tailwindcss.com/)
- Development assisted by [Claude Code](https://claude.com/claude-code) (Anthropic)

---

## 📞 Support

- **Documentation**: [../docs/Projektübersicht/](../docs/Projektübersicht/)
- **Backend API**: [../agent/README.md](../agent/README.md)
- **Issues**: [GitHub Issues](https://github.com/TomWesselmann/Confidential-Assurance-Protocol/issues)

---

**Project Status:** ✅ Production-Ready (MVP v0.1.0)
**Current Version:** v0.1.0
**Last Updated:** November 18, 2025
