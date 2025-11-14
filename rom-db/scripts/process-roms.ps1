# PowerShell script to process multiple ROM files with rom-db in parallel
param(
    [Parameter(Mandatory=$true)]
    [string]$RomDirectory,

    [Parameter(Mandatory=$true)]
    [string]$BiosPath,

    [string]$ResumeFile = ".\resume.txt",

    [int]$MaxConcurrent = 0  # 0 = use CPU core count
)

# Determine max concurrent processes
if ($MaxConcurrent -eq 0) {
    $MaxConcurrent = (Get-CimInstance -ClassName Win32_Processor).NumberOfLogicalProcessors - 4
}

Write-Host "Max concurrent processes: $MaxConcurrent" -ForegroundColor Cyan

# Global flag for graceful shutdown
$script:shouldExit = $false

# Simple Ctrl+C handler
$null = Register-EngineEvent -SourceIdentifier PowerShell.Exiting -Action {
    Write-Host "`nCtrl+C detected. Waiting for running processes to complete..." -ForegroundColor Yellow
    $script:shouldExit = $true
} -ErrorAction SilentlyContinue

# Ensure resume file exists
if (-not (Test-Path $ResumeFile)) {
    New-Item -Path $ResumeFile -ItemType File -Force | Out-Null
    Write-Host "Created resume file: $ResumeFile" -ForegroundColor Green
}

# Get already processed files
$processedFiles = @()
if (Test-Path $ResumeFile) {
    $processedFiles = Get-Content $ResumeFile -ErrorAction SilentlyContinue | Where-Object { $_ -ne "" }
}

Write-Host "Already processed: $($processedFiles.Count) files" -ForegroundColor Yellow

# Get all ZIP ROM files
Write-Host "Scanning directory: $RomDirectory" -ForegroundColor Cyan

# Verify directory exists
if (-not (Test-Path $RomDirectory)) {
    Write-Host "Error: Directory not found: $RomDirectory" -ForegroundColor Red
    exit 1
}

# Get all files and filter for .zip
Write-Host "Finding all .zip files recursively..." -ForegroundColor Cyan
$allFiles = @()
try {
    $allFiles = Get-ChildItem -Path $RomDirectory -Filter "*.zip" -File -Recurse -ErrorAction Stop | Select-Object -ExpandProperty FullName
} catch {
    Write-Host "Error scanning directory: $_" -ForegroundColor Red
    Write-Host "Trying alternate method..." -ForegroundColor Yellow
    $allFiles = Get-ChildItem -Path $RomDirectory -File -Recurse -ErrorAction SilentlyContinue |
                Where-Object { $_.Name -like "*.zip" } |
                Select-Object -ExpandProperty FullName
}

Write-Host "Found $($allFiles.Count) total .zip files" -ForegroundColor Cyan

# Filter out already processed files
$filesToProcess = $allFiles | Where-Object { $processedFiles -notcontains $_ }

Write-Host "Total files found: $($allFiles.Count)" -ForegroundColor Cyan
Write-Host "Files to process: $($filesToProcess.Count)" -ForegroundColor Cyan

if ($filesToProcess.Count -eq 0) {
    Write-Host "No files to process. All files have been processed." -ForegroundColor Green
    exit 0
}

# Verify BIOS exists
if (-not (Test-Path $BiosPath)) {
    Write-Host "Error: BIOS file not found at $BiosPath" -ForegroundColor Red
    exit 1
}

# Verify rom-db executable (script is in rom-db/, so go up one level to project root)
$projectRoot = Split-Path $PSScriptRoot -Parent
$romDbPath = Join-Path $projectRoot "target\release\rom-db.exe"
if (-not (Test-Path $romDbPath)) {
    $romDbPath = Join-Path $projectRoot "target\debug\rom-db.exe"
    if (-not (Test-Path $romDbPath)) {
        Write-Host "Error: rom-db.exe not found in target/release or target/debug" -ForegroundColor Red
        Write-Host "Searched in: $projectRoot\target\" -ForegroundColor Red
        exit 1
    }
}

Write-Host "Using rom-db: $romDbPath" -ForegroundColor Cyan
Write-Host "`nStarting processing...`n" -ForegroundColor Green

# Job tracking
$jobs = @{}
$fileQueue = New-Object System.Collections.Queue
$filesToProcess | ForEach-Object { $fileQueue.Enqueue($_) }

$totalFiles = $filesToProcess.Count
$completedCount = 0
$startTime = Get-Date

function Start-RomProcess {
    param([string]$RomPath)

    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Starting: $RomPath" -ForegroundColor Cyan

    # Start the process (using -WindowStyle Hidden to suppress console window)
    $process = Start-Process -FilePath $romDbPath -ArgumentList "`"$BiosPath`" `"$RomPath`"" -PassThru -WindowStyle Hidden

    return $process
}

