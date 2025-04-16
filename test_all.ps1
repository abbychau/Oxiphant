# PowerShell script to compile and run all test scripts

# Get all PHP scripts in the tests\scripts folder
$scripts = Get-ChildItem -Path "tests\scripts" -Filter "*.php" | ForEach-Object { $_.BaseName }

# Create output directory if it doesn't exist
if (-not (Test-Path "tests\output")) {
    New-Item -ItemType Directory -Path "tests\output" | Out-Null
}

# empty output folder
Remove-Item -Path "tests\output\*" -Force

# Compile and run each script
foreach ($script in $scripts) {
    Write-Host "===== Testing $script =====" -ForegroundColor Green
    
    # Compile the script
    Write-Host "Compiling $script..." -ForegroundColor Cyan
    .\compile_test.ps1 $script
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Compilation failed for $script" -ForegroundColor Red
        continue
    }
    
    # Run the script
    Write-Host "Running $script..." -ForegroundColor Cyan
    .\run_test.ps1 $script
    
    Write-Host "===== Test completed for $script =====" -ForegroundColor Green
    Write-Host ""
}

Write-Host "All tests completed!" -ForegroundColor Green
