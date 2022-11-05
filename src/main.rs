use std::collections::HashMap;
use std::fs::File;
use std::io::{self, prelude::*, BufReader, Lines};

fn lines(path: &str) -> io::Result<Lines<BufReader<File>>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines())
}

fn add_to_vocabulary(vocab: &mut HashMap<String, Vec<String>>, str: &String) {
    //No, xml parser makes it even worse
    let x1 = str.split_at(92).1;
    let pos = x1.find("\">").expect("Unparseable line");
    let x2 = x1.split_at(pos);
    let key = x2.0;
    let value = String::from(&(x2.1)[2..(x2.1.len() - 10)]);

    println!("key: {}", key);

    //remove stress
    let chill = key.replace('\u{301}', "");

    vocab.entry(chill).or_insert(Vec::new()).push(value);
}

fn main() -> io::Result<()> {
    let vocab_path = "./bg-en.xml";

    let mut vocabulary: HashMap<String, Vec<String>> = HashMap::new();

    for line in lines(vocab_path)? {
        match line {
            Err(e) => {
                eprintln!("Failed to read line. {}", e);
                panic!("Vocabulary could not be read");
            }
            Ok(str) => add_to_vocabulary(&mut vocabulary, &str)
        }
    }

    println!("{:?}", vocabulary.iter().filter(|e| e.1.len() == 3).count());

    Ok(())
}
