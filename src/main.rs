//! SyMark: Static site generator for SiYuan notes.

use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use chrono::Local;
use base64::decode;
use std::time::Instant;

#[derive(Debug, Deserialize)]
struct Note {
    ID: String,
    #[serde(default)]
    Spec: String,
    #[serde(default)]
    Type: String,
    #[serde(default)]
    Properties: Properties,
    #[serde(default)]
    Children: Vec<Block>,
}

#[derive(Debug, Deserialize, Default)]
struct Properties {
    #[serde(default)]
    id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    #[serde(rename = "title-img")]
    title_img: String,
    #[serde(default)]
    tags: String,
    #[serde(default)]
    #[serde(rename = "type")]
    note_type: String,
    #[serde(default)]
    updated: String,
    #[serde(default)]
    created: String,
    #[serde(default)]
    style: String,
    #[serde(default)]
    #[serde(rename = "parent-style")]
    parent_style: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct Block {
    #[serde(default)]
    ID: String,
    #[serde(default)]
    Type: String,
    #[serde(default)]
    Data: String,
    #[serde(default)]
    Properties: Properties,
    #[serde(default)]
    Children: Vec<Block>,
    #[serde(default)]
    HeadingLevel: u8,
    #[serde(default)]
    ListData: serde_json::Value,
    #[serde(default)]
    TableAligns: Vec<i32>,

    #[serde(default)]
    TextMarkType: String,
    #[serde(default)]
    TextMarkTextContent: String,
    #[serde(default)]
    TextMarkAHref: String,
    #[serde(default)]
    TextMarkBlockRefID: String,
    #[serde(default)]
    TextMarkBlockRefSubtype: String,
    #[serde(default)]
    TextMarkInlineMemoContent: String,

    #[serde(default)]
    IsFencedCodeBlock: bool,
    #[serde(default)]
    CodeBlockInfo: String,

    #[serde(default)]
    TaskListItemChecked: bool,
}

fn get_style_class(style: &str, is_inline: bool) -> Option<String> {
    let base_class = if style.contains("var(--b3-card-info-background)") && style.contains("var(--b3-card-info-color)") {
        "info-box"
    } else if style.contains("var(--b3-card-success-background)") && style.contains("var(--b3-card-success-color)") {
        "success-box"
    } else if style.contains("var(--b3-card-warning-background)") && style.contains("var(--b3-card-warning-color)") {
        "warning-box"
    } else if style.contains("var(--b3-card-error-background)") && style.contains("var(--b3-card-error-color)") {
        "error-box"
    } else {
        return None;
    };

    if is_inline {
        Some(format!("inline-{}", base_class))
    } else {
        Some(base_class.to_string())
    }
}

fn read_template(path: &str) -> String {
    match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading template file {}: {}", path, e);
            String::new()
        }
    }
}

struct TocItem {
    id: String,
    text: String,
    level: u8,
}

const BACK_NAVIGATION_HTML: &str = r#"<a href="index.html" class="back-link">Back to home<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-arrow-left"><path d="m12 19-7-7 7-7"></path><path d="M19 12H5"></path></svg></a>"#;

fn extract_toc_items(blocks: &[Block], headings: &mut Vec<TocItem>, id_counter: &mut usize) {
    for block in blocks {
        if block.Type == "NodeHeading" {
            let level = block.HeadingLevel.max(1).min(6);

            let id = if !block.ID.is_empty() {
                block.ID.clone()
            } else {
                format!("heading-{}", id_counter)
            };
            *id_counter += 1;

            let mut text = String::new();
            for child in &block.Children {
                if child.Type == "NodeText" {
                    text.push_str(&child.Data);
                } else if child.Type == "NodeTextMark" {
                    text.push_str(&child.TextMarkTextContent);
                }
            }

            headings.push(TocItem {
                id,
                text,
                level,
            });
        }

        extract_toc_items(&block.Children, headings, id_counter);
    }
}

fn naturalize_date(date_str: &str) -> String {
    if date_str.is_empty() {
        return String::from("Unknown date");
    }

    if date_str.len() >= 8 && date_str.chars().take(8).all(|c| c.is_digit(10)) {
        let year = &date_str[0..4];
        let month_num: u32 = date_str[4..6].parse().unwrap_or(1);
        let day: u32 = date_str[6..8].parse().unwrap_or(1);

        let month_name = match month_num {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Unknown",
        };

        // Add ordinal suffix to day
        let day_with_suffix = match day {
            1 | 21 | 31 => format!("{}st", day),
            2 | 22 => format!("{}nd", day),
            3 | 23 => format!("{}rd", day),
            _ => format!("{}th", day),
        };

        let formatted_date = format!("{} {}, {}", month_name, day_with_suffix, year);

        if date_str.len() > 8 && date_str.chars().nth(8) == Some('T') {
            if date_str.len() >= 14 { // Has at least hour and minute
                let hour: u32 = date_str[9..11].parse().unwrap_or(0);
                let minute: u32 = date_str[11..13].parse().unwrap_or(0);

                let hour12 = if hour == 0 {
                    12
                } else if hour > 12 {
                    hour - 12
                } else {
                    hour
                };

                let am_pm = if hour >= 12 { "PM" } else { "AM" };
                return format!("{} at {}:{:02} {}", formatted_date, hour12, minute, am_pm);
            }
        }

        return formatted_date;
    }

    // ISO format date handling (YYYY-MM-DD)
    if date_str.len() >= 10 &&
       date_str.chars().nth(4) == Some('-') &&
       date_str.chars().nth(7) == Some('-') {
        // Extract from ISO format
        if let Ok(year) = date_str[0..4].parse::<u32>() {
            if let Ok(month) = date_str[5..7].parse::<u32>() {
                if let Ok(day) = date_str[8..10].parse::<u32>() {
                    // Get month name
                    let month_name = match month {
                        1 => "January",
                        2 => "February",
                        3 => "March",
                        4 => "April",
                        5 => "May",
                        6 => "June",
                        7 => "July",
                        8 => "August",
                        9 => "September",
                        10 => "October",
                        11 => "November",
                        12 => "December",
                        _ => "Unknown",
                    };

                    // Add ordinal suffix to day
                    let day_with_suffix = match day {
                        1 | 21 | 31 => format!("{}st", day),
                        2 | 22 => format!("{}nd", day),
                        3 | 23 => format!("{}rd", day),
                        _ => format!("{}th", day),
                    };

                    // Check if there's time information
                    if date_str.len() >= 16 && date_str.chars().nth(10) == Some('T') {
                        // Has time component
                        if let Ok(hour) = date_str[11..13].parse::<u32>() {
                            if let Ok(minute) = date_str[14..16].parse::<u32>() {
                                // Format with AM/PM
                                let hour12 = if hour == 0 {
                                    12
                                } else if hour > 12 {
                                    hour - 12
                                } else {
                                    hour
                                };

                                let am_pm = if hour >= 12 { "PM" } else { "AM" };
                                return format!("{} {}, {} at {}:{:02} {}",
                                    month_name, day_with_suffix, year, hour12, minute, am_pm);
                            }
                        }
                    }

                    return format!("{} {}, {}", month_name, day_with_suffix, year);
                }
            }
        }
    }

    // If we can't parse it in any of our formats, return original
    return date_str.to_string();
}

fn generate_toc_html(headings: &[TocItem]) -> String {
    let mut toc_html = String::new();

    for heading in headings {
        // Only include h2 and h3 in the TOC
        if heading.level >= 2 && heading.level <= 3 {
            let class = if heading.level == 3 { " toc-subitem" } else { "" };
            toc_html.push_str(&format!(
                "<li class=\"toc-item{}\"><a class=\"toc-link\" href=\"#{}\">{}",
                class, heading.id, heading.text
            ));
            toc_html.push_str("</a></li>\n");
        }
    }

    if toc_html.is_empty() {
        toc_html = "<li class=\"toc-item\"><em>No headings found</em></li>\n".to_string();
    }

    toc_html
}

fn filter_index_tag(tags_str: &str) -> String {
    tags_str
        .split(',')
        .map(|t| t.trim())
        .filter(|t| *t != "index")
        .collect::<Vec<_>>()
        .join(", ")
}

