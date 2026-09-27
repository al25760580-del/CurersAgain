#!/bin/bash
set -euo pipefail
B=/home/mila/gamemaker-analysis
W="$B/tools/windows-build"
export LD_LIBRARY_PATH="$W/clang/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
C="$W/clang/usr/bin/clang-cl"
I="$B/tools/HoloCureDataDumpMod/HoloCureDataDumpMod/include"
S="$W/sdk"
cd "$B/hover-inspector"
mkdir -p objects
FLAGS=(--target=x86_64-pc-windows-msvc /nologo /std:c++20 /EHsc /MD /O2 /DNDEBUG /DNOMINMAX /DUNICODE /D_UNICODE /I"$I" /imsvc"$S/crt/include" /imsvc"$S/sdk/include/ucrt" /imsvc"$S/sdk/include/shared" /imsvc"$S/sdk/include/um" /clang:-Wno-ignored-attributes /clang:-Wno-microsoft-cast)
"$C" "${FLAGS[@]}" /c HoverInspector.cpp /Foobjects/HoverInspector.obj
if [ ! -f objects/YYTK_Shared_Types.obj ]; then
"$C" "${FLAGS[@]}" /c "$I/YYToolkit/YYTK_Shared_Types.cpp" /Foobjects/YYTK_Shared_Types.obj
fi
lld-link /dll /out:HoloCureHoverInspector.dll /implib:objects/HoloCureHoverInspector.lib /machine:x64 /opt:ref /opt:icf objects/HoverInspector.obj objects/YYTK_Shared_Types.obj /libpath:"$S/crt/lib/x86_64" /libpath:"$S/sdk/lib/um/x86_64" /libpath:"$S/sdk/lib/ucrt/x86_64" kernel32.lib user32.lib
sha256sum HoloCureHoverInspector.dll
