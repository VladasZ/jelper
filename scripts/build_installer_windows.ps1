param(
  [Parameter(Mandatory)][string]$Version
)

iscc /DAppVersion=$Version installer.iss
