# metadata

`metadata` is a media metadata parser and formatter designed for human consumption. Powered by FFmpeg.

Example:

```
$ metadata '20160907 Apple Special Event.m4v'
Title:                  Apple Special Event, September 2016 (1080p)
Filename:               20160907 Apple Special Event.m4v
File size:              6825755188 (6.83GB, 6.36GiB)
Container format:       MPEG-4 Part 14 (M4V)
Duration:               01:59:15.88
Pixel dimensions:       1920x800
Sample aspect ratio:    1:1
Display aspect ratio:   12:5
Scan type:              Progressive scan
Frame rate:             29.97 fps
Bit rate:               7631 kb/s
    #0: Video, H.264 (High Profile level 4), yuv420p, 1920x800 (SAR 1:1, DAR 12:5), 29.97 fps, 7500 kb/s
    #1: Audio, AAC (LC), 48000 Hz, stereo, 125 kb/s
    #2: Subtitle (eng), EIA-608 closed captions

```

Compare this to `ffprobe` or `mediainfo` (both great tools, just not so human-readable):

![ffprobe](https://user-images.githubusercontent.com/4149852/45572668-f5b82c00-b837-11e8-8295-f066bca019e9.png)
![mediainfo](https://user-images.githubusercontent.com/4149852/45572674-fa7ce000-b837-11e8-8fdc-dcccc57d55d9.png)

(`mediainfo` prints so much, the output doesn't even fit on my screen with 85 lines. Now try using it in a 80x24 terminal.)