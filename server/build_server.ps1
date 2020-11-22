# NOTE: EXECUTE THIS FROM THE WORKSPACE ROOT ONLY
$default = "latest"
if (!($tag = Read-Host "Docker Image Tag [$default]")) { $tag = $default }

Copy-Item Cargo.lock server/Cargo.lock
sleep 1
docker build server/ -f server/Dockerfile -t docker.pkg.github.com/songtronix/airshipper/airshipper:$tag
sleep 1
Remove-Item server/Cargo.lock -ErrorAction Ignore
sleep 1