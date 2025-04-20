// Struct to keep track of lines and the current position
pub struct LineTracker<'a> {
	lines: Vec<&'a str>,
	current: usize,
}

impl<'a> LineTracker<'a> {
	// Create a new LineTracker from a string slice
	pub fn new(content: &'a str) -> Self {
		return LineTracker {
			lines: content.lines().collect(),
			current: 0,
		};
	}

	// Move one line back, if possible
	pub fn back(&mut self) {
		if self.current > 0 {
			self.current -= 1;
		}
	}
}

impl<'a> Iterator for LineTracker<'a> {
	type Item = &'a str;

	// Return the current line and advance the cursor
	fn next(&mut self) -> Option<Self::Item> {
		if self.current < self.lines.len() {
			let line = self.lines[self.current];
			self.current += 1;
			return Some(line);
		} else {
			return None;
		}
	}
}

pub enum ListType {
	UL,
	OL,
}

pub enum BlockType {
	CODE,
	HTML,
}
