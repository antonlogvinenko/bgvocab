use clap::Parser;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use core::panic;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, prelude::*, BufReader, Lines};
use std::sync::mpsc;
use std::thread::{self};
use std::time::{Duration, Instant};
use thiserror::Error;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Text};
use tui::widgets::{Block, BorderType, Borders, Paragraph};
use tui::{backend::CrosstermBackend, Terminal};

//todo 4 shift + enter for scrolling back
//todo 2 switching vocabularies
//todo 3 not showing translation key
//todo 2 show both words in translation
//todo better layout/formatting


//todo managing words

//todo better help section
//how to build compile

//todo handling errors
//todo project description
//todo installation/compliation description

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    batch_size: usize,

    #[arg(long)]
    batch_number: usize,
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
    // println!("===============");
    // println!("{}", html2text::from_read(value.as_bytes(), 100));
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    if args.batch_size <= 2 {
        panic!("Batch size of less than 2 words? You make no sense.");
    }

    let vocab = get_vocabulary();

    println!("Length {}", vocab.len());

    println!("Batches amount: {}", vocab.len() / args.batch_size);

    let batch_vocab: BTreeMap<String, Vec<String>> = vocab
        .into_iter()
        .skip(args.batch_number * args.batch_size)
        .take(args.batch_size)
        .collect();

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

    println!("{}", batch_vocab.iter().count());

    enable_raw_mode().expect("must be able to run in raw mode");
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut index = 0;

    loop {
        let word = *keys.get(index).expect("must be in vocab");
        let translation: String = html2text::from_read(
            batch_vocab
                .get(word)
                .expect("must be in vocab")
                .get(0)
                .expect("must be in vocab")
                .as_bytes(),
            100,
        );

        //todo add loop here
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(80),
                        Constraint::Percentage(10),
                    ]
                    .as_ref(),
                )
                .split(f.size());


            let text = format!(" [{}/{}]    {}", index + 1, args.batch_size, word);
            let wordWidget = Paragraph::new(Text::styled(
                text,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            ))
            .style(Style::default())
            .block(
                Block::default()
                    .title(Span::styled("The word", Style::default().fg(Color::White)))
                    .borders(Borders::ALL),
            );


            f.render_widget(wordWidget, chunks[0]);

            let translationWidget = Paragraph::new(translation).style(Style::default()).block(
                Block::default()
                    .title(Span::styled(
                        "The translation",
                        Style::default().fg(Color::White),
                    ))
                    .borders(Borders::ALL),
            );
            f.render_widget(translationWidget, chunks[1]);

            let helpWidget = Paragraph::new(Text::styled(
                "Press Enter for the next word; q to exit",
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
            f.render_widget(helpWidget, chunks[2]);
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
                KeyCode::Right | KeyCode::Enter => {
                    if event.modifiers.contains(KeyModifiers::SHIFT) {
                        index -= 1;
                    } else {
                        index += 1;
                    }
                    index = index % keys.len();
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
