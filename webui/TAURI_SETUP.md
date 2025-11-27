# Tauri Frontend Setup Instructions

## 1. Install Tauri Dependencies

```bash
cd /Users/tomwesselmann/Desktop/LsKG-Agent/webui
npm install @tauri-apps/api @tauri-apps/plugin-dialog @tauri-apps/plugin-fs
```

## 2. File Replacements

Replace the following files to enable Tauri integration:

### Replace imports in these files:

1. **`src/App.tsx`** → Replace with `src/App.tauri.tsx`
   ```bash
   cp src/App.tauri.tsx src/App.tsx
   ```

2. **`src/components/upload/BundleUploader.tsx`** → Replace with `.tauri.tsx` version
   ```bash
   cp src/components/upload/BundleUploader.tauri.tsx src/components/upload/BundleUploader.tsx
   ```

3. **`src/store/verificationStore.ts`** → Replace with `.tauri.ts` version
   ```bash
   cp src/store/verificationStore.tauri.ts src/store/verificationStore.ts
   ```

## 3. Update package.json Scripts

Add Tauri development and build scripts:

```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc -b && vite build",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build",
    "tauri:build:debug": "tauri build --debug"
  }
}
```

## 4. Verify CSP Configuration

Ensure `src-tauri/tauri.conf.json` has:

```json
{
  "app": {
    "security": {
      "csp": {
        "default-src": "'self'",
        "connect-src": "'none'",  // ← CRITICAL: Offline enforcement
        "script-src": "'self' 'unsafe-inline'",
        "style-src": "'self' 'unsafe-inline'",
        "img-src": "'self' data:",
        "font-src": "'self' data:"
      }
    }
  }
}
```

## 5. Test Offline Enforcement

Add this test to verify CSP blocks network calls:

```typescript
// src/App.tsx or src/lib/tauri.ts
async function testOfflineEnforcement() {
  try {
    await fetch('http://example.com');
    console.error('❌ CSP NOT ENFORCED! Network call succeeded');
  } catch (error) {
    console.log('✅ CSP enforced: Network call blocked');
  }
}
```

## 6. Development Workflow

### Start Dev Server

```bash
# Terminal 1: Frontend dev server
cd webui
npm run dev

# Terminal 2: Tauri app
cd ../src-tauri
cargo tauri dev
```

### Build for Production

```bash
cd webui
npm run build

cd ../src-tauri
cargo tauri build
```

Output will be in `src-tauri/target/release/bundle/`

## 7. File Structure After Setup

```
webui/
├── src/
│   ├── App.tsx                           # ← Tauri version (no API config)
│   ├── components/
│   │   ├── upload/
│   │   │   └── BundleUploader.tsx        # ← Tauri version (dialog API)
│   │   └── verification/
│   │       └── VerificationView.tsx      # ← Can be reused (adapt response format)
│   ├── lib/
│   │   └── tauri.ts                      # ← NEW: Tauri command wrappers
│   └── store/
│       └── verificationStore.ts          # ← Tauri version (simplified)
├── package.json                           # ← Updated with Tauri deps & scripts
└── node_modules/
    ├── @tauri-apps/api/
    ├── @tauri-apps/plugin-dialog/
    └── @tauri-apps/plugin-fs/
```

## 8. Troubleshooting

### CSP Violations

If you see CSP violations in the console:
1. Check `src-tauri/tauri.conf.json` → `app.security.csp`
2. Ensure `connect-src: 'none'` is set
3. Remove any `axios` or `fetch()` calls in the code

### Tauri Commands Not Found

If commands return `Command not found` errors:
1. Verify `src-tauri/src/lib.rs` has `#[tauri::command]` decorators
2. Check `invoke_handler(tauri::generate_handler![verify_bundle, get_bundle_info])`
3. Rebuild Rust backend: `cargo clean && cargo build`

### File Dialog Not Opening

If file dialog doesn't open:
1. Verify `tauri-plugin-dialog` is installed in `Cargo.toml`
2. Check `.plugin(tauri_plugin_dialog::init())` in `src/lib.rs`
3. Ensure permissions in `tauri.conf.json` → `plugins.dialog.open: true`

## 9. Next Steps

After setup:
1. Test bundle verification with a sample ZIP file
2. Verify CSP enforcement (no network calls)
3. Test error handling (invalid bundle, missing files)
4. Run integration tests
5. Build production bundle

## 10. Differences from WebUI Version

| Feature | WebUI (HTTP) | Desktop Proofer (Tauri) |
|---------|--------------|-------------------------|
| **API Communication** | axios with REST API | Tauri IPC commands |
| **File Upload** | FormData multipart | File Dialog API |
| **Configuration UI** | API URL + Bearer Token | None (local only) |
| **Network Access** | Full HTTP/HTTPS | Blocked by CSP |
| **Bundle Loading** | Uploaded to server | Read from local filesystem |
| **Verification** | Remote (server-side) | Local (embedded cap-agent) |
