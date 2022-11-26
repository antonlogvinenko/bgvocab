use clap::Parser;
use core::panic;
use std::process::exit;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, prelude::*, BufReader, Lines};
use std::ops::Add;
use std::sync::mpsc;
use std::thread::{self};
use std::time::{Duration, Instant};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Text};
use tui::widgets::{Block, Borders, Paragraph};
use tui::{backend::CrosstermBackend, Terminal};

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    batch_size: usize,

    #[arg(long)]
    batch_number: usize,

    #[arg(long)]
    en: bool,

    #[arg(long)]
    quiz: bool,
}

enum Event<I> {
    Input(I),
    Tick,
}

fn lines(path: &str) -> io::Result<Lines<BufReader<File>>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines())
}

fn add_to_vocabulary(vocab: &mut BTreeMap<String, Vec<String>>, str: &String) {
    //No, xml parsers make this code even worse
    //Dealing with a single-tag constant-length xml wrapper here
    let x1 = str.split_at(92).1;
    let pos = x1.find("\">").expect("Unparseable line");
    let x2 = x1.split_at(pos);
    let key = x2.0;
    let value = String::from(&(x2.1)[2..(x2.1.len() - 10)]);
    //remove stress
    let chill = key.replace('\u{0301}', "");
    vocab.entry(chill).or_insert(Vec::new()).push(value);
}

fn get_vocabulary() -> BTreeMap<String, Vec<String>> {
    let vocab_path = "./bg-en.xml";
    let mut vocabulary: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for line in lines(vocab_path).expect("Can't read vocabulary") {
        match line {
            Err(e) => {
                eprintln!("Failed to read line. {}", e);
                panic!("Vocabulary could not be read");
            }
            Ok(str) => add_to_vocabulary(&mut vocabulary, &str),
        }
    }

    //skip entries with names
    vocabulary.into_iter().skip(2287).collect()
}

fn get_vocabulary2() -> BTreeMap<String, Vec<String>> {
    let vocab_path = "./vocab.txt";
    let mut vocabulary: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut x = lines(vocab_path).expect("must");

    loop {
        let maybe_word = x.next();
        match maybe_word {
            Some(Ok(mut word)) => {
                let translation = x
                    .next()
                    .expect("translation must be present")
                    .expect("must");

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
            _ => return vocabulary,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    if args.batch_size <= 2 {
        panic!("Batch size of less than 2 words? You make no sense dear.");
    }

    let vocab = if args.en {
        get_vocabulary()
    } else {
        get_vocabulary2()
    };

    let vocab_size = vocab.len();
    println!("Words in the dictionary: {:?}", vocab.len());

    let mut x = String::from("aasd");
    x.insert_str(1, "\u{0301}");

    println!("Batches amount: {}", vocab.len() / args.batch_size);

    let batch_vocab: BTreeMap<String, Vec<String>> = vocab
        .into_iter()
        .skip(args.batch_number * args.batch_size)
        .take(args.batch_size)
        .collect();

    if batch_vocab.len() == 0 {
        eprintln!("Sorry, no words in vocabulary in this range. Try batches of smaller size or batches with smaller indices.");
        exit(0);
    }

    let keys: Vec<&String> = batch_vocab.keys().collect();

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    // println!(">>> Sending Event");
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    // println!(">>> Sending OK");
                    last_tick = Instant::now();
                }
            }
        }
    });

    enable_raw_mode().expect("must be able to run in raw mode");
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // let mut terminal = Terminal::with_options(
    //     backend,
    //     TerminalOptions {
    //         viewport: Viewport::fixed(Rect {x: 100, y: 0, width: 200, height:200})
    //     },
    // )?;
    terminal.clear()?;

    let step = if args.quiz { 1 } else { 2 };
    let mut index = 0;
    loop {
        let mut word_index = index / 2;

        let word = *keys.get(word_index).expect("must be in vocab");

        let translation: String = if args.quiz && index % 2 == 0 {
            String::default()
        } else {
            html2text::from_read(
                batch_vocab
                    .get(word)
                    .expect("must be in vocab")
                    .get(0)
                    .expect("must be in vocab")
                    .as_bytes(),
                100,
            )
        };

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(70),
                        Constraint::Percentage(20),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let text = format!(" [{}/{}] \n\n {}\n", word_index + 1, args.batch_size, word);
            let word_widget = Paragraph::new(Text::styled(
                text,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::default())
            .block(
                Block::default()
                    .title(Span::styled("", Style::default().fg(Color::White)))
                    .borders(Borders::ALL),
            );

            f.render_widget(word_widget, chunks[0]);

            let translation_widget = Paragraph::new(Text::styled(
                format!("\n {}", translation),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::default())
            .block(
                Block::default()
                    .title(Span::styled("", Style::default().fg(Color::White)))
                    .borders(Borders::ALL),
            );
            f.render_widget(translation_widget, chunks[1]);

            let mut content: String = String::from(format!("\n Vocabulary size: {} words", vocab_size));
            content = content
                .add("\n")
                .add("\n Press:")
                .add("\n  <Enter> to see the next word")
                .add("\n  <Backspace> to see the previous word")
                .add("\n  q to exit");
            let help_widget = Paragraph::new(Text::styled(
                content,
                Style::default()
                    // .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::default())
            .block(
                Block::default()
                    .title(Span::styled("Help", Style::default().fg(Color::White)))
                    .borders(Borders::ALL),
            );
            f.render_widget(help_widget, chunks[2]);
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    crossterm::execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    break;
                }
                KeyCode::Backspace => {
                    index = if index == 0 {
                        keys.len() * 2 - step
                    } else {
                        (index - step) % (keys.len() * 2)
                    };
                }
                KeyCode::Enter => {
                    index = (index + step) % (keys.len() * 2);
                }
                // KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                // KeyCode::Char('p') => active_menu_item = MenuItem::Pets,
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}


//todo better help section
//todo how to build compile
//todo project description
//todo installation/compliation description

//todo show both words in translation for en vocab
//todo better layout/formatting
//todo managing words
//todo proper errors handling