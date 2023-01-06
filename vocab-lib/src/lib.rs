use std::{collections::BTreeMap, io::{BufReader, Lines, self, BufRead, ErrorKind}, fs::File, cmp::Ordering};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Eq, PartialEq, PartialOrd)]
pub struct VocabWord(pub String);

pub type Vocab = BTreeMap<VocabWord, Vec<String>>;
pub type VocabError = Box<dyn std::error::Error>;
// type VocabError = VocabErrorX;


impl Ord for VocabWord {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.to_lowercase().cmp(&other.0.to_lowercase())
    }
}

pub fn lines(path: &str) -> io::Result<Lines<BufReader<File>>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines())
}

pub fn add_to_vocabulary(vocab: &mut BTreeMap<VocabWord, Vec<String>>, str: &String) -> Result<(), VocabError> {
    //No, xml parsers make this code even worse
    //Dealing with a single-tag constant-length xml wrapper here
    let x1 = str.split_at(92).1;
    let pos = x1.find("\">").ok_or(std::io::Error::new(ErrorKind::NotFound, "Unparseable line"))?;
    let x2 = x1.split_at(pos);
    let key = x2.0;
    let value = String::from(&(x2.1)[2..(x2.1.len() - 10)]);
    //remove stress
    let chill = key.replace('\u{0301}', "");
    vocab.entry(VocabWord(chill)).or_insert(Vec::new()).push(value);
    Ok(())
}


pub fn get_en_vocabulary() -> Result<Vocab, VocabError> {
    let vocab_path = "../bg-en.xml";
    let mut vocabulary: BTreeMap<VocabWord, Vec<String>> = BTreeMap::new();
    for line in lines(vocab_path)? {
        match line {
            Err(e) => {
                eprintln!("Failed to read line. {}", e);
                return Err(std::io::Error::new(ErrorKind::NotFound, "Vocabulary could not be read"))?;
            }
            Ok(str) => add_to_vocabulary(&mut vocabulary, &str)?
        }
    }

    //skip entries with names
    Ok(vocabulary.into_iter().skip(2287).collect())
}

pub fn draw_stress(word: &String) -> String {
    let mut drawn = String::from(word);
    let indices: Vec<(usize, char)> = word.char_indices().collect();
    // .position(|(_, c)| c.is_uppercase());
    let p = indices.iter()
        .enumerate()
        .map(|(idx, (b, c))| (idx, c))
        .filter(|(_, c)| c.is_uppercase())
        .rev()
        .last();
    //todo how to replace a single char to lowercase?
    match p {
        Some(pos) => {
            // drawn = drawn.to_lowercase();
            let ins_at = if pos.0 == indices.len() - 1 {
                word.len()
            } else {
                indices.get(pos.0 + 1).unwrap().0
            };
            drawn.insert(ins_at, '\u{0301}');
        }
        None => {}
    }
    drawn
}

pub fn get_ru_vocabulary() -> Result<Vocab, VocabError> {
    let vocab_path = "../vocab.txt";
    let mut vocabulary: BTreeMap<VocabWord, Vec<String>> = BTreeMap::new();
    let mut x = lines(vocab_path)?;

    loop {
        let maybe_word = x.next();
        match maybe_word {
            Some(Ok(mut word)) => {
                let translation = x
                    .next()
                    .ok_or(std::io::Error::new(ErrorKind::NotFound, "translation must be present"))??;

                vocabulary.insert(VocabWord(word), vec![translation]);
                x.next();
            }
            _ => return Ok(vocabulary),
        }
    }
}