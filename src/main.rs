//! This crate takes in a copied text string from ibooks from the clipboard,
//! strips the excerpts as well as the beginning and end quotes

use copypasta::{ClipboardContext, ClipboardProvider};

fn strip(input: String) -> String {
    let mut splits = input.split("\n\nExcerpt From\n");

    let mut quoted_string = splits.next().unwrap().to_string();

    // if there's more than 1 split or less than 1 split, do not run the
    // program, leave text untouched
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

fn main() {
    let mut ctx = ClipboardContext::new().expect("error creating clipboard context");
    let clipboard_contents = ctx.get_contents().expect("error getting clipboard content");
    let res = strip(clipboard_contents);
    ctx.set_contents(res).expect("error setting clipboard");
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

            assert_eq!(after, strip(before.to_string()));
        }
    }
}