# Main processing loop
while (($fileQueue.Count -gt 0 -or $jobs.Count -gt 0) -and -not $script:shouldExit) {
    # Start new jobs if slots available
    while ($jobs.Count -lt $MaxConcurrent -and $fileQueue.Count -gt 0 -and -not $script:shouldExit) {
        $nextFile = $fileQueue.Dequeue()
        $process = Start-RomProcess -RomPath $nextFile
        $jobs[$process.Id] = @{
            Process = $process
            File = $nextFile
            StartTime = Get-Date
        }
    }

    # Check for completed jobs
    $completedJobs = @()
    foreach ($jobId in $jobs.Keys) {
        $jobInfo = $jobs[$jobId]
        $process = $jobInfo.Process

        if ($process.HasExited) {
            $completedJobs += $jobId
            $duration = (Get-Date) - $jobInfo.StartTime
            $completedCount++

            if ($process.ExitCode -eq 0) {
                Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Completed: $($jobInfo.File) (Duration: $($duration.ToString('mm\:ss')))" -ForegroundColor Green
                # Only write to resume file on successful completion
                Add-Content -Path $ResumeFile -Value $jobInfo.File
            } else {
                Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Failed: $($jobInfo.File) (Exit code: $($process.ExitCode))" -ForegroundColor Red
            }

            # Progress update
            $percentComplete = [math]::Round(($completedCount / $totalFiles) * 100, 2)
            $elapsed = (Get-Date) - $startTime
            $avgTimePerFile = $elapsed.TotalSeconds / $completedCount
            $estimatedRemaining = [TimeSpan]::FromSeconds($avgTimePerFile * ($totalFiles - $completedCount))

            Write-Host "Progress: $completedCount/$totalFiles ($percentComplete%) | Elapsed: $($elapsed.ToString('hh\:mm\:ss')) | ETA: $($estimatedRemaining.ToString('hh\:mm\:ss'))" -ForegroundColor Yellow
        }
    }

    # Remove completed jobs
    foreach ($jobId in $completedJobs) {
        $jobs.Remove($jobId)
    }

    # Brief sleep to avoid CPU spinning
    Start-Sleep -Milliseconds 500
}

# If Ctrl+C was pressed, wait for remaining jobs
if ($script:shouldExit -and $jobs.Count -gt 0) {
    Write-Host "`nWaiting for $($jobs.Count) running process(es) to complete..." -ForegroundColor Yellow

    foreach ($jobId in $jobs.Keys) {
        $jobInfo = $jobs[$jobId]
        Write-Host "Waiting for: $($jobInfo.File)" -ForegroundColor Gray
        $jobInfo.Process.WaitForExit()

        if ($jobInfo.Process.ExitCode -eq 0) {
            Write-Host "Completed: $($jobInfo.File)" -ForegroundColor Green
            # Only write to resume file on successful completion
            Add-Content -Path $ResumeFile -Value $jobInfo.File
        } else {
            Write-Host "Failed: $($jobInfo.File) (Exit code: $($jobInfo.Process.ExitCode))" -ForegroundColor Red
        }
    }
}

$totalElapsed = (Get-Date) - $startTime

# Clean up screenshots matching specific hash
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "Cleaning up unwanted screenshots..." -ForegroundColor Yellow

$targetHashes = @("49C4A6CF9A24D3F1B24F48E960772B0A794C22ECB04AA85A636B32905C938A35", "600B577EA0F0E372438FBCC48269226164FC58A6FDE0AB76B913DBFCBE92DDD7")
$screenshotsPath = "screenshots"
$removedCount = 0

if (Test-Path $screenshotsPath) {
    Write-Host "Scanning screenshots directory for images matching hash..." -ForegroundColor Cyan

    $allImages = Get-ChildItem -Path $screenshotsPath -Filter "*.png" -Recurse -File
    $totalImages = $allImages.Count

    Write-Host "Found $totalImages images to check" -ForegroundColor Cyan

    $processedImages = 0
    foreach ($image in $allImages) {
        $processedImages++

        # Show progress every 100 images
        if ($processedImages % 100 -eq 0) {
            Write-Host "Progress: $processedImages/$totalImages images checked..." -ForegroundColor Gray
        }

        try {
            $hash = (Get-FileHash -Path $image.FullName -Algorithm SHA256).Hash

            if ($hash -in $targetHashes) {
                Write-Host "Removing: $($image.FullName)" -ForegroundColor Yellow
                Remove-Item -Path $image.FullName -Force
                $removedCount++
            }
        } catch {
            Write-Host "Error processing $($image.FullName): $_" -ForegroundColor Red
        }
    }

    Write-Host "Removed $removedCount unwanted screenshot(s)" -ForegroundColor Green
} else {
    Write-Host "Screenshots directory not found, skipping cleanup" -ForegroundColor Yellow
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "Processing complete!" -ForegroundColor Green
Write-Host "Total files processed: $completedCount" -ForegroundColor Cyan
Write-Host "Total time: $($totalElapsed.ToString('hh\:mm\:ss'))" -ForegroundColor Cyan
Write-Host "Unwanted screenshots removed: $removedCount" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
