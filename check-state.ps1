$dbPath = "$env:APPDATA\com.arcanecodex.app\arcanecodex.db"

# 使用 rusqlite 的 bundled 特性编译的 sqlite3 应该可用
# 尝试用 .NET 的 System.Data.SQLite
Add-Type -AssemblyName System.Data.SQLite -ErrorAction SilentlyContinue

if (-not ([System.Reflection.Assembly]::GetAssembly([System.Data.SQLite.SQLiteConnection]))) {
    # 下载 System.Data.SQLite
    Write-Host "System.Data.SQLite not available. Trying to find sqlite3.exe..."
    $sqlite3 = Get-Command sqlite3 -ErrorAction SilentlyContinue
    if ($sqlite3) {
        Write-Host "Found sqlite3 at: $($sqlite3.Source)"
        & $sqlite3 $dbPath "SELECT id, file_path, thumbnail_path, ai_status FROM images ORDER BY id;"
    } else {
        Write-Host "No SQLite tool available. Using file inspection..."
    }
    exit
}

$conn = New-Object System.Data.SQLite.SQLiteConnection("Data Source=$dbPath")
$conn.Open()

# 查询图片
$cmd = $conn.CreateCommand()
$cmd.CommandText = "SELECT id, file_path, thumbnail_path, ai_status FROM images ORDER BY id"
$reader = $cmd.ExecuteReader()
Write-Host "=== Images ==="
while ($reader.Read()) {
    $id = $reader.GetInt64(0)
    $fp = $reader.GetString(1)
    $tp = if ($reader.IsDBNull(2)) { "NULL" } else { $reader.GetString(2) }
    $st = $reader.GetString(3)
    Write-Host "ID=$id thumb=$tp status=$st"
}
$reader.Close()

# 查询总数
$cmd2 = $conn.CreateCommand()
$cmd2.CommandText = "SELECT COUNT(*) FROM images"
$count = $cmd2.ExecuteScalar()
Write-Host "`nTotal images: $count"

$conn.Close()
