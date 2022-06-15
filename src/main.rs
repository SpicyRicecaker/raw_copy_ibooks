use std::io;

    /*
    June 15, 2022
The Judgment, p. 120

“ elderly father—had reached the friend, who sent a letter expressing his condolences so dryly that it could be concluded that the grief over such an event could not be felt from such a distance. But s”

Excerpt from:
The Metamorphosis and Other Stories
Franz Kafka
This material may be protected by copyright.
    */

fn strip(mut iter: impl Iterator<Item = String>) -> String {
    // skip the first three lines
    let iter = iter.skip(3);

    // turn it into a string, remove \n\nExcerpt from ...
    let latter_half_string: String = iter.collect::<Vec<String>>().join("\n");

    latter_half_string.split("\n\nExcerpt from:\n").next().expect("invalid split").to_string()
}

fn main() {
    // receive input from stdin
    let input_iter = io::stdin().lines().filter_map(|l|l.ok());

    println!("{}", strip(input_iter));
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_ex() {
        let input = r##"    June 15, 2022
The Judgment, p. 120

“ elderly father—had reached the friend, who sent a letter expressing his condolences so dryly that it could be concluded that the grief over such an event could not be felt from such a distance. But s”

Excerpt from:
The Metamorphosis and Other Stories
Franz Kafka
This material may be protected by copyright."##;

        let out = strip(input.lines().map(|l|l.to_string()));

        assert_eq!(&out, r##"“ elderly father—had reached the friend, who sent a letter expressing his condolences so dryly that it could be concluded that the grief over such an event could not be felt from such a distance. But s”"##);
    }
}