fn main() -> std::io::Result<()> {
    let start_time = Instant::now();
    let mut page_count = 0;

    println!("Starting SyMark generator...");

    let template_dir = PathBuf::from("template");
    println!("Template directory: {:?}", template_dir);
    if !template_dir.exists() {
        println!("Creating template directory...");
        fs::create_dir_all(&template_dir)?;

        let html_template = read_template("template/page.html");
        let css_template = read_template("template/styles.css");

        let cleaned_html_template = remove_zero_width_spaces(&html_template);
        let mut html_file = File::create(template_dir.join("page.html"))?;
        html_file.write_all(cleaned_html_template.as_bytes())?;

        let cleaned_css_template = remove_zero_width_spaces(&css_template);
        let mut css_file = File::create(template_dir.join("styles.css"))?;
        css_file.write_all(cleaned_css_template.as_bytes())?;

        println!("Created template files in {:?}", template_dir);
    }

    let output_dir = PathBuf::from("output");
    println!("Output directory: {:?}", output_dir);
    if output_dir.exists() {
        println!("Removing existing output directory...");
        fs::remove_dir_all(&output_dir)?;
    }
    println!("Creating output directory...");
    fs::create_dir_all(&output_dir)?;

    let assets_dir = output_dir.join("assets");
    println!("Assets directory: {:?}", assets_dir);
    fs::create_dir_all(&assets_dir)?;

    println!("Finding and copying assets...");
    find_and_copy_assets(Path::new("input"), &assets_dir)?;

    println!("Reading CSS template...");
    let css_template = read_template("template/styles.css");
    let cleaned_css = remove_zero_width_spaces(&css_template);
    let css_path = output_dir.join("styles.css");
    println!("Writing CSS to: {:?}", css_path);
    let mut css_file = File::create(&css_path)?;
    css_file.write_all(cleaned_css.as_bytes())?;

    println!("Finding .sy files...");
    let mut note_files = Vec::new();
    find_sy_files(Path::new("input"), &mut note_files)?;
    println!("Found {} .sy files", note_files.len());

    println!("Parsing notes...");
    let mut notes_map = HashMap::new();
    let mut id_to_path = HashMap::new();
    let mut all_tags = HashSet::new();
    let mut index_note_id: Option<String> = None;

    for path in &note_files {
        let content = fs::read_to_string(path)?;
        match serde_json::from_str::<Note>(&content) {
            Ok(mut note) => {
                let id = note.ID.clone(); // Clone the ID before moving the note
                id_to_path.insert(id.clone(), path.clone());

                // Extract creation date from note ID if it's not set
                if note.Properties.created.is_empty() && id.len() >= 14 {
                    // Format is YYYYMMDDhhmmss-xxx
                    note.Properties.created = id[0..14].to_string();
                }

                if !note.Properties.tags.is_empty() {
                    for tag in note.Properties.tags.split(',') {
                        let tag = tag.trim().to_string();

                        if tag != "index" {
                            all_tags.insert(tag.clone());
                        }

                        // Check if this note has the "index" tag
                        if tag == "index" {
                            if index_note_id.is_some() {
                                println!("Warning: Multiple notes with 'index' tag found. Using the last one found.");
                            }
                            index_note_id = Some(id.clone());
                        }
                    }
                }

                notes_map.insert(id, note);
            }
            Err(e) => {
                eprintln!("Error parsing file {:?}: {}", path, e);
            }
        }
    }

    println!("Reading HTML template...");
    let html_template = read_template("template/page.html");


    if let Some(index_id) = &index_note_id {
        println!("Generating custom index page with ID: {}", index_id);

        generate_custom_index_page(index_id, &notes_map, &id_to_path, &output_dir, &all_tags, &html_template)?;
        page_count += 1;


        generate_all_notes_page(&notes_map, &output_dir, &all_tags, &html_template)?;
        page_count += 1;
    } else {

        generate_index_page(&notes_map, &output_dir, &all_tags, &html_template)?;
        page_count += 1;
    }

    println!("Generating HTML for each note...");
    for (id, note) in &notes_map {
        println!("Generating HTML for note: {}", id);
        generate_html_for_note(id, &notes_map, &id_to_path, &output_dir, &all_tags, &html_template)?;
        page_count += 1;
    }

    println!("Generating tag pages...");
    for tag in &all_tags {
        println!("Generating page for tag: {}", tag);
        generate_tag_page(tag, &notes_map, &output_dir, &all_tags, &html_template)?;
        page_count += 1;
    }


    let elapsed = start_time.elapsed();
    let elapsed_ms = elapsed.as_millis();

    if elapsed_ms < 1000 {
        println!("✓ Built {} pages in {} ms", page_count, elapsed_ms);
    } else {
        let elapsed_sec = elapsed.as_secs_f64();
        println!("✓ Built {} pages in {:.2} s", page_count, elapsed_sec);
    }

    println!("Checking output directory:");
    match fs::read_dir(&output_dir) {
        Ok(entries) => {
            let entries: Vec<_> = entries.collect();
            if entries.is_empty() {
                println!("  Warning: Output directory is empty!");
            } else {
                for entry in entries {
                    if let Ok(entry) = entry {
                        println!("  - {:?}", entry.path());
                    }
                }
            }
        },
        Err(e) => println!("  Error reading output directory: {}", e),
    }

    println!("HTML generation complete. Output written to {:?}", output_dir);
    Ok(())
}

fn copy_directory(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(file_name);

        if entry_path.is_dir() {
            copy_directory(&entry_path, &dst_path)?;
        } else {
            fs::copy(&entry_path, &dst_path)?;
        }
    }

    Ok(())
}

fn find_and_copy_assets(dir: &Path, output_assets_dir: &Path) -> std::io::Result<()> {
    if dir.is_dir() {
        // Check if the current directory is named "assets"
        if dir.file_name().map_or(false, |name| name == "assets") {
            let rel_path = dir.file_name().unwrap();
            let target_dir = output_assets_dir;

            copy_directory(dir, target_dir)?;
            println!("Copied assets from {:?} to {:?}", dir, target_dir);
        }


        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                find_and_copy_assets(&path, output_assets_dir)?;
            }
        }
    }
    Ok(())
}

fn find_sy_files(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                find_sy_files(&path, files)?;
            } else if let Some(extension) = path.extension() {
                if extension == "sy" {
                    files.push(path);
                }
            }
        }
    }
    Ok(())
}

fn generate_custom_index_page(
    index_id: &str,
    notes_map: &HashMap<String, Note>,
    id_to_path: &HashMap<String, PathBuf>,
    output_dir: &Path,
    all_tags: &HashSet<String>,
    html_template: &str,
) -> std::io::Result<()> {
    let note = &notes_map[index_id];
    let title = if !note.Properties.title.is_empty() {
        note.Properties.title.clone()
    } else {
        "Notes Index".to_string()
    };

    // Extract creation date from ID if not present in properties
    let created_date = if !note.Properties.created.is_empty() {
        note.Properties.created.clone()
    } else if note.ID.len() >= 14 {
        // Extract timestamp from ID (format: YYYYMMDDhhmmss-xxx)
        let timestamp = &note.ID[0..14];
        timestamp.to_string()
    } else {
        String::new()
    };

    let mut html = html_template.replace("{{title}}", &title);
    html = html.replace("{{css_path}}", "styles.css");
    html = html.replace("{{site_name}}", "SyMark");

    // Extract a good description for meta and OpenGraph tags
    let mut description = String::new();
    let mut paragraph_count = 0;

    // Process blocks to find meaningful content
    for block in &note.Children {
        if block.Type == "NodeParagraph" || block.Type == "Paragraph" {
            // Only process first few paragraphs
            if paragraph_count > 0 {
                description.push(' '); // Add space between paragraphs
            }

            // Get text from paragraph
            let paragraph_content = render_blocks(&block.Children, notes_map, id_to_path);

            // Strip HTML
            let mut plain_text = String::new();
            let mut in_tag = false;

            for c in paragraph_content.chars() {
                if c == '<' {
                    in_tag = true;
                } else if c == '>' {
                    in_tag = false;
                } else if !in_tag {
                    plain_text.push(c);
                }
            }

            description.push_str(&plain_text);
            paragraph_count += 1;

            // Limit to 2-3 paragraphs for description
            if paragraph_count >= 2 && description.len() > 100 {
                break;
            }
        }
    }

    // If no paragraphs found, use title
    if description.is_empty() {
        description = title.clone();
    }

    // Escape HTML in the description
    let description = escape_html(&description);

    // Truncate description if too long (typical limit is around 200 chars)
    let truncated_description = if description.len() > 200 {
        // Ensure we cut at a character boundary
        let mut end_index = 0;
        let mut char_count = 0;
        for (idx, _) in description.char_indices() {
            end_index = idx;
            char_count += 1;
            if char_count >= 197 {
                break;
            }
        }
        format!("{}...", &description[0..end_index])
    } else {
        description
    };

    html = html.replace("{{meta_description}}", &truncated_description);
    html = html.replace("{{blog_description}}", "A collection of notes");
    html = html.replace("{{back_navigation}}", "");

    // Define site base URL for OpenGraph - this should be configurable in the future
    let site_base_url = "https://du82.github.io/symark";

    // OpenGraph URL
    html = html.replace("{{og_url}}", &format!("{}", site_base_url));

    // Set OpenGraph published time in ISO 8601 format
    if !created_date.is_empty() && created_date.len() >= 14 {
        let og_published_time = format!("{}-{}-{}T{}:{}:{}Z",
            &created_date[0..4], &created_date[4..6], &created_date[6..8],
            &created_date[8..10], &created_date[10..12], &created_date[12..14]);
        html = html.replace("{{og_published_time}}", &og_published_time);
    } else {

        let now = Local::now();
        let og_published_time = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        html = html.replace("{{og_published_time}}", &og_published_time);
    }


    if !note.Properties.updated.is_empty() && note.Properties.updated != created_date {
        if note.Properties.updated.len() >= 14 {
            let og_modified_time = format!("{}-{}-{}T{}:{}:{}Z",
                &note.Properties.updated[0..4], &note.Properties.updated[4..6], &note.Properties.updated[6..8],
                &note.Properties.updated[8..10], &note.Properties.updated[10..12], &note.Properties.updated[12..14]);
            html = html.replace("{{og_modified_time}}", &og_modified_time);
        } else {
            html = html.replace("{{og_modified_time}}", "");
        }
    } else {
        html = html.replace("{{og_modified_time}}", "");
    }

    // Add header image if it exists
    if !note.Properties.title_img.is_empty() {
        html = html.replace("{{#header_image}}", "");
        html = html.replace("{{/header_image}}", "");
        html = html.replace("{{header_image}}", &note.Properties.title_img);
    } else {
        // Remove header image section if no image
        html = html.replace("{{#header_image}}", "<!-- ");
        html = html.replace("{{/header_image}}", " -->");
    }
    html = html.replace("{{blog_description}}", "A collection of notes");
    html = html.replace("{{reading_time}}", "2");
    html = html.replace("{{author_name}}", "Notes Author");
    html = html.replace("{{publish_date}}", &naturalize_date(&created_date));


    let formatted_date = if !note.Properties.updated.is_empty() && note.Properties.updated != created_date {

        if note.Properties.updated.len() >= 14 {
            let og_modified_time = format!("{}-{}-{}T{}:{}:{}Z",
                &note.Properties.updated[0..4], &note.Properties.updated[4..6], &note.Properties.updated[6..8],
                &note.Properties.updated[8..10], &note.Properties.updated[10..12], &note.Properties.updated[12..14]);
            html = html.replace("{{og_modified_time}}", &og_modified_time);
        } else {

            html = html.replace("<meta property=\"article:modified_time\" content=\"{{og_modified_time}}\">", "");
        }

        format!("Created on {}, updated on {}",
            naturalize_date(&created_date),
            naturalize_date(&note.Properties.updated))
    } else {
        // If no image, remove the OpenGraph image tag
        html = html.replace("<meta property=\"og:image\" content=\"{{og_image}}\">", "");

        format!("Created on {}", naturalize_date(&created_date))
    };
    html = html.replace("{{last_updated_date}}", &formatted_date);

    html = html.replace("{{category}}", "Notes");
    html = html.replace("{{next_article_url}}", "#");
    html = html.replace("{{next_article_title}}", "");
    html = html.replace("{{back_navigation}}", "");

    let mut toc_items = Vec::new();
    let mut id_counter = 0;
    extract_toc_items(&note.Children, &mut toc_items, &mut id_counter);

    let toc_html = generate_toc_html(&toc_items);
    html = html.replace("{{table_of_contents}}", &toc_html);

    let content = render_blocks_with_ids(&note.Children, notes_map, id_to_path);

    let content_with_link = format!(
        "{}\n<div class=\"all-notes-link\"><a href=\"all.html\">View All Notes</a></div>",
        content
    );

    html = html.replace("{{content}}", &content_with_link);


    let mut meta = String::new();

    // Display creation date as a tag
    if !created_date.is_empty() {
        meta.push_str(&format!(
            "<span class=\"meta-tag date-tag\">Created: {}</span>",
            naturalize_date(&created_date)
        ));
    }

    // Display update date if it's different from creation date
    if !note.Properties.updated.is_empty() && note.Properties.updated != created_date {
        meta.push_str(&format!(
            "<span class=\"meta-tag date-tag\">Updated: {}</span>",
            naturalize_date(&note.Properties.updated)
        ));
    }

    // Add tags
    if !note.Properties.tags.is_empty() {
        let mut tags: Vec<_> = note.Properties.tags.split(',')
            .map(|t| t.trim())
            .filter(|t| *t != "index")
            .collect();
        tags.sort();

        for tag in tags {
            if !tag.is_empty() {
                meta.push_str(&format!(
                    "<a href=\"tag_{}.html\" class=\"meta-tag\">{}</a>",
                    tag.replace(" ", "_"),
                    tag
                ));
            }
        }
    }

    html = html.replace("{{note_meta}}", &meta);
    html = html.replace("{{generation_date}}", &Local::now().format("%Y-%m-%d %H:%M:%S").to_string());

    // Remove zero-width spaces and clean up any remaining template variables
    let cleaned_html = remove_zero_width_spaces(&html);
    let cleaned_html = cleanup_template_variables(&html);

    let final_html = comment_processor(&cleaned_html);

    // Write to file
    let file_path = output_dir.join("index.html");
    let mut file = File::create(&file_path)?;
    file.write_all(final_html.as_bytes())?;

    Ok(())
}


