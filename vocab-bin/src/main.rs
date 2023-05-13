use clap::Parser;
use vocab_lib::{draw_stress, VocabWord};
use core::panic;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::collections::BTreeMap;
use std::io::{self};
use std::ops::Add;
use std::sync::mpsc::{self};
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

    #[arg(long)]
    double: bool
}

enum Event<I> {
    Input(I),
    Tick,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    if args.batch_size <= 2 {
        panic!("Batch size of less than 2 words? You make no sense dear.");
    }

    let vocab = if args.en {
        vocab_lib::get_en_vocabulary()
    } else {
        vocab_lib::get_ru_vocabulary()
    }?;

    let vocab_size = vocab.len();
    println!("Words in the dictionary: {:?}", vocab.len());

    println!("Batches amount: {}", vocab.len() / args.batch_size);

    let batch_vocab: BTreeMap<VocabWord, Vec<String>> = vocab
        .into_iter()
        .skip(args.batch_number * args.batch_size)
        .take(args.batch_size)
        .collect();

    if batch_vocab.len() == 0 {
        eprintln!("Sorry, no words in vocabulary in this range. Try batches of smaller size or batches with smaller indices.");
        return Ok(());
    }

    let mut keys: Vec<&VocabWord> = batch_vocab.keys().collect();
    if args.double {
        keys.append(&mut batch_vocab.keys().collect());
    }

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
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let step = if args.quiz { 1 } else { 2 };
    let mut index = 0;
    loop {
        let word_index = index / 2;

        let word = *keys.get(word_index).ok_or("must be in vocab")?;
        let drawn_word: String = draw_stress(&word.0);

        let translation: String = if args.quiz && index % 2 == 0 {
            String::default()
        } else {
            html2text::from_read(
                batch_vocab
                    .get(word)
                    .ok_or("must be in vocab")?
                    .get(0)
                    .ok_or("must be in vocab")?
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
                        Constraint::Percentage(50),
                        Constraint::Percentage(30),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let text = format!(" [{}/{}] \n\n {}\n", word_index + 1, keys.len(), drawn_word);
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

            let mut content: String =
                String::from(format!("\n Vocabulary size: {} words", vocab_size));
            content = content
                .add("\n")
                .add("\n Press:")
                .add("\n  <Enter> to see the next word")
                .add("\n  <Backspace> to see the previous word")
                .add("\n  q to exit");
            let help_widget = Paragraph::new(Text::styled(
                content,
                Style::default()
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
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}

// backend method for requesting several words
// ui to select and iterate through words

//todo wrap vocab in VocabIterator(type: VocabType{en | ru}, batch_number: int, batch_size, testing: boolean)
//todo add webasm wrapper
//todo call VocabIterator.next() from JS
//todo create full web UI
//todo deploy somewhere
//todo several words - several stress points


//todo better help section
//todo how to build compile
//todo project description
//todo installation/compliation description

//todo show both words in translation for en vocab
//todo better layout/formatting
//todo managing words
//todo proper errors handling
