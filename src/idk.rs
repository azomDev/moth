use std::{fs, time::Instant};

use crate::{
	htmlerror,
	parsing::{image_parser, parse_heading, parse_markdown_line},
	types::{BlockType, LineTracker, ListType},
};

pub fn transpile(file_path: &str, base64_image_compiling: bool) -> String {
	let start_time = Instant::now();

	let content = fs::read_to_string(file_path).unwrap();
	let mut lines = LineTracker::new(&content);
	let html_content = process_layer(&mut lines, 0, base64_image_compiling);

	let duration = start_time.elapsed();
	println!("Total time taken: {:?}", duration);
	return html_content;
}

pub fn check_list_type(line: &str) -> Option<ListType> {
	let mut chars = line.chars();

	match chars.next() {
		Some(c) => match c {
			'-' | '*' if chars.next() == Some(' ') => Some(ListType::UL),
			c if c.is_digit(10) && chars.next() == Some('.') => Some(ListType::OL),
			_ => None,
		},
		None => None,
	}
}

// todo line breaks like I want to
//
// block todo:
// $$
// aaaa
// $$

pub fn process_layer<'a>(
	raw_lines: &mut LineTracker,
	depth: usize,
	base64_image_compiling: bool,
) -> String {
	let mut output = String::new();

	let mut current_list_type: Option<ListType> = None;
	let mut current_block_type: Option<BlockType> = None;

	while let Some(raw_line) = raw_lines.next() {
		// Check if the line is empty
		if raw_line.chars().all(|c| c.is_whitespace()) {
			output.push_str("<span class='small-space'></span>");
			continue;
		}

		// Check that we are still at the same depth
		let nb_leading_tabs = match clamped_nb_leading_tabs(raw_line, depth) {
			Some(nb_tabs) => nb_tabs,
			None => {
				// Number of tabs is smaller than depth
				raw_lines.back();
				break;
			}
		};

		let line = &raw_line[nb_leading_tabs..];

		if let Some(ref block_type) = current_block_type {
			match block_type {
				BlockType::CODE => {
					if line.starts_with("```") {
						current_block_type = None;
						output.push_str("</code></pre>");
					} else {
						output.push_str(line);
						output.push_str("\n");
					}
				}
				BlockType::HTML => {
					if line.starts_with("</html>") {
						current_block_type = None;
					} else {
						output.push_str(line);
						output.push_str("\n");
					}
				}
			}
			continue;
		}

		if let Some(list_type) = check_list_type(&line) {
			if current_list_type.is_none() {
				let list_type_str = match list_type {
					ListType::UL => "<ul>",
					ListType::OL => "<ol>",
				};
				output.push_str(list_type_str);
				current_list_type = Some(list_type);
			}

			let inner_html = process_layer(raw_lines, depth + 1, base64_image_compiling);
			let item_title = parse_markdown_line(&line[2..]);
			output.push_str(&format!("<li>{}", item_title));
			if !inner_html.is_empty() {
				output.push_str(&format!("<p>{}</p>\n", inner_html));
			}
			output.push_str("</li>\n");

			continue;
		} else {
			// If we are in a list and we stopped seeing list items, break out of the list
			if let Some(list_type) = current_list_type {
				match list_type {
					ListType::OL => output.push_str("</ol>"),
					ListType::UL => output.push_str("</ul>"),
				};
				current_list_type = None;
			}
		}

		// other things

		let mut line_chars = line.chars();
		let first_char = line_chars.next();
		match first_char {
			Some(c) => match c {
				'#' => {
					output.push_str(&parse_heading(line));
					continue;
				}
				'!' => {
					if let Some(s) = image_parser(line, base64_image_compiling) {
						output.push_str(&s);
						continue;
					}
				}
				'`' => {
					if line.starts_with("```") {
						current_block_type = Some(BlockType::CODE);
						output.push_str("<pre><code>");
						continue;
					}
				}
				'<' => {
					if line == "<html>" {
						current_block_type = Some(BlockType::HTML);
						continue;
					}
				}
				'-' => {
					if line_chars.next() == Some('-') && line_chars.next() == Some('-') {
						output.push_str("<hr>");
						continue;
					}
				}
				_ => {}
			},
			None => unreachable!(),
		}

		// if nothing matches then parse inlines
		let ends_with_backslash = line.ends_with('\\');
		let clean_line = if ends_with_backslash {
			&line[..line.len() - 1]
		} else {
			line
		};
		// todo add space or newline after text if it ends with a backslash so the text of two lines are not glued together
		output.push_str(&format!("{}", parse_markdown_line(clean_line)));
		if !ends_with_backslash {
			output.push_str("<span class='small-space'></span>");
		}
	}

	if !current_list_type.is_none() {
		match current_list_type {
			Some(ListType::OL) => output.push_str("</ol>"),
			Some(ListType::UL) => output.push_str("</ul>"),
			None => unreachable!(),
		}
	}

	if !current_block_type.is_none() {
		return htmlerror!("Block was not closed");
	}

	return output;
}

// clamped at max of depth
fn clamped_nb_leading_tabs(line: &str, depth: usize) -> Option<usize> {
	let mut count = 0;

	// Manually iterate over the first `depth` characters
	for c in line.chars().take(depth) {
		if c == '\t' {
			count += 1;
		} else {
			return None;
		}
	}

	return Some(count);
}