fn generate_all_notes_page(
    notes_map: &HashMap<String, Note>,
    output_dir: &Path,
    all_tags: &HashSet<String>,
    html_template: &str,
) -> std::io::Result<()> {
    let mut html = html_template.replace("{{title}}", "All Notes");
    html = html.replace("{{css_path}}", "styles.css");
    html = html.replace("{{site_name}}", "SyMark");
    html = html.replace("{{meta_description}}", "Collection of all notes");

    // Define site base URL for OpenGraph - this should be configurable in the future
    let site_base_url = "https://du82.github.io/symark";


    html = html.replace("{{og_url}}", &format!("{}", site_base_url));

    // Set OpenGraph published time in ISO 8601 format
    let now_iso = Local::now();
    let og_published_time = now_iso.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    html = html.replace("{{og_published_time}}", &og_published_time);

    // No modified time for all notes page
    html = html.replace("{{og_modified_time}}", "");

    // Remove OpenGraph image tag if no image
    html = html.replace("<meta property=\"og:image\" content=\"{{og_image}}\">", "");
    html = html.replace("{{blog_description}}", "A collection of all notes");
    html = html.replace("{{reading_time}}", "2");
    html = html.replace("{{author_name}}", "Notes Author");

    // Use current timestamp for publish date
    let now = Local::now().format("%Y%m%d%H%M%S").to_string();
    html = html.replace("{{publish_date}}", &naturalize_date(&now));

    let formatted_date = format!("Created on {}", naturalize_date(&now));
    html = html.replace("{{last_updated_date}}", &formatted_date);
    // Create tag cloud metadata for all notes page
    let meta = format!("<span class=\"meta-tag date-tag\">Created on {}</span>", naturalize_date(&now));
    html = html.replace("{{note_meta}}", &meta);
    html = html.replace("{{last_updated_date}}", ""); // Clear this as we're using note_meta

    html = html.replace("{{category}}", "Notes");
    html = html.replace("{{next_article_url}}", "#");
    html = html.replace("{{next_article_title}}", "");
    html = html.replace("{{back_navigation}}", BACK_NAVIGATION_HTML);

    // No header image for all notes page
    html = html.replace("{{#header_image}}", "<!-- ");
    html = html.replace("{{/header_image}}", " -->");

    // Generate navigation items - sort by title for better navigation
    let mut sorted_notes: Vec<_> = notes_map.values().filter(|n| !n.Properties.title.is_empty()).collect();
    sorted_notes.sort_by(|a, b| a.Properties.title.cmp(&b.Properties.title));

    let mut nav_items = String::new();
    for note in &sorted_notes {
        nav_items.push_str(&format!(
            "<li><a href=\"{}.html\">{}</a></li>\n",
            note.ID,
            note.Properties.title
        ));
    }

    // Generate table of contents
    let mut toc_items = Vec::new();
    let mut id_counter = 0;

    toc_items.push(TocItem {
        id: "section-all-notes".to_string(),
        text: "All Notes".to_string(),
        level: 2,
    });

    toc_items.push(TocItem {
        id: "section-tags".to_string(),
        text: "Tags".to_string(),
        level: 2,
    });

    let toc_html = generate_toc_html(&toc_items);
    html = html.replace("{{table_of_contents}}", &toc_html);

    // Generate tags section - sort alphabetically
    let mut tags: Vec<_> = all_tags.iter().collect();
    tags.sort();

    let mut tags_html = String::new();
    for tag in tags {
        tags_html.push_str(&format!(
            "<a href=\"tag_{}.html\" class=\"tag\">{}</a>\n",
            tag.replace(" ", "_"),
            tag
        ));
    }

    // Generate content
    let mut content = String::from("\n<ul>\n");
    for note in &sorted_notes {
        content.push_str(&format!(
            "<li><a href=\"{}.html\">{}</a></li>\n",
            note.ID,
            note.Properties.title
        ));
    }
    content.push_str("</ul>");

    content.push_str("<h2 id=\"section-tags\">Tags</h2>\n<div class=\"tags-container\">\n");
    content.push_str(&tags_html);
    content.push_str("</div>");

    html = html.replace("{{content}}", &content);

    // Set metadata
    html = html.replace("{{note_meta}}", "");
    html = html.replace("{{generation_date}}", &Local::now().format("%Y-%m-%d %H:%M:%S").to_string());

    // Remove zero-width spaces and clean up template variables before writing to file
    let cleaned_html = remove_zero_width_spaces(&html);
    let cleaned_html = cleanup_template_variables(&cleaned_html);

    let final_html = comment_processor(&cleaned_html);

    // Write to file - use all.html, not index.html to avoid overwriting the custom index
    let all_notes_path = output_dir.join("all.html");
    let mut file = File::create(&all_notes_path)?;
    file.write_all(final_html.as_bytes())?;

    Ok(())
}

fn generate_index_page(
    notes_map: &HashMap<String, Note>,
    output_dir: &Path,
    all_tags: &HashSet<String>,
    html_template: &str,
) -> std::io::Result<()> {
    let mut html = html_template.replace("{{title}}", "Notes Index");
    html = html.replace("{{css_path}}", "styles.css");
    html = html.replace("{{site_name}}", "SyMark");
    html = html.replace("{{meta_description}}", "Collection of all notes");
    html = html.replace("{{blog_description}}", "A collection of all notes");
    html = html.replace("{{back_navigation}}", "");

    // Define site base URL for OpenGraph - this should be configurable in the future
    let site_base_url = "https://du82.github.io/symark";

    // OpenGraph URL
    html = html.replace("{{og_url}}", &format!("{}", site_base_url));

    // Set OpenGraph published time in ISO 8601 format
    let now_iso = Local::now();
    let og_published_time = now_iso.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    html = html.replace("{{og_published_time}}", &og_published_time);

    // No modified time for index page
    html = html.replace("{{og_modified_time}}", "");

    // Remove OpenGraph image tag if no image
    html = html.replace("<meta property=\"og:image\" content=\"{{og_image}}\">", "");

    html = html.replace("{{#header_image}}", "<!-- ");
    html = html.replace("{{/header_image}}", " -->");
    html = html.replace("{{header_image}}", "");
    html = html.replace("{{blog_description}}", "A collection of all notes");
    html = html.replace("{{reading_time}}", "2");
    html = html.replace("{{author_name}}", "Notes Author");
    let now = Local::now().format("%Y%m%d%H%M%S").to_string();
    html = html.replace("{{publish_date}}", &naturalize_date(&now));

    let formatted_date = format!("Created on {}", naturalize_date(&now));
    html = html.replace("{{last_updated_date}}", &formatted_date);

    html = html.replace("{{category}}", "Notes");
    html = html.replace("{{next_article_url}}", "#");
    html = html.replace("{{next_article_title}}", "");
    html = html.replace("{{back_navigation}}", "");

    // Generate navigation items - sort by title for better navigation
    let mut sorted_notes: Vec<_> = notes_map.values().filter(|n| !n.Properties.title.is_empty()).collect();
    sorted_notes.sort_by(|a, b| a.Properties.title.cmp(&b.Properties.title));

    let mut nav_items = String::new();
    for note in &sorted_notes {
        nav_items.push_str(&format!(
            "<li><a href=\"{}.html\">{}</a></li>\n",
            note.ID,
            note.Properties.title
        ));
    }

    // Generate table of contents
    let mut toc_items = Vec::new();
    let mut id_counter = 0;

    toc_items.push(TocItem {
        id: "section-all-notes".to_string(),
        text: "All Notes".to_string(),
        level: 2,
    });

    toc_items.push(TocItem {
        id: "section-tags".to_string(),
        text: "Tags".to_string(),
        level: 2,
    });

    let toc_html = generate_toc_html(&toc_items);
    html = html.replace("{{table_of_contents}}", &toc_html);

    // Generate tags section - sort alphabetically
    let mut tags: Vec<_> = all_tags.iter().collect();
    tags.sort();

    let mut tags_html = String::new();
    for tag in tags {
        tags_html.push_str(&format!(
            "<a href=\"tag_{}.html\" class=\"tag\">{}</a>\n",
            tag.replace(" ", "_"),
            tag
        ));
    }

    // Generate content
    let mut content = String::from("<h2 id=\"section-all-notes\">All Notes</h2>\n<ul>\n");
    for note in &sorted_notes {
        content.push_str(&format!(
            "<li><a href=\"{}.html\">{}</a></li>\n",
            note.ID,
            note.Properties.title
        ));
    }
    content.push_str("</ul>");

    content.push_str("<h2 id=\"section-tags\">Tags</h2>\n<div class=\"tags-container\">\n");
    content.push_str(&tags_html);
    content.push_str("</div>");

    html = html.replace("{{content}}", &content);

    // Set metadata
    html = html.replace("{{note_meta}}", "");
    html = html.replace("{{generation_date}}", &Local::now().format("%Y-%m-%d %H:%M:%S").to_string());

    // Remove zero-width spaces and clean up template variables before writing to file
    let cleaned_html = remove_zero_width_spaces(&html);
    let cleaned_html = cleanup_template_variables(&cleaned_html);

    let final_html = comment_processor(&cleaned_html);

    // Write to file
    let all_notes_path = output_dir.join("all.html");
    let mut file = File::create(&all_notes_path)?;
    file.write_all(final_html.as_bytes())?;

    Ok(())
}

