use std::time::{Duration, Instant};

use cosmic::app::Core;
use cosmic::iced::keyboard::{self, Key, key::Named};
use cosmic::iced::Length;
use cosmic::widget::{self, button, container, settings, text};
use cosmic::{Apply, Application, Element, Task};

use diaframe::{AudioPlayer, SessionDuration};
use crate::config::{PersistentSettings, TummoConfig};
use crate::widgets::{TummoCircle, TummoPhase, BreathState};

/// Views in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Practice,
    Settings,
}

/// Application messages
#[derive(Debug, Clone)]
pub enum Message {
    Tick(Instant),
    ToggleRunning,
    OpenSettings,
    CloseSettings,
    SetInhaleDuration(f32),
    SetExhaleDuration(f32),
    SetVaseHold(f32),
    ToggleBrownNoise(bool),
    ToggleBinaural(bool),
    ToggleTones(bool),
    SetVolume(f32),
    SetSessionDuration(SessionDuration),
    // Window controls
    Minimize,
    Maximize,
    CloseWindow,
    // Keyboard
    KeyPressed(Key),
    ToggleFullscreen,
}

/// Main application state
pub struct App {
    core: Core,
    config: TummoConfig,
    is_running: bool,
    phase_progress: f32,
    phase_start: Instant,
    current_view: View,
    audio: Option<AudioPlayer>,
    brown_enabled: bool,
    binaural_enabled: bool,
    tones_enabled: bool,
    volume: f32,
    // Tummo state
    tummo_phase: TummoPhase,
    tummo_breath: BreathState,
    tummo_round: u8,
    // UI state
    fullscreen: bool,
    // Session timer
    session_duration: SessionDuration,
    session_elapsed: f32,
    // Statistics
    stats: diaframe::PracticeStats,
}

