// create a macro "htmlerror" that takes in the same things as format macro and returns <span style=\"color: red;\">THE_TEXT</span>
#[macro_export]
macro_rules! htmlerror {
    ($($arg:tt)*) => {{
        let formatted = format!($($arg)*);
        format!("<span style=\"color: red;\">{}</span>", formatted)
    }};
}
