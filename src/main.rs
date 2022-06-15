use std::io;

    /* “But Georg’s friend had no inkling of this change. Earlier, perhaps the letter of condolence was the last time, he had tried to lure Georg into emigrating to Russia and expounded upon the prospects that St. Petersburg offered in precisely Georg’s line of business.”

    Excerpt From
    The Metamorphosis and Other Stories
    Franz Kafka
    This material may be protected by copyright.*/


fn strip(iter: impl Iterator<Item = String>) -> String {
    // turn it into a string, remove \n\nExcerpt from ...
    let latter_half_string: String = iter.collect::<Vec<String>>().join("\n");

    let quoted_string = latter_half_string.split("\n\nExcerpt From\n").next().expect("invalid split");
    
    // remove quotes
    // very scuffed unicode string slicing, for some reason `“` and `”` are both 3 bytes... so to remove them, we hardcode in 3 
    quoted_string[3..quoted_string.len()-3].to_string()
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
        let input = r##"“But Georg’s friend had no inkling of this change. Earlier, perhaps the letter of condolence was the last time, he had tried to lure Georg into emigrating to Russia and expounded upon the prospects that St. Petersburg offered in precisely Georg’s line of business.”

Excerpt From
The Metamorphosis and Other Stories
Franz Kafka
This material may be protected by copyright."##;

        let out = strip(input.lines().map(|l|l.to_string()));

        assert_eq!(&out, r##"But Georg’s friend had no inkling of this change. Earlier, perhaps the letter of condolence was the last time, he had tried to lure Georg into emigrating to Russia and expounded upon the prospects that St. Petersburg offered in precisely Georg’s line of business."##);


        let input = r##"“orld over. In the course of these three years, however, much had changed in Georg’s own life. The news of his mother’s death—she died two years ago and Georg had since been living with his elderly father—had reached the friend, who sent a letter expressing his condolences so dryly that it could be concluded that the grief over such an event could not be felt from such a distance. But since that time Georg had tackled his business, as well as everything else, with more fervor. Perhaps his fat”

Excerpt From
The Metamorphosis and Other Stories
Franz Kafka
This material may be protected by copyright."##;
        let out = strip(input.lines().map(|l|l.to_string()));
        assert_eq!(&out, r##"orld over. In the course of these three years, however, much had changed in Georg’s own life. The news of his mother’s death—she died two years ago and Georg had since been living with his elderly father—had reached the friend, who sent a letter expressing his condolences so dryly that it could be concluded that the grief over such an event could not be felt from such a distance. But since that time Georg had tackled his business, as well as everything else, with more fervor. Perhaps his fat"##);
    }
}