fn generate_tag_page(
    tag: &str,
    notes_map: &HashMap<String, Note>,
    output_dir: &Path,
    all_tags: &HashSet<String>,
    html_template: &str,
) -> std::io::Result<()> {
    // Count notes with this tag (we already calculated this above)

    let mut html = html_template.replace("{{title}}", &format!("Tag: {}", tag));
    html = html.replace("{{css_path}}", "styles.css");
    html = html.replace("{{site_name}}", "SyMark");
    // Filter notes with this tag for meta description and TOC
    let tagged_notes: Vec<&Note> = notes_map.values()
        .filter(|n| n.Properties.tags.split(',').any(|t| t.trim() == tag))
        .collect();
    let note_count = tagged_notes.len();

    let meta_description = if note_count == 1 {
        format!("1 note has the tag \"{}\"", tag)
    } else {
        format!("{} notes have the tag \"{}\"", note_count, tag)
    };

    html = html.replace("{{meta_description}}", &meta_description);
    html = html.replace("{{blog_description}}", &meta_description);
    html = html.replace("{{reading_time}}", "2");
    html = html.replace("{{author_name}}", "Notes Author");

    // Get current date for tag page generation
    let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();

    // Create tag cloud metadata for tag page
    let meta = format!("<span class=\"meta-tag date-tag\">Created on {}</span>", naturalize_date(&timestamp));
    html = html.replace("{{note_meta}}", &meta);
    html = html.replace("{{last_updated_date}}", ""); // Clear this as we're using note_meta

    html = html.replace("{{category}}", "Tags");
    html = html.replace("{{next_article_url}}", "#");
    html = html.replace("{{next_article_title}}", "");
    html = html.replace("{{back_navigation}}", BACK_NAVIGATION_HTML);

    // No header image for tag pages
    html = html.replace("{{#header_image}}", "<!-- ");
    html = html.replace("{{/header_image}}", " -->");

    // Generate navigation items - sort by title
    let mut sorted_notes: Vec<_> = notes_map.values().filter(|n| !n.Properties.title.is_empty()).collect();
    sorted_notes.sort_by(|a, b| a.Properties.title.cmp(&b.Properties.title));

    // Generate table of contents
    let mut toc_items = Vec::new();

    toc_items.push(TocItem {
        id: "section-tagged-notes".to_string(),
        text: if tagged_notes.len() == 1 {
            format!("1 note has the tag \"{}\"", tag)
        } else {
            format!("{} notes have the tag \"{}\"", tagged_notes.len(), tag)
        },
        level: 2,
    });

    toc_items.push(TocItem {
        id: "section-all-tags".to_string(),
        text: "All Tags".to_string(),
        level: 2,
    });

    let toc_html = generate_toc_html(&toc_items);
    html = html.replace("{{table_of_contents}}", &toc_html);

    // Generate tags section - sort alphabetically
    let mut tags: Vec<_> = all_tags.iter().collect();
    tags.sort();

    let mut tags_html = String::new();
    for t in tags {
        let class = if t == tag { "tag active" } else { "tag" };

        // Count notes with this tag
        let tag_notes: Vec<&Note> = notes_map.values()
            .filter(|n| n.Properties.tags.split(',').any(|tag| tag.trim() == *t))
            .collect();
        let tag_count = tag_notes.len();

        let mut tooltip_text = if tag_count == 1 {
            format!("1 note has the tag \"{}\"", t)
        } else {
            format!("{} notes have the tag \"{}\"", tag_count, t)
        };

        // Add titles of up to 3 notes in the tooltip
        if !tag_notes.is_empty() {
            tooltip_text.push_str(": ");
            let note_titles: Vec<String> = tag_notes.iter()
                .take(3)
                .map(|n| format!("\"{}\"", n.Properties.title))
                .collect();

            tooltip_text.push_str(&note_titles.join(", "));

            if tag_count > 3 {
                tooltip_text.push_str(", ...");
            }
        }

        tags_html.push_str(&format!(
            "<span class=\"tooltip\"><a href=\"tag_{}.html\" class=\"{}\">{}</a><span class=\"right bottom\"><span class=\"tooltip-excerpt\">{}</span><i></i></span></span>\n",
            t.replace(" ", "_"),
            class,
            t,
            tooltip_text
        ));
    }

    // Process notes with this tag
    let mut content = format!("<ul>");
    for note in &tagged_notes {
        // Get excerpt from note content for tooltip
        let excerpt = {
            // First, look for the first paragraph with actual content
            let mut content_text = String::new();

            // Process blocks to find meaningful content
            for block in &note.Children {
                // Find the first paragraph with actual content
                if block.Type == "P" && !block.Data.is_empty() {
                    content_text = escape_html(&block.Data);

                    // If it's a short paragraph, try to get more content
                    if content_text.len() < 120 && note.Children.len() > 1 {
                        // Look for a second paragraph
                        for second_block in &note.Children {
                            if second_block.ID != block.ID && second_block.Type == "P" && !second_block.Data.is_empty() {
                                content_text.push_str(" ");
                                content_text.push_str(&escape_html(&second_block.Data));
                                break;
                            }
                        }
                    }

                    break;
                }
            }

            // If we didn't find a paragraph, look for any text in the first few blocks
            if content_text.is_empty() {
                for block in note.Children.iter().take(3) {
                    if !block.Data.is_empty() {
                        content_text = escape_html(&block.Data);
                        break;
                    }

                    // Check children if this block has no direct content
                    if block.Children.len() > 0 {
                        for child in &block.Children {
                            if !child.Data.is_empty() {
                                content_text = escape_html(&child.Data);
                                break;
                            }
                        }
                        if !content_text.is_empty() {
                            break;
                        }
                    }
                }
            }

            // Limit to ~200 chars and add ellipsis
            if content_text.len() > 200 {
                let mut truncate_pos = 200;
                // Find a good breakpoint (space or punctuation)
                while truncate_pos > 150 {
                    if content_text.chars().nth(truncate_pos).map_or(false, |c| c == ' ' || c == '.' || c == ',' || c == ';') {
                        break;
                    }
                    truncate_pos -= 1;
                }
                content_text.truncate(truncate_pos);
                content_text.push_str("...");
            }

            if !content_text.is_empty() {
                content_text
            } else {
                // If no content found, use tags as fallback
                let tags = note.Properties.tags.split(',')
                    .map(|t| t.trim())
                    .filter(|t| !t.is_empty())
                    .collect::<Vec<_>>();

                if !tags.is_empty() {
                    format!("Tagged with: {}", tags.join(", "))
                } else {
                    // Last resort: use title
                    note.Properties.title.clone()
                }
            }
        };

        // Create tooltip with title and excerpt
        content.push_str(&format!(
            "<li><span class=\"tooltip\"><a href=\"{}.html\">{}</a><span class=\"right bottom\"><span class=\"tooltip-title\">{}</span><span class=\"tooltip-excerpt\">{}</span><i></i></span></span>\n",
            note.ID,
            note.Properties.title,
            note.Properties.title,
            excerpt
        ));
    }

    content.push_str("</ul>");

    content.push_str("<h2 id=\"section-all-tags\">All Tags</h2>\n<div class=\"tags-container\">\n");
    content.push_str(&tags_html);
    content.push_str("</div>");

    html = html.replace("{{content}}", &content);

    // Set metadata
    html = html.replace("{{note_meta}}", "");
    html = html.replace("{{generation_date}}", &Local::now().format("%Y-%m-%d %H:%M:%S").to_string());

    // Remove zero-width spaces and cleanup template variables before writing to file
    let cleaned_html = remove_zero_width_spaces(&html);
    let cleaned_html = cleanup_template_variables(&cleaned_html);

    let final_html = comment_processor(&cleaned_html);

    // Write to file
    let file_path = output_dir.join(format!("tag_{}.html", tag.replace(" ", "_")));
    let mut file = File::create(&file_path)?;
    file.write_all(final_html.as_bytes())?;

    Ok(())
}

