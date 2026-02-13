use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, Paragraph},
    Terminal,
};

const TICK_RATE: Duration = Duration::from_millis(16); // ~60 FPS
const BALL_CHARS: &[&str] = &["●", "◉", "○", "◎", "◆", "■", "▲", "★"];
const MAX_HISTORY: usize = 300;

const BALL_COLORS: &[Color] = &[
    Color::Yellow,
    Color::Green,
    Color::Red,
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
    Color::LightRed,
    Color::LightGreen,
];

const BALL_RADIUS: f64 = 0.75;

struct Ball {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    radius: f64,
    color: Color,
    char_idx: usize,
    x_history: Vec<(f64, f64)>,
    y_history: Vec<(f64, f64)>,
    vx_history: Vec<(f64, f64)>,
    vy_history: Vec<(f64, f64)>,
}

impl Ball {
    fn new(x: f64, y: f64, vx: f64, vy: f64, index: usize) -> Self {
        Ball {
            x,
            y,
            vx,
            vy,
            radius: BALL_RADIUS,
            color: BALL_COLORS[index % BALL_COLORS.len()],
            char_idx: index % BALL_CHARS.len(),
            x_history: Vec::new(),
            y_history: Vec::new(),
            vx_history: Vec::new(),
            vy_history: Vec::new(),
        }
    }
}

struct App {
    balls: Vec<Ball>,
    paused: bool,
    tick_count: u64,
    ball_counter: usize, // total balls ever created, for unique color/char assignment
    area_width: f64,
    area_height: f64,
    speed_multiplier: f64,
}

impl App {
    fn new() -> App {
        let mut app = App {
            balls: Vec::new(),
            paused: false,
            tick_count: 0,
            ball_counter: 0,
            area_width: 80.0,
            area_height: 20.0,
            speed_multiplier: 1.0,
        };
        app.add_ball();
        app
    }

    fn add_ball(&mut self) {
        // Vary initial position and velocity so balls don't overlap
        let idx = self.ball_counter;
        let x = 5.0 + (idx as f64 * 7.3) % self.area_width.max(20.0);
        let y = 3.0 + (idx as f64 * 4.1) % self.area_height.max(10.0);
        let vx = 0.5 + (idx as f64 * 0.17) % 0.8;
        let vy = 0.3 + (idx as f64 * 0.13) % 0.6;
        // Alternate directions
        let vx = if idx % 2 == 0 { vx } else { -vx };
        let vy = if idx % 3 == 0 { vy } else { -vy };

        self.balls.push(Ball::new(x, y, vx, vy, idx));
        self.ball_counter += 1;
    }

    fn remove_ball(&mut self) {
        if !self.balls.is_empty() {
            self.balls.pop();
        }
    }

    fn speed_up(&mut self) {
        self.speed_multiplier = (self.speed_multiplier + 0.25).min(5.0);
    }

    fn speed_down(&mut self) {
        self.speed_multiplier = (self.speed_multiplier - 0.25).max(0.25);
    }

