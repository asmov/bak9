Param (
    [Parameter(Mandatory)]
    $Target
)

function Log {
    Param (
        [Parameter(Mandatory)]
        $Output
    )

    Write-Host "[$(Get-Date -Format "HH:mm:ss") bak9] " -NoNewline -ForegroundColor Green 
    Write-Host $Output
}

$PROJECT_DIR = Split-Path $(Split-Path ($MyInvocation.MyCommand.Path) -Parent) -Parent 
Set-Location -Path $PROJECT_DIR -PassThru
Write-Host "$(Get-Location)"

Log "Began building windows release: ${Target}"

Log "Debug testing: ${Target}"
cargo test --target="${Target}"

Log "Building release: ${Target}"
cargo build --release --target="${Target}"

Log "Testing release: ${Target}"
cargo test --release --target="${Target}"

Log "Building .msi: ${Target}"
Write-Output "`n$(Get-Location)"
cargo wix

Log "Finished building windows release: ${Target}"

