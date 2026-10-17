[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$CliArgs
)

$ErrorActionPreference = 'Stop'
$cli = Get-Command playwright-cli.cmd -CommandType Application -ErrorAction SilentlyContinue
if (-not $cli) {
    $cli = Get-Command playwright-cli -CommandType Application -ErrorAction SilentlyContinue
}
if (-not $cli) {
    throw 'Playwright CLI was not found on PATH. Add its installation directory before running this script.'
}

& $cli.Source @CliArgs
exit $LASTEXITCODE