    fn tick(&mut self) {
        if self.paused {
            return;
        }

        self.tick_count += 1;
        let t = self.tick_count as f64;

        // Update positions
        for ball in &mut self.balls {
            ball.x += ball.vx * self.speed_multiplier;
            ball.y += ball.vy * self.speed_multiplier;
        }

        // Ball-to-ball elastic collisions
        let n = self.balls.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = self.balls[j].x - self.balls[i].x;
                let dy = self.balls[j].y - self.balls[i].y;
                let dist_sq = dx * dx + dy * dy;
                let min_dist = self.balls[i].radius + self.balls[j].radius;

                if dist_sq < min_dist * min_dist && dist_sq > 0.0 {
                    let dist = dist_sq.sqrt();
                    // Collision normal
                    let nx = dx / dist;
                    let ny = dy / dist;

                    // Relative velocity along collision normal
                    let dvx = self.balls[i].vx - self.balls[j].vx;
                    let dvy = self.balls[i].vy - self.balls[j].vy;
                    let dvn = dvx * nx + dvy * ny;

                    // Only resolve if balls are moving toward each other
                    if dvn > 0.0 {
                        // Equal mass elastic collision: swap normal components
                        self.balls[i].vx -= dvn * nx;
                        self.balls[i].vy -= dvn * ny;
                        self.balls[j].vx += dvn * nx;
                        self.balls[j].vy += dvn * ny;
                    }

                    // Separate overlapping balls
                    let overlap = min_dist - dist;
                    let sep = overlap / 2.0 + 0.01;
                    self.balls[i].x -= sep * nx;
                    self.balls[i].y -= sep * ny;
                    self.balls[j].x += sep * nx;
                    self.balls[j].y += sep * ny;
                }
            }
        }

        // Wall bounces and history recording
        let w = self.area_width;
        let h = self.area_height;
        for ball in &mut self.balls {
            if ball.x <= 0.0 {
                ball.x = 0.0;
                ball.vx = ball.vx.abs();
            }
            if ball.x >= w - 1.0 {
                ball.x = w - 1.0;
                ball.vx = -ball.vx.abs();
            }
            if ball.y <= 0.0 {
                ball.y = 0.0;
                ball.vy = ball.vy.abs();
            }
            if ball.y >= h - 1.0 {
                ball.y = h - 1.0;
                ball.vy = -ball.vy.abs();
            }

            ball.x_history.push((t, ball.x));
            ball.y_history.push((t, ball.y));
            ball.vx_history.push((t, ball.vx));
            ball.vy_history.push((t, ball.vy));

            if ball.x_history.len() > MAX_HISTORY {
                ball.x_history.remove(0);
            }
            if ball.y_history.len() > MAX_HISTORY {
                ball.y_history.remove(0);
            }
            if ball.vx_history.len() > MAX_HISTORY {
                ball.vx_history.remove(0);
            }
            if ball.vy_history.len() > MAX_HISTORY {
                ball.vy_history.remove(0);
            }
        }
    }
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), io::Error> {
    let mut app = App::new();
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = TICK_RATE
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char(' ') | KeyCode::Char('p') => {
                        app.paused = !app.paused;
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') | KeyCode::Char('a') => {
                        app.add_ball();
                    }
                    KeyCode::Char('-') | KeyCode::Char('_') | KeyCode::Char('r') => {
                        app.remove_ball();
                    }
                    KeyCode::Up => {
                        app.speed_up();
                    }
                    KeyCode::Down => {
                        app.speed_down();
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= TICK_RATE {
            app.tick();
            last_tick = Instant::now();
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &mut App) {
    let size = f.area();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(size);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(20),
            Constraint::Length(32),
        ])
        .split(main_chunks[0]);

    let mid_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(main_chunks[1]);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(main_chunks[2]);

    let ball_area = top_chunks[0];
    let inner_width = if ball_area.width > 2 { ball_area.width - 2 } else { 1 };
    let inner_height = if ball_area.height > 2 { ball_area.height - 2 } else { 1 };
    app.area_width = inner_width as f64;
    app.area_height = inner_height as f64;

    draw_ball_arena(f, app, ball_area);
    draw_status(f, app, top_chunks[1]);
    draw_x_graph(f, app, mid_chunks[0]);
    draw_y_graph(f, app, mid_chunks[1]);
    draw_vx_graph(f, app, bottom_chunks[0]);
    draw_vy_graph(f, app, bottom_chunks[1]);
}

