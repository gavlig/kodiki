PLATFORM="x86_64"
OSNAME="windows"
ARCHIVEFORMAT="zip"

# PLATFORM="aarch64"
# OSNAME="macos"
#ARCHIVEFORMAT="tar.xz"

# PLATFORM = "x86_64"
# OSNAME = "linux"
#ARCHIVEFORMAT="tar.xz"

mkdir build
mkdir runtime_temp
cd ./runtime_temp
curl -LJO https://github.com/helix-editor/helix/releases/download/23.05/helix-23.05-$PLATFORM-$OSNAME.$ARCHIVEFORMAT
if [ $ARCHIVEFORMAT = "tar.xz" ]; then
	tar -xvf helix-23.05-$PLATFORM-$OSNAME.$ARCHIVEFORMAT
elif [ $ARCHIVEFORMAT = "zip" ]; then
	unzip helix-23.05-$PLATFORM-$OSNAME.$ARCHIVEFORMAT
else
	echo "ERROR: UNKNOWN ARCHIVE FORMAT! RUNTIME IS NOT PREPARED!"
	exit 1
fi
rm helix-23.05-$PLATFORM-$OSNAME.$ARCHIVEFORMAT
mkdir ../build/bin/
cp -R ./helix-23.05-$PLATFORM-$OSNAME/runtime ../build/bin/
cp -R ../assets ../build/bin/
cd ../
rm -r runtime_temp
