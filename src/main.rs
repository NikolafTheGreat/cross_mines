use crossterm::{terminal, cursor, style, style::Color, execute};
use crossterm::event::{self, Event, MouseEventKind, MouseButton, KeyCode};
use std::{io, env};
use rand::{thread_rng, Rng};

const NUMBERS: [&str; 9] = ["  ", "󰲠 ","󰲢 ","󰲤 ","󰲦 ","󰲨 ","󰲪 ","󰲬 ","󰲮 "];
const COLORS: [Color; 9] = [
    Color::White,
    Color::Blue,
    Color::Green,
    Color::Red,
    Color::DarkBlue,
    Color::DarkRed,
    Color::Cyan,
    Color::White,
    Color::Grey
];

#[derive(Clone, Copy)]
struct FieldCell {
    is_mine: bool,
    is_flagged: bool,
    is_uncovered: bool,
    neighbors: usize,
    flagged_neighbors: isize
}

impl FieldCell {
    fn new() -> Self {
        Self {
            is_mine: false,
            is_flagged: false,
            is_uncovered: false,
            neighbors: 0,
            flagged_neighbors: 0
        }
    }
}

fn main() -> io::Result<()> {
    let (window_width, window_height) = terminal::size()?;

    let mut args = env::args();
    args.next();
    let (width, height, mines) = match args.len() {
        0 => {
            let width = (window_width as usize / 2) & !1;
            let height = (window_height as usize) & !1;
            let mines = (width * height) / 5;
            (width, height, mines)
        },
        1 => {
            let width = (window_width as usize / 2) & !1;
            let height = (window_height as usize) & !1;
            let mines = args.next().unwrap().trim().parse()
                .expect("Please provide the number of mines as integers");
            (width, height, mines)
        },
        2 => {
            let width: usize = args.next()
                .unwrap().trim().parse()
                .expect("Please provide the width and height as integers");
            let height: usize = args.next()
                .unwrap().trim().parse()
                .expect("Please provide the width and height as integers");
            let mines = (width * height) / 5;
            (width, height, mines)
        },
        3 => {
            let width: usize = args.next()
                .unwrap().trim().parse()
                .expect("Please provide the width and height as integers");
            let height: usize = args.next()
                .unwrap().trim().parse()
                .expect("Please provide the width and height as integers");
            let mines: usize = args.next()
                .unwrap().trim().parse()
                .expect("Please provide the number of mines as integers");
            (width, height, mines)
        },
        _ => {
            panic!("Invalid number of arguments supplied. Got {} but expected 0 1 or 3", args.len())
        }
    };

    if mines >= width * height {
        panic!("Please choose a number of mines that fits on the board.");
    }
    if width * 2 > window_width as usize || height > window_height as usize {
        panic!("Please provide a width and height that \
            fits within the current terminal frame. \
            \nThe current terminal frame can at max fit: ({}, {})",
            window_width / 2, window_height
        );
    }

    let mut stdout = io::stdout();
    
    terminal::enable_raw_mode()?;
    execute!(stdout,
        terminal::EnterAlternateScreen,
        cursor::Hide,
        event::EnableMouseCapture,
    )?;

    'games: loop {
        execute!(stdout, style::SetForegroundColor(COLORS[0]))?;
        for y in 0..height {
            for x in 0..width {
                execute!(stdout,
                    cursor::MoveTo(x as u16 * 2, y as u16),
                    style::Print(" ")
                )?;
            }
        }

        let (first_x, first_y) = loop {
            let first_event = event::read()?;
            if let Event::Mouse(info) = first_event {
                if let MouseEventKind::Down(MouseButton::Left) = info.kind {
                    let x = info.column as usize / 2;
                    let y = info.row as usize;
                    if x >= width || y >= height {continue;}
                    break (info.column as usize / 2, info.row as usize);
                }
            }
            if let Event::Key(key_event) = first_event {
                if let KeyCode::Char('q') = key_event.code {
                    break 'games;
                }
            }
        };

        let mut minefield = generate_board(
            mines,
            first_x,
            first_y,
            width,
            height
        );

        let mut covered = width * height;

        covered -= reveal(
            &mut stdout,
            &mut minefield,
            first_x as isize,
            first_y as isize,
            width,
            height,
            true
        )?.unwrap();

        let won = 'game: loop {
            if covered == mines {
                break 'game true;
            }
            match event::read()? {
                Event::Key(key_event) => {
                    if let KeyCode::Char('q') = key_event.code {
                        break 'games;
                    }
                    if let KeyCode::Char('r') = key_event.code {
                        continue 'games;
                    }
                },
                Event::Mouse(mouse_event) => {
                    if let MouseEventKind::Down(button) = mouse_event.kind {
                        match button {
                            MouseButton::Left => {
                                if let Some(revealed) = reveal(
                                    &mut stdout,
                                    &mut minefield,
                                    mouse_event.column as isize / 2,
                                    mouse_event.row as isize,
                                    width,
                                    height,
                                    true
                                )? {
                                    covered -= revealed;
                                } else {
                                    break 'game false;
                                }
                            },
                            MouseButton::Right => flag(
                                &mut stdout,
                                &mut minefield,
                                mouse_event.column as usize / 2,
                                mouse_event.row as usize,
                                width,
                                height
                            )?,
                            _ => ()
                        }
                    }
                },
                _ => ()
            }
        };

        let (message, color) = if won {
            ("  YOU WIN!  ", Color::Green)
        } else {
            (" GAME OVER! ", Color::Red)
        };
        let y_offset = if height == 1 { 1 } else { 0 };
        execute!(stdout,
            style::SetForegroundColor(color),
            cursor::MoveTo((width * 2).saturating_sub(message.len()).saturating_sub(4) as u16 / 2, (height + y_offset) as u16 / 2 - 1),
            style::Print(" ╭"),
            style::Print("─".repeat(message.len())),
            style::Print("╮ "),
            cursor::MoveTo((width * 2).saturating_sub(message.len()).saturating_sub(4) as u16 / 2, (height + y_offset) as u16 / 2),
            style::Print(" │"),
            style::Print(message),
            style::Print("│ "),
            cursor::MoveTo((width * 2).saturating_sub(message.len()).saturating_sub(4) as u16 / 2, (height + y_offset) as u16 / 2 + 1),
            style::Print(" ╰"),
            style::Print("─".repeat(message.len())),
            style::Print("╯ "),
        )?;
        loop {
            if let Event::Key(key_event) = event::read()? {
                if let KeyCode::Char('q') = key_event.code {
                    break 'games;
                }
                if let KeyCode::Char('r') = key_event.code {
                    break;
                }
            }
        }
    }

    terminal::disable_raw_mode()?;
    execute!(stdout,
        terminal::LeaveAlternateScreen,
        cursor::Show,
        event::DisableMouseCapture
    )?;

    Ok(())
}