fn draw_ball_arena(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(format!(" Ball Arena ({} balls) ", app.balls.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    for ball in &app.balls {
        let bx = ball.x.round() as u16;
        let by = ball.y.round() as u16;

        if bx < inner.width && by < inner.height {
            let ball_rect = Rect::new(inner.x + bx, inner.y + by, 1, 1);
            let ball_widget = Paragraph::new(BALL_CHARS[ball.char_idx])
                .style(Style::default().fg(ball.color).add_modifier(Modifier::BOLD));
            f.render_widget(ball_widget, ball_rect);
        }
    }
}

fn draw_status(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let status = if app.paused { "⏸  PAUSED" } else { "▶  RUNNING" };
    let status_color = if app.paused { Color::Red } else { Color::Green };

    let mut text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Status: ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(status, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  Balls:  ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{}", app.balls.len()), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  Speed:  ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:.2}x", app.speed_multiplier), Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];

    // Show info for up to 4 balls
    for (_i, ball) in app.balls.iter().enumerate().take(4) {
        text.push(Line::from(vec![
            Span::styled(
                format!("  {} ", BALL_CHARS[ball.char_idx]),
                Style::default().fg(ball.color),
            ),
            Span::styled(
                format!("x:{:.0} y:{:.0}", ball.x, ball.y),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }
    if app.balls.len() > 4 {
        text.push(Line::from(Span::styled(
            format!("  ... +{} more", app.balls.len() - 4),
            Style::default().fg(Color::DarkGray),
        )));
    }

    text.push(Line::from(""));
    text.push(Line::from(Span::styled(
        "  ────────────────────────",
        Style::default().fg(Color::DarkGray),
    )));
    text.push(Line::from(""));
    text.push(Line::from(Span::styled(
        "  [Space/P]  Pause/Start",
        Style::default().fg(Color::Yellow),
    )));
    text.push(Line::from(Span::styled(
        "  [+/A]      Add ball",
        Style::default().fg(Color::Green),
    )));
    text.push(Line::from(Span::styled(
        "  [-/R]      Remove ball",
        Style::default().fg(Color::Red),
    )));
    text.push(Line::from(Span::styled(
        "  [↑]        Speed up",
        Style::default().fg(Color::LightGreen),
    )));
    text.push(Line::from(Span::styled(
        "  [↓]        Speed down",
        Style::default().fg(Color::LightRed),
    )));
    text.push(Line::from(Span::styled(
        "  [Q/Esc]    Quit",
        Style::default().fg(Color::Yellow),
    )));

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .title(" Controls ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(paragraph, area);
}

fn draw_x_graph(f: &mut ratatui::Frame, app: &App, area: Rect) {
    // Compute global time bounds
    let (t_min, t_max) = global_time_bounds(app);
    let x_max = app.area_width.max(1.0);

    let datasets: Vec<Dataset> = app
        .balls
        .iter()
        .enumerate()
        .map(|(i, ball)| {
            Dataset::default()
                .name(format!("B{}", i + 1))
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(ball.color))
                .data(&ball.x_history)
        })
        .collect();

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(" X Position Over Time ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .x_axis(
            Axis::default()
                .title("Time")
                .style(Style::default().fg(Color::Gray))
                .bounds([t_min, t_max])
                .labels(vec![
                    Span::raw(format!("{:.0}", t_min)),
                    Span::raw(format!("{:.0}", t_max)),
                ]),
        )
        .y_axis(
            Axis::default()
                .title("X")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, x_max])
                .labels(vec![
                    Span::raw("0"),
                    Span::raw(format!("{:.0}", x_max)),
                ]),
        );

    f.render_widget(chart, area);
}

fn draw_y_graph(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let (t_min, t_max) = global_time_bounds(app);
    let y_max = app.area_height.max(1.0);

    let datasets: Vec<Dataset> = app
        .balls
        .iter()
        .enumerate()
        .map(|(i, ball)| {
            Dataset::default()
                .name(format!("B{}", i + 1))
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(ball.color))
                .data(&ball.y_history)
        })
        .collect();

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(" Y Position Over Time ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        )
        .x_axis(
            Axis::default()
                .title("Time")
                .style(Style::default().fg(Color::Gray))
                .bounds([t_min, t_max])
                .labels(vec![
                    Span::raw(format!("{:.0}", t_min)),
                    Span::raw(format!("{:.0}", t_max)),
                ]),
        )
        .y_axis(
            Axis::default()
                .title("Y")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, y_max])
                .labels(vec![
                    Span::raw("0"),
                    Span::raw(format!("{:.0}", y_max)),
                ]),
        );

    f.render_widget(chart, area);
}

fn draw_vx_graph(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let (t_min, t_max) = global_time_bounds(app);
    let (v_min, v_max) = velocity_bounds(app, true);

    let datasets: Vec<Dataset> = app
        .balls
        .iter()
        .enumerate()
        .map(|(i, ball)| {
            Dataset::default()
                .name(format!("B{}", i + 1))
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(ball.color))
                .data(&ball.vx_history)
        })
        .collect();

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(" X Velocity Over Time ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightCyan)),
        )
        .x_axis(
            Axis::default()
                .title("Time")
                .style(Style::default().fg(Color::Gray))
                .bounds([t_min, t_max])
                .labels(vec![
                    Span::raw(format!("{:.0}", t_min)),
                    Span::raw(format!("{:.0}", t_max)),
                ]),
        )
        .y_axis(
            Axis::default()
                .title("Vx")
                .style(Style::default().fg(Color::Gray))
                .bounds([v_min, v_max])
                .labels(vec![
                    Span::raw(format!("{:.1}", v_min)),
                    Span::raw("0"),
                    Span::raw(format!("{:.1}", v_max)),
                ]),
        );

    f.render_widget(chart, area);
}

