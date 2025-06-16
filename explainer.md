# Symark: A SiYuan Note Conversion System

## Overview

Symark is a program that converts structured notes from the SiYuan note-taking application into HTML pages, creating an interconnected web of documents. It's designed specifically for transforming SiYuan notebooks into a static website, preserving all the rich formatting and interconnections from the original notes.

## Project Structure

Symark is organized as a Rust project with the following structure:

```
symark/
├── src/                # Source code
│   └── main.rs         # Main Rust implementation file
├── input/              # Directory for SiYuan notes
│   ├── .siyuan/        # SiYuan metadata
│   └── [note-id].sy    # SiYuan note files (JSON format)
├── template/           # HTML and CSS templates
│   ├── page.html       # HTML template for generated pages
│   └── styles.css      # CSS styles for the website
├── output/             # Generated website (created by Symark)
├── Cargo.toml          # Rust project configuration
└── Cargo.lock          # Rust dependency lock file
```

The main workflow involves:
1. Reading SiYuan notes from the `input/` directory
2. Processing them using the code in `src/main.rs`
3. Applying templates from the `template/` directory
4. Generating the final website in the `output/` directory

## SiYuan Integration

Symark is built specifically to work with [SiYuan](https://github.com/siyuan-note/siyuan), an open-source personal knowledge management system. SiYuan stores notes as `.sy` files in JSON format, which Symark processes to generate HTML.

Key SiYuan features that Symark preserves:
- Bidirectional links between notes
- Block references and transclusion
- Hierarchical document structure
- Rich formatting and markdown extensions
- Custom styling for blocks and notes
- Tagging system

The data structures in Symark directly map to SiYuan's internal representation of notes, ensuring faithful reproduction of content.

## Core Data Structures

### Note

The `Note` structure represents a complete note document:

- **ID**: Unique identifier for the note
- **Spec**: Specification version
- **Type**: Type of the note
- **Properties**: Metadata about the note (title, tags, etc.)
- **Children**: The content blocks that make up the note

### Properties

The `Properties` structure contains metadata about a note:

- **id**: Unique identifier
- **title**: Title of the note
- **title_img**: Optional image for the note header
- **tags**: List of tags associated with the note
- **note_type**: Type of note
- **updated**: Last updated timestamp
- **created**: Creation timestamp
- **style**: Custom style information
- **parent_style**: Style inherited from parent

### Block

The `Block` structure represents a content element within a note, directly mapping to SiYuan's block-based document model:

- **ID**: Unique identifier for the block (SiYuan uses a timestamp-based ID system)
- **Type**: The block type (heading, paragraph, list, etc.)
- **Data**: Content data
- **Properties**: Block-specific properties
- **Children**: Nested blocks
- Various specialized fields for different block types:
  - **HeadingLevel**: For heading blocks (h1-h6)
  - **ListData**: For ordered and unordered lists
  - **TableAligns**: For table column alignment
  - **TextMarkType**: For inline formatting
  - **TextMarkAHref**: For hyperlinks
  - **TextMarkBlockRefID**: For block references (a key SiYuan feature)
  - **CodeBlockInfo**: For syntax highlighting in code blocks
  - **TaskListItemChecked**: For task lists with checkboxes

### TocItem

The `TocItem` structure is used to build a table of contents:

- **id**: ID of the heading block
- **text**: Text content of the heading
- **level**: Heading level (1-6)

## Main Functionality

### Document Processing

1. **Main Function**: Orchestrates the entire process, from finding notes to generating HTML
2. **Finding Notes**: `find_sy_files` discovers all note files in the source directory
3. **Asset Management**: `find_and_copy_assets` and `copy_directory` handle static assets

### HTML Generation

1. **Note Rendering**: `generate_html_for_note` converts a single note to HTML
2. **Block Rendering**:
   - `render_blocks` processes blocks and their children recursively
   - `render_blocks_with_ids` processes blocks with specific IDs (for cross-referencing)
   - `render_block` dispatches rendering to appropriate handler based on block type
   - `render_text_mark` handles inline formatting, links, and references
   - `find_block_by_id` and `find_content_by_id` support cross-referencing between blocks

### Navigation and Organization

1. **Index Pages**:
   - `generate_index_page` creates the main landing page
   - `generate_custom_index_page` creates customized index pages
   - `generate_all_notes_page` creates a page listing all notes
   - `generate_tag_page` creates pages for browsing notes by tag

2. **Table of Contents**:
   - `extract_toc_items` identifies headings within a note
   - `generate_toc_html` creates a navigable table of contents

### Utility Functions

1. **Template Management**: `read_template` loads HTML templates
2. **Date Formatting**: `naturalize_date` converts timestamps to human-readable format
3. **HTML Helpers**:
   - `escape_html` prevents HTML injection
   - `remove_zero_width_spaces` cleans up text
   - `get_style_class` determines styling for elements

## Flow of Execution

1. The program starts by finding all `.sy` files (SiYuan's JSON note format) in a specified directory using `find_sy_files`
2. For each note file:
   - Parse the note structure into the `Note` data structure
   - Extract headings to build a table of contents using `extract_toc_items`
   - Generate HTML representation of the note content via `generate_html_for_note`
     - The note's blocks are rendered recursively through `render_blocks`
     - Different block types (paragraphs, headings, lists, code blocks, etc.) are handled with specialized logic
     - Text marks (formatting, links, references) are processed by `render_text_mark`
   - Create navigation elements (table of contents, links to related notes)
   - Apply styling and formatting based on note properties

3. Generate index pages:
   - Main index page via `generate_index_page`
   - Custom index pages via `generate_custom_index_page`
   - Tag-based index pages via `generate_tag_page`
   - Complete note listing via `generate_all_notes_page`

4. Copy any associated assets (images, CSS, etc.) using `find_and_copy_assets` and `copy_directory`

## Use Cases

This system appears designed for:

1. Converting private notes into a publishable format
2. Creating a knowledge base or documentation site
3. Building a personal wiki from structured notes

The flexibility in the block rendering system suggests it can handle a variety of content types, including text, lists, tables, code blocks, and embedded media.

## Tag System

Symark features a robust tagging system that creates connections between related notes:

1. **Tag Storage**: Tags are stored in the `Properties.tags` field of each note
2. **Tag Page Generation**: The `generate_tag_page` function creates dedicated pages for each tag
3. **Tag Filtering**: `filter_index_tag` helps process and organize tags
4. **Tag-Based Navigation**: Notes with matching tags are linked together, creating a web of related content

The tag system enables:
- Categorization of notes by topic or purpose
- Discovery of related content
- Creation of custom collections based on tag combinations
- Improved navigation between thematically connected notes

## Detailed Rendering Process

The heart of Symark is its block rendering system. Here's how it works:

1. **Block Type Identification**: The `render_blocks` function processes each block based on its `Type` field:
   - Paragraphs
   - Headings (with levels 1-6)
   - Lists (ordered, unordered, and task lists)
   - Code blocks (with syntax highlighting support)
   - Tables
   - Blockquotes
   - HTML blocks
   - And more specialized types

2. **Text Formatting**: Within text blocks, `render_text_mark` handles:
   - Basic formatting (bold, italic, underline, strikethrough)
   - Links (external URLs and internal references)
   - Code spans
   - Superscript and subscript
   - Highlighting
   - Block references (linking to other blocks by ID)
   - Inline memos

3. **Cross-References**: The system supports references between notes:
   - `find_block_by_id` locates blocks by their unique identifiers
   - `find_content_by_id` extracts content from referenced blocks
   - `NodeBlockQueryEmbed` handles transclusions within other notes
   - This enables a wiki-like navigation experience between notes

4. **HTML Safety and SiYuan Compatibility**: Several safeguards ensure the generated HTML is safe and valid:
   - `escape_html` prevents HTML injection in user content
   - `remove_zero_width_spaces` cleans up text to improve rendering (SiYuan uses zero-width spaces in certain contexts)
   - `get_style_class` translates SiYuan's custom styling variables into appropriate HTML/CSS classes
   - Careful handling of nested structures (lists within lists, etc.)

5. **Navigation Generation**: For each note, the system:
   - Extracts headings to build a table of contents via `extract_toc_items`
   - Generates a hierarchical TOC via `generate_toc_html`
   - Adds a back navigation link (defined in `BACK_NAVIGATION_HTML`)
   - Creates links to related notes based on tags

This sophisticated rendering pipeline transforms structured note data into a rich, interconnected HTML document collection that preserves the relationships and formatting of the original notes.

## Templates

Symark uses a templating system to generate consistent HTML pages:

1. **HTML Template**: The `template/page.html` file provides the structure for each generated page, with placeholders for:
   - Page title
   - Content
   - Navigation elements
   - Metadata

2. **CSS Styling**: The `template/styles.css` file contains all styling for the generated website, including:
   - Basic typography and layout
   - Special formatting for different block types
   - Responsive design elements
   - Navigation styling

The templating system allows for consistent presentation across all generated pages while injecting the specific content and structure of each note.

## Date Handling

Symark includes a sophisticated date handling system through the `naturalize_date` function, which converts SiYuan's timestamp format (YYYYMMDD) into human-readable dates. This function:

1. Recognizes SiYuan's date format patterns
2. Converts numeric dates to natural language formats
3. Handles relative dates (yesterday, last week, etc.)
4. Supports both date-only and datetime formats

## Conclusion

Symark is a powerful tool specifically designed for converting SiYuan notes into an interconnected web of HTML documents. Its key strengths include:

1. **SiYuan Compatibility**: Deep integration with SiYuan's note format and features
2. **Flexible Content Rendering**: Support for all SiYuan content types and formatting
3. **Relationship Preservation**: Maintains connections between notes through references and tags
4. **Navigation Enhancement**: Generates tables of contents, tag pages, and index pages
5. **Asset Management**: Handles images and other resources required by the notes

The system allows knowledge captured in SiYuan to be transformed into a publishable format while preserving the rich structure and interconnections that make personal knowledge management systems valuable. By generating static HTML, the resulting site can be easily hosted and shared without requiring specialized server infrastructure.

Symark bridges the gap between personal note-taking in SiYuan and public knowledge sharing, making it easier to transform private notes into documentation, wikis, or knowledge bases that can be shared with others. It's an excellent example of extending a personal knowledge management system with publishing capabilities.

## Building and Usage

As a Rust project, Symark can be built using standard Cargo commands:

```
cargo build --release
```

Basic usage involves:
1. Placing SiYuan notes in the `input/` directory
2. Running the Symark executable
3. Accessing the generated website from the `output/` directory

The system is designed to be simple to use while providing powerful conversion capabilities.
