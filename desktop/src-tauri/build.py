import json
import os
import shutil
import subprocess
from pathlib import Path
import sys
from ctypes.util import find_library
import re
import glob
import requests
import re

# https://sourceforge.net/projects/avbuild/files/macOS/

SKIP_BUILD = os.getenv('SKIP_BUILD') == "1"
SKIP_CLEANUP = os.getenv('SKIP_CLEANUP') == "1"
# if sys.platform == 'darwin':
#     some_dylib = 'libavutil.dylib'
#     dylib_path = find_library(some_dylib)
#     dylib_path = Path(dylib_path).resolve() # resolve symlink
#     FFMPEG_FRAMEWORK_LIBS = dylib_path.parent
#     print('Found ffmpeg framework in ', FFMPEG_FRAMEWORK_LIBS)


SRC_TAURI = Path(__file__).parent
FFMPEG_FRAMEWORK_TARGET = SRC_TAURI / 'ffmpeg.framework'
CONF = SRC_TAURI / 'tauri.conf.json'
WIN_RESOURCES = [
    # FFMPEG
    "avcodec-60.dll",
    "libbrotlidec.dll",
    "libffi-8.dll",
    "libgmp-10.dll",
    "libintl-8.dll",
    "libopenjp2-7.dll",
    "libpng16-16.dll",
    "libstdc++-6.dll",
    "libvidstab.dll",
    "libxml2-2.dll",
    "avdevice-60.dll",
    "libbrotlienc.dll",
    "libfontconfig-1.dll",
    "libgnutls-30.dll",
    "liblcms2-2.dll",
    "libopus-0.dll",
    "librsvg-2-2.dll",
    "libSvtAv1Enc.dll",
    "libvorbis-0.dll",
    "libzimg-2.dll",
    "avfilter-9.dll",
    "libbz2-1.dll",
    "libfreetype-6.dll",
    "libgobject-2.0-0.dll",
    "liblzma-5.dll",
    "libp11-kit-0.dll",
    "librtmp-1.dll",
    "libtasn1-6.dll",
    "libvorbisenc-2.dll",
    "libzstd.dll",
    "avformat-60.dll",
    "libcaca-0.dll",
    "libfribidi-0.dll",
    "libgomp-1.dll",
    "libmodplug-1.dll",
    "libpango-1.0-0.dll",
    "libshaderc_shared.dll",
    "libthai-0.dll",
    "libvpl.dll",
    "postproc-57.dll",
    "avutil-58.dll",
    "libcairo-2.dll",
    "libgcc_s_seh-1.dll",
    "libgraphite2.dll",
    "libmp3lame-0.dll",
    "libpangocairo-1.0-0.dll",
    "libsharpyuv-0.dll",
    "libtheoradec-1.dll",
    "libvpx-1.dll",
    "rav1e.dll",
    "dovi.dll",
    "libcairo-gobject-2.dll",
    "libgdk_pixbuf-2.0-0.dll",
    "libgsm.dll",
    "libnettle-8.dll",
    "libpangoft2-1.0-0.dll",
    "libsoxr.dll",
    "libtheoraenc-1.dll",
    "libwebp-7.dll",
    "SDL2.dll",
    "libaom.dll",
    "libcrypto-3-x64.dll",
    "libgio-2.0-0.dll",
    "libharfbuzz-0.dll",
    "libogg-0.dll",
    "libpangowin32-1.0-0.dll",
    "libspeex-1.dll",
    "libunibreak-5.dll",
    "libwebpmux-3.dll",
    "swresample-4.dll",
    "libass-9.dll",
    "libdatrie-1.dll",
    "libglib-2.0-0.dll",
    "libhogweed-6.dll",
    "libopenal-1.dll",
    "libpcre2-8-0.dll",
    "libspirv-cross-c-shared.dll",
    "libunistring-5.dll",
    "libwinpthread-1.dll",
    "swscale-7.dll",
    "libbluray-2.dll",
    "libdav1d-7.dll",
    "libgme.dll",
    "libiconv-2.dll",
    "libopencore-amrnb-0.dll",
    "libpixman-1-0.dll",
    "libsrt.dll",
    "libva.dll",
    "libx264-164.dll",
    "xvidcore.dll",
    "libbrotlicommon.dll",
    "libexpat-1.dll",
    "libgmodule-2.0-0.dll",
    "libidn2-0.dll",
    "libopencore-amrwb-0.dll",
    "libplacebo-338.dll",
    "libssh.dll",
    "libva_win32.dll",
    "libx265.dll",
    "zlib1.dll",
    # OpenBLAS
    "libopenblas.dll",
    "libgfortran-5.dll",
    "libquadmath-0.dll",
    "vulkan-1.dll"
]
# Webview2
WIN_RESOURCES.append("../../target/release/WebView2Loader.dll")

