use cosmic::iced::mouse;
use cosmic::iced::widget::canvas::{self, Canvas, Event, Geometry, Path, Text};
use cosmic::iced::{Color, Point, Rectangle};
use cosmic::Element;
use cosmic::Theme;

use crate::app::Message;

/// Tummo practice phases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TummoPhase {
    /// Rounds 1-3: Right nostril in, left nostril out (clears desire)
    PurifyDesire,
    /// Rounds 4-6: Left nostril in, right nostril out (clears anger)
    PurifyAnger,
    /// Rounds 7-9: Both nostrils (clears ignorance)
    PurifyIgnorance,
    /// Vase breathing with inner fire
    VaseBreathing,
}

/// Breath state within a round
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreathState {
    Inhale,
    Exhale,
    Hold,
}

/// A custom widget for Tummo meditation visualization
pub struct TummoCircle {
    phase: TummoPhase,
    breath_state: BreathState,
    round: u8,
    progress: f32,
    is_running: bool,
}

impl TummoCircle {
    pub fn new(phase: TummoPhase, breath_state: BreathState, round: u8, progress: f32, is_running: bool) -> Self {
        Self { phase, breath_state, round, progress, is_running }
    }

    pub fn view(self) -> Element<'static, Message> {
        Canvas::new(TummoCircleProgram {
            phase: self.phase,
            breath_state: self.breath_state,
            round: self.round,
            progress: self.progress,
            is_running: self.is_running,
        })
        .width(cosmic::iced::Length::Fill)
        .height(cosmic::iced::Length::Fill)
        .into()
    }
}

struct TummoCircleProgram {
    phase: TummoPhase,
    breath_state: BreathState,
    round: u8,
    progress: f32,
    is_running: bool,
}

impl TummoCircleProgram {
    const BG_DARK: Color = Color { r: 0.05, g: 0.03, b: 0.08, a: 1.0 };
    const CHANNEL_LEFT: Color = Color { r: 0.85, g: 0.15, b: 0.15, a: 1.0 };
    const CHANNEL_RIGHT: Color = Color { r: 0.95, g: 0.95, b: 0.95, a: 1.0 };
    const CHANNEL_CENTER: Color = Color { r: 0.2, g: 0.4, b: 0.9, a: 1.0 };
    const FIRE_CORE: Color = Color { r: 1.0, g: 0.4, b: 0.1, a: 1.0 };
    const TEXT_COLOR: Color = Color { r: 1.0, g: 0.95, b: 0.85, a: 1.0 };
    const DIM_TEXT: Color = Color { r: 0.6, g: 0.55, b: 0.5, a: 1.0 };

