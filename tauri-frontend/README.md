# CAP Desktop Prover - Frontend

**Tauri 2.0 Frontend for CAP Desktop Prover (Offline-First)**

[![TypeScript](https://img.shields.io/badge/TypeScript-5.9-blue)](https://www.typescriptlang.org/)
[![React](https://img.shields.io/badge/React-19.2-blue)](https://react.dev/)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-orange)](https://tauri.app/)
[![Vite](https://img.shields.io/badge/Vite-7.2-purple)](https://vite.dev/)

---

## Overview

This is the **frontend component** of the CAP Desktop Prover application. It is designed to work exclusively with Tauri 2.0 as the backend runtime.

**Key Features:**

- **Prover Mode** - 6-step workflow for creating compliance proofs
- **Verifier Mode** - Verify existing CAP bundles locally
- **Offline-First** - No network requests, no API endpoints
- **Tauri IPC** - Secure communication with Rust backend

---

## Architecture

```
┌────────────────────────────────────────┐
│  React Components (Imperative Shell)   │  ← UI, State Management
├────────────────────────────────────────┤
│  Tauri IPC Layer                       │  ← invoke(), emit()
├────────────────────────────────────────┤
│  Rust Backend (src-tauri/)             │  ← cap-agent library
└────────────────────────────────────────┘
```

### Project Structure

```
tauri-frontend/
├── src/
│   ├── App.tsx                 # Main Application (Prover + Verifier modes)
│   ├── components/
│   │   ├── upload/             # Bundle Uploader (Tauri file dialog)
│   │   ├── verification/       # Verification View
│   │   ├── workflow/           # 6-Step Prover Workflow
│   │   ├── manifest/           # Manifest Viewer
│   │   └── audit/              # Audit Timeline
│   ├── core/
│   │   ├── models/             # Data Models (Manifest, Proof)
│   │   └── utils/              # Pure Validation Functions
│   ├── lib/
│   │   └── tauri.ts            # Tauri API Client
│   ├── store/
│   │   ├── workflowStore.ts    # Prover Workflow State
│   │   └── verificationStore.ts # Verifier State
│   └── __tests__/              # Unit Tests
└── package.json
```

---

## Development

### Prerequisites

- **Node.js** 24+ (LTS)
- **npm** 11+
- **Rust** (for Tauri backend)

### Install Dependencies

```bash
cd tauri-frontend
npm install
```

### Run with Tauri

```bash
cd src-tauri
cargo tauri dev
```

This starts the Vite dev server and opens the Tauri window.

### Build Frontend Only

```bash
npm run build
```

Built files will be in `dist/` (used by Tauri).

---

## Testing

```bash
# Run unit tests
npm test

# Watch mode
npm run test:watch

# Coverage report
npm run test:coverage
```

---

## Code Quality

```bash
# Linting
npm run lint

# Type Checking
npx tsc --noEmit
```

---

## Dependencies

### Runtime

- **React** 19.2 - UI framework
- **@tauri-apps/api** 2.9 - Tauri IPC
- **@tauri-apps/plugin-dialog** 2.4 - Native file dialogs
- **@tauri-apps/plugin-fs** 2.4 - File system access
- **zustand** 5.0 - State management
- **react-dropzone** 14.3 - File drag-and-drop
- **jszip** 3.10 - ZIP file handling

### Development

- **TypeScript** 5.9 - Type safety
- **Vite** 7.2 - Build tool
- **Vitest** 4.0 - Unit testing
- **TailwindCSS** 4.0 - Styling

---

## License

**All Rights Reserved**

Copyright 2025 Tom Wesselmann

---

**Project Status:** Production-Ready (v0.12.2)
**Last Updated:** December 2025
