function Resolve-Symlinks {
    [CmdletBinding()]
    [OutputType([string])]
    param(
        [Parameter(Position = 0, Mandatory, ValueFromPipeline, ValueFromPipelineByPropertyName)]
        [string] $Path
    )

    [string] $separator = '/'
    [string[]] $parts = $Path.Split($separator)

    [string] $realPath = ''
    foreach ($part in $parts) {
        if ($realPath -and !$realPath.EndsWith($separator)) {
            $realPath += $separator
        }

        $realPath += $part.Replace('\', '/')

        # The slash is important when using Get-Item on Drive letters in pwsh.
        if (-not($realPath.Contains($separator)) -and $realPath.EndsWith(':')) {
            $realPath += '/'
        }

        if (Test-Path -LiteralPath $realPath) {
            $item = Get-Item -LiteralPath $realPath
            if ($item.LinkTarget) {
                $target = $item.LinkTarget.Replace('\', '/')
                if ([System.IO.Path]::IsPathRooted($target)) {
                    $realPath = $target
                } else {
                    $parentDir = [System.IO.Path]::GetDirectoryName($realPath)
                    $realPath = [System.IO.Path]::GetFullPath([System.IO.Path]::Combine($parentDir, $target)).Replace('\', '/')
                }
            }
        }
    }
    $realPath
}

$path = Resolve-Symlinks -Path $args[0]
Write-Host $path
