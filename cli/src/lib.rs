use clap::{arg, command, value_parser};

use arboard::Clipboard;
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::PathBuf,
};

use ripbk_lib::{parse_information, rep, DataType};
use serde::{Deserialize, Serialize};

const ANKI_URL: &str = "http://localhost:8765";

#[derive(Debug, Clone, Copy)]
pub enum NoteType {
    Basic,
    Cloze,
}

impl clap::ValueEnum for NoteType {
    fn value_variants<'a>() -> &'a [Self] {
        &[NoteType::Basic, NoteType::Cloze]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            NoteType::Basic => clap::builder::PossibleValue::new("basic"),
            NoteType::Cloze => clap::builder::PossibleValue::new("cloze"),
        })
    }
    // ...
}

pub enum Action {
    Reset,
    Undo,
    Data,
    Add(NoteType),
    Copy,
    Complete,
}

pub struct Program {
    pub data_path: PathBuf,
}

impl Program {
    pub fn get_action() -> Action {
        let matches = command!() // requires `cargo` feature
            // .arg(arg!([name] "Optional name to operate on"))
            .arg(
                arg!(
                    -a --add <ADD> "Sets the type of card we're adding to"
                )
                // We don't have syntax yet for optional options, so manually calling `required`
                .required(false)
                .value_parser(value_parser!(NoteType)),
            )
            .arg(arg!(
                -r --reset ... "Remove all builder progress"
            ))
            .arg(arg!(
                -u --undo ... "Remove one stage of builder progress"
            ))
            .arg(arg!(
               --data ... "Prints out the path to the data directory"
            ))
            .arg(arg!(
                -c --complete ... "Attempt to complete a card"
            ))
            .get_matches();

        if let Some(n) = matches.get_one::<u8>("reset") {
            if *n != 0 {
                return Action::Reset;
            }
        }

        if let Some(n) = matches.get_one::<u8>("undo") {
            if *n != 0 {
                return Action::Undo;
            }
        }

        if let Some(&n) = matches.get_one::<u8>("data") {
            if n != 0 {
                return Action::Data;
            }
        }

        if let Some(&n) = matches.get_one::<u8>("complete") {
            if n != 0 {
                return Action::Complete;
            }
        }

        if let Some(add) = matches.get_one::<NoteType>("add") {
            Action::Add(*add)
        } else {
            Action::Copy
        }
    }

    pub fn undo(&self) {
        let mut state = self.read_file();

        match PROGRESS_ORDERING[state.progress] {
            Progress::Meaning => state.meaning = "".into(),
            Progress::Word => state.word = "".into(),
            Progress::Reading => state.reading = "".into(),
            Progress::Example => state.example = "".into(),
        }
        state.progress = state.progress.saturating_sub(1);

        self.write_file(state);
    }

    pub fn new() -> Self {
        let data_path = get_data_path();
        Self { data_path }
    }

    pub fn data(&self) {
        println!("{}", self.data_path.display());
    }

    fn add_card_anki(state: &mut State, note_type: NoteType) {
        let client = reqwest::blocking::Client::new();

        // also send a post request to the server with the current information.
        // then
        match note_type {
            NoteType::Basic => {
                let body = Body {
                    action: "guiAddCards".to_string(),
                    version: 6,
                    params: Params {
                        note: Note {
                            deck_name: "misc".to_string(),
                            model_name: "basic".to_string(),
                            fields: Fields::Basic {
                                front: state.meaning.clone(),
                                back: format!(
                                    "{}<br>{}<br><br>${}",
                                    state.word, state.reading, state.example
                                ),
                            },
                            tags: vec![format!("book")],
                        },
                    },
                };

                client
                    .post(ANKI_URL)
                    .body(serde_json::to_string(&body).unwrap())
                    .send().unwrap();
            }
            NoteType::Cloze => {
                let body = Body {
                    action: "guiAddCards".to_string(),
                    version: 6,
                    params: Params {
                        note: Note {
                            deck_name: "misc".to_string(),
                            model_name: "Cloze".to_string(),
                            fields: Fields::Cloze {
                                text: format!(
                                    "{}<br><br>{}",
                                    state.meaning,
                                    rep(&state.example, &state.word)
                                ),
                                back_extra: state.reading.clone(),
                            },
                            tags: vec![format!("book")],
                        },
                    },
                };
                // println!("{}", serde_json::to_string_pretty(&body).unwrap());
                client
                    .post(ANKI_URL)
                    .body(serde_json::to_string(&body).unwrap())
                    .send().unwrap();
            }
        }

        // always reset state
        *state = State::default();
    }

