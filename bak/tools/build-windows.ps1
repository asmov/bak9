Param (
    [Parameter(Mandatory)]
    $Target
)

$PROJECT_DIR = Split-Path ($MyInvocation.MyCommand.Path) -Parent 
cd $PROJECT_DIR

function Log {
    Param (
        [Parameter(Mandatory)]
        $Output
    )

    Write-Output "`n$Output"
}

Log "Began building windows release: ${Target}"

Log "Debug testing: ${Target}"
cargo test --target="${Target}"

Log "Building release: ${Target}"
cargo build --release --target="${Target}"

Log "Testing release: ${Target}"
cargo test --release --target="${Target}"

Log "Building msi: ${Target}"
cargo wix

Log "Finished building windows release: ${Target}"