fn generate_html_for_note(
    id: &str,
    notes_map: &HashMap<String, Note>,
    id_to_path: &HashMap<String, PathBuf>,
    output_dir: &Path,
    all_tags: &HashSet<String>,
    html_template: &str,
) -> std::io::Result<()> {
    println!("Generating HTML for note ID: {}", id);
    let note = &notes_map[id];
    let title = if !note.Properties.title.is_empty() {
        note.Properties.title.clone()
    } else {
        id.to_string()
    };

    // Extract creation date from ID if not present in properties
    let created_date = if !note.Properties.created.is_empty() {
        note.Properties.created.clone()
    } else if note.ID.len() >= 14 {
        // Extract timestamp from ID (format: YYYYMMDDhhmmss-xxx)
        let timestamp = &note.ID[0..14];
        timestamp.to_string()
    } else {
        String::new()
    };

    let mut html = html_template.replace("{{title}}", &title);
    html = html.replace("{{css_path}}", "styles.css");
    html = html.replace("{{site_name}}", "SyMark");

    // Extract a good description for meta and OpenGraph tags
    // Similar to how tooltip excerpts are generated
    let mut description = String::new();
    let mut paragraph_count = 0;

    // Process blocks to find meaningful content
    for block in &note.Children {
        if block.Type == "NodeParagraph" || block.Type == "Paragraph" {
            // Only process first few paragraphs
            if paragraph_count > 0 {
                description.push(' '); // Add space between paragraphs
            }

            // Get text from paragraph
            let paragraph_content = render_blocks(&block.Children, notes_map, id_to_path);

            // Strip HTML
            let mut plain_text = String::new();
            let mut in_tag = false;

            for c in paragraph_content.chars() {
                if c == '<' {
                    in_tag = true;
                } else if c == '>' {
                    in_tag = false;
                } else if !in_tag {
                    plain_text.push(c);
                }
            }

            description.push_str(&plain_text);
            paragraph_count += 1;

            // Limit to 2-3 paragraphs for description
            if paragraph_count >= 2 && description.len() > 100 {
                break;
            }
        }
    }

    // If no paragraphs found, use title
    if description.is_empty() {
        description = title.clone();
    }

    // Escape HTML in the description
    let description = escape_html(&description);

    // Truncate description if too long (typical limit is around 200 chars)
    let truncated_description = if description.len() > 200 {
        // Ensure we cut at a character boundary
        let mut end_index = 0;
        let mut char_count = 0;
        for (idx, _) in description.char_indices() {
            end_index = idx;
            char_count += 1;
            if char_count >= 197 {
                break;
            }
        }
        format!("{}...", &description[0..end_index])
    } else {
        description
    };

    html = html.replace("{{meta_description}}", &truncated_description);
    html = html.replace("{{blog_description}}", "A collection of notes");
    html = html.replace("{{back_navigation}}", BACK_NAVIGATION_HTML);

    // Define site base URL - this should be configurable in the future
    let site_base_url = "https://du82.github.io/symark";

    // OpenGraph URL - will use the same title and description from meta tags
    // For the index page, use a different URL format
    if id == "index" {
        html = html.replace("{{og_url}}", &format!("{}", site_base_url));
    } else {
        html = html.replace("{{og_url}}", &format!("{}{}.html", site_base_url, id));
    }

    // Handle OpenGraph image if there's a header image
    if !note.Properties.title_img.is_empty() {
        // Extract URL from title_img CSS string
        let img_url = note.Properties.title_img.clone();
        // If the image is a background-image CSS property
        if img_url.contains("background-image") {
            let start = img_url.find("url(");
            let end = img_url.find(")");
            if let (Some(s), Some(e)) = (start, end) {
                let image_path = &img_url[(s + 4)..(e)].trim_matches('"').trim_matches('\'');
                let og_image = format!("{}{}", site_base_url, image_path);
                // Replace the og:image tag content
                html = html.replace("{{og_image}}", &og_image);
            } else {
                // If we couldn't parse the image URL, remove the tag
                html = html.replace("<meta property=\"og:image\" content=\"{{og_image}}\">", "");
            }
        } else {
            // Not a background-image CSS property
            html = html.replace("<meta property=\"og:image\" content=\"{{og_image}}\">", "");
        }
    } else {
        // If no image, remove the OpenGraph image tag
        html = html.replace("<meta property=\"og:image\" content=\"{{og_image}}\">", "");
    }

    // This section intentionally left empty as the OpenGraph published time
    // is now handled later in the function

    // Add header image if it exists
    if !note.Properties.title_img.is_empty() {
        html = html.replace("{{#header_image}}", "");
        html = html.replace("{{/header_image}}", "");
        html = html.replace("{{header_image}}", &note.Properties.title_img);
    } else {
        // Remove header image section if no image
        html = html.replace("{{#header_image}}", "<!-- ");
        html = html.replace("{{/header_image}}", " -->");
    }

    // Reading time will be calculated by JavaScript
    html = html.replace("{{reading_time}}", "");

    html = html.replace("{{author_name}}", "Notes Author");

    // Use created date for publish_date
    html = html.replace("{{publish_date}}", &naturalize_date(&created_date));

    // Set OpenGraph published time in ISO 8601 format
    if !created_date.is_empty() && created_date.len() >= 14 {
        let og_published_time = format!("{}-{}-{}T{}:{}:{}Z",
            &created_date[0..4], &created_date[4..6], &created_date[6..8],
            &created_date[8..10], &created_date[10..12], &created_date[12..14]);
        html = html.replace("{{og_published_time}}", &og_published_time);
    } else {
        // For index or other pages without a creation date, use current time
        let now = Local::now();
        let og_published_time = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        html = html.replace("{{og_published_time}}", &og_published_time);
    }

    // Format conditional date string
    let formatted_date = if !note.Properties.updated.is_empty() && note.Properties.updated != created_date {
        // Add OpenGraph modified time in ISO 8601 format
        if note.Properties.updated.len() >= 14 {
            let og_modified_time = format!("{}-{}-{}T{}:{}:{}Z",
                &note.Properties.updated[0..4], &note.Properties.updated[4..6], &note.Properties.updated[6..8],
                &note.Properties.updated[8..10], &note.Properties.updated[10..12], &note.Properties.updated[12..14]);
            html = html.replace("{{#og_modified_time}}", "");
            html = html.replace("{{/og_modified_time}}", "");
            html = html.replace("{{og_modified_time}}", &og_modified_time);
        }

        format!("Created on {}, updated on {}",
            naturalize_date(&created_date),
            naturalize_date(&note.Properties.updated))
    } else {
        format!("Created on {}", naturalize_date(&created_date))
    };
    html = html.replace("{{last_updated_date}}", &formatted_date);

    html = html.replace("{{category}}", &note.Properties.note_type);
    html = html.replace("{{next_article_url}}", "#");
    html = html.replace("{{next_article_title}}", "");

    // Generate navigation items
    let mut sorted_notes: Vec<_> = notes_map.values().filter(|n| !n.Properties.title.is_empty()).collect();
    sorted_notes.sort_by(|a, b| a.Properties.title.cmp(&b.Properties.title));

    // Generate table of contents by extracting headings from the note content
    let mut toc_items = Vec::new();
    let mut id_counter = 0;
    extract_toc_items(&note.Children, &mut toc_items, &mut id_counter);

    let toc_html = generate_toc_html(&toc_items);
    html = html.replace("{{table_of_contents}}", &toc_html);

    // Generate tags section
    let mut tags: Vec<_> = all_tags.iter().collect();
    tags.sort();

    // Generate content with heading IDs for TOC
    let content = render_blocks_with_ids(&note.Children, notes_map, id_to_path);
    html = html.replace("{{content}}", &content);

    // Already handled OpenGraph URL earlier

    // Generate note metadata as a tag cloud
    let mut meta = String::new();

    // Display creation date as a tag
    if !created_date.is_empty() {
        meta.push_str(&format!(
            "<span class=\"meta-tag date-tag\">Created: {}</span>",
            naturalize_date(&created_date)
        ));
    }

    // Display update date if it's different from creation date
    if !note.Properties.updated.is_empty() && note.Properties.updated != created_date {
        meta.push_str(&format!(
            "<span class=\"meta-tag date-tag\">Updated: {}</span>",
            naturalize_date(&note.Properties.updated)
        ));
    }

    // Add tags
    if !note.Properties.tags.is_empty() {
        let mut tags: Vec<_> = note.Properties.tags.split(',').map(|t| t.trim()).collect();
        tags.sort();

        // Process tags for both display and OpenGraph
        let mut og_tags_html = String::new();

        for tag in &tags {
            if !tag.is_empty() {
                meta.push_str(&format!(
                    "<a href=\"tag_{}.html\" class=\"meta-tag\">{}</a>",
                    tag.replace(" ", "_"),
                    tag
                ));

                // Add tag to OpenGraph tags
                og_tags_html.push_str(&format!(
                    "<meta property=\"article:tag\" content=\"{}\">",
                    tag
                ));
            }
        }

        // Replace OpenGraph tags placeholder
        if !og_tags_html.is_empty() {
            html = html.replace("{{#og_tags}}", "");
            html = html.replace("{{/og_tags}}", "");
            html = html.replace("<meta property=\"article:tag\" content=\"{{.}}\">", &og_tags_html);
        }
    } else {
        // Remove OpenGraph tags section if no tags
        html = html.replace("{{#og_tags}}", "<!-- ");
        html = html.replace("{{/og_tags}}", " -->");
    }

    // Generate absolute URL for OpenGraph URL
    html = html.replace("{{og_url}}", &format!("{}{}.html", site_base_url, id));

    // TODO: Add configuration option for site_base_url
    // In the future, this should be read from a config file and used consistently
    // for all OpenGraph URLs and image paths

    html = html.replace("{{note_meta}}", &meta);

    // Set generation date
    html = html.replace("{{generation_date}}", &Local::now().format("%Y-%m-%d %H:%M:%S").to_string());

    // All template variables should be handled appropriately above

    // Remove zero-width spaces and clean up any remaining template variables
    let cleaned_html = remove_zero_width_spaces(&html);
    let cleaned_html = cleanup_template_variables(&cleaned_html);

    let final_html = comment_processor(&cleaned_html);

    // Write to file
    let file_path = output_dir.join(format!("{}.html", id));
    println!("Writing HTML to file: {:?}", file_path);
    let mut file = File::create(&file_path)?;
    file.write_all(final_html.as_bytes())?;

    Ok(())
}

fn render_blocks_with_ids(blocks: &[Block], notes_map: &HashMap<String, Note>, id_to_path: &HashMap<String, PathBuf>) -> String {
    let mut id_counter = 0;
    let mut html = String::new();

    for block in blocks {
        match block.Type.as_str() {
            "NodeHeading" => {
                let level = block.HeadingLevel.max(1).min(6);

                // Generate an ID for the heading if it doesn't have one
                let id = if !block.ID.is_empty() {
                    block.ID.clone()
                } else {
                    let id = format!("heading-{}", id_counter);
                    id_counter += 1;
                    id
                };

                html.push_str(&format!("<h{} id=\"{}\">", level, id));

                // Render the heading content
                for child in &block.Children {
                    if child.Type == "NodeText" {
                        html.push_str(&escape_html(&child.Data));
                    } else if child.Type == "NodeTextMark" {
                        html.push_str(&render_text_mark(child, notes_map, id_to_path));
                    } else {
                        html.push_str(&render_block(child, notes_map, id_to_path));
                    }
                }

                html.push_str(&format!("</h{}>\n", level));
            },
            // For other block types, use the regular render_block function
            _ => {
                html.push_str(&render_block(block, notes_map, id_to_path));
            }
        }
    }

    html
}

