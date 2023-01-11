use arboard::{Clipboard, ImageData};
use chrono::{DateTime, Local};
use image::{ImageBuffer, RgbaImage};
use regex::Regex;
use std::error::Error;

pub fn remove_excerpt(input: String) -> String {
    let input = input.trim().to_string();

    let mut splits = input.split("\n\nExcerpt From\n");

    let mut quoted_string = splits.next().unwrap().to_string();

    // if there's more than 1 split or less than 1 split, do not run the
    // program, leave text untouched except for trimming whitespace
    if splits.count() != 1 {
        return input;
    }

    let mut index_of_quote_that_matches_first_opening = 1;
    let mut last_index = 1;

    let mut stack: Vec<(usize, char)> = Vec::new();
    quoted_string
        .chars()
        .enumerate()
        .filter(|(_, c)| *c == '“' || *c == '”')
        .for_each(|(i, c)| {
            // not sure how else to keep track of the largest
            last_index = i;

            if c == '“' {
                stack.push((i, c));
            } else if let Some((i_l, c_l)) = stack.last() {
                if *c_l == '“' {
                    if *i_l == 0 {
                        index_of_quote_that_matches_first_opening = i;
                    }
                    stack.pop();
                } else {
                    stack.push((i, c));
                }
            } else {
                stack.push((i, c));
            }
        });

    // queue all remaining quotes on the stack to be straight up deleted from the string
    // I don't think it's ever possible to have > 1 quote left at the end, and all dangling quotes should also be at the beginning or end, I think.
    // again, number of characters does not equal length of string, so basically never modify string directly if you can help it.
    if let Some((i, _)) = stack.last() {
        if *i == 0 {
            quoted_string.remove(0);
        } else {
            quoted_string.pop();
        }
    } else if index_of_quote_that_matches_first_opening == last_index {
        quoted_string.pop();
        quoted_string.remove(0);
    }
    quoted_string
}

pub enum DataType {
    Text(String),
    Image(ImageData<'static>),
}

impl DataType {
    pub fn from_clipboard(clipboard: &mut Clipboard) -> Result<DataType, Box<dyn Error>> {
        if let Ok(text) = clipboard.get_text() {
            Ok(DataType::Text(text))
        } else if let Ok(image) = clipboard.get_image() {
            Ok(DataType::Image(image))
        } else {
            Err("Unable to access clipboard for some reason".into())
        }
    }
}

pub fn parse_information(data_type: DataType) -> String {
    match data_type {
        DataType::Text(text) => remove_excerpt(text),
        DataType::Image(image) => {
            // try saving the buffer to
            // image
            let image: RgbaImage = ImageBuffer::from_raw(
                image.width.try_into().unwrap(),
                image.height.try_into().unwrap(),
                image.bytes.into_owned(),
            )
            .unwrap();
            let filename = {
                let now: DateTime<Local> = Local::now();
                // taking care not to include any special characters that
                // need to be % enconded in Anki, as either anki-connect or
                // Anki reencodes % signs, making them impossible to encode.
                format!("paste-{}.png", now.format("%F-%H-%M-%S"))
            };
            let filepath = {
                let mut t = dirs::config_dir().unwrap();
                t.push("Anki2");
                // TODO hardcoded profile, we should let the user modify their profile in a config
                t.push("User 1");
                t.push("collection.media");
                // generate current date
                t.push(&filename);
                t
            };
            // move image to home dir
            // there's a chance that this could overwrite important files if the name isn't unique enough
            image.save(&filepath).unwrap();
            format!(r#"<img src="{}">"#, &filename)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ex() {
        let input: String =
            std::fs::read_to_string("test/test_cases.txt").expect("error reading test file");
        let mut iter = input.split("\n=====\n");

        while let Some(before) = iter.next() {
            let after = iter.next().expect("error with test cases");

            assert_eq!(after, remove_excerpt(before.to_string()));
        }
    }
}

/// takes in a pattern and another pattern.
/// the problem is that sometimes the word starts at the beginning of a sentence
/// or something. Should we fuzzy match like 90% ?
pub fn rep<'a>(parent: &'a str, replace: &str) -> std::borrow::Cow<'a, str> {
    let regex = Regex::new(&format!("(?i){}", replace)).unwrap();
    regex.replace_all(parent, &format!("{{{{c1::{}}}}}", replace))
}