impl Application for App {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "com.example.firechannel";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(mut core: Core, _flags: Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {
        core.window.show_close = false;
        core.window.show_maximize = false;
        core.window.show_minimize = false;
        core.window.show_headerbar = true;

        let saved = PersistentSettings::load();

        let audio = AudioPlayer::new();
        if let Some(ref a) = audio {
            a.set_volume(saved.audio.volume);
            a.set_brown_enabled(saved.audio.brown_noise);
            a.set_binaural_enabled(saved.audio.binaural);
            a.set_tones_enabled(saved.audio.tones);
        }

        let mut stats = saved.stats.clone();
        stats.check_today();

        let app = Self {
            core,
            config: TummoConfig {
                inhale_duration: saved.inhale_duration,
                exhale_duration: saved.exhale_duration,
                vase_hold_duration: saved.vase_hold_duration,
            },
            is_running: false,
            phase_progress: 0.0,
            phase_start: Instant::now(),
            current_view: View::Practice,
            audio,
            brown_enabled: saved.audio.brown_noise,
            binaural_enabled: saved.audio.binaural,
            tones_enabled: saved.audio.tones,
            volume: saved.audio.volume,
            tummo_phase: TummoPhase::PurifyDesire,
            tummo_breath: BreathState::Inhale,
            tummo_round: 1,
            fullscreen: false,
            session_duration: saved.session_duration,
            session_elapsed: 0.0,
            stats,
        };
        (app, Task::none())
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        vec![text::heading("Fire Channel").into()]
    }

    fn header_end(&self) -> Vec<Element<'_, Self::Message>> {
        let nav_button = if self.current_view == View::Settings {
            button::text("← Back")
                .on_press(Message::CloseSettings)
        } else {
            button::text("⚙ Settings")
                .on_press(Message::OpenSettings)
        };

        let minimize = button::text("−")
            .on_press(Message::Minimize);
        let maximize = button::text("□")
            .on_press(Message::Maximize);
        let close = button::text("×")
            .on_press(Message::CloseWindow)
            .class(cosmic::theme::Button::Destructive);

        vec![
            nav_button.into(),
            widget::horizontal_space().width(Length::Fixed(16.0)).into(),
            minimize.into(),
            maximize.into(),
            close.into(),
        ]
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match self.current_view {
            View::Practice => self.view_practice(),
            View::Settings => self.view_settings(),
        }
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Tick(now) => {
                if self.is_running {
                    // Track session time (~16ms per tick)
                    self.session_elapsed += 0.016;

                    // Check if session is complete
                    if let Some(duration) = self.session_duration.seconds() {
                        if self.session_elapsed >= duration {
                            self.is_running = false;
                            if let Some(ref audio) = self.audio {
                                audio.pause();
                            }
                            self.stats.record_session(self.session_elapsed);
                            self.save_settings();
                            return Task::none();
                        }
                    }

                    self.tick_tummo(now);
                }
            }
            Message::ToggleRunning => {
                self.is_running = !self.is_running;
                if self.is_running {
                    self.phase_start = Instant::now();
                    self.phase_progress = 0.0;
                    self.session_elapsed = 0.0;
                    self.tummo_phase = TummoPhase::PurifyDesire;
                    self.tummo_breath = BreathState::Inhale;
                    self.tummo_round = 1;
                    if let Some(ref audio) = self.audio {
                        audio.set_state(true, 0.0);
                        audio.play();
                    }
                } else {
                    if let Some(ref audio) = self.audio {
                        audio.pause();
                    }
                    if self.session_elapsed >= 30.0 {
                        self.stats.record_session(self.session_elapsed);
                        self.save_settings();
                    }
                }
            }
            Message::OpenSettings => {
                self.current_view = View::Settings;
                self.is_running = false;
                if let Some(ref audio) = self.audio {
                    audio.pause();
                }
            }
            Message::CloseSettings => {
                self.current_view = View::Practice;
            }
            Message::SetInhaleDuration(value) => {
                self.config.inhale_duration = value;
                self.save_settings();
            }
            Message::SetExhaleDuration(value) => {
                self.config.exhale_duration = value;
                self.save_settings();
            }
            Message::SetVaseHold(value) => {
                self.config.vase_hold_duration = value;
                self.save_settings();
            }
            Message::SetSessionDuration(duration) => {
                self.session_duration = duration;
                self.save_settings();
            }
            Message::ToggleBrownNoise(enabled) => {
                self.brown_enabled = enabled;
                if let Some(ref audio) = self.audio {
                    audio.set_brown_enabled(enabled);
                }
                self.save_settings();
            }
            Message::ToggleBinaural(enabled) => {
                self.binaural_enabled = enabled;
                if let Some(ref audio) = self.audio {
                    audio.set_binaural_enabled(enabled);
                }
                self.save_settings();
            }
            Message::ToggleTones(enabled) => {
                self.tones_enabled = enabled;
                if let Some(ref audio) = self.audio {
                    audio.set_tones_enabled(enabled);
                }
                self.save_settings();
            }
            Message::SetVolume(value) => {
                self.volume = value;
                if let Some(ref audio) = self.audio {
                    audio.set_volume(value);
                }
                self.save_settings();
            }
            Message::Minimize => {
                if let Some(id) = self.core.main_window_id() {
                    return cosmic::command::minimize(id);
                }
            }
            Message::Maximize => {
                if let Some(id) = self.core.main_window_id() {
                    return cosmic::command::toggle_maximize(id);
                }
            }
            Message::CloseWindow => {
                std::process::exit(0);
            }
            Message::KeyPressed(key) => {
                match key {
                    Key::Named(Named::Space) => {
                        if self.current_view == View::Practice {
                            return self.update(Message::ToggleRunning);
                        }
                    }
                    Key::Named(Named::Escape) => {
                        if self.current_view == View::Settings {
                            return self.update(Message::CloseSettings);
                        }
                    }
                    Key::Named(Named::F11) => {
                        return self.update(Message::ToggleFullscreen);
                    }
                    _ => {}
                }
            }
            Message::ToggleFullscreen => {
                self.fullscreen = !self.fullscreen;
                self.core.window.show_headerbar = !self.fullscreen;
            }
        }
        Task::none()
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        let tick = if self.is_running {
            cosmic::iced::time::every(Duration::from_millis(16)).map(Message::Tick)
        } else {
            cosmic::iced::Subscription::none()
        };

        let keys = keyboard::on_key_press(|key, _modifiers| {
            Some(Message::KeyPressed(key))
        });

        cosmic::iced::Subscription::batch([tick, keys])
    }
}

impl App {
    fn view_practice(&self) -> Element<'_, Message> {
        let visualization = TummoCircle::new(
            self.tummo_phase,
            self.tummo_breath,
            self.tummo_round,
            self.phase_progress,
            self.is_running,
        ).view();