fn render_blocks(
    blocks: &[Block],
    notes_map: &HashMap<String, Note>,
    id_to_path: &HashMap<String, PathBuf>,
) -> String {
    let mut html = String::new();

    for block in blocks {
        match block.Type.as_str() {

            "NodeSuperBlock" => {
                let layout_type = if let Some(layout_marker) = block.Children.iter().find(|child| child.Type == "NodeSuperBlockLayoutMarker") {
                    layout_marker.Data.as_str()
                } else {
                    "row" // Default to column layout
                };
                let layout_type = if layout_type == "row" { "col" } else { "row" };

                let id_attr = if !block.ID.is_empty() {
                    format!(" id=\"{}\"", block.ID)
                } else {
                    String::new()
                };
                html.push_str(&format!("<div{} class=\"superblock superblock-{}\">\n", id_attr, layout_type));

                let content_blocks: Vec<&Block> = block.Children.iter()
                    .filter(|child|
                        child.Type != "NodeSuperBlockOpenMarker" &&
                        child.Type != "NodeSuperBlockLayoutMarker" &&
                        child.Type != "NodeSuperBlockCloseMarker")
                    .collect();

                let nested_superblocks: Vec<&Block> = content_blocks.iter()
                    .filter(|b| b.Type == "NodeSuperBlock")
                    .cloned()
                    .collect();

                if layout_type == "row" && !nested_superblocks.is_empty() {
                    for nested in &nested_superblocks {
                        html.push_str(&render_block(nested, notes_map, id_to_path));
                    }

                    let other_blocks: Vec<&Block> = content_blocks.iter()
                        .filter(|b| b.Type != "NodeSuperBlock")
                        .cloned()
                        .collect();

                    if !other_blocks.is_empty() {
                        // If there are blocks that aren't in a nested superblock,
                        // wrap them in a column
                        html.push_str("<div class=\"superblock superblock-col\">\n");
                        for block in &other_blocks {
                            html.push_str(&render_block(block, notes_map, id_to_path));
                        }
                        html.push_str("</div>\n");
                    }
                } else {
                    // For non-row layouts or rows without nested superblocks,
                    // we need to organize content
                    if layout_type == "row" {
                        // For rows without nested superblocks, we should create columns
                        // Group related blocks (e.g. heading + paragraph)
                        if !content_blocks.is_empty() {
                            html.push_str("<div class=\"superblock superblock-col\">\n");
                            for block in &content_blocks {
                                html.push_str(&render_block(block, notes_map, id_to_path));
                            }
                            html.push_str("</div>\n");
                        }
                    } else {
                        // For column layouts, render blocks directly
                        for block in &content_blocks {
                            html.push_str(&render_block(block, notes_map, id_to_path));
                        }
                    }
                }

                html.push_str("</div>\n");
            },
            "NodeParagraph" => {
                let mut class_attr = String::new();
                let mut style_attr = String::new();

                // Check if paragraph has special styling
                if !block.Properties.style.is_empty() {
                    if let Some(class_name) = get_style_class(&block.Properties.style, false) {
                        class_attr = format!(" class=\"{}\"", class_name);
                    } else {
                        // Keep the original style if no special class is applied
                        style_attr = format!(" style=\"{}\"", block.Properties.style);
                    }
                }

                // Check if this paragraph contains only an image
                let contains_only_image = block.Children.len() == 1 &&
                    block.Children[0].Type == "NodeImage";

                // Always output the paragraph with its styling, even for images
                // This allows for centered or aligned images through paragraph styling
                let id_attr = if !block.ID.is_empty() {
                    format!(" id=\"{}\"", block.ID)
                } else {
                    String::new()
                };
                html.push_str(&format!("<p{}{}{}>", id_attr, class_attr, style_attr));
                html.push_str(&render_blocks(&block.Children, notes_map, id_to_path));
                html.push_str("</p>\n");
            },
            "NodeHeading" => {
                let level = block.HeadingLevel.max(1).min(6);
                let id = if !block.ID.is_empty() {
                    format!(" id=\"{}\"", block.ID)
                } else {
                    String::new()
                };
                html.push_str(&format!("<h{}{}>", level, id));
                html.push_str(&render_blocks(&block.Children, notes_map, id_to_path));
                html.push_str(&format!("</h{}>\n", level));
            },
            "NodeList" => {
                // Determine if ordered or unordered list
                let list_type = if let serde_json::Value::Object(map) = &block.ListData {
                    if map.contains_key("Typ") {
                        // Check for task list (Typ == 3)
                        if let Some(serde_json::Value::Number(num)) = map.get("Typ") {
                            if num.as_i64() == Some(3) {
                                "task"
                            } else if num.as_i64() == Some(1) {
                                "ordered"
                            } else {
                                "unordered"
                            }
                        } else {
                            "unordered"
                        }
                    } else {
                        "unordered"
                    }
                } else {
                    "unordered"
                };

                let id_attr = if !block.ID.is_empty() {
                    format!(" id=\"{}\"", block.ID)
                } else {
                    String::new()
                };

                match list_type {
                    "ordered" => html.push_str(&format!("<ol{}>\n", id_attr)),
                    _ => html.push_str(&format!("<ul{}>\n", id_attr)),
                }

                html.push_str(&render_blocks(&block.Children, notes_map, id_to_path));

                match list_type {
                    "ordered" => html.push_str("</ol>\n"),
                    _ => html.push_str("</ul>\n"),
                }
            },
            "NodeListItem" => {
                // Check if this is a task list item
                let id_attr = if !block.ID.is_empty() {
                    format!(" id=\"{}\"", block.ID)
                } else {
                    String::new()
                };

                if block.Children.iter().any(|child| child.Type == "NodeTaskListItemMarker") {
                    // Make the list item properly positioned
                    html.push_str(&format!("<li{} style=\"position: relative; padding-left: 30px; margin-bottom: 12px; list-style: none; \">", id_attr));

                    // Check if the task is checked or unchecked
                    if let Some(task_marker_block) = block.Children.iter().find(|child| child.Type == "NodeTaskListItemMarker") {
                        if task_marker_block.TaskListItemChecked {
                            // Checked item
                            html.push_str("<span class=\"task-checkbox-checked\" style=\"position: absolute; left: 0; top: 2px; display: inline-block; width: 20px; height: 20px; border: 2px solid #bdc3c7; background-color: #3498db; border-color: #3498db; border-radius: 2px;\"></span>");
                            html.push_str("<span class=\"task-complete\" style=\"text-decoration: line-through; color: #7f8c8d;\">");

                            // Filter out the marker from rendering
                            for child in &block.Children {
                                if child.Type != "NodeTaskListItemMarker" {
                                    html.push_str(&render_block(child, notes_map, id_to_path));
                                }
                            }
                            html.push_str("</span>");
                        } else {
                            html.push_str("<span class=\"task-checkbox-unchecked\" style=\"position: absolute; left: 0; top: 2px; display: inline-block; width: 20px; height: 20px; border: 2px solid #bdc3c7; background-color: #ecf0f1; border-radius: 2px;\"></span>");

                            for child in &block.Children {
                                if child.Type != "NodeTaskListItemMarker" {
                                    html.push_str(&render_block(child, notes_map, id_to_path));
                                }
                            }
                        }
                    }
                } else {
                    html.push_str(&format!("<li{}>", id_attr));

                    let mut content = String::new();
                    let mut last_was_paragraph = false;

                    for child in &block.Children {
                        if child.Type == "NodeParagraph" {
                            if last_was_paragraph {
                                content.push_str(&render_blocks(&child.Children, notes_map, id_to_path));
                            } else {
                                content.push_str(&render_block(child, notes_map, id_to_path));
                                last_was_paragraph = true;
                            }
                        } else {
                            content.push_str(&render_block(child, notes_map, id_to_path));
                            last_was_paragraph = false;
                        }
                    }

                    html.push_str(&content);
                }
                html.push_str("</li>\n");
            },
            "NodeTaskListItemMarker" => {
                // Skip rendering
            },
            "NodeBlockquote" => {
                let id_attr = if !block.ID.is_empty() {
                    format!(" id=\"{}\"", block.ID)
                } else {
                    String::new()
                };
                html.push_str(&format!("<blockquote{}>", id_attr));
                html.push_str(&render_blocks(&block.Children, notes_map, id_to_path));
                html.push_str("</blockquote>\n");
            },
            "NodeThematicBreak" => {
                let id_attr = if !block.ID.is_empty() {
                    format!(" id=\"{}\"", block.ID)
                } else {
                    String::new()
                };
                html.push_str(&format!("<hr{}>\n", id_attr));
            },
            "NodeTable" => {
                let id_attr = if !block.ID.is_empty() {
                    format!(" id=\"{}\"", block.ID)
                } else {
                    String::new()
                };
                html.push_str(&format!("<table{}>\n", id_attr));
                html.push_str(&render_blocks(&block.Children, notes_map, id_to_path));
                html.push_str("</table>\n");
            },
            "NodeTableHead" => {
                let id_attr = if !block.ID.is_empty() {
                    format!(" id=\"{}\"", block.ID)
                } else {
                    String::new()
                };
                html.push_str(&format!("<thead{}>\n", id_attr));
                html.push_str(&render_blocks(&block.Children, notes_map, id_to_path));
                html.push_str("</thead>\n");
            },
            "NodeTableRow" => {
                let id_attr = if !block.ID.is_empty() {
                    format!(" id=\"{}\"", block.ID)
                } else {
                    String::new()
                };
                html.push_str(&format!("<tr{}>\n", id_attr));
                html.push_str(&render_blocks(&block.Children, notes_map, id_to_path));
                html.push_str("</tr>\n");
            },
            "NodeTableCell" => {
                let id_attr = if !block.ID.is_empty() {
                    format!(" id=\"{}\"", block.ID)
                } else {
                    String::new()
                };
                if block.Data == "th" {
                    html.push_str(&format!("<th{}>", id_attr));
                    html.push_str(&render_blocks(&block.Children, notes_map, id_to_path));
                    html.push_str("</th>\n");
                } else {
                    html.push_str(&format!("<td{}>", id_attr));
                    html.push_str(&render_blocks(&block.Children, notes_map, id_to_path));
                    html.push_str("</td>\n");
                }
            },
            "NodeCodeBlock" => {
                let id_attr = if !block.ID.is_empty() {
                    format!(" id=\"{}\"", block.ID)
                } else {
                    String::new()
                };
                html.push_str(&format!("<pre{}><code", id_attr));

                // Add language class if available
                if !block.CodeBlockInfo.is_empty() {
                    // CodeBlockInfo might be base64 encoded
                    if let Ok(lang) = decode(&block.CodeBlockInfo) {
                        if let Ok(lang_str) = String::from_utf8(lang) {
                            html.push_str(&format!(" class=\"language-{}\"", lang_str));
                        }
                    } else {
                        html.push_str(&format!(" class=\"language-{}\"", block.CodeBlockInfo));
                    }
                }

                html.push_str(">");

                // Find the code content in children
                for child in &block.Children {
                    if child.Type == "NodeCodeBlockCode" {
                        html.push_str(&escape_html(&child.Data));
                    }
                }

                html.push_str("</code></pre>\n");
            },
            "NodeText" => {
                // For text nodes, we generally don't add IDs as they're inline elements,
                // but we can wrap them in a span with an ID if needed
                if !block.ID.is_empty() {
                    html.push_str(&format!("<span id=\"{}\">", block.ID));
                    html.push_str(&escape_html(&block.Data));
                    html.push_str("</span>");
                } else {
                    html.push_str(&escape_html(&block.Data));
                }
            },
            "NodeTextMark" => {
                // Update this line to pass all required arguments
                html.push_str(&render_text_mark(block, notes_map, id_to_path));
            },
            "NodeImage" => {
                // Handle image nodes
                let mut image_src = String::new();
                let mut alt_text = String::new();
                let mut style_attr = String::new();
                let mut parent_style_attr = String::new();
                let id_attr = if !block.ID.is_empty() {
                    format!(" id=\"{}\"", block.ID)
                } else {
                    String::new()
                };

                // Find the link destination in children
                for child in &block.Children {
                    if child.Type == "NodeLinkDest" {
                        image_src = child.Data.clone();
                    } else if child.Type == "NodeLinkText" {
                        alt_text = child.Data.clone();
                    }
                }

                // Check if there are style properties for the image
                if !block.Properties.style.is_empty() {
                    style_attr = format!(" style=\"{}\"", block.Properties.style);
                }

                // Check if there's a parent-style attribute
                if let Some(parent_style) = block.Properties.parent_style.as_ref() {
                    if !parent_style.is_empty() {
                        parent_style_attr = format!(" style=\"{}\"", parent_style);
                    }
                }

                if !image_src.is_empty() {
                    // If parent styling is present, wrap the image in a div with that styling
                    if !parent_style_attr.is_empty() {
                        let wrapper_id = if !block.ID.is_empty() {
                            // If we have an ID, use it for the wrapper instead of the img
                            format!(" id=\"{}\"", block.ID)
                        } else {
                            String::new()
                        };
                        html.push_str(&format!("<div{}{}>", wrapper_id, parent_style_attr));

                        // In this case, don't add the ID to the img tag since it's on the wrapper
                        html.push_str(&format!(
                            "<img src=\"{}\" alt=\"{}\"{}/>",
                            image_src,
                            alt_text,
                            style_attr
                        ));
                    } else {
                        // No wrapper, add ID directly to the img tag
                        html.push_str(&format!(
                            "<img{} src=\"{}\" alt=\"{}\"{}/>",
                            id_attr,
                            image_src,
                            alt_text,
                            style_attr
                        ));
                    }

                    // Close the parent div if it was opened
                    if !parent_style_attr.is_empty() {
                        html.push_str("</div>");
                    }
                }
            },
            "NodeBr" => {
                let id_attr = if !block.ID.is_empty() {
                    format!(" id=\"{}\"", block.ID)
                } else {
                    String::new()
                };
                html.push_str(&format!("<br{}>", id_attr));
            },
            "NodeBlockQueryEmbed" => {
                // Process block query embed (transclusion)
                // First, find the NodeBlockQueryEmbedScript child that contains the query
                if let Some(script_block) = block.Children.iter().find(|child| child.Type == "NodeBlockQueryEmbedScript") {
                    // Extract the block ID from the query
                    // The query format is typically: "select * from blocks where id='BLOCK_ID'"
                    if let Some(id_start) = script_block.Data.find("id='") {
                        let id_start = id_start + 4; // Skip "id='"
                        if let Some(id_end) = script_block.Data[id_start..].find('\'') {
                            let content_id = &script_block.Data[id_start..id_start + id_end];

                            // Create a div wrapper for the transcluded content
                            let wrapper_id = if !block.ID.is_empty() {
                                format!(" id=\"{}\"", block.ID)
                            } else {
                                String::new()
                            };
                            html.push_str(&format!("<div{} class=\"transcluded-block\">", wrapper_id));

                            // Add source link button
                            let source_url = if notes_map.contains_key(content_id) {
                                format!("{}.html", content_id) // Link to the note
                            } else {
                                // For block IDs, try to find which note contains it
                                let mut source_note_id = String::new();
                                for (note_id, note) in notes_map.iter() {
                                    if find_block_by_id(content_id, &note.Children).is_some() {
                                        source_note_id = note_id.clone();
                                        break;
                                    }
                                }
                                format!("{}.html#{}", source_note_id, content_id) // Link to the note with block ID anchor
                            };

                            html.push_str(&format!("<a href=\"{}\" class=\"source-link\">Go to source</a>", source_url));

                            // Check if this is a block ID or a note ID
                            let mut found = false;

                            // First try to find the specific block by ID
                            for note in notes_map.values() {
                                if let Some(found_block) = find_block_by_id(content_id, &note.Children) {
                                    html.push_str(&render_block(found_block, notes_map, id_to_path));
                                    found = true;
                                    break;
                                }
                            }

                            // If not found as a block, check if it's a note ID
                            if !found {
                                if let Some(note) = notes_map.get(content_id) {
                                    // Render all blocks from the note
                                    html.push_str(&render_blocks(&note.Children, notes_map, id_to_path));
                                    found = true;
                                }
                            }

                            if !found {
                                html.push_str(&format!("<p><em>Transcluded content not found: {}</em></p>", content_id));
                                // No need to remove the source link with CSS-based approach
                            }

                            html.push_str("</div>");
                        }
                    }
                } else {
                    // Fallback - just render children
                    html.push_str(&render_blocks(&block.Children, notes_map, id_to_path));
                }
            },
            _ => {
                // For unhandled node types, just render their children
                html.push_str(&render_blocks(&block.Children, notes_map, id_to_path));
            }
        }
    }

    html
}