    fn get_instruction(&self) -> &'static str {
        if !self.is_running {
            return "Begin";
        }
        match self.phase {
            TummoPhase::PurifyDesire => match self.breath_state {
                BreathState::Inhale => "Inhale Right",
                BreathState::Exhale => "Exhale Left",
                BreathState::Hold => "Hold",
            },
            TummoPhase::PurifyAnger => match self.breath_state {
                BreathState::Inhale => "Inhale Left",
                BreathState::Exhale => "Exhale Right",
                BreathState::Hold => "Hold",
            },
            TummoPhase::PurifyIgnorance => match self.breath_state {
                BreathState::Inhale => "Inhale Both",
                BreathState::Exhale => "Exhale Both",
                BreathState::Hold => "Hold",
            },
            TummoPhase::VaseBreathing => match self.breath_state {
                BreathState::Inhale => "Draw In",
                BreathState::Hold => "Hold Below",
                BreathState::Exhale => "Release",
            },
        }
    }

    fn get_phase_description(&self) -> &'static str {
        match self.phase {
            TummoPhase::PurifyDesire => "Clearing Desire",
            TummoPhase::PurifyAnger => "Clearing Anger",
            TummoPhase::PurifyIgnorance => "Clearing Ignorance",
            TummoPhase::VaseBreathing => "Inner Fire",
        }
    }

    fn draw_body_silhouette(&self, frame: &mut canvas::Frame, center: Point, scale: f32) {
        let body_color = Color::from_rgba(0.15, 0.12, 0.18, 0.8);

        let head_radius = 25.0 * scale;
        let head_center = Point::new(center.x, center.y - 80.0 * scale);
        let head = Path::circle(head_center, head_radius);
        frame.fill(&head, body_color);

        let body = Path::new(|builder| {
            builder.move_to(Point::new(center.x - 50.0 * scale, center.y + 80.0 * scale));
            builder.quadratic_curve_to(
                Point::new(center.x - 60.0 * scale, center.y),
                Point::new(center.x - 20.0 * scale, center.y - 45.0 * scale),
            );
            builder.quadratic_curve_to(
                Point::new(center.x, center.y - 55.0 * scale),
                Point::new(center.x + 20.0 * scale, center.y - 45.0 * scale),
            );
            builder.quadratic_curve_to(
                Point::new(center.x + 60.0 * scale, center.y),
                Point::new(center.x + 50.0 * scale, center.y + 80.0 * scale),
            );
            builder.close();
        });
        frame.fill(&body, body_color);
    }

    fn draw_channels(&self, frame: &mut canvas::Frame, center: Point, scale: f32) {
        let channel_width = 6.0 * scale;

        let (left_active, right_active, center_active) = match self.phase {
            TummoPhase::PurifyDesire => (
                self.breath_state == BreathState::Exhale,
                self.breath_state == BreathState::Inhale,
                false
            ),
            TummoPhase::PurifyAnger => (
                self.breath_state == BreathState::Inhale,
                self.breath_state == BreathState::Exhale,
                false
            ),
            TummoPhase::PurifyIgnorance => (true, true, true),
            TummoPhase::VaseBreathing => (false, false, true),
        };

        let left_alpha = if left_active { 0.9 } else { 0.25 };
        let left_color = Color::from_rgba(
            Self::CHANNEL_LEFT.r, Self::CHANNEL_LEFT.g, Self::CHANNEL_LEFT.b, left_alpha
        );

        let left_channel = Path::new(|builder| {
            builder.move_to(Point::new(center.x - 12.0 * scale, center.y - 95.0 * scale));
            builder.quadratic_curve_to(
                Point::new(center.x - 35.0 * scale, center.y - 40.0 * scale),
                Point::new(center.x - 8.0 * scale, center.y + 50.0 * scale),
            );
        });
        frame.stroke(&left_channel, canvas::Stroke::default().with_color(left_color).with_width(channel_width));

        let right_alpha = if right_active { 0.9 } else { 0.25 };
        let right_color = Color::from_rgba(
            Self::CHANNEL_RIGHT.r, Self::CHANNEL_RIGHT.g, Self::CHANNEL_RIGHT.b, right_alpha
        );

        let right_channel = Path::new(|builder| {
            builder.move_to(Point::new(center.x + 12.0 * scale, center.y - 95.0 * scale));
            builder.quadratic_curve_to(
                Point::new(center.x + 35.0 * scale, center.y - 40.0 * scale),
                Point::new(center.x + 8.0 * scale, center.y + 50.0 * scale),
            );
        });
        frame.stroke(&right_channel, canvas::Stroke::default().with_color(right_color).with_width(channel_width));

        let center_alpha = if center_active { 0.9 } else { 0.2 };
        let center_color = Color::from_rgba(
            Self::CHANNEL_CENTER.r, Self::CHANNEL_CENTER.g, Self::CHANNEL_CENTER.b, center_alpha
        );

        let central_channel = Path::new(|builder| {
            builder.move_to(Point::new(center.x, center.y - 100.0 * scale));
            builder.line_to(Point::new(center.x, center.y + 60.0 * scale));
        });
        frame.stroke(&central_channel, canvas::Stroke::default().with_color(center_color).with_width(channel_width * 1.2));

        if self.is_running {
            self.draw_energy_flow(frame, center, scale, left_active, right_active, center_active);
        }
    }

    fn draw_energy_flow(&self, frame: &mut canvas::Frame, center: Point, scale: f32,
                        left_active: bool, right_active: bool, center_active: bool) {
        let num_particles = 5;

        for i in 0..num_particles {
            let offset = (i as f32 / num_particles as f32 + self.progress) % 1.0;
            let particle_alpha = 0.8 * (1.0 - (offset - 0.5).abs() * 2.0).max(0.0);

            if left_active {
                let is_descending = self.breath_state == BreathState::Exhale;
                let t = if is_descending { offset } else { 1.0 - offset };
                let x = center.x - 12.0 * scale + (center.x - 8.0 * scale - (center.x - 12.0 * scale)) * t
                        - 23.0 * scale * (1.0 - (t - 0.5).abs() * 2.0);
                let y = (center.y - 95.0 * scale) + ((center.y + 50.0 * scale) - (center.y - 95.0 * scale)) * t;
                let particle = Path::circle(Point::new(x, y), 4.0 * scale);
                let color = Color::from_rgba(1.0, 0.3, 0.3, particle_alpha);
                frame.fill(&particle, color);
            }

            if right_active {
                let is_descending = match self.phase {
                    TummoPhase::PurifyDesire => self.breath_state == BreathState::Inhale,
                    TummoPhase::PurifyAnger => self.breath_state == BreathState::Exhale,
                    _ => self.breath_state == BreathState::Inhale,
                };
                let t = if is_descending { 1.0 - offset } else { offset };
                let x = center.x + 12.0 * scale + (center.x + 8.0 * scale - (center.x + 12.0 * scale)) * t
                        + 23.0 * scale * (1.0 - (t - 0.5).abs() * 2.0);
                let y = (center.y - 95.0 * scale) + ((center.y + 50.0 * scale) - (center.y - 95.0 * scale)) * t;
                let particle = Path::circle(Point::new(x, y), 4.0 * scale);
                let color = Color::from_rgba(1.0, 1.0, 1.0, particle_alpha);
                frame.fill(&particle, color);
            }

            if center_active && self.phase == TummoPhase::VaseBreathing {
                let t = match self.breath_state {
                    BreathState::Inhale => offset,
                    BreathState::Hold => 0.9 + offset * 0.1,
                    BreathState::Exhale => 1.0 - offset,
                };
                let y = (center.y - 100.0 * scale) + ((center.y + 60.0 * scale) - (center.y - 100.0 * scale)) * t;
                let particle = Path::circle(Point::new(center.x, y), 5.0 * scale);
                let color = Color::from_rgba(0.3, 0.5, 1.0, particle_alpha);
                frame.fill(&particle, color);
            }
        }
    }

    fn draw_inner_fire(&self, frame: &mut canvas::Frame, center: Point, scale: f32) {
        if self.phase != TummoPhase::VaseBreathing {
            return;
        }

        let fire_center = Point::new(center.x, center.y + 35.0 * scale);

        let intensity = match self.breath_state {
            BreathState::Inhale => 0.3 + self.progress * 0.4,
            BreathState::Hold => 0.7 + self.progress * 0.3,
            BreathState::Exhale => 1.0 - self.progress * 0.5,
        };

        for i in (1..=5).rev() {
            let glow_radius = (15.0 + i as f32 * 12.0) * scale * intensity;
            let alpha = 0.15 / (i as f32) * intensity;
            let glow = Path::circle(fire_center, glow_radius);
            frame.fill(&glow, Color::from_rgba(1.0, 0.5, 0.0, alpha));
        }

        let core_radius = 12.0 * scale * intensity;
        let core = Path::circle(fire_center, core_radius);
        frame.fill(&core, Color::from_rgba(
            Self::FIRE_CORE.r, Self::FIRE_CORE.g, Self::FIRE_CORE.b, intensity
        ));

        let inner_radius = 5.0 * scale * intensity;
        let inner = Path::circle(fire_center, inner_radius);
        frame.fill(&inner, Color::from_rgba(1.0, 0.9, 0.7, intensity));

        if intensity > 0.5 {
            let flame_height = 30.0 * scale * intensity;
            let flame = Path::new(|builder| {
                builder.move_to(Point::new(fire_center.x - 8.0 * scale, fire_center.y));
                builder.line_to(Point::new(fire_center.x, fire_center.y - flame_height));
                builder.line_to(Point::new(fire_center.x + 8.0 * scale, fire_center.y));
                builder.close();
            });
            frame.fill(&flame, Color::from_rgba(1.0, 0.6, 0.1, intensity * 0.7));
        }
    }

    fn draw_nostril_indicators(&self, frame: &mut canvas::Frame, center: Point, scale: f32) {
        if self.phase == TummoPhase::VaseBreathing {
            return;
        }

        let nostril_y = center.y - 95.0 * scale;
        let left_x = center.x - 12.0 * scale;
        let right_x = center.x + 12.0 * scale;
        let indicator_offset = 25.0 * scale;

        let (left_in, left_out, right_in, right_out) = match self.phase {
            TummoPhase::PurifyDesire => (false, self.breath_state == BreathState::Exhale,
                                         self.breath_state == BreathState::Inhale, false),
            TummoPhase::PurifyAnger => (self.breath_state == BreathState::Inhale, false,
                                        false, self.breath_state == BreathState::Exhale),
            TummoPhase::PurifyIgnorance => (self.breath_state == BreathState::Inhale, self.breath_state == BreathState::Exhale,
                                            self.breath_state == BreathState::Inhale, self.breath_state == BreathState::Exhale),
            _ => (false, false, false, false),
        };

        let arrow_color = Color::from_rgba(0.9, 0.85, 0.7, 0.9);

        if left_in {
            self.draw_arrow(frame, Point::new(left_x - indicator_offset, nostril_y - 15.0 * scale),
                           Point::new(left_x, nostril_y), arrow_color, scale);
        }
        if left_out {
            self.draw_arrow(frame, Point::new(left_x, nostril_y),
                           Point::new(left_x - indicator_offset, nostril_y - 15.0 * scale), arrow_color, scale);
        }
        if right_in {
            self.draw_arrow(frame, Point::new(right_x + indicator_offset, nostril_y - 15.0 * scale),
                           Point::new(right_x, nostril_y), arrow_color, scale);
        }
        if right_out {
            self.draw_arrow(frame, Point::new(right_x, nostril_y),
                           Point::new(right_x + indicator_offset, nostril_y - 15.0 * scale), arrow_color, scale);
        }
    }

    fn draw_arrow(&self, frame: &mut canvas::Frame, from: Point, to: Point, color: Color, scale: f32) {
        let line = Path::line(from, to);
        frame.stroke(&line, canvas::Stroke::default().with_color(color).with_width(2.5 * scale));

        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let len = (dx * dx + dy * dy).sqrt();
        let ux = dx / len;
        let uy = dy / len;

        let arrow_size = 8.0 * scale;
        let head = Path::new(|builder| {
            builder.move_to(to);
            builder.line_to(Point::new(
                to.x - arrow_size * (ux + uy * 0.5),
                to.y - arrow_size * (uy - ux * 0.5),
            ));
            builder.line_to(Point::new(
                to.x - arrow_size * (ux - uy * 0.5),
                to.y - arrow_size * (uy + ux * 0.5),
            ));
            builder.close();
        });
        frame.fill(&head, color);
    }

    fn draw_round_indicator(&self, frame: &mut canvas::Frame, bounds: Rectangle) {
        if self.phase == TummoPhase::VaseBreathing {
            return;
        }

        let indicator_y = bounds.height - 80.0;
        let start_x = bounds.width / 2.0 - 60.0;

        for i in 0..9 {
            let x = start_x + i as f32 * 15.0;
            let is_current = i + 1 == self.round as usize;
            let is_complete = i + 1 < self.round as usize;

            let base_color = if i < 3 {
                Self::CHANNEL_RIGHT
            } else if i < 6 {
                Self::CHANNEL_LEFT
            } else {
                Self::CHANNEL_CENTER
            };

            let (color, radius) = if is_current {
                (base_color, 6.0)
            } else if is_complete {
                (Color::from_rgba(base_color.r, base_color.g, base_color.b, 0.8), 4.0)
            } else {
                (Color::from_rgba(base_color.r, base_color.g, base_color.b, 0.25), 4.0)
            };

            let dot = Path::circle(Point::new(x, indicator_y), radius);
            frame.fill(&dot, color);

            if is_current && self.is_running {
                let progress_arc = Path::circle(Point::new(x, indicator_y), 8.0);
                frame.stroke(&progress_arc, canvas::Stroke::default()
                    .with_color(Color::from_rgba(1.0, 1.0, 1.0, 0.5))
                    .with_width(2.0));
            }
        }
    }
}