        container(visualization)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_settings(&self) -> Element<'_, Message> {
        let nine_round_time = 9.0 * (self.config.inhale_duration + self.config.exhale_duration);

        // Session duration buttons
        let duration_row = widget::row::with_capacity(6)
            .push(
                button::standard("5m")
                    .on_press(Message::SetSessionDuration(SessionDuration::Minutes5))
                    .class(if self.session_duration == SessionDuration::Minutes5 {
                        cosmic::theme::Button::Suggested
                    } else {
                        cosmic::theme::Button::Standard
                    })
            )
            .push(
                button::standard("10m")
                    .on_press(Message::SetSessionDuration(SessionDuration::Minutes10))
                    .class(if self.session_duration == SessionDuration::Minutes10 {
                        cosmic::theme::Button::Suggested
                    } else {
                        cosmic::theme::Button::Standard
                    })
            )
            .push(
                button::standard("15m")
                    .on_press(Message::SetSessionDuration(SessionDuration::Minutes15))
                    .class(if self.session_duration == SessionDuration::Minutes15 {
                        cosmic::theme::Button::Suggested
                    } else {
                        cosmic::theme::Button::Standard
                    })
            )
            .push(
                button::standard("20m")
                    .on_press(Message::SetSessionDuration(SessionDuration::Minutes20))
                    .class(if self.session_duration == SessionDuration::Minutes20 {
                        cosmic::theme::Button::Suggested
                    } else {
                        cosmic::theme::Button::Standard
                    })
            )
            .push(
                button::standard("30m")
                    .on_press(Message::SetSessionDuration(SessionDuration::Minutes30))
                    .class(if self.session_duration == SessionDuration::Minutes30 {
                        cosmic::theme::Button::Suggested
                    } else {
                        cosmic::theme::Button::Standard
                    })
            )
            .push(
                button::standard("∞")
                    .on_press(Message::SetSessionDuration(SessionDuration::Infinite))
                    .class(if self.session_duration == SessionDuration::Infinite {
                        cosmic::theme::Button::Suggested
                    } else {
                        cosmic::theme::Button::Standard
                    })
            )
            .spacing(6);

        // Practice section
        let practice_section = settings::section()
            .title("Tummo Practice")
            .add(
                settings::item::builder("Nine-Round Breathing")
                    .description(format!(
                        "~{:.0} min • Clears desire, anger, ignorance",
                        nine_round_time / 60.0
                    ))
                    .control(text::body(format!(
                        "{:.0}s in / {:.0}s out",
                        self.config.inhale_duration,
                        self.config.exhale_duration
                    )))
            )
            .add(
                settings::item::builder("Inhale")
                    .description(format!("{:.1} seconds", self.config.inhale_duration))
                    .control(
                        widget::slider(2.0..=15.0, self.config.inhale_duration, Message::SetInhaleDuration)
                            .step(0.5)
                            .width(Length::Fixed(200.0))
                    )
            )
            .add(
                settings::item::builder("Exhale")
                    .description(format!("{:.1} seconds", self.config.exhale_duration))
                    .control(
                        widget::slider(2.0..=15.0, self.config.exhale_duration, Message::SetExhaleDuration)
                            .step(0.5)
                            .width(Length::Fixed(200.0))
                    )
            )
            .add(
                settings::item::builder("Vase Breath Hold")
                    .description("Increase as you progress (advanced: 2+ min)")
                    .control(
                        widget::row::with_capacity(2)
                            .push(
                                widget::slider(5.0..=120.0, self.config.vase_hold_duration, Message::SetVaseHold)
                                    .step(5.0)
                                    .width(Length::Fixed(160.0))
                            )
                            .push(text::body(format!("{:.0}s", self.config.vase_hold_duration)))
                            .spacing(12)
                            .align_y(cosmic::iced::Alignment::Center)
                    )
            )
            .add(
                settings::item::builder("Session Length")
                    .description("Auto-stop after duration")
                    .control(duration_row)
            );