fn render_text_mark(block: &Block, notes_map: &HashMap<String, Note>, id_to_path: &HashMap<String, PathBuf>) -> String {
    let mut html = String::new();
    let id_attr = if !block.ID.is_empty() {
        format!(" id=\"{}\"", block.ID)
    } else {
        String::new()
    };

    match block.TextMarkType.as_str() {
        "a" => {
            html.push_str(&format!(
                "<a{} href=\"{}\" target=\"_blank\" class=\"link\">{}",
                id_attr,
                block.TextMarkAHref,
                block.TextMarkTextContent
            ));
            html.push_str("</a>");
        },
        "code" => {
            html.push_str(&format!(
                "<code{}>{}",
                id_attr,
                escape_html(&block.TextMarkTextContent)
            ));
            html.push_str("</code>");
        },
        "strong" | "strong text" => {
            // Handle both "strong" and "strong text" the same way
            let content = escape_html(&block.TextMarkTextContent);

            // Check if there are style properties for special highlights
            if !block.Properties.style.is_empty() {
                if let Some(class_name) = get_style_class(&block.Properties.style, true) {
                    html.push_str(&format!(
                        "<strong{} class=\"{}\">{}",
                        id_attr,
                        class_name,
                        content
                    ));
                } else {
                    // Use inline style for custom colors
                    html.push_str(&format!(
                        "<strong{} style=\"{}\">{}",
                        id_attr,
                        block.Properties.style,
                        content
                    ));
                }
            } else {
                html.push_str(&format!("<strong{}>{}", id_attr, content));
            }
            html.push_str("</strong>");
        },
        "em" => {
            // Check if this is also a link (has a href)
            if !block.TextMarkAHref.is_empty() {
                html.push_str(&format!(
                    "<a{} href=\"{}\" target=\"_blank\" class=\"link\"><em>{}",
                    id_attr,
                    block.TextMarkAHref,
                    escape_html(&block.TextMarkTextContent)
                ));
                html.push_str("</em></a>");
            } else {
                html.push_str(&format!(
                    "<em{}>{}",
                    id_attr,
                    escape_html(&block.TextMarkTextContent)
                ));
                html.push_str("</em>");
            }
        },
        "u" => {
            html.push_str(&format!(
                "<u{}>{}",
                id_attr,
                escape_html(&block.TextMarkTextContent)
            ));
            html.push_str("</u>");
        },
        "s" => {
            html.push_str(&format!(
                "<s{}>{}",
                id_attr,
                escape_html(&block.TextMarkTextContent)
            ));
            html.push_str("</s>");
        },
        "sub" => {
            html.push_str(&format!(
                "<sub{}>{}",
                id_attr,
                escape_html(&block.TextMarkTextContent)
            ));
            html.push_str("</sub>");
        },
        "sup" => {
            html.push_str(&format!(
                "<sup{}>{}",
                id_attr,
                escape_html(&block.TextMarkTextContent)
            ));
            html.push_str("</sup>");
        },
        "kbd" => {
            html.push_str(&format!(
                "<kbd{}>{}",
                id_attr,
                escape_html(&block.TextMarkTextContent)
            ));
            html.push_str("</kbd>");
        },
        "mark" => {
            html.push_str(&format!(
                "<mark{}>{}",
                id_attr,
                escape_html(&block.TextMarkTextContent)
            ));
            html.push_str("</mark>");
        },
        "text" | "text strong" => {
            // Check if there are style properties for special highlights
            if !block.Properties.style.is_empty() {
                let content = escape_html(&block.TextMarkTextContent);
                let tag_open = if block.TextMarkType == "text strong" { "<strong" } else { "<span" };
                let tag_close = if block.TextMarkType == "text strong" { "</strong>" } else { "</span>" };

                if let Some(class_name) = get_style_class(&block.Properties.style, true) {
                    html.push_str(&format!(
                        "{}{} class=\"{}\">{}",
                        tag_open,
                        id_attr,
                        class_name,
                        content
                    ));
                } else {
                    // Use inline style for custom colors
                    html.push_str(&format!(
                        "{}{} style=\"{}\">{}",
                        tag_open,
                        id_attr,
                        block.Properties.style,
                        content
                    ));
                }
                html.push_str(tag_close);
            } else {
                html.push_str(&escape_html(&block.TextMarkTextContent));
            }
        },
        "tag" => {
            html.push_str(&format!(
                "<a{} href=\"tag_{}.html\" class=\"tag\"># {}",
                id_attr,
                block.TextMarkTextContent.replace(" ", "_"),
                block.TextMarkTextContent
            ));
            html.push_str("</a>");
        },
        "inline-math" => {
            // Just preserve the math content in a specially styled span
            html.push_str(&format!(
                "<span{} class=\"math-inline\">{}</span>",
                id_attr,
                escape_html(&block.TextMarkTextContent)
            ));
        }
        "inline-memo" => {
            // Use HTML title attribute for inline memos
            html.push_str(&format!(
                "<span{} title=\"{}\">{}",
                id_attr,
                escape_html(&block.TextMarkInlineMemoContent),
                escape_html(&block.TextMarkTextContent)
            ));
            html.push_str("</span>");
        },
        "block-ref" => {
            if notes_map.contains_key(&block.TextMarkBlockRefID) {
                let ref_note = &notes_map[&block.TextMarkBlockRefID];
                let title = if !block.TextMarkTextContent.is_empty() {
                    block.TextMarkTextContent.clone()
                } else if !ref_note.Properties.title.is_empty() {
                    ref_note.Properties.title.clone()
                } else {
                    block.TextMarkBlockRefID.clone()
                };

                // Extract first few paragraphs for excerpt
                let mut excerpt = String::new();
                let mut paragraph_count = 0;
                for child in &ref_note.Children {
                    if child.Type == "NodeParagraph" {
                        if paragraph_count > 0 {
                            // Add line break between paragraphs, not after
                            excerpt.push_str("<br>");
                        }

                        for grandchild in &child.Children {
                            if grandchild.Type == "NodeText" {
                                excerpt.push_str(&escape_html(&grandchild.Data));
                                excerpt.push(' ');
                            }
                        }

                        paragraph_count += 1;
                        if paragraph_count >= 2 {
                            break;
                        }
                    }
                }

                // Limit excerpt length
                if excerpt.len() > 300 {
                    // Ensure we cut at a character boundary
                    let mut end_index = 0;
                    let mut char_count = 0;
                    for (idx, _) in excerpt.char_indices() {
                        end_index = idx;
                        char_count += 1;
                        if char_count >= 300 {
                            break;
                        }
                    }
                    excerpt = excerpt[0..end_index].to_string();
                    excerpt.push_str("...");
                }

                // Create tooltip HTML
                html.push_str(&format!("<span{} class=\"tooltip\">", id_attr));
                html.push_str(&format!(
                    "<a href=\"{}.html\">{}",
                    block.TextMarkBlockRefID,
                    escape_html(&title)
                ));
                html.push_str("</a>");
                html.push_str("<span class=\"right bottom\">");
                html.push_str(&format!("<span class=\"tooltip-title\">{}</span>", escape_html(&ref_note.Properties.title)));
                html.push_str(&format!("<span class=\"tooltip-excerpt\">{}</span>", excerpt));
                html.push_str("<i></i></span></span>");
            } else {
                html.push_str(&format!(
                    "<span{} title=\"Missing reference: {}\">{}",
                    id_attr,
                    block.TextMarkBlockRefID,
                    escape_html(&block.TextMarkTextContent)
                ));
                html.push_str("</span>");
            }
        },
        _ => {
            html.push_str(&escape_html(&block.TextMarkTextContent));
        }
    }

    html
}

