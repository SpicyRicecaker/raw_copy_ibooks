//! This crate takes in a copied text string from ibooks from the clipboard,
//! strips the excerpts as well as the beginning and end quotes
//!
//! three options
//! regular copy copies text but with formatting stripped
//!
//! we can add our selection to basic or cloze
//! --add basic
//! --add cloze
//! --reset clears all steps, useful when accidentally copying something
//! --undo clears the recent add step
//! --complete finishes the cards and ships it
//!
//! additionally, we should also check the last copy time. If the last copy
//! time is greater than 1hr then we should run the reset function regardless
//! }
use ripbk_cli::*;

fn main() {
    let program = Program::new();

    match Program::get_action() {
        Action::Reset => {
            // generate a clear state then serialize to the path
            program.reset();
        }
        Action::Undo => {
            program.undo();
        }
        Action::Data => {
            program.data();
        }
        Action::Add(b) => {
            program.add(b, Program::get_data_from_clipboard());
        }
        Action::Copy => Program::copy(),
        Action::Complete => {
            program.complete();
        },
    };
}

#[cfg(test)]
mod test {
    use ripbk_lib::parse_information;

    use super::*;
    #[test]
    // test add
    fn test_reset() {
        let program = Program::new();
        program.reset();

        assert_eq!(
            std::fs::read_to_string("example.json").unwrap(),
            "{\n  \"meaning\": \"\",\n  \"word\": \"\",\n  \"reading\": \"\",\n  \"example\": \"\",\n  \"progress\": \"0\"\n}\n"
        );
    }
    #[test]
    fn test_add() {
        let program = Program::new();
        program.reset();

        let s = parse_information(ripbk_lib::DataType::Text("“Hello World”\n\nExcerpt From\nLife\nby me\nbob.com\nThis material may be protected by copyright.".into()));

        assert_eq!(s, "Hello World")
    }
    #[test]
    fn test_undo() {
        let program = Program::new();
        program.reset();
        // dbg!(&program.data_path);

        program.add(
            NoteType::Cloze,
            ripbk_lib::DataType::Text("XDDDDDDDDDDDDDDDDDDDDDDDDD".into()),
        );
        program.add(
            NoteType::Cloze,
            ripbk_lib::DataType::Text("XDDDDDDDDDDDDDDDDDDDDDDDDD".into()),
        );

        program.undo();

        let s = program.read_file();

        assert_eq!(s.progress, 1);
        assert_eq!(
            match PROGRESS_ORDERING[s.progress] {
                Progress::Meaning => s.meaning,
                Progress::Word => s.word,
                Progress::Reading => s.reading,
                Progress::Example => s.example,
            },
            String::from("XDDDDDDDDDDDDDDDDDDDDDDDDD")
        );
    }
}
