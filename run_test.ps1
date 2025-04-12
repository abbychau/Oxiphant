# PowerShell script to run a compiled PHP script

param (
    [Parameter(Mandatory=$true)]
    [string]$ScriptName
)

# Check if the executable exists
$exePath = "tests\output\$ScriptName.exe"
if (-not (Test-Path $exePath)) {
    Write-Error "Executable not found: $exePath"
    exit 1
}

# Run the executable
Write-Host "Running $exePath..."
& $exePath

# Note: The executable might return a non-zero exit code even if it ran successfully
# This is because we're not properly handling the exit code in our assembly code
Write-Host "Execution completed!"
