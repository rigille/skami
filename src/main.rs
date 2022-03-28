use std::{io, fmt, thread, time::Duration};
use tui::{
    backend::CrosstermBackend,
    style::Style,
    text::{Span, Spans},
    widgets::{Widget, Block, Borders, Paragraph},
    layout::{Layout, Constraint, Direction},
    Terminal
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEvent, KeyCode, poll, read},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use hvm::language as lang;
use im::Vector;

#[derive(Clone)]
struct State {
    mode: Mode,
    input: String,
    stack: Vector<StackEntry>,
}

#[derive(Clone)]
enum StackEntry {
    Rule(lang::Rule),
    Term(lang::Term),
}

#[derive(Clone, PartialEq)]
enum Mode {
    Normal,
    Insert,
    Exit,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Self::Normal => "Mode::Normal",
                Self::Insert => "Mode::Insert",
                Self::Exit => "Mode::Exit",
            }
        )
    }
}

impl fmt::Display for StackEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Self::Term(val) => val.to_string(),
                Self::Rule(val) => val.to_string(),
            }
        )
    }
}

fn normal_update(st: State, event: Event) -> State {
    if let Event::Key(key_event) = event {
        match key_event.code {
            KeyCode::Char(c) => {
                match c {
                    'i' =>
                        State { mode: Mode::Insert, ..st },
                    'q' =>
                        State { mode: Mode::Exit, ..st },
                    'a' => {
                        let mut stack = st.stack.clone();
                        let fst = stack.pop_back();
                        let snd = stack.pop_back();
                        let stack =
                            match (fst, snd) {
                                (Some(StackEntry::Term(func)), Some(StackEntry::Term(argm))) => {
                                    let func = Box::new(func);
                                    let argm = Box::new(argm);
                                    stack.push_back(
                                        StackEntry::Term(lang::Term::App {
                                            func,
                                            argm,
                                        }));
                                    stack
                                },
                                _ => st.stack
                            };
                        State { stack, ..st }},
                    _ => st,
                }
            },
            _ => st,
        }
    } else {
        st
    }
}

fn insert_update(st: State, event: Event) -> State {
    if let Event::Key(key_event) = event {
        match key_event.code {
            KeyCode::Char(c) => {
                let mut input = st.input;
                input.push(c);
                State { input, ..st }
            },
            KeyCode::Backspace => {
                let mut input = st.input;
                input.pop();
                State { input, ..st }
            },
            KeyCode::Enter => {
                if let Ok(term) = lang::read_term(&st.input) {
                    let mut stack = st.stack.clone();
                    stack.push_back(StackEntry::Term(*term));
                    let mut input = st.input;
                    input.clear();
                    State { input, stack, ..st }
                } else {
                    st
                }
            },
            _ => st,
        }
    } else {
        st
    }
}

fn update(st: State, event: Event) -> State {
    match (&st.mode, event) {
        (_, Event::Key(KeyEvent { code: KeyCode::Esc, .. })) =>
            State { mode: Mode::Normal, ..st },
        (Mode::Normal, _) => normal_update(st, event),
        (Mode::Insert, _) => insert_update(st, event),
        _ => st,
    }
}

fn render(st: &State) -> impl Widget {
    let mut v = Vec::new();
    for term in &st.stack {
        v.push(Spans::from(Span::raw(term.to_string())));
    }
    v.push(Spans::from(Span::raw(st.input.clone())));
    v.push(Spans::from(Span::raw(st.mode.to_string())));
    Paragraph::new(v)
}


fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;
    let mut st = State { mode: Mode::Insert, input: String::new(), stack: Vector::new() };

    let widget = render(&st);
    terminal.draw(|f| {
        let size = f.size();
        f.render_widget(widget, size);
    })?;
    loop {
        if poll(Duration::from_millis(50))? {
            let event = read()?;
            st = update(st, event);
            let widget = render(&st);
            terminal.draw(|f| {
                let size = f.size();
                f.render_widget(widget, size);
            })?;
            if st.mode == Mode::Exit {
                break;
            }
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
