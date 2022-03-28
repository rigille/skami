use std::{io, thread, time::Duration};
use tui::{
    backend::CrosstermBackend,
    style::Style,
    text::{Span, Spans},
    widgets::{Widget, Block, Borders, Paragraph},
    layout::{Layout, Constraint, Direction},
    Terminal
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, poll, read},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

#[derive(PartialEq)]
struct State {
    mode: Mode,
    input: String
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    Input,
    Exit,
}

fn update_input(mut input: String, event: Event) -> String {
    match event {
        Event::Key(key_event) =>
            match key_event.code {
                KeyCode::Char(c) => {
                    input.push(c);
                    input
                },
                KeyCode::Backspace => {
                  input.pop();
                  input
                },
                _ => input,
            }
        _ => input,
    }
}

fn update_state(st: State, event: Event) -> State {
    if event == Event::Key(KeyCode::Esc.into()) { 
        State {
            mode: Mode::Exit,
            ..st
        }
    } else {
        match st.mode {
            Mode::Input => State {
                input: update_input(st.input, event),
                ..st
            },
            _ => st
        }
    }
}

fn render_state(st: &State) -> impl Widget {
    Paragraph::new(vec![Spans::from(Span::raw(st.input.clone()))])
}


fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;
    let mut st = State { mode: Mode::Input, input: String::new() };

    loop {
        if poll(Duration::from_millis(50))? {
            let event = read()?;
            st = update_state(st, event);
            terminal.draw(|f| {
                let size = f.size();
                let widget = render_state(&st);
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
