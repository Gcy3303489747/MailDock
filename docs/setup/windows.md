# Windows Development Setup

This guide explains the tools needed to develop MailDock on Windows.

## Required Tools

- Git: tracks source code changes and connects the local repository to GitHub.
- Node.js and npm: run the React/Vite frontend and install JavaScript packages.
- Rust stable MSVC toolchain: builds the Tauri backend and native Windows app.
- WebView2 Runtime: provides the embedded browser used by Tauri windows.
- Visual Studio C++ Build Tools: provides the Windows C++ linker and libraries used by Rust native builds.

## Option 1: Install Manually

Install these tools from their official installers:

- Git for Windows
- Node.js LTS
- Rustup with the stable MSVC toolchain
- Microsoft Edge WebView2 Runtime
- Visual Studio 2022 Build Tools with the C++ workload

After installation, open a new PowerShell and check:

```powershell
git --version
node -v
npm -v
rustc -V
cargo -V
```

## Option 2: Use the Helper Script

MailDock includes a Windows helper script for development machines:

```powershell
powershell -ExecutionPolicy Bypass -File docs\setup\install-dev-tools.ps1
```

To skip Visual Studio Build Tools installation:

```powershell
powershell -ExecutionPolicy Bypass -File docs\setup\install-dev-tools.ps1 -SkipVisualStudioBuildTools
```

Restart Codex or open a new PowerShell after the script finishes so PATH changes are available.

## Run MailDock

Install project dependencies:

```powershell
npm install
```

Start the desktop app in development mode:

```powershell
npm run tauri dev
```

For browser-only UI development:

```powershell
npm run dev
```
