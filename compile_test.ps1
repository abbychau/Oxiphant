# PowerShell script to compile a PHP script to an executable

param (
    [Parameter(Mandatory=$true)]
    [string]$ScriptName
)

# Check if the script exists
$scriptPath = "tests\scripts\$ScriptName.php"
if (-not (Test-Path $scriptPath)) {
    Write-Error "Script not found: $scriptPath"
    exit 1
}

# Create output directory if it doesn't exist
if (-not (Test-Path "tests\output")) {
    New-Item -ItemType Directory -Path "tests\output" -Force | Out-Null
}

# Create output paths with a timestamp to avoid conflicts
$timestamp = Get-Date -Format "yyyyMMddHHmmss"
$outputExe = "tests\output\$ScriptName-$timestamp.exe"
$outputAsm = "tests\output\$ScriptName-$timestamp.s"

# Compile the script
Write-Host "Compiling $scriptPath to $outputExe..."
cargo run --bin tinyphp-rs $scriptPath $outputExe

# Check if compilation was successful
if ($LASTEXITCODE -ne 0) {
    Write-Error "Compilation failed with exit code $LASTEXITCODE"
    exit $LASTEXITCODE
}

# Copy the assembly file to the output folder
$asmPath = "tests\scripts\$ScriptName.php.s"
if (Test-Path $asmPath) {
    Copy-Item $asmPath $outputAsm
    Remove-Item $asmPath
    Write-Host "Assembly file saved to $outputAsm"
}

Write-Host "Compilation successful!"
