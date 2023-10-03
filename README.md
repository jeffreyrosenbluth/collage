```plaintext
Create a collage from a directory of images

Usage: collage [OPTIONS] <IMAGE_DIR>

Arguments:
  <IMAGE_DIR>  The directory wiht the images to be used in the collage

Options:
  -W, --width <IMAGE_WIDTH>        The width of the images in the collage. If not specified, the width of the first image will be used
  -H, --height <IMAGE_HEIGHT>      The height of the images in the collage. If not specified, the height of the first image will be used
  -o, --orientation <ORIENTATION>  The orientation of the collage. If not specified, the default is `portrait` [default: portrait] [possible values: portrait, landscape]
  -t, --top <TOP_MARGIN>           The top and bottom margin of the collage. If not specified, the default is 0 [default: 0]
  -l, --left <LEFT_MARGIN>         The left and right margin of the collage. If not specified, the default is 0 [default: 0]
  -s, --spacing <SPACING>          The spacing between images. If not specified, the default is 20 [default: 20]
  -c, --color <BACKGROUND_COLOR>   The background color of the collage. If not specified, the default is white [default: #ffffff]
  -p, --preserve                   If true, then the aspect ratio of the images will be preserved. If not specified, the default is false
  -h, --help                       Print help
  -V, --version                    Print version
```