impl canvas::Program<Message, Theme> for TummoCircleProgram {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
            if cursor.position_in(bounds).is_some() {
                return (canvas::event::Status::Captured, Some(Message::ToggleRunning));
            }
        }
        (canvas::event::Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &cosmic::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<cosmic::Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0 - 20.0);

        let scale = (bounds.width.min(bounds.height) / 400.0).clamp(0.6, 1.5);

        let bg = Path::rectangle(Point::new(0.0, 0.0), bounds.size());
        frame.fill(&bg, Self::BG_DARK);

        for i in (1..=6).rev() {
            let gradient_radius = (bounds.width.min(bounds.height) / 2.0) * (i as f32 / 6.0);
            let alpha = 0.04 * (1.0 - (i as f32 / 6.0));
            let gradient_circle = Path::circle(center, gradient_radius);
            frame.fill(&gradient_circle, Color::from_rgba(0.2, 0.1, 0.3, alpha));
        }

        self.draw_body_silhouette(&mut frame, center, scale);
        self.draw_channels(&mut frame, center, scale);
        self.draw_inner_fire(&mut frame, center, scale);
        self.draw_nostril_indicators(&mut frame, center, scale);
        self.draw_round_indicator(&mut frame, bounds);

        let phase_text = Text {
            content: self.get_phase_description().to_string(),
            position: Point::new(bounds.width / 2.0, 40.0),
            color: Self::DIM_TEXT,
            size: cosmic::iced::Pixels(18.0),
            horizontal_alignment: cosmic::iced::alignment::Horizontal::Center,
            vertical_alignment: cosmic::iced::alignment::Vertical::Center,
            ..Text::default()
        };
        frame.fill_text(phase_text);

        let instruction = Text {
            content: self.get_instruction().to_string(),
            position: Point::new(bounds.width / 2.0, bounds.height - 130.0),
            color: Self::TEXT_COLOR,
            size: cosmic::iced::Pixels(28.0 * scale),
            horizontal_alignment: cosmic::iced::alignment::Horizontal::Center,
            vertical_alignment: cosmic::iced::alignment::Vertical::Center,
            ..Text::default()
        };
        frame.fill_text(instruction);

        if self.phase != TummoPhase::VaseBreathing && self.is_running {
            let round_text = Text {
                content: format!("Round {}/9", self.round),
                position: Point::new(bounds.width / 2.0, bounds.height - 50.0),
                color: Self::DIM_TEXT,
                size: cosmic::iced::Pixels(16.0),
                horizontal_alignment: cosmic::iced::alignment::Horizontal::Center,
                vertical_alignment: cosmic::iced::alignment::Vertical::Center,
                ..Text::default()
            };
            frame.fill_text(round_text);
        }

        vec![frame.into_geometry()]
    }
}