        // Sound section
        let sound_section = settings::section()
            .title("Sound")
            .add(
                settings::item::builder("Brown Noise")
                    .description("Gentle background noise for focus")
                    .toggler(self.brown_enabled, Message::ToggleBrownNoise)
            )
            .add(
                settings::item::builder("Binaural Beats")
                    .description("Theta waves (6 Hz) for deep relaxation")
                    .toggler(self.binaural_enabled, Message::ToggleBinaural)
            )
            .add(
                settings::item::builder("Breathing Tones")
                    .description("Sub-bass drone following breath")
                    .toggler(self.tones_enabled, Message::ToggleTones)
            )
            .add(
                settings::item::builder("Volume")
                    .control(
                        widget::row::with_capacity(2)
                            .push(
                                widget::slider(0.0..=1.0, self.volume, Message::SetVolume)
                                    .step(0.05)
                                    .width(Length::Fixed(160.0))
                            )
                            .push(text::body(format!("{:.0}%", self.volume * 100.0)))
                            .spacing(12)
                            .align_y(cosmic::iced::Alignment::Center)
                    )
            );

        // Statistics section
        let stats_section = settings::section()
            .title("Statistics")
            .add(
                settings::item::builder("Today")
                    .control(text::body(diaframe::format_duration(self.stats.today_practice_seconds)))
            )
            .add(
                settings::item::builder("Total Practice")
                    .control(text::body(diaframe::format_duration(self.stats.total_practice_seconds)))
            )
            .add(
                settings::item::builder("Sessions")
                    .control(text::body(format!("{}", self.stats.sessions_completed)))
            );

        let content = settings::view_column(vec![
            practice_section.into(),
            sound_section.into(),
            stats_section.into(),
        ]);

        widget::scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .apply(container)
            .padding([0, 24])
            .into()
    }

    fn tick_tummo(&mut self, now: Instant) {
        let duration = match self.tummo_breath {
            BreathState::Inhale => self.config.inhale_duration,
            BreathState::Exhale => self.config.exhale_duration,
            BreathState::Hold => self.config.vase_hold_duration,
        };

        let elapsed = now.duration_since(self.phase_start).as_secs_f32();
        self.phase_progress = (elapsed / duration).min(1.0);

        if let Some(ref audio) = self.audio {
            audio.set_state(self.tummo_breath == BreathState::Inhale, self.phase_progress);
        }

        if self.phase_progress >= 1.0 {
            self.phase_start = now;
            self.phase_progress = 0.0;
            self.advance_tummo();
        }
    }

    fn advance_tummo(&mut self) {
        match self.tummo_phase {
            TummoPhase::PurifyDesire | TummoPhase::PurifyAnger | TummoPhase::PurifyIgnorance => {
                match self.tummo_breath {
                    BreathState::Inhale => {
                        self.tummo_breath = BreathState::Exhale;
                    }
                    BreathState::Exhale => {
                        self.tummo_breath = BreathState::Inhale;
                        self.tummo_round += 1;

                        if self.tummo_round > 9 {
                            self.tummo_phase = TummoPhase::VaseBreathing;
                            self.tummo_round = 0;
                            self.tummo_breath = BreathState::Inhale;
                        } else if self.tummo_round > 6 {
                            self.tummo_phase = TummoPhase::PurifyIgnorance;
                        } else if self.tummo_round > 3 {
                            self.tummo_phase = TummoPhase::PurifyAnger;
                        }
                    }
                    BreathState::Hold => {
                        self.tummo_breath = BreathState::Exhale;
                    }
                }
            }
            TummoPhase::VaseBreathing => {
                match self.tummo_breath {
                    BreathState::Inhale => {
                        self.tummo_breath = BreathState::Hold;
                    }
                    BreathState::Hold => {
                        self.tummo_breath = BreathState::Exhale;
                    }
                    BreathState::Exhale => {
                        self.tummo_breath = BreathState::Inhale;
                    }
                }
            }
        }
    }

    fn save_settings(&self) {
        let settings = PersistentSettings {
            inhale_duration: self.config.inhale_duration,
            exhale_duration: self.config.exhale_duration,
            vase_hold_duration: self.config.vase_hold_duration,
            session_duration: self.session_duration,
            audio: diaframe::AudioSettings {
                brown_noise: self.brown_enabled,
                binaural: self.binaural_enabled,
                tones: self.tones_enabled,
                volume: self.volume,
            },
            stats: self.stats.clone(),
        };
        settings.save();
    }
}