fn draw_vy_graph(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let (t_min, t_max) = global_time_bounds(app);
    let (v_min, v_max) = velocity_bounds(app, false);

    let datasets: Vec<Dataset> = app
        .balls
        .iter()
        .enumerate()
        .map(|(i, ball)| {
            Dataset::default()
                .name(format!("B{}", i + 1))
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(ball.color))
                .data(&ball.vy_history)
        })
        .collect();

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(" Y Velocity Over Time ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightMagenta)),
        )
        .x_axis(
            Axis::default()
                .title("Time")
                .style(Style::default().fg(Color::Gray))
                .bounds([t_min, t_max])
                .labels(vec![
                    Span::raw(format!("{:.0}", t_min)),
                    Span::raw(format!("{:.0}", t_max)),
                ]),
        )
        .y_axis(
            Axis::default()
                .title("Vy")
                .style(Style::default().fg(Color::Gray))
                .bounds([v_min, v_max])
                .labels(vec![
                    Span::raw(format!("{:.1}", v_min)),
                    Span::raw("0"),
                    Span::raw(format!("{:.1}", v_max)),
                ]),
        );

    f.render_widget(chart, area);
}

fn velocity_bounds(app: &App, is_x: bool) -> (f64, f64) {
    let mut v_min = f64::MAX;
    let mut v_max = f64::MIN;

    for ball in &app.balls {
        let history = if is_x { &ball.vx_history } else { &ball.vy_history };
        for &(_, v) in history {
            if v < v_min { v_min = v; }
            if v > v_max { v_max = v; }
        }
    }

    if v_min >= v_max {
        (-1.0, 1.0)
    } else {
        // Add a small margin
        let margin = (v_max - v_min) * 0.1;
        (v_min - margin, v_max + margin)
    }
}

fn global_time_bounds(app: &App) -> (f64, f64) {
    let mut t_min = f64::MAX;
    let mut t_max = f64::MIN;

    for ball in &app.balls {
        if let Some(first) = ball.x_history.first() {
            t_min = t_min.min(first.0);
        }
        if let Some(last) = ball.x_history.last() {
            t_max = t_max.max(last.0);
        }
    }

    if t_min >= t_max {
        (0.0, 1.0)
    } else {
        (t_min, t_max)
    }
}