# run after build
def clean():
    for path in RESOURCES:
        path = Path(path)
        new_path = SRC_TAURI / path.name
        new_path.unlink()

    CONF.unlink()
    # shutil.rmtree(FFMPEG_FRAMEWORK_TARGET)
    shutil.copy(CONF.with_suffix('.old.json'), CONF)    


# copy DLLs
RESOURCES = WIN_RESOURCES if sys.platform == 'win32' else [] # MAC_RESOURCES
for path in RESOURCES:
    if '/' not in path:
        path = find_library(path)
    path = Path(path)
    new_path = SRC_TAURI / path.name
    shutil.copy(path, new_path, follow_symlinks=True)

# config environment
if sys.platform == 'win32':
    env = os.environ.copy()
    env["PATH"] = f'C:\\Program Files\\Nodejs;{env["PATH"]}'
    env["OPENBLAS_PATH"]=os.getenv("MINGW_PREFIX")

# def download_ffmpeg():
#     name = 'ffmpeg-6.1-macOS-default.tar.xz'
#     url = f'https://master.dl.sourceforge.net/project/avbuild/macOS/{name}?viasf=1'
#     response = requests.get(url, stream=True)
#     response.raise_for_status()
#     f = open(name, 'wb')
#     for chunk in response.iter_content(chunk_size=8192):
#         f.write(chunk)
#     f.close()
#     # uncompress
#     name = Path(name)
    
#     import lzma

#     compressed = lzma.open(name)
#     f = open(name.with_suffix(''))
#     while True:
#         chunk = compressed.read(8192)
#         if not chunk:
#             break
#         f.write(chunk)
#     f.close()


    

def patch_config():
    # config tauri.conf.json
    shutil.copy(CONF, CONF.with_suffix('.old.json'))
    with open(CONF, 'r') as f:
        # webview_dll = TARGET / 'target/release/WebView2Loader.dll'
        data = json.load(f)
        if sys.platform == 'win32':
            data['tauri']['bundle']['resources'] = data['tauri']['bundle'].get("resources", []) + [Path(i).name for i in RESOURCES]
        elif sys.platform == 'darwin':
            data['build']['beforeBundleCommand'] = f"codesign -s - {SRC_TAURI / '../../target/release/vibe'}"
    with open(CONF, 'w') as f:
        print('Patching config', data, CONF)
        json.dump(data, f, indent=4)

def post_build():
    print("Post build...")
    if sys.platform != 'darwin':
        return
    DMG_PARENT = SRC_TAURI / '../../target/release/bundle/dmg'
    DMG_MOUNT_POINT = Path('/Volumes/vibe1')
    DMG_PATH = DMG_PARENT.glob('*.dmg').__next__().absolute()
    # Mount
    subprocess.run(f'hdiutil attach -shadow -nobrowse -mountpoint {DMG_MOUNT_POINT} {DMG_PATH}', shell=True, check=True)
    # Copy
    LOCAL_DMG = SRC_TAURI / 'vibe'
    shutil.copytree(DMG_MOUNT_POINT, LOCAL_DMG, symlinks=True) # dont copy sylinks content
    # Unmount
    subprocess.run(f'hdiutil detach {DMG_MOUNT_POINT}', shell=True, check=True)
    
    # Get dylibs
    DMG_CONTENTS = LOCAL_DMG / 'vibe.app/Contents'
    DMG_EXECUTABLE = DMG_CONTENTS / 'MacOS/vibe'
    FFMPEG_FRAMEWORKS_PATH = DMG_CONTENTS / 'Frameworks/ffmpeg.framework'
    FFMPEG_FRAMEWORKS_PATH.mkdir(exist_ok=True, parents=True)

    # Sign the binary

    # Create new dmg file
    FINAL_DMG = 'final.dmg'
    subprocess.run(f'hdiutil create -format UDZO -srcfolder {LOCAL_DMG} {FINAL_DMG}', shell=True, check=True)
    # Rename new dmg file



# build
try:
    patch_config()
    if not SKIP_BUILD and sys.platform == 'win32':
        result = subprocess.run('cargo tauri build', shell=True, check=True, env=env)
    elif not SKIP_BUILD:
        result = subprocess.run('cargo tauri build', shell=True, check=True)
    # post_build()
finally:
    if not SKIP_CLEANUP:
        pass
        # clean()


