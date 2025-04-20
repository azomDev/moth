use base64::{Engine, engine::general_purpose};
use katex::OutputType;

pub fn parse_heading(line: &str) -> String {
	// pub fn parse_heading(line: &str) -> String {
	let mut num_hashes: u32 = 0;
	let mut content_start = 0;

	for c in line.chars() {
		if c == '#' {
			num_hashes += 1;
		} else {
			break; // No need to check further once content starts
		}
		content_start += 1;
	}

	num_hashes = num_hashes.clamp(1, 6);
	let level_strs = ["1", "2", "3", "4", "5", "6"]; // cursed but performance
	let level_str = level_strs[(num_hashes - 1) as usize];

	return format!(
		"<h{}>{}</h{}>",
		level_str,
		&line[content_start..],
		level_str
	);
}

pub fn image_parser(line: &str, base64_image_compiling: bool) -> Option<String> {
	// Skip the first '!' character
	let mut chars = line.chars().skip(1);
	let next_char = chars.next();
	if next_char == None || next_char != Some('[') {
		return None;
	}

	let mut end_index = 0;

	for c in line.chars() {
		if c == ']' {
			break;
		}
		end_index += 1;
	}

	let image_path = &line[2..end_index];
	if !base64_image_compiling {
		return Some(format!("<img src=\"./{}\"/>", image_path));
	}

	match std::fs::read(format!("./{}", image_path)) {
		Ok(bytes) => {
			let base64_data = general_purpose::STANDARD.encode(&bytes);

			// Return the image tag with the Base64 data
			return Some(format!(
				"<img src=\"data:image/png;base64,{}\" />\n",
				base64_data
			));
		}
		Err(e) => return Some(format!("<p>Error loading image: {}</p>\n", e)),
	}
}

pub fn parse_markdown_line(line: &str) -> String {
	let mut result = String::new();
	let mut i = 0;
	let chars: Vec<char> = line.chars().collect();
	let len = chars.len();
	let mut emphasis_stack: Vec<usize> = Vec::new();
	let mut in_code = false;
	let mut in_html = false;

	while i < len {
		if in_code {
			if chars[i] == '`' {
				// close code
				result.push_str("</code></pre>");
				in_code = false;
				i += 1;
			} else {
				result.push(chars[i]);
				i += 1;
			}
			continue;
		}

		if in_html {
			// look for closing </html>
			if i + 6 < len && &line[i..i + 7] == "</html>" {
				// append tag and close html state
				in_html = false;
				i += 7;
			} else {
				result.push(chars[i]);
				i += 1;
			}
			continue;
		}

		match chars[i] {
			'`' => {
				// start code
				result.push_str("<pre><code>");
				in_code = true;
				i += 1;
			}
			'<' => {
				// check for raw html
				if i + 6 < len && &line[i..i + 6] == "<html>" {
					in_html = true;
					i += 6;
				} else {
					// quicklink
					if let Some(j) = (i + 1..len).find(|&j| chars[j] == '>') {
						let link_text: String = chars[i + 1..j].iter().collect();
						result.push_str(&format!("<a href=\"{}\">{}</a>", link_text, link_text));
						i = j + 1;
					} else {
						// no closing '>', treat literally
						result.push('<');
						i += 1;
					}
				}
			}
			'*' => {
				// count consecutive '*' up to 3
				let mut count = 1;
				while count < 3 && i + count < len && chars[i + count] == '*' {
					count += 1;
				}
				// determine open or close
				if emphasis_stack.last() == Some(&count) {
					// close
					emphasis_stack.pop();
					match count {
						1 => result.push_str("</em>"),
						2 => result.push_str("</strong>"),
						3 => result.push_str("</em></strong>"),
						_ => {}
					}
				} else {
					// open
					emphasis_stack.push(count);
					match count {
						1 => result.push_str("<em>"),
						2 => result.push_str("<strong>"),
						3 => result.push_str("<strong><em>"),
						_ => {}
					}
				}
				i += count;
			}
			'$' => {
				// start or end equation (inline math mode)
				let mut eq_end = i + 1;
				while eq_end < len && chars[eq_end] != '$' {
					eq_end += 1;
				}
				if eq_end < len && chars[eq_end] == '$' {
					// Found closing '$', parse the equation between
					let equation: String = chars[i + 1..eq_end].iter().collect();
					// Render the equation using KaTeX
					// let rendered_eq = render(&equation).unwrap();
					let opts = katex::Opts::builder()
						.output_type(OutputType::Mathml)
						.build()
						.unwrap();
					let rendered_eq = katex::render_with_opts(&equation, &opts).unwrap();
					result.push_str(&format!("<span class=\"katex\">{}</span>", rendered_eq));
					i = eq_end + 1;
				} else {
					// Unmatched '$', treat literally
					result.push('$');
					i += 1;
				}
			}
			c => {
				result.push(c);
				i += 1;
			}
		}
	}

	// Close any unclosed emphasis tags
	while let Some(count) = emphasis_stack.pop() {
		match count {
			1 => result.push_str("</em>"),
			2 => result.push_str("</strong>"),
			3 => result.push_str("</em></strong>"),
			_ => {}
		}
	}

	result
}
