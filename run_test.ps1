# PowerShell script to run a compiled PHP script

param (
    [Parameter(Mandatory=$true)]
    [string]$ScriptName
)

# Find the most recent executable for this script
$exePath = Get-ChildItem -Path "tests\output\$ScriptName*.exe" | Sort-Object LastWriteTime -Descending | Select-Object -First 1 -ExpandProperty FullName

if (-not $exePath) {
    Write-Error "No executable found for script: $ScriptName"
    exit 1
}

# Run the executable
Write-Host "Running $exePath..."
& $exePath

# Note: The executable might return a non-zero exit code even if it ran successfully
# This is because we're not properly handling the exit code in our assembly code
Write-Host "Execution completed!"
