use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, prelude::*, BufReader, Lines};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, BorderType, Borders, Paragraph};
use tui::{backend::CrosstermBackend, Terminal};

//todo handling errors
//todo project description
//todo installation/compliation description

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
    let vocab = get_vocabulary();

    let batch: usize = 0;
    let batch_size = 100;

    println!("Length {}", vocab.len());

    println!("Batches amount: {}", vocab.len() / batch_size);

    let batch_vocab: BTreeMap<String, Vec<String>> = vocab
        .into_iter()
        .skip(batch * batch_size)
        .take(batch_size)
        .collect();

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

    loop {
        //todo add loop here
        terminal.draw(|f| {
            let copyright = Paragraph::new("pet-CLI 2020 - all rights reserved")
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Copyright")
                        .border_type(BorderType::Plain),
                );
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);
            f.render_widget(copyright, chunks[2]);

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
            let word = Block::default().title(" Word ").borders(Borders::ALL);
            // f.render_widget(word, chunks[0]);
            let translation = Block::default()
                .title(" Translation ")
                .borders(Borders::ALL);
            // f.render_widget(translation, chunks[1]);
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
                // KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                // KeyCode::Char('p') => active_menu_item = MenuItem::Pets,
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}