fn generate_board(
    mines: usize,
    first_x: usize,
    first_y: usize,
    width: usize,
    height: usize
) -> Vec<Vec<FieldCell>> {
    let mut minefield = vec![vec![FieldCell::new(); height]; width];
    let mut rng = thread_rng();
    for _ in 0..mines {
        let mut x = rng.gen_range(0..width);
        let mut y = rng.gen_range(0..height);
        while minefield[x][y].is_mine || (x.abs_diff(first_x) <= 1 && y.abs_diff(first_y) <= 1) {
            x = rng.gen_range(0..width);
            y = rng.gen_range(0..height);
        }
        minefield[x][y].is_mine = true;
    }

    for y in 0..height {
        for x in 0..width {
            let w = width - 1;
            let h = height - 1;
            if x != 0 && y != 0 && minefield[x - 1][y - 1].is_mine {minefield[x][y].neighbors += 1}
            if x != 0 && y != h && minefield[x - 1][y + 1].is_mine {minefield[x][y].neighbors += 1}
            if x != w && y != 0 && minefield[x + 1][y - 1].is_mine {minefield[x][y].neighbors += 1}
            if x != w && y != h && minefield[x + 1][y + 1].is_mine {minefield[x][y].neighbors += 1}
            if x != 0 && minefield[x - 1][y].is_mine {minefield[x][y].neighbors += 1}
            if x != w && minefield[x + 1][y].is_mine {minefield[x][y].neighbors += 1}
            if y != 0 && minefield[x][y - 1].is_mine {minefield[x][y].neighbors += 1}
            if y != h && minefield[x][y + 1].is_mine {minefield[x][y].neighbors += 1}
        }
    }

    minefield
}

