# youtube_archiver

rust project that downloads all the information
about a youtube video and the video itself

## usage

### required arguments

- url: String
    * There are currently three ways to use this argument:
        - A plain YouTube URL, ex: `https://youtube.com/watch?v=dQw4w9WgXcQ` or `https://youtu.be/dQw4w9WgXcQ`
            * Simply archives the video into `dir`.
        - A YouTube channel URL using a handle, ex: `https://youtube.com/@youtube`
            * Archives all videos from provided channel into `dir`, using the video id as folder names.
        - A list of YouTube URLs separated by `,`, ex: `"https://youtu.be/dQw4w9WgXcQ,https://youtu.be/DLzxrzFCyOs"`
            * Archives all videos specified into `dir`, using the video id as folder names.
- dir: String
    * Specifies the directory in which to archive all video data.

### optional arguments

- video: bool, default: true
    * Specifies whether or not to download the whole video as part of archiving.
- metadata: bool, default: true
    * Specifies whether or not to download metadata and thumbnails as part of archiving.
- streams_and_premieres: bool, default: true
    * Specifies whether ot not to archive streams and premieres when archiving a whole channel.
    This may at times not be wanted as streams can get very long and thus take a lot of memory and time to download.
- api: String, default: https://yt.lemnoslife.com
    * Specifies the address of api to use for archiving.
    You must use an instance of the [YouTube Operational API](https://github.com/Benjamin-Loison/YouTube-operational-API) for this,
    although support for the official api and api keys maybe be added in the future if needed.

    * By default this uses the official instance of the [YouTube Operational API](https://github.com/Benjamin-Loison/YouTube-operational-API)
    but is configurable because the official instance has been unreliable at times.

## apis used

- [cobalt](https://github.com/imputnet/cobalt)
    * the actual video downloading stuff
- [YouTube Operational API](https://github.com/Benjamin-Loison/YouTube-operational-API) (yt.lemnoslife.com by default)
    * youtube data api v3 without a key (accessible for all & free)
