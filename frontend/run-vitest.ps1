$output = & npx vitest run 2>&1
$output -join "`n" | Out-File -FilePath "e:\ArcaneCodex\frontend\vitest-output.txt" -Encoding UTF8
