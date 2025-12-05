$ErrorActionPreference = 'Stop'

function Test-Command {
    param([string]$Name)
    return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}

Write-Host 'Checking for rustup...' -ForegroundColor Cyan
if (-not (Test-Command 'rustup')) {
    Write-Host 'ERROR: rustup is not installed or not in PATH.' -ForegroundColor Red
    Write-Host 'Install Rust from https://rustup.rs/ and try again.' -ForegroundColor Red
    exit 1
}

Write-Host 'Checking for cargo...' -ForegroundColor Cyan
if (-not (Test-Command 'cargo')) {
    Write-Host 'ERROR: cargo is not installed or not in PATH.' -ForegroundColor Red
    Write-Host 'Install Rust from https://rustup.rs/ and try again.' -ForegroundColor Red
    exit 1
}

Write-Host 'Adding wasm32 target...' -ForegroundColor Cyan
try {
    rustup target add wasm32-unknown-unknown
}
catch {
    Write-Host 'ERROR: Failed to add wasm32-unknown-unknown target.' -ForegroundColor Red
    exit 1
}

Write-Host 'Checking for wasm-pack...' -ForegroundColor Cyan
if (-not (Test-Command 'wasm-pack')) {
    Write-Host 'wasm-pack not found. Installing...' -ForegroundColor Yellow
    try {
        cargo install wasm-pack
    }
    catch {
        Write-Host 'ERROR: Failed to install wasm-pack.' -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host 'wasm-pack already installed.' -ForegroundColor Green
}

Write-Host 'Building WASM package...' -ForegroundColor Cyan
try {
    wasm-pack build --target web --out-dir pkg --release
}
catch {
    Write-Host 'ERROR: wasm-pack build failed.' -ForegroundColor Red
    exit 1
}

Write-Host ''
Write-Host 'Build complete.' -ForegroundColor Green
Write-Host 'Output directory: ./pkg' -ForegroundColor Green
Write-Host ''

Write-Host 'To publish manually:' -ForegroundColor Magenta
Write-Host '  npm login' -ForegroundColor Magenta
Write-Host '  cd pkg' -ForegroundColor Magenta
Write-Host '  npm publish --access public --no-git-checks' -ForegroundColor Magenta
Write-Host ''
