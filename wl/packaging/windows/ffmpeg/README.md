# Bundled FFmpeg

Include avcodec, avformat, avutil, swscale and swresample DLLs from one dynamically linked LGPL build. Pin the version, source, hashes, --disable-gpl --disable-nonfree options and legal notices before distribution.

Recording and preview also require `ffmpeg.exe`; DLLs alone are insufficient.
Keep the executable and its matching DLLs together in the portable package's
`ffmpeg/` directory. Include the build's licenses, notices and source information.
Do not ship a GPL or nonfree build.
