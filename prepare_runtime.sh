mkdir build
mkdir runtime_temp
cd ./runtime_temp
# wget https://github.com/helix-editor/helix/releases/download/23.05/helix-23.05-x86_64-linux.tar.xz
curl -LJO https://github.com/helix-editor/helix/releases/download/23.05/helix-23.05-x86_64-linux.tar.xz 
tar -xvf helix-23.05-x86_64-linux.tar.xz
rm helix-23.05-x86_64-linux.tar.xz
mkdir ../build/bin/
cp -R ./helix-23.05-x86_64-linux/runtime ../build/bin/
cp -R ../assets ../build/bin/
cd ../
rm -r runtime_temp