fn reveal(
    stdout: &mut io::Stdout, 
    minefield: &mut Vec<Vec<FieldCell>>,
    x_s: isize,
    y_s: isize,
    width: usize,
    height: usize,
    clicked: bool
) -> io::Result<Option<usize>> {
    if x_s < 0 || y_s < 0 {return Ok(Some(0))}
    let x = x_s as usize;
    let y = y_s as usize;
    if x >= width || y >= height {return Ok(Some(0))}

    if minefield[x][y].is_flagged {return Ok(Some(0))}
    if minefield[x][y].is_mine {
        execute!(stdout,
            cursor::MoveTo(x as u16 * 2, y as u16),
            style::SetForegroundColor(Color::Red),
            style::Print("󰷚 "),
        )?;

        return Ok(None)
    }
    if minefield[x][y].is_uncovered && !clicked {return Ok(Some(0))}

    let mut return_value = if minefield[x][y].is_uncovered { 0 } else { 1 };

    minefield[x][y].is_uncovered = true;
    execute!(stdout,
        cursor::MoveTo(x as u16 * 2, y as u16),
        style::SetForegroundColor(COLORS[minefield[x][y].neighbors]),
        style::Print(NUMBERS[minefield[x][y].neighbors])
    )?;

    if minefield[x][y].neighbors == minefield[x][y].flagged_neighbors as usize {
        return_value += if let Some(v) = reveal(stdout, minefield, x_s - 1, y_s - 1, width, height, false)? {v} else {return Ok(None)};
        return_value += if let Some(v) = reveal(stdout, minefield, x_s - 1, y_s + 1, width, height, false)? {v} else {return Ok(None)};
        return_value += if let Some(v) = reveal(stdout, minefield, x_s + 1, y_s - 1, width, height, false)? {v} else {return Ok(None)};
        return_value += if let Some(v) = reveal(stdout, minefield, x_s + 1, y_s + 1, width, height, false)? {v} else {return Ok(None)};
        return_value += if let Some(v) = reveal(stdout, minefield, x_s - 1, y_s, width, height, false)? {v} else {return Ok(None)};
        return_value += if let Some(v) = reveal(stdout, minefield, x_s + 1, y_s, width, height, false)? {v} else {return Ok(None)};
        return_value += if let Some(v) = reveal(stdout, minefield, x_s, y_s - 1, width, height, false)? {v} else {return Ok(None)};
        return_value += if let Some(v) = reveal(stdout, minefield, x_s, y_s + 1, width, height, false)? {v} else {return Ok(None)};
    }

    Ok(Some(return_value))
}

fn flag(
    stdout: &mut io::Stdout, 
    minefield: &mut Vec<Vec<FieldCell>>,
    x: usize,
    y: usize,
    width: usize,
    height: usize
) -> io::Result<()> {
    if x >= width || y >= height {return Ok(())}


    if minefield[x][y].is_uncovered {return Ok(())}

    minefield[x][y].is_flagged = !minefield[x][y].is_flagged;
    let diff = if minefield[x][y].is_flagged {1} else {-1};
    {
        let w = width - 1;
        let h = height - 1;
        if x != 0 && y != 0 {minefield[x - 1][y - 1].flagged_neighbors += diff}
        if x != 0 && y != h {minefield[x - 1][y + 1].flagged_neighbors += diff}
        if x != w && y != 0 {minefield[x + 1][y - 1].flagged_neighbors += diff}
        if x != w && y != h {minefield[x + 1][y + 1].flagged_neighbors += diff}
        if x != 0 {minefield[x - 1][y].flagged_neighbors += diff}
        if x != w {minefield[x + 1][y].flagged_neighbors += diff}
        if y != 0 {minefield[x][y - 1].flagged_neighbors += diff}
        if y != h {minefield[x][y + 1].flagged_neighbors += diff}
    }
    execute!(stdout,
        style::SetForegroundColor(COLORS[0]),
        cursor::MoveTo(x as u16 * 2, y as u16)
    )?;
    if minefield[x][y].is_flagged {
        execute!(stdout, style::Print(" "))?;
    } else {
        execute!(stdout, style::Print(" "))?;
    }
    Ok(())
}
