<h1 align="center">SyMark - Turn SiYuan notes into webpages</h1>
SyMark is an open-source static site generator that converts SiYuan notebooks into responsive websites with a single terminal command. Every aspect of your notes—bidirectional links, tags, and all formatting options—renders elegantly on the web, just as it does in the desktop editor.

![symark](https://github.com/user-attachments/assets/5e5eedc5-412e-4635-a768-64d46e86b75e)

## Overview

SyMark processes `.sy` files from SiYuan notebooks and generates a static website with HTML pages for each note, tag collections, and index pages. It preserves the structure and formatting of your notes while creating a clean, responsive web interface.

## Features

- Converts SiYuan notes to responsive HTML webpages
- Maintains bidirectional links and transclusions between notes
- Supports tags with dedicated tag pages
- Automatically generates table of contents
- Renders markdown formatting, code blocks, and other SiYuan features
- Custom index page support
- Zero-width whitespace character removal for clean HTML output
- Lightning-fast generation even for large notebooks (1,000,000+ notes)
- Privacy-focused with no trackers or telemetry

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- SiYuan notebook with exported `.sy` files

### Running SyMark

1. Extract the archive `NotebookName.sy.zip` into the `input` directory in SyMark
2. Run SyMark:
   ```sh
   cargo run
   ```

3. The generated website will be in the `output/` directory
4. Open `output/index.html` in your browser to view your website

The program will display information about the generation process, including:
- Number of notes processed
- Tags found
- Generated HTML files
- Build time statistics

## Customization & Theming

### Project Structure

SyMark expects the following directory structure:

```
symark/
├── input/              # Directory containing SiYuan notebook data
│   ├── assets/         # Assets from SiYuan (images, etc.)
│   ├── [note-id]/      # SiYuan note directories
│   └── [note-id].sy    # SiYuan note files (JSON format)
├── template/           # HTML and CSS templates
│   ├── page.html       # HTML template for generated pages
│   └── styles.css      # CSS styles for the website
└── output/             # Generated website (created by SyMark)
```

### Templates

SyMark uses HTML and CSS templates from the `template/` directory:

- `page.html`: The base HTML template for all generated pages
- `styles.css`: The CSS styles for the website

If these files don't exist, SyMark will create default templates when first run.

### Customization

#### Custom Index Page

To create a custom index page, add the tag `index` to any note in SiYuan. SyMark will use that note as the content for the main `index.html` page.

For example, the note "20250506164324-csw026m.sy" in the sample data has the tag "index" and will be used as the landing page. The system will also generate an "all.html" page that contains links to all notes.

#### Styling

Customize the appearance by editing `template/styles.css`.

## Tags

Tags in your SiYuan notes become browsable collections in the generated website. For each unique tag, SyMark creates a dedicated page listing all notes with that tag.

## Navigation

The generated website includes:

- `index.html`: Main entry point (custom or default listing)
- `all.html`: Complete list of all notes
- `tag_[tagname].html`: Pages for each tag collection (e.g., `tag_Features.html`)
- `[note-id].html`: Individual note pages (e.g., `20250506164324-csw026m.html`)

Each page includes navigation links to easily browse between notes, tags, and the index page.

## Troubleshooting

If you encounter issues:

1. Ensure your `.sy` files are valid JSON
2. Check that all referenced assets exist in the `input/assets/` directory
3. Verify you have read/write permissions for the `output/` directory
4. Make sure your note IDs follow the SiYuan format (YYYYMMDDhhmmss-xxxxx)
5. For missing assets, check the console output for warnings

### Common Issues

- **Missing images**: Ensure image assets are in the `input/assets/` directory
- **Broken links**: Check that referenced note IDs exist in your input directory
- **Formatting issues**: Verify your SiYuan notes use supported formatting features

## Performance

SyMark is optimized for speed and can process large notebooks efficiently:
- Processes hundreds of notes in milliseconds
- Automatically removes zero-width whitespace characters for clean HTML
- Minimal memory footprint suitable for low-end hardware

## License
```
This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or
distribute this software, either in source code form or as a compiled
binary, for any purpose, commercial or non-commercial, and by any
means.

In jurisdictions that recognize copyright laws, the author or authors
of this software dedicate any and all copyright interest in the
software to the public domain. We make this dedication for the benefit
of the public at large and to the detriment of our heirs and
successors. We intend this dedication to be an overt act of
relinquishment in perpetuity of all present and future rights to this
software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.
```
For more information, please refer to <https://unlicense.org/>

## Acknowledgements

SyMark is designed to work with [SiYuan](https://github.com/siyuan-note/siyuan), an excellent open-source personal knowledge management system.