    pub fn read_file(&self) -> State {
        // dbg!(&self.data_path);
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.data_path)
            .unwrap();

        let mut buf = vec![];
        file.read_to_end(&mut buf).unwrap();

        let state: State = match serde_json::from_slice(&buf) {
            Ok(o) => o,
            Err(_) => {
                // just create a new one
                State::default()
            }
        };
        state
    }

    fn write_file(&self, state: State) {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.data_path)
            .unwrap();
        file.write_all(serde_json::to_string(&state).unwrap().as_bytes())
            .unwrap();
    }

    pub fn get_data_from_clipboard() -> DataType {
        let mut clipboard = Clipboard::new().expect("error creating clipboard context");
        DataType::from_clipboard(&mut clipboard).unwrap()
    }

    pub fn add(&self, note_type: NoteType, data_type: DataType) {
        let mut state = self.read_file();

        let mut s = parse_information(data_type);
        if PROGRESS_ORDERING[state.progress] == Progress::Example && s.lines().count() > 1 {
            let t = s
                .lines()
                .map(|s| s.to_string().replace('\t', ""))
                .collect::<Vec<String>>();
            s = t.join("<br><br>");
        }

        // run the main algorithm
        // dbg!(state.progress);
        match PROGRESS_ORDERING[state.progress] {
            Progress::Meaning => state.meaning = s,
            Progress::Word => state.word = s,
            Progress::Reading => state.reading = s,
            Progress::Example => state.example = s,
        }

        if state.progress as usize == 3 {
            Self::add_card_anki(&mut state, note_type);
        } else {
            state.progress = (state.progress + 1) % 4;
        }

        self.write_file(state);
    }

    pub fn reset(&self) {
        let state = State::default();
        self.write_file(state);
    }

    pub fn copy() {
        // if none, then we just run copy and return the stuffs
        let mut clipboard = Clipboard::new().expect("error creating clipboard context");

        let data_type = DataType::from_clipboard(&mut clipboard).unwrap();
        let s = parse_information(data_type);

        clipboard.set_text(s).unwrap();
    }
    pub fn complete(&self) {
        let mut state = self.read_file();
        // complete as cloze by default for now
        Self::add_card_anki(&mut state, NoteType::Cloze);
        self.write_file(state);
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Progress {
    Meaning,
    Word,
    Reading,
    Example,
}

pub const PROGRESS_ORDERING: [Progress; 4] = [
    Progress::Example,
    Progress::Word,
    Progress::Meaning,
    Progress::Reading,
];

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct State {
    pub meaning: String,
    pub word: String,
    pub reading: String,
    pub example: String,
    pub progress: usize,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Fields {
    // we're definitely not doing this as efficiently as we can lol
    Basic {
        #[serde(rename(serialize = "Front"))]
        front: String,
        #[serde(rename(serialize = "Back"))]
        back: String,
    },
    Cloze {
        #[serde(rename(serialize = "Text"))]
        text: String,
        #[serde(rename(serialize = "Back Extra"))]
        back_extra: String,
    },
}

#[derive(Debug, Serialize)]
struct Note {
    #[serde(rename(serialize = "deckName"))]
    deck_name: String,
    #[serde(rename(serialize = "modelName"))]
    model_name: String,
    fields: Fields,
    tags: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Params {
    note: Note,
}

#[derive(Debug, Serialize)]
struct Body {
    action: String,
    version: i32,
    params: Params,
}

fn get_data_path() -> PathBuf {
    let mut t = dirs::data_dir().unwrap();
    t.push("ripbk");
    // ensure directory exists
    std::fs::create_dir_all(&t).unwrap();
    t.push("data.json");
    t
}
