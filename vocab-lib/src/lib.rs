use std::{collections::BTreeMap, io::{BufReader, Lines, self, BufRead, ErrorKind}, fs::File};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub type Vocab = BTreeMap<String, Vec<String>>;
pub type VocabError = Box<dyn std::error::Error>;
// type VocabError = VocabErrorX;

pub fn lines(path: &str) -> io::Result<Lines<BufReader<File>>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines())
}

pub fn add_to_vocabulary(vocab: &mut BTreeMap<String, Vec<String>>, str: &String) -> Result<(), VocabError> {
    //No, xml parsers make this code even worse
    //Dealing with a single-tag constant-length xml wrapper here
    let x1 = str.split_at(92).1;
    let pos = x1.find("\">").ok_or(std::io::Error::new(ErrorKind::NotFound, "Unparseable line"))?;
    let x2 = x1.split_at(pos);
    let key = x2.0;
    let value = String::from(&(x2.1)[2..(x2.1.len() - 10)]);
    //remove stress
    let chill = key.replace('\u{0301}', "");
    vocab.entry(chill).or_insert(Vec::new()).push(value);
    Ok(())
}


pub fn get_en_vocabulary() -> Result<Vocab, VocabError> {
    let vocab_path = "../bg-en.xml";
    let mut vocabulary: BTreeMap<String, Vec<String>> = BTreeMap::new();
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

pub fn get_ru_vocabulary() -> Result<Vocab, VocabError> {
    let vocab_path = "../vocab.txt";
    let mut vocabulary: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut x = lines(vocab_path)?;

    loop {
        let maybe_word = x.next();
        match maybe_word {
            Some(Ok(mut word)) => {
                let translation = x
                    .next()
                    .ok_or(std::io::Error::new(ErrorKind::NotFound, "translation must be present"))??;

                let indices: Vec<(usize, char)> = word.char_indices().collect();
                let p = indices.iter().position(|(_, c)| c.is_uppercase());
                match p {
                    Some(pos) => {
                        word = word.to_lowercase();
                        let ins_at = if pos == indices.len() - 1 {
                            word.len()
                        } else {
                            indices.get(pos + 1).unwrap().0
                        };
                        word.insert(ins_at, '\u{0301}');
                    }
                    None => {}
                }

                vocabulary.insert(word, vec![translation]);
                x.next();
            }
            _ => return Ok(vocabulary),
        }
    }
}