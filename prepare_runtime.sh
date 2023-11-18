PLATFORM="aarch64"
OS="macos"

# PLATFORM = "x86_64"
# OS = "linux"

mkdir build
mkdir runtime_temp
cd ./runtime_temp
curl -LJO https://github.com/helix-editor/helix/releases/download/23.05/helix-23.05-$PLATFORM-$OS.tar.xz
tar -xvf helix-23.05-$PLATFORM-$OS.tar.xz
rm helix-23.05-$PLATFORM-$OS.tar.xz
mkdir ../build/bin/
cp -R ./helix-23.05-$PLATFORM-$OS/runtime ../build/bin/
cp -R ../assets ../build/bin/
cd ../
rm -r runtime_temp
