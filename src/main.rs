use std::time::{Duration, Instant};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::widgets::{Paragraph, Block, BorderType};
use ratatui::{DefaultTerminal, Frame};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use sysinfo::{MINIMUM_CPU_UPDATE_INTERVAL, System};

const TICK_RATE: Duration = Duration::from_millis(500);
const MUTED: Color = Color::DarkGray;
const ACCENT: Color = Color::LightBlue;

struct App {
    running: bool,
    system: System,
    host_name: String,
    os_name: String,
}

fn main() -> std::io::Result<()> {
    ratatui::run(|terminal| App::new().run(terminal))
}


impl App {
    fn new() -> Self {
        let mut system = System::new();
        std::thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
        system.refresh_cpu_usage();

        let mut App = Self {
            running: true,
            system,
            // host_name: System::host_name().unwrap_or_else(|| "unknown".into()),
            host_name: "Bofulo".to_string(),
            os_name: System::long_os_version().unwrap_or_else(|| "unknown OS".into())
        };

        App.on_tick();
        App
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        let mut last_tick = Instant::now();

        while self.running {
            terminal.draw(|frame| self.render(frame))?;

            let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key);
                }
            }

            if last_tick.elapsed() >= TICK_RATE {
                self.on_tick();
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.running = false,
            _ => {}
        }
    }

    fn on_tick(&mut self) {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
    }

    fn render(&self, frame: &mut Frame) {
        let [header, top, history, processes, footer] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]).areas(frame.area());

        let [cpu, memory] = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]).areas(top);

        self.render_header(frame, header);
        self.render_cpu(frame, cpu);
        frame.render_widget(panel("Memory Usage"), memory);
        frame.render_widget(panel("History"), history);
        frame.render_widget(panel("Processes"), processes);
        self.render_footer(frame, footer);
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let sep = Span::styled(" | ", Style::new().fg(MUTED));
        let uptime = System::uptime();
        let line = Line::from(vec![
            "Ratatui System Monitor".to_string().fg(ACCENT).bold(),
            sep.clone(),
            self.host_name.clone().fg(ACCENT).bold(),
            sep.clone(),
            Span::raw(format!("up {}h {}m", uptime / 3600, uptime % 3600 / 60)),
        ]);

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(MUTED);

        frame.render_widget(Paragraph::new(line).block(block), area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let key = |k: &'static str, desc: &'static str| {
            [
                Span::styled(format!(" {k} "), Style::new().fg(Color::Black).bg(ACCENT)),
                Span::styled(format!(" {desc} \t"), MUTED),
            ]
        };

        let spans: Vec<Span> = [
            key("q", "Quit"),
            // key("Esc", "Quit"),
        ]
            .into_iter()
            .flatten()
            .collect();
        
        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }

    fn render_cpu(&self, frame: &mut Frame, area: Rect) {
        let average = self.system.global_cpu_usage();
        let lines: Vec<Line> = self
            .system
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, cpu)| {
                Line::from(format!("Core {i:>2}\t{:>3.0}%", cpu.cpu_usage()))
            })
            .collect();

        let block = panel(&format!("CPU Usage: {:.0}%", average));

        frame.render_widget(Paragraph::new(lines).block(block), area);
    }

}

fn panel(title: &str) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(MUTED)
        .title(Line::from(format!(" {title} ")).fg(ACCENT).bold())
}