fn comment_processor(html: &str) -> String {
    const COMMENT: &str = "\n<!-- Generated with SyMark, a static site generator for SiYuan Note. Available at https://github.com/du82/symark -->\n";

    let mut insertion_points = Vec::new();
    let mut depth = 0;
    let mut in_tag = false;
    let mut in_comment = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut total_points = 0;

    for (i, c) in html.char_indices() {
        match c {
            '<' => {
                if !in_comment {
                    in_tag = true;
                    if html[i..].starts_with("<!--") {
                        in_comment = true;
                    } else if i + 7 <= html.len() && html[i..].starts_with("<script") {
                        in_script = true;
                    } else if i + 6 <= html.len() && html[i..].starts_with("<style") {
                        in_style = true;
                    } else if i + 9 <= html.len() && html[i..].starts_with("</script>") {
                        in_script = false;
                    } else if i + 8 <= html.len() && html[i..].starts_with("</style>") {
                        in_style = false;
                    }
                }
            },
            '>' => {
                if in_comment && i > 2 && &html[i-2..=i] == "-->" {
                    in_comment = false;
                    in_tag = false;
                } else if in_tag && !in_comment {
                    in_tag = false;
                    if i > 0 {
                        let tag_start = html[..i].rfind('<').unwrap_or(i);
                        let tag_content = &html[tag_start..=i];

                        // Opening tag increases depth
                        if !tag_content.starts_with("</") && !tag_content.ends_with("/>") {
                            depth += 1;
                        }
                        else if tag_content.starts_with("</") {
                            if depth > 0 {
                                depth -= 1;
                            }
                        }
                    }
                }
            },
            _ => {}
        }

        if !in_tag && !in_comment && !in_script && !in_style {
            if c == '\n' || c == ' ' || c == '>' {
                let position_ratio = i as f64 / html.len() as f64;
                if depth >= 3 && position_ratio > 0.2 && position_ratio < 0.8 {
                    insertion_points.push((i, depth));
                    total_points += 1;
                }
            }
        }
    }

    if total_points > 0 {
        let mid_section: Vec<(usize, usize)> = insertion_points.iter()
            .filter(|(pos, d)| {
                let pos_ratio = *pos as f64 / html.len() as f64;
                *d >= 3 && pos_ratio >= 0.2 && pos_ratio <= 0.8
            })
            .cloned()
            .collect();

        let weighted_points: Vec<(usize, usize)> = mid_section.iter()
            .flat_map(|(pos, d)| {
                let weight = d * d; // Square the depth to increase probability for deeper points
                std::iter::repeat((*pos, *d)).take(weight)
            })
            .collect();

        if !weighted_points.is_empty() {
            let seed = html.len() + html.chars().fold(0, |acc, c| acc + c as usize);
            let index_pos = seed % weighted_points.len();
            let (insertion_point, _) = weighted_points[index_pos];

            let mut result = String::with_capacity(html.len() + COMMENT.len());
            result.push_str(&html[..=insertion_point]);
            result.push_str(COMMENT);
            result.push_str(&html[insertion_point+1..]);
            return result;
        }
    }

    let middle_point = html.len() / 2;
    let mut result = String::with_capacity(html.len() + COMMENT.len());
    result.push_str(&html[..middle_point]);
    result.push_str(COMMENT);
    result.push_str(&html[middle_point..]);
    result
}

fn find_block_by_id<'a>(block_id: &str, blocks: &'a [Block]) -> Option<&'a Block> {
    // First check if any block at this level has the ID
    if let Some(block) = blocks.iter().find(|b| b.ID == block_id) {
        return Some(block);
    }

    // If not found, recursively check all children
    for block in blocks {
        if !block.Children.is_empty() {
            if let Some(found) = find_block_by_id(block_id, &block.Children) {
                return Some(found);
            }
        }
    }

    None
}

// This function handles finding a block by ID, or an entire note by ID if the block isn't found
fn find_content_by_id<'a>(
    id: &str,
    notes_map: &'a HashMap<String, Note>,
) -> Option<&'a [Block]> {
    // First, check if this is a note ID
    if let Some(note) = notes_map.get(id) {
        // If it's a note ID, return all its top-level blocks
        return Some(&note.Children);
    }

    // Otherwise, search for the specific block ID in all notes
    for note in notes_map.values() {
        if let Some(_) = find_block_by_id(id, &note.Children) {
            // If we found the block, return it wrapped in a slice
            // We don't actually need the block itself here, just to know it exists
            // The actual rendering will be done in the calling function
            return Some(&note.Children);
        }
    }

    None
}

fn render_block(
    block: &Block,
    notes_map: &HashMap<String, Note>,
    id_to_path: &HashMap<String, PathBuf>,
) -> String {
    // Create a slice with a single element instead of a Vec
    render_blocks(std::slice::from_ref(block), notes_map, id_to_path)
}

fn escape_html(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
}

/// Removes zero-width spaces while preserving emoji combinations
fn remove_zero_width_spaces(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut chars = html.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\u{200B}' || c == '\u{200C}' || c == '\u{2060}' || c == '\u{200E}' || c == '\u{200F}' {
            continue;
        }

        let is_emoji_start = c >= '\u{1F000}' && c <= '\u{1FFFF}' ||
                             c >= '\u{2600}' && c <= '\u{27BF}' ||
                             c >= '\u{2300}' && c <= '\u{23FF}' ||
                             c >= '\u{2700}' && c <= '\u{27FF}' ||
                             c >= '\u{1F1E6}' && c <= '\u{1F1FF}';

        result.push(c);

        if is_emoji_start {
            while let Some(&next) = chars.peek() {
                if next == '\u{200D}' {
                    result.push(next);
                    chars.next();

                    if let Some(emoji_part) = chars.next() {
                        result.push(emoji_part);

                        while let Some(&modifier) = chars.peek() {
                            if (modifier >= '\u{1F3FB}' && modifier <= '\u{1F3FF}') || modifier == '\u{FE0F}' {
                                result.push(modifier);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                } else if (next >= '\u{1F3FB}' && next <= '\u{1F3FF}') || next == '\u{FE0F}' {
                    result.push(next);
                    chars.next();
                } else {
                    break;
                }
            }
        }
    }

    result
}

// Function to clean up any remaining template variables in the HTML
fn cleanup_template_variables(html: &str) -> String {
    let mut result = String::new();
    let mut current_pos = 0;

    // Find and remove template variables {{...}}
    while let Some(start_pos) = html[current_pos..].find("{{") {
        let absolute_start = current_pos + start_pos;
        // Add everything up to the template variable
        result.push_str(&html[current_pos..absolute_start]);

        // Find the end of the template variable
        if let Some(end_pos) = html[absolute_start..].find("}}") {
            // Skip past the template variable
            current_pos = absolute_start + end_pos + 2;
        } else {
            // No closing }} found, so just add the {{ and continue
            result.push_str("{{");
            current_pos = absolute_start + 2;
        }
    }

    // Add the rest of the string
    result.push_str(&html[current_pos..]);

    // Second pass: remove empty meta tags
    let mut cleaned_html = String::new();
    let lines: Vec<&str> = result.lines().collect();

    for line in lines {
        let trimmed = line.trim();
        if !(
            (trimmed.starts_with("<meta property=\"og:") ||
             trimmed.starts_with("<meta property=\"article:")) &&
            (trimmed.contains("content=\"\"") ||
             trimmed.contains("content=\"{{") ||
             trimmed.contains("content=\"\">"))
        ) {
            cleaned_html.push_str(line);
            cleaned_html.push('\n');
        }
    }

    cleaned_html